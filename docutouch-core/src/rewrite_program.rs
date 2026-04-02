use crate::splice_selection::{
    SelectionBlock, SelectionParseError, SelectionSide, parse_selection_block_with_locations,
};
use std::fmt;
use std::path::PathBuf;

const BEGIN_REWRITE: &str = "*** Begin Rewrite";
const END_REWRITE: &str = "*** End Rewrite";
const ADD_FILE: &str = "*** Add File: ";
const DELETE_FILE: &str = "*** Delete File: ";
const UPDATE_FILE: &str = "*** Update File: ";
const MOVE_TO: &str = "*** Move to: ";
const WITH: &str = "*** With";
const END_WITH: &str = "*** End With";
const DELETE: &str = "*** Delete";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteProgram {
    pub operations: Vec<RewriteFileOperation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteFileOperation {
    Add(AddFileOperation),
    Delete(DeleteFileOperation),
    Update(UpdateFileOperation),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddFileOperation {
    pub authored_header_line: usize,
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteFileOperation {
    pub authored_header_line: usize,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateFileOperation {
    pub authored_header_line: usize,
    pub path: String,
    pub move_path: Option<String>,
    pub actions: Vec<RewriteAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteAction {
    pub authored_selection_line: usize,
    pub selection_intent_comment: Option<String>,
    pub selection: SelectionBlock,
    pub replacement_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteProgramParseError {
    code: String,
    message: String,
    source_line: Option<usize>,
    source_column: Option<usize>,
}

impl RewriteProgramParseError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source_line: None,
            source_column: Some(1),
        }
    }

    fn at_line(code: impl Into<String>, source_line: usize, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source_line: Some(source_line),
            source_column: Some(1),
        }
    }

    fn with_selection_error(selection_header_line: usize, error: SelectionParseError) -> Self {
        let source_line = error
            .item_index()
            .map(|item_index| selection_header_line + 1 + item_index)
            .unwrap_or(selection_header_line);
        Self::at_line(
            error.code().rewrite_code(),
            source_line,
            format!("invalid rewrite selection: {}", error.message()),
        )
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn source_line(&self) -> Option<usize> {
        self.source_line
    }

    pub fn source_column(&self) -> Option<usize> {
        self.source_column
    }
}

impl fmt::Display for RewriteProgramParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RewriteProgramParseError {}

pub fn parse_rewrite_program(input: &str) -> Result<RewriteProgram, RewriteProgramParseError> {
    let lines = input.lines().collect::<Vec<_>>();
    if lines.first().copied() != Some(BEGIN_REWRITE) {
        return Err(RewriteProgramParseError::at_line(
            "REWRITE_PROGRAM_INVALID",
            1,
            "rewrite program must begin with `*** Begin Rewrite`",
        ));
    }

    let mut cursor = 1usize;
    let mut operations = Vec::new();
    while cursor < lines.len() {
        if lines[cursor] == END_REWRITE {
            cursor += 1;
            if cursor != lines.len() {
                return Err(RewriteProgramParseError::at_line(
                    "REWRITE_PROGRAM_INVALID",
                    cursor + 1,
                    "rewrite program must not contain trailing content after `*** End Rewrite`",
                ));
            }
            if operations.is_empty() {
                return Err(RewriteProgramParseError::at_line(
                    "REWRITE_PROGRAM_INVALID",
                    1,
                    "rewrite program must contain at least one file operation",
                ));
            }
            return Ok(RewriteProgram { operations });
        }

        let (operation, next_cursor) = parse_file_operation(&lines, cursor)?;
        operations.push(operation);
        cursor = next_cursor;
    }

    Err(RewriteProgramParseError::new(
        "REWRITE_PROGRAM_INVALID",
        "rewrite program must end with `*** End Rewrite`",
    ))
}

pub fn extract_rewrite_paths(input: &str) -> Result<Vec<PathBuf>, RewriteProgramParseError> {
    let program = parse_rewrite_program(input)?;
    let mut paths = Vec::new();
    for operation in program.operations {
        match operation {
            RewriteFileOperation::Add(operation) => paths.push(operation.path.into()),
            RewriteFileOperation::Delete(operation) => paths.push(operation.path.into()),
            RewriteFileOperation::Update(operation) => {
                paths.push(operation.path.into());
                if let Some(move_path) = operation.move_path {
                    paths.push(move_path.into());
                }
            }
        }
    }
    Ok(paths)
}

fn parse_file_operation(
    lines: &[&str],
    start: usize,
) -> Result<(RewriteFileOperation, usize), RewriteProgramParseError> {
    let header = lines[start];
    if let Some(path) = header.strip_prefix(ADD_FILE) {
        let (content, next_cursor) = parse_with_block(lines, start + 1)?;
        return Ok((
            RewriteFileOperation::Add(AddFileOperation {
                authored_header_line: start + 1,
                path: path.to_string(),
                content,
            }),
            next_cursor,
        ));
    }
    if let Some(path) = header.strip_prefix(DELETE_FILE) {
        return Ok((
            RewriteFileOperation::Delete(DeleteFileOperation {
                authored_header_line: start + 1,
                path: path.to_string(),
            }),
            start + 1,
        ));
    }
    if let Some(path) = header.strip_prefix(UPDATE_FILE) {
        return parse_update_operation(lines, start, path.to_string());
    }

    Err(RewriteProgramParseError::at_line(
        "REWRITE_PROGRAM_INVALID",
        start + 1,
        format!("unsupported rewrite file operation header: {header}"),
    ))
}

fn parse_update_operation(
    lines: &[&str],
    header_index: usize,
    path: String,
) -> Result<(RewriteFileOperation, usize), RewriteProgramParseError> {
    let mut cursor = header_index + 1;
    let move_path = if let Some(line) = lines.get(cursor) {
        if let Some(move_path) = line.strip_prefix(MOVE_TO) {
            cursor += 1;
            Some(move_path.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let mut actions = Vec::new();
    while cursor < lines.len() {
        if is_file_operation_header(lines[cursor]) || lines[cursor] == END_REWRITE {
            break;
        }
        let (action, next_cursor) = parse_rewrite_action(lines, cursor)?;
        actions.push(action);
        cursor = next_cursor;
    }

    if actions.is_empty() {
        return Err(RewriteProgramParseError::at_line(
            "REWRITE_PROGRAM_INVALID",
            header_index + 1,
            "update operation must contain at least one rewrite action",
        ));
    }

    Ok((
        RewriteFileOperation::Update(UpdateFileOperation {
            authored_header_line: header_index + 1,
            path,
            move_path,
            actions,
        }),
        cursor,
    ))
}

fn parse_rewrite_action(
    lines: &[&str],
    cursor: usize,
) -> Result<(RewriteAction, usize), RewriteProgramParseError> {
    let selection_header = lines.get(cursor).copied().ok_or_else(|| {
        RewriteProgramParseError::at_line(
            "REWRITE_SELECTION_INVALID",
            cursor + 1,
            "rewrite action must begin with `@@`",
        )
    })?;
    let selection_intent_comment = if selection_header == "@@" {
        None
    } else if let Some(comment) = selection_header.strip_prefix("@@ ") {
        Some(comment.to_string())
    } else {
        return Err(RewriteProgramParseError::at_line(
            "REWRITE_SELECTION_INVALID",
            cursor + 1,
            "rewrite action must begin with `@@`",
        ));
    };
    let mut body_end = cursor + 1;
    while body_end < lines.len() && !is_selection_terminator(lines[body_end]) {
        body_end += 1;
    }
    let selection = parse_selection_block_with_locations(
        SelectionSide::Rewrite,
        &lines[cursor + 1..body_end],
        Some(cursor + 2),
    )
    .map_err(|error| RewriteProgramParseError::with_selection_error(cursor + 1, error))?;

    let Some(terminator) = lines.get(body_end).copied() else {
        return Err(RewriteProgramParseError::at_line(
            "REWRITE_PROGRAM_INVALID",
            cursor + 1,
            "rewrite action must end with `*** Delete` or a `*** With` block",
        ));
    };
    if terminator == DELETE {
        return Ok((
            RewriteAction {
                authored_selection_line: cursor + 1,
                selection_intent_comment,
                selection,
                replacement_text: None,
            },
            body_end + 1,
        ));
    }
    if terminator == WITH {
        let (replacement_text, next_cursor) = parse_with_block(lines, body_end)?;
        return Ok((
            RewriteAction {
                authored_selection_line: cursor + 1,
                selection_intent_comment,
                selection,
                replacement_text: Some(replacement_text),
            },
            next_cursor,
        ));
    }

    Err(RewriteProgramParseError::at_line(
        "REWRITE_PROGRAM_INVALID",
        body_end + 1,
        "rewrite action must end with `*** Delete` or a `*** With` block",
    ))
}

fn parse_with_block(
    lines: &[&str],
    start: usize,
) -> Result<(String, usize), RewriteProgramParseError> {
    if lines.get(start).copied() != Some(WITH) {
        return Err(RewriteProgramParseError::at_line(
            "REWRITE_PROGRAM_INVALID",
            start + 1,
            "expected `*** With`",
        ));
    }
    let mut cursor = start + 1;
    let mut payload_lines = Vec::new();
    while cursor < lines.len() {
        if lines[cursor] == END_WITH {
            let payload = payload_lines.join("\n");
            return Ok((payload, cursor + 1));
        }
        payload_lines.push(lines[cursor]);
        cursor += 1;
    }
    Err(RewriteProgramParseError::at_line(
        "REWRITE_PROGRAM_INVALID",
        start + 1,
        "with block must end with `*** End With`",
    ))
}

fn is_selection_terminator(line: &str) -> bool {
    line == DELETE || line == WITH || is_file_operation_header(line) || line == END_REWRITE
}

fn is_file_operation_header(line: &str) -> bool {
    line.starts_with(ADD_FILE) || line.starts_with(DELETE_FILE) || line.starts_with(UPDATE_FILE)
}
