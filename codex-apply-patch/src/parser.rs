//! This module is responsible for parsing & validating a patch into a list of "hunks".
//! (It does not attempt to actually check that the patch can be applied to the filesystem.)
//!
//! The official Lark grammar for the apply-patch format is:
//!
//! start: begin_patch hunk+ end_patch
//! begin_patch: "*** Begin Patch" LF
//! end_patch: "*** End Patch" LF?
//!
//! hunk: add_hunk | delete_hunk | update_hunk
//! add_hunk: "*** Add File: " filename LF add_line*
//! delete_hunk: "*** Delete File: " filename LF
//! update_hunk: "*** Update File: " filename LF change_move? change?
//! filename: /(.+)/
//! add_line: "+" /(.*)/ LF -> line
//!
//! change_move: "*** Move to: " filename LF
//! change: (change_context | change_line)+ eof_line?
//! change_context: ("@@" | "@@ " /(.+)/) LF
//! change_line: ("+" | "-" | " ") /(.+)/ LF
//! eof_line: "*** End of File" LF
//!
//! The parser below is a little more lenient than the explicit spec and allows for
//! leading/trailing whitespace around patch markers. It also accepts a local
//! line-number-assisted extension for old-side authored evidence such as
//! `@@ 120 | def handler():` and `-121 | old value`.
use crate::ApplyPatchArgs;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;

const BEGIN_PATCH_MARKER: &str = "*** Begin Patch";
const END_PATCH_MARKER: &str = "*** End Patch";
const ADD_FILE_MARKER: &str = "*** Add File: ";
const DELETE_FILE_MARKER: &str = "*** Delete File: ";
const UPDATE_FILE_MARKER: &str = "*** Update File: ";
const MOVE_TO_MARKER: &str = "*** Move to: ";
const EOF_MARKER: &str = "*** End of File";
const CHANGE_CONTEXT_MARKER: &str = "@@ ";
const EMPTY_CHANGE_CONTEXT_MARKER: &str = "@@";

/// Currently, the only OpenAI model that knowingly requires lenient parsing is
/// gpt-4.1. While we could try to require everyone to pass in a strictness
/// param when invoking apply_patch, it is a pain to thread it through all of
/// the call sites, so we resign ourselves allowing lenient parsing for all
/// models. See [`ParseMode::Lenient`] for details on the exceptions we make for
/// gpt-4.1.
const PARSE_IN_STRICT_MODE: bool = false;

#[derive(Debug, PartialEq, Error, Clone)]
pub enum ParseError {
    #[error("invalid patch: {0}")]
    InvalidPatchError(String),
    #[error("invalid hunk at line {line_number}, {message}")]
    InvalidHunkError { message: String, line_number: usize },
}
use ParseError::*;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Hunk {
    AddFile {
        path: PathBuf,
        contents: String,
    },
    DeleteFile {
        path: PathBuf,
    },
    UpdateFile {
        path: PathBuf,
        move_path: Option<PathBuf>,

        /// Chunks should be in order, i.e. the `change_context` of one chunk
        /// should occur later in the file than the previous chunk.
        chunks: Vec<UpdateFileChunk>,
    },
}

impl Hunk {
    pub fn resolve_path(&self, cwd: &Path) -> PathBuf {
        match self {
            Hunk::AddFile { path, .. } => cwd.join(path),
            Hunk::DeleteFile { path } => cwd.join(path),
            Hunk::UpdateFile { path, .. } => cwd.join(path),
        }
    }
}

use Hunk::*;

#[derive(Debug, PartialEq, Clone)]
pub struct UpdateFileChunk {
    /// A single line of context used to narrow down the position of the chunk
    /// (this is usually a class, method, or function definition.)
    ///
    /// When the author uses line-number-assisted locking, the raw authored
    /// evidence is preserved here, e.g. `120 | def handler():`.
    pub change_context: Option<String>,

    /// A contiguous block of old-side authored evidence that should be
    /// replaced with `new_lines`. `old_lines` must occur strictly after
    /// `change_context`.
    ///
    /// When the author uses line-number-assisted locking on old-side lines,
    /// the raw authored evidence is preserved here, e.g.
    /// `121 |     value = 1`.
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,

    /// If set to true, `old_lines` must occur at the end of the source file.
    /// (Tolerance around trailing newlines should be encouraged.)
    pub is_end_of_file: bool,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct PatchSourceMap {
    pub actions: Vec<PatchActionSource>,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct PatchActionSource {
    pub line_number: usize,
    pub column_number: usize,
    pub move_line_number: Option<usize>,
    pub move_column_number: Option<usize>,
    pub chunks: Vec<PatchChunkSource>,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct PatchChunkSource {
    pub line_number: usize,
    pub column_number: usize,
    pub first_old_line_number: Option<usize>,
    pub first_old_line_column: Option<usize>,
    pub first_removed_line_number: Option<usize>,
    pub first_removed_line_column: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
enum NumberedHeaderEvidence {
    None,
    Valid,
}

fn classify_numbered_header_evidence(payload: &str) -> Result<NumberedHeaderEvidence, String> {
    let Some(first) = payload.chars().next() else {
        return Ok(NumberedHeaderEvidence::None);
    };
    if !first.is_ascii_digit() {
        return Ok(NumberedHeaderEvidence::None);
    }

    let digit_count = payload.chars().take_while(|ch| ch.is_ascii_digit()).count();
    let digits = &payload[..digit_count];
    let rest = &payload[digit_count..];

    if let Some(visible) = rest.strip_prefix(" | ") {
        if digits.starts_with('0') {
            return Err(
                "line-number-assisted evidence must use a positive integer without leading zeroes"
                    .to_string(),
            );
        }
        let parsed = digits.parse::<usize>().map_err(|_| {
            "line-number-assisted evidence must use a valid positive integer".to_string()
        })?;
        if parsed == 0 {
            return Err(
                "line-number-assisted evidence must use a positive integer without leading zeroes"
                    .to_string(),
            );
        }
        let _ = visible;
        return Ok(NumberedHeaderEvidence::Valid);
    }

    if rest.starts_with('|') || rest.starts_with(" |") || rest.is_empty() {
        return Err(
            "line-number-assisted evidence must use the exact 'N | visible text' delimiter"
                .to_string(),
        );
    }

    Ok(NumberedHeaderEvidence::None)
}

pub fn parse_patch(patch: &str) -> Result<ApplyPatchArgs, ParseError> {
    parse_patch_with_source_map(patch).map(|(args, _)| args)
}

pub fn parse_patch_with_source_map(
    patch: &str,
) -> Result<(ApplyPatchArgs, PatchSourceMap), ParseError> {
    let mode = if PARSE_IN_STRICT_MODE {
        ParseMode::Strict
    } else {
        ParseMode::Lenient
    };
    parse_patch_text_with_source_map(patch, mode)
}

enum ParseMode {
    /// Parse the patch text argument as is.
    Strict,

    /// GPT-4.1 is known to formulate the `command` array for the `local_shell`
    /// tool call for `apply_patch` call using something like the following:
    ///
    /// ```json
    /// [
    ///   "apply_patch",
    ///   "<<'EOF'\n*** Begin Patch\n*** Update File: README.md\n@@...\n*** End Patch\nEOF\n",
    /// ]
    /// ```
    ///
    /// This is a problem because `local_shell` is a bit of a misnomer: the
    /// `command` is not invoked by passing the arguments to a shell like Bash,
    /// but are invoked using something akin to `execvpe(3)`.
    ///
    /// This is significant in this case because where a shell would interpret
    /// `<<'EOF'...` as a heredoc and pass the contents via stdin (which is
    /// fine, as `apply_patch` is specified to read from stdin if no argument is
    /// passed), `execvpe(3)` interprets the heredoc as a literal string. To get
    /// the `local_shell` tool to run a command the way shell would, the
    /// `command` array must be something like:
    ///
    /// ```json
    /// [
    ///   "bash",
    ///   "-lc",
    ///   "apply_patch <<'EOF'\n*** Begin Patch\n*** Update File: README.md\n@@...\n*** End Patch\nEOF\n",
    /// ]
    /// ```
    ///
    /// In lenient mode, we check if the argument to `apply_patch` starts with
    /// `<<'EOF'` and ends with `EOF\n`. If so, we strip off these markers,
    /// trim() the result, and treat what is left as the patch text.
    Lenient,
}

#[cfg(test)]
fn parse_patch_text(patch: &str, mode: ParseMode) -> Result<ApplyPatchArgs, ParseError> {
    parse_patch_text_with_source_map(patch, mode).map(|(args, _)| args)
}

fn parse_patch_text_with_source_map(
    patch: &str,
    mode: ParseMode,
) -> Result<(ApplyPatchArgs, PatchSourceMap), ParseError> {
    let lines: Vec<&str> = patch.trim().lines().collect();
    let lines: &[&str] = match check_patch_boundaries_strict(&lines) {
        Ok(()) => &lines,
        Err(e) => match mode {
            ParseMode::Strict => {
                return Err(e);
            }
            ParseMode::Lenient => check_patch_boundaries_lenient(&lines, e)?,
        },
    };

    let mut hunks: Vec<Hunk> = Vec::new();
    let mut action_sources = Vec::new();
    // The above checks ensure that lines.len() >= 2.
    let last_line_index = lines.len().saturating_sub(1);
    let mut remaining_lines = &lines[1..last_line_index];
    let mut line_number = 2;
    while !remaining_lines.is_empty() {
        let (hunk, action_source, hunk_lines) = parse_one_hunk(remaining_lines, line_number)?;
        hunks.push(hunk);
        action_sources.push(action_source);
        line_number += hunk_lines;
        remaining_lines = &remaining_lines[hunk_lines..]
    }
    let patch = lines.join("\n");
    Ok((
        ApplyPatchArgs {
            hunks,
            patch,
            workdir: None,
        },
        PatchSourceMap {
            actions: action_sources,
        },
    ))
}

/// Checks the start and end lines of the patch text for `apply_patch`,
/// returning an error if they do not match the expected markers.
fn check_patch_boundaries_strict(lines: &[&str]) -> Result<(), ParseError> {
    let (first_line, last_line) = match lines {
        [] => (None, None),
        [first] => (Some(first), Some(first)),
        [first, .., last] => (Some(first), Some(last)),
    };
    check_start_and_end_lines_strict(first_line, last_line)
}

/// If we are in lenient mode, we check if the first line starts with `<<EOF`
/// (possibly quoted) and the last line ends with `EOF`. There must be at least
/// 4 lines total because the heredoc markers take up 2 lines and the patch text
/// must have at least 2 lines.
///
/// If successful, returns the lines of the patch text that contain the patch
/// contents, excluding the heredoc markers.
fn check_patch_boundaries_lenient<'a>(
    original_lines: &'a [&'a str],
    original_parse_error: ParseError,
) -> Result<&'a [&'a str], ParseError> {
    match original_lines {
        [first, .., last] => {
            if (first == &"<<EOF" || first == &"<<'EOF'" || first == &"<<\"EOF\"")
                && last.ends_with("EOF")
                && original_lines.len() >= 4
            {
                let inner_lines = &original_lines[1..original_lines.len() - 1];
                match check_patch_boundaries_strict(inner_lines) {
                    Ok(()) => Ok(inner_lines),
                    Err(e) => Err(e),
                }
            } else {
                Err(original_parse_error)
            }
        }
        _ => Err(original_parse_error),
    }
}

fn check_start_and_end_lines_strict(
    first_line: Option<&&str>,
    last_line: Option<&&str>,
) -> Result<(), ParseError> {
    let first_line = first_line.map(|line| line.trim());
    let last_line = last_line.map(|line| line.trim());

    match (first_line, last_line) {
        (Some(first), Some(last)) if first == BEGIN_PATCH_MARKER && last == END_PATCH_MARKER => {
            Ok(())
        }
        (Some(first), _) if first != BEGIN_PATCH_MARKER => Err(InvalidPatchError(String::from(
            "The first line of the patch must be '*** Begin Patch'",
        ))),
        _ => Err(InvalidPatchError(String::from(
            "The last line of the patch must be '*** End Patch'",
        ))),
    }
}

/// Attempts to parse a single hunk from the start of lines.
/// Returns the parsed hunk and the number of lines parsed (or a ParseError).
fn parse_one_hunk(
    lines: &[&str],
    line_number: usize,
) -> Result<(Hunk, PatchActionSource, usize), ParseError> {
    // Be tolerant of case mismatches and extra padding around marker strings.
    let first_line = lines[0].trim();
    if let Some(path) = first_line.strip_prefix(ADD_FILE_MARKER) {
        // Add File
        let mut contents = String::new();
        let mut parsed_lines = 1;
        for add_line in &lines[1..] {
            if let Some(line_to_add) = add_line.strip_prefix('+') {
                contents.push_str(line_to_add);
                contents.push('\n');
                parsed_lines += 1;
            } else if add_line.trim_start().starts_with("***") {
                break;
            } else {
                return Err(InvalidHunkError {
                    message: "Add File block requires lines prefixed with '+'".to_string(),
                    line_number: line_number + parsed_lines,
                });
            }
        }
        return Ok((
            AddFile {
                path: PathBuf::from(path),
                contents,
            },
            PatchActionSource {
                line_number,
                column_number: 1,
                move_line_number: None,
                move_column_number: None,
                chunks: Vec::new(),
            },
            parsed_lines,
        ));
    } else if let Some(path) = first_line.strip_prefix(DELETE_FILE_MARKER) {
        // Delete File
        return Ok((
            DeleteFile {
                path: PathBuf::from(path),
            },
            PatchActionSource {
                line_number,
                column_number: 1,
                move_line_number: None,
                move_column_number: None,
                chunks: Vec::new(),
            },
            1,
        ));
    } else if let Some(path) = first_line.strip_prefix(UPDATE_FILE_MARKER) {
        // Update File
        let mut remaining_lines = &lines[1..];
        let mut parsed_lines = 1;

        // Optional: move file line
        let move_path = remaining_lines
            .first()
            .and_then(|x| x.strip_prefix(MOVE_TO_MARKER));
        let move_source_line_number = move_path.map(|_| line_number + parsed_lines);

        if move_path.is_some() {
            remaining_lines = &remaining_lines[1..];
            parsed_lines += 1;
        }

        let mut chunks = Vec::new();
        let mut chunk_sources = Vec::new();
        // NOTE: we need to know to stop once we reach the next special marker header.
        while !remaining_lines.is_empty() {
            // Skip over any completely blank lines that may separate chunks.
            if remaining_lines[0].trim().is_empty() {
                parsed_lines += 1;
                remaining_lines = &remaining_lines[1..];
                continue;
            }

            if remaining_lines[0].starts_with("***") {
                break;
            }

            let (chunk, chunk_source, chunk_lines) = parse_update_file_chunk(
                remaining_lines,
                line_number + parsed_lines,
                chunks.is_empty(),
            )?;
            chunks.push(chunk);
            chunk_sources.push(chunk_source);
            parsed_lines += chunk_lines;
            remaining_lines = &remaining_lines[chunk_lines..]
        }

        if chunks.is_empty() {
            return Err(InvalidHunkError {
                message: format!("Update file hunk for path '{path}' is empty"),
                line_number,
            });
        }

        return Ok((
            UpdateFile {
                path: PathBuf::from(path),
                move_path: move_path.map(PathBuf::from),
                chunks,
            },
            PatchActionSource {
                line_number,
                column_number: 1,
                move_line_number: move_source_line_number,
                move_column_number: move_source_line_number.map(|_| 1),
                chunks: chunk_sources,
            },
            parsed_lines,
        ));
    }

    Err(InvalidHunkError {
        message: format!(
            "'{first_line}' is not a valid hunk header. Valid hunk headers: '*** Add File: {{path}}', '*** Delete File: {{path}}', '*** Update File: {{path}}'"
        ),
        line_number,
    })
}

fn parse_update_file_chunk(
    lines: &[&str],
    line_number: usize,
    allow_missing_context: bool,
) -> Result<(UpdateFileChunk, PatchChunkSource, usize), ParseError> {
    if lines.is_empty() {
        return Err(InvalidHunkError {
            message: "Update hunk does not contain any lines".to_string(),
            line_number,
        });
    }
    // If we see an explicit context marker @@ or @@ <context>, consume it; otherwise, optionally
    // allow treating the chunk as starting directly with diff lines.
    let (change_context, start_index) = if lines[0] == EMPTY_CHANGE_CONTEXT_MARKER {
        (None, 1)
    } else if let Some(context) = lines[0].strip_prefix(CHANGE_CONTEXT_MARKER) {
        classify_numbered_header_evidence(context).map_err(|message| InvalidHunkError {
            message,
            line_number,
        })?;
        (Some(context.to_string()), 1)
    } else {
        if !allow_missing_context {
            return Err(InvalidHunkError {
                message: format!(
                    "Expected update hunk to start with a @@ context marker, got: '{}'",
                    lines[0]
                ),
                line_number,
            });
        }
        (None, 0)
    };
    if start_index >= lines.len() {
        return Err(InvalidHunkError {
            message: "Update hunk does not contain any lines".to_string(),
            line_number: line_number + 1,
        });
    }
    let mut chunk = UpdateFileChunk {
        change_context,
        old_lines: Vec::new(),
        new_lines: Vec::new(),
        is_end_of_file: false,
    };
    let mut first_old_line_number = None;
    let mut first_old_line_column = None;
    let mut first_removed_line_number = None;
    let mut first_removed_line_column = None;
    let mut parsed_lines = 0;
    for line in &lines[start_index..] {
        let current_line_number = line_number + start_index + parsed_lines;
        match *line {
            EOF_MARKER => {
                if parsed_lines == 0 {
                    return Err(InvalidHunkError {
                        message: "Update hunk does not contain any lines".to_string(),
                        line_number: line_number + 1,
                    });
                }
                chunk.is_end_of_file = true;
                parsed_lines += 1;
                break;
            }
            line_contents => {
                match line_contents.chars().next() {
                    None => {
                        // Interpret this as an empty line.
                        if first_old_line_number.is_none() {
                            first_old_line_number = Some(current_line_number);
                            first_old_line_column = Some(1);
                        }
                        chunk.old_lines.push(String::new());
                        chunk.new_lines.push(String::new());
                    }
                    Some(' ') => {
                        if first_old_line_number.is_none() {
                            first_old_line_number = Some(current_line_number);
                            first_old_line_column = Some(1);
                        }
                        chunk.old_lines.push(line_contents[1..].to_string());
                        chunk.new_lines.push(line_contents[1..].to_string());
                    }
                    Some('+') => {
                        chunk.new_lines.push(line_contents[1..].to_string());
                    }
                    Some('-') => {
                        if first_old_line_number.is_none() {
                            first_old_line_number = Some(current_line_number);
                            first_old_line_column = Some(1);
                        }
                        if first_removed_line_number.is_none() {
                            first_removed_line_number = Some(current_line_number);
                            first_removed_line_column = Some(1);
                        }
                        chunk.old_lines.push(line_contents[1..].to_string());
                    }
                    _ => {
                        if parsed_lines == 0 {
                            return Err(InvalidHunkError {
                                message: format!(
                                    "Unexpected line found in update hunk: '{line_contents}'. Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)"
                                ),
                                line_number: line_number + 1,
                            });
                        }
                        // Assume this is the start of the next hunk.
                        break;
                    }
                }
                parsed_lines += 1;
            }
        }
    }

    Ok((
        chunk,
        PatchChunkSource {
            line_number,
            column_number: 1,
            first_old_line_number,
            first_old_line_column,
            first_removed_line_number,
            first_removed_line_column,
        },
        parsed_lines + start_index,
    ))
}

#[test]
fn test_parse_patch() {
    assert_eq!(
        parse_patch_text("bad", ParseMode::Strict),
        Err(InvalidPatchError(
            "The first line of the patch must be '*** Begin Patch'".to_string()
        ))
    );
    assert_eq!(
        parse_patch_text("*** Begin Patch\nbad", ParseMode::Strict),
        Err(InvalidPatchError(
            "The last line of the patch must be '*** End Patch'".to_string()
        ))
    );

    assert_eq!(
        parse_patch_text(
            concat!(
                "*** Begin Patch",
                " ",
                "\n*** Add File: foo\n+hi\n",
                " ",
                "*** End Patch"
            ),
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![AddFile {
            path: PathBuf::from("foo"),
            contents: "hi\n".to_string()
        }]
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Update File: test.py\n\
             *** End Patch",
            ParseMode::Strict
        ),
        Err(InvalidHunkError {
            message: "Update file hunk for path 'test.py' is empty".to_string(),
            line_number: 2,
        })
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        Vec::new()
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Add File: empty.txt\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![AddFile {
            path: PathBuf::from("empty.txt"),
            contents: String::new()
        }]
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Add File: path/add.py\n\
             +abc\n\
             +def\n\
             *** Delete File: path/delete.py\n\
             *** Update File: path/update.py\n\
             *** Move to: path/update2.py\n\
             @@ def f():\n\
             -    pass\n\
             +    return 123\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![
            AddFile {
                path: PathBuf::from("path/add.py"),
                contents: "abc\ndef\n".to_string()
            },
            DeleteFile {
                path: PathBuf::from("path/delete.py")
            },
            UpdateFile {
                path: PathBuf::from("path/update.py"),
                move_path: Some(PathBuf::from("path/update2.py")),
                chunks: vec![UpdateFileChunk {
                    change_context: Some("def f():".to_string()),
                    old_lines: vec!["    pass".to_string()],
                    new_lines: vec!["    return 123".to_string()],
                    is_end_of_file: false
                }]
            }
        ]
    );
    // Update hunk followed by another hunk (Add File).
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Update File: file.py\n\
             @@\n\
             +line\n\
             *** Add File: other.py\n\
             +content\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![
            UpdateFile {
                path: PathBuf::from("file.py"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: vec![],
                    new_lines: vec!["line".to_string()],
                    is_end_of_file: false
                }],
            },
            AddFile {
                path: PathBuf::from("other.py"),
                contents: "content\n".to_string()
            }
        ]
    );

    // Update hunk without an explicit @@ header for the first chunk should parse.
    // Use a raw string to preserve the leading space diff marker on the context line.
    assert_eq!(
        parse_patch_text(
            r#"*** Begin Patch
*** Update File: file2.py
 import foo
+bar
*** End Patch"#,
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![UpdateFile {
            path: PathBuf::from("file2.py"),
            move_path: None,
            chunks: vec![UpdateFileChunk {
                change_context: None,
                old_lines: vec!["import foo".to_string()],
                new_lines: vec!["import foo".to_string(), "bar".to_string()],
                is_end_of_file: false,
            }],
        }]
    );
}

#[test]
fn test_parse_patch_lenient() {
    let patch_text = r#"*** Begin Patch
*** Update File: file2.py
 import foo
+bar
*** End Patch"#;
    let expected_patch = vec![UpdateFile {
        path: PathBuf::from("file2.py"),
        move_path: None,
        chunks: vec![UpdateFileChunk {
            change_context: None,
            old_lines: vec!["import foo".to_string()],
            new_lines: vec!["import foo".to_string(), "bar".to_string()],
            is_end_of_file: false,
        }],
    }];
    let expected_error =
        InvalidPatchError("The first line of the patch must be '*** Begin Patch'".to_string());

    let patch_text_in_heredoc = format!("<<EOF\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_heredoc, ParseMode::Lenient),
        Ok(ApplyPatchArgs {
            hunks: expected_patch.clone(),
            patch: patch_text.to_string(),
            workdir: None,
        })
    );

    let patch_text_in_single_quoted_heredoc = format!("<<'EOF'\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_single_quoted_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_single_quoted_heredoc, ParseMode::Lenient),
        Ok(ApplyPatchArgs {
            hunks: expected_patch.clone(),
            patch: patch_text.to_string(),
            workdir: None,
        })
    );

    let patch_text_in_double_quoted_heredoc = format!("<<\"EOF\"\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_double_quoted_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_double_quoted_heredoc, ParseMode::Lenient),
        Ok(ApplyPatchArgs {
            hunks: expected_patch,
            patch: patch_text.to_string(),
            workdir: None,
        })
    );

    let patch_text_in_mismatched_quotes_heredoc = format!("<<\"EOF'\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_mismatched_quotes_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_mismatched_quotes_heredoc, ParseMode::Lenient),
        Err(expected_error.clone())
    );

    let patch_text_with_missing_closing_heredoc =
        "<<EOF\n*** Begin Patch\n*** Update File: file2.py\nEOF\n".to_string();
    assert_eq!(
        parse_patch_text(&patch_text_with_missing_closing_heredoc, ParseMode::Strict),
        Err(expected_error)
    );
    assert_eq!(
        parse_patch_text(&patch_text_with_missing_closing_heredoc, ParseMode::Lenient),
        Err(InvalidPatchError(
            "The last line of the patch must be '*** End Patch'".to_string()
        ))
    );
}

#[test]
fn test_parse_one_hunk() {
    assert_eq!(
        parse_one_hunk(&["bad"], 234),
        Err(InvalidHunkError {
            message: "'bad' is not a valid hunk header. \
            Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'".to_string(),
            line_number: 234
        })
    );
    // Other edge cases are already covered by tests above/below.
}

#[test]
fn test_parse_patch_with_source_map_records_move_line() {
    let (_args, source_map) = parse_patch_with_source_map(
        "*** Begin Patch\n*** Update File: src/app.rs\n*** Move to: src/renamed.rs\n@@\n-old\n+new\n*** End Patch",
    )
    .unwrap();

    assert_eq!(source_map.actions.len(), 1);
    assert_eq!(source_map.actions[0].line_number, 2);
    assert_eq!(source_map.actions[0].move_line_number, Some(3));
    assert_eq!(source_map.actions[0].move_column_number, Some(1));
}

#[test]
fn test_update_file_chunk() {
    assert_eq!(
        parse_update_file_chunk(&["bad"], 123, false),
        Err(InvalidHunkError {
            message: "Expected update hunk to start with a @@ context marker, got: 'bad'"
                .to_string(),
            line_number: 123
        })
    );
    assert_eq!(
        parse_update_file_chunk(&["@@"], 123, false),
        Err(InvalidHunkError {
            message: "Update hunk does not contain any lines".to_string(),
            line_number: 124
        })
    );
    assert_eq!(
        parse_update_file_chunk(&["@@", "bad"], 123, false),
        Err(InvalidHunkError {
            message:  "Unexpected line found in update hunk: 'bad'. \
                       Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)".to_string(),
            line_number: 124
        })
    );
    assert_eq!(
        parse_update_file_chunk(&["@@", "*** End of File"], 123, false),
        Err(InvalidHunkError {
            message: "Update hunk does not contain any lines".to_string(),
            line_number: 124
        })
    );
    assert_eq!(
        parse_update_file_chunk(
            &[
                "@@ change_context",
                "",
                " context",
                "-remove",
                "+add",
                " context2",
                "*** End Patch",
            ],
            123,
            false
        ),
        Ok((
            UpdateFileChunk {
                change_context: Some("change_context".to_string()),
                old_lines: vec![
                    "".to_string(),
                    "context".to_string(),
                    "remove".to_string(),
                    "context2".to_string()
                ],
                new_lines: vec![
                    "".to_string(),
                    "context".to_string(),
                    "add".to_string(),
                    "context2".to_string()
                ],
                is_end_of_file: false
            },
            PatchChunkSource {
                line_number: 123,
                column_number: 1,
                first_old_line_number: Some(124),
                first_old_line_column: Some(1),
                first_removed_line_number: Some(126),
                first_removed_line_column: Some(1),
            },
            6
        ))
    );
    assert_eq!(
        parse_update_file_chunk(
            &[
                "@@ 120 | def handler():",
                " 121 |     value = 1",
                "-122 |     return value",
                "+    value = 2",
                " 123 | ",
                "*** End Patch",
            ],
            123,
            false
        ),
        Ok((
            UpdateFileChunk {
                change_context: Some("120 | def handler():".to_string()),
                old_lines: vec![
                    "121 |     value = 1".to_string(),
                    "122 |     return value".to_string(),
                    "123 | ".to_string(),
                ],
                new_lines: vec![
                    "121 |     value = 1".to_string(),
                    "    value = 2".to_string(),
                    "123 | ".to_string(),
                ],
                is_end_of_file: false
            },
            PatchChunkSource {
                line_number: 123,
                column_number: 1,
                first_old_line_number: Some(124),
                first_old_line_column: Some(1),
                first_removed_line_number: Some(125),
                first_removed_line_column: Some(1),
            },
            5
        ))
    );
    assert_eq!(
        parse_update_file_chunk(&["@@ 01 | def handler():", "-old", "+new"], 123, false),
        Err(InvalidHunkError {
            message:
                "line-number-assisted evidence must use a positive integer without leading zeroes"
                    .to_string(),
            line_number: 123
        })
    );
    assert_eq!(
        parse_update_file_chunk(&["@@", "+121 | added"], 123, false),
        Ok((
            UpdateFileChunk {
                change_context: None,
                old_lines: vec![],
                new_lines: vec!["121 | added".to_string()],
                is_end_of_file: false
            },
            PatchChunkSource {
                line_number: 123,
                column_number: 1,
                first_old_line_number: None,
                first_old_line_column: None,
                first_removed_line_number: None,
                first_removed_line_column: None,
            },
            2
        ))
    );
    assert_eq!(
        parse_update_file_chunk(&["@@", "-0 | old", "+new"], 123, false),
        Ok((
            UpdateFileChunk {
                change_context: None,
                old_lines: vec!["0 | old".to_string()],
                new_lines: vec!["new".to_string()],
                is_end_of_file: false
            },
            PatchChunkSource {
                line_number: 123,
                column_number: 1,
                first_old_line_number: Some(124),
                first_old_line_column: Some(1),
                first_removed_line_number: Some(124),
                first_removed_line_column: Some(1),
            },
            3
        ))
    );
    assert_eq!(
        parse_update_file_chunk(&["@@", " 12| old", "+new"], 123, false),
        Ok((
            UpdateFileChunk {
                change_context: None,
                old_lines: vec!["12| old".to_string()],
                new_lines: vec!["12| old".to_string(), "new".to_string()],
                is_end_of_file: false
            },
            PatchChunkSource {
                line_number: 123,
                column_number: 1,
                first_old_line_number: Some(124),
                first_old_line_column: Some(1),
                first_removed_line_number: None,
                first_removed_line_column: None,
            },
            3
        ))
    );
    assert_eq!(
        parse_update_file_chunk(&["@@", "+line", "*** End of File"], 123, false),
        Ok((
            UpdateFileChunk {
                change_context: None,
                old_lines: vec![],
                new_lines: vec!["line".to_string()],
                is_end_of_file: true
            },
            PatchChunkSource {
                line_number: 123,
                column_number: 1,
                first_old_line_number: None,
                first_old_line_column: None,
                first_removed_line_number: None,
                first_removed_line_column: None,
            },
            3
        ))
    );
}
