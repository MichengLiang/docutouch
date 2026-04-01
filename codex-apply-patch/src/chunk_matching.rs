use std::path::Path;

use crate::seek_sequence;
use crate::{ApplyPatchTargetAnchor, NumberedEvidenceMode, ReplaceMatchError, UpdateFileChunk};

#[derive(Debug, Clone, PartialEq, Eq)]
struct NumberedOldSideEvidence {
    line_number: usize,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedChunkLine {
    raw: String,
    visible: String,
    numbered: Option<NumberedOldSideEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedChunkLines {
    old_lines: Vec<ParsedChunkLine>,
    new_lines: Vec<String>,
}

fn parse_numbered_old_side_evidence(line: &str) -> Option<NumberedOldSideEvidence> {
    let (line_number, text) = line.split_once(" | ")?;
    if line_number.is_empty()
        || (line_number.len() > 1 && line_number.starts_with('0'))
        || !line_number.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let line_number = line_number.parse::<usize>().ok()?;
    if line_number == 0 {
        return None;
    }
    Some(NumberedOldSideEvidence {
        line_number,
        text: text.to_string(),
    })
}

fn parse_chunk_line(line: &str, mode: NumberedEvidenceMode) -> ParsedChunkLine {
    let numbered = match mode {
        NumberedEvidenceMode::HeaderOnly => None,
        NumberedEvidenceMode::Full => parse_numbered_old_side_evidence(line),
    };
    let visible = numbered
        .as_ref()
        .map(|evidence| evidence.text.clone())
        .unwrap_or_else(|| line.to_string());
    ParsedChunkLine {
        raw: line.to_string(),
        visible,
        numbered,
    }
}

fn chunk_lines_are_context_equivalent(old: &ParsedChunkLine, new: &ParsedChunkLine) -> bool {
    old.raw == new.raw
        || (old.numbered.is_some() && new.numbered.is_none() && old.visible == new.visible)
}

fn prepare_chunk_lines(
    chunk: &UpdateFileChunk,
    mode: NumberedEvidenceMode,
) -> Result<PreparedChunkLines, String> {
    let old_lines = chunk
        .old_lines
        .iter()
        .map(|line| parse_chunk_line(line, mode))
        .collect::<Vec<_>>();
    let new_lines = chunk
        .new_lines
        .iter()
        .map(|line| parse_chunk_line(line, mode))
        .collect::<Vec<_>>();

    let old_len = old_lines.len();
    let new_len = new_lines.len();
    let mut lcs = vec![vec![0usize; new_len + 1]; old_len + 1];
    for old_index in (0..old_len).rev() {
        for new_index in (0..new_len).rev() {
            lcs[old_index][new_index] =
                if chunk_lines_are_context_equivalent(&old_lines[old_index], &new_lines[new_index])
                {
                    lcs[old_index + 1][new_index + 1] + 1
                } else {
                    lcs[old_index + 1][new_index].max(lcs[old_index][new_index + 1])
                };
        }
    }

    let mut sanitized_new_lines = new_lines
        .iter()
        .map(|line| line.raw.clone())
        .collect::<Vec<_>>();
    let mut old_index = 0usize;
    let mut new_index = 0usize;
    while old_index < old_len && new_index < new_len {
        if chunk_lines_are_context_equivalent(&old_lines[old_index], &new_lines[new_index])
            && lcs[old_index][new_index] == lcs[old_index + 1][new_index + 1] + 1
        {
            if old_lines[old_index].numbered.is_some() || new_lines[new_index].numbered.is_some() {
                sanitized_new_lines[new_index] = old_lines[old_index].visible.clone();
            }
            old_index += 1;
            new_index += 1;
        } else if lcs[old_index + 1][new_index] >= lcs[old_index][new_index + 1] {
            old_index += 1;
        } else {
            new_index += 1;
        }
    }

    Ok(PreparedChunkLines {
        old_lines,
        new_lines: sanitized_new_lines,
    })
}

fn lines_match_authored_evidence(actual: &str, expected: &str) -> bool {
    if actual == expected
        || actual.trim_end() == expected.trim_end()
        || actual.trim() == expected.trim()
    {
        return true;
    }

    fn normalise(s: &str) -> String {
        s.trim()
            .chars()
            .map(|c| match c {
                '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
                | '\u{2212}' => '-',
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
                '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
                | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
                | '\u{3000}' => ' ',
                other => other,
            })
            .collect::<String>()
    }

    normalise(actual) == normalise(expected)
}

fn target_anchor_at_line(
    original_lines: &[String],
    path: &Path,
    line_number: usize,
    label: &str,
) -> Option<ApplyPatchTargetAnchor> {
    let index = line_number.checked_sub(1)?;
    Some(ApplyPatchTargetAnchor {
        path: path.to_path_buf(),
        line_number,
        column_number: 1,
        label: Some(label.to_string()),
        excerpt: original_lines.get(index).cloned(),
    })
}

fn match_numbered_context_anchor(
    original_lines: &[String],
    path: &Path,
    evidence: &NumberedOldSideEvidence,
    minimum_index: usize,
) -> Result<(usize, ApplyPatchTargetAnchor), ReplaceMatchError> {
    let target_index = evidence.line_number - 1;
    if target_index < minimum_index {
        return Err(ReplaceMatchError {
            message: format!(
                "Failed to match numbered context '{} | {}' in {}: line-numbered anchor precedes the current chunk order",
                evidence.line_number,
                evidence.text,
                path.display()
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: target_anchor_at_line(
                original_lines,
                path,
                evidence.line_number,
                "numbered context",
            ),
            blame_first_old_line: false,
        });
    }
    let Some(actual_line) = original_lines.get(target_index) else {
        return Err(ReplaceMatchError {
            message: format!(
                "Failed to match numbered context '{} | {}' in {}: line {} is outside the target file",
                evidence.line_number,
                evidence.text,
                path.display(),
                evidence.line_number
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: None,
            blame_first_old_line: false,
        });
    };

    if !lines_match_authored_evidence(actual_line, &evidence.text) {
        return Err(ReplaceMatchError {
            message: format!(
                "Failed to match numbered context '{} | {}' in {}",
                evidence.line_number,
                evidence.text,
                path.display()
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: target_anchor_at_line(
                original_lines,
                path,
                evidence.line_number,
                "numbered context mismatch",
            ),
            blame_first_old_line: false,
        });
    }

    Ok((
        target_index,
        ApplyPatchTargetAnchor {
            path: path.to_path_buf(),
            line_number: evidence.line_number,
            column_number: 1,
            label: Some("matched numbered context".to_string()),
            excerpt: Some(actual_line.clone()),
        },
    ))
}

fn find_numbered_old_side_match(
    original_lines: &[String],
    path: &Path,
    pattern: &[ParsedChunkLine],
    minimum_index: usize,
) -> Result<usize, ReplaceMatchError> {
    let numbered_lines = pattern
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| line.numbered.as_ref().map(|numbered| (offset, numbered)))
        .collect::<Vec<_>>();

    let Some((first_offset, first_line)) = numbered_lines.first().copied() else {
        return Err(ReplaceMatchError {
            message: format!(
                "internal error: numbered old-side match requested without numbered evidence in {}",
                path.display()
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: None,
            blame_first_old_line: true,
        });
    };

    if first_line.line_number <= first_offset {
        return Err(ReplaceMatchError {
            message: format!(
                "Failed to reconcile numbered old-side evidence in {}: line {} cannot appear at offset {} within one contiguous chunk",
                path.display(),
                first_line.line_number,
                first_offset + 1
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: None,
            blame_first_old_line: true,
        });
    }

    let start_index = first_line.line_number - 1 - first_offset;
    for (offset, numbered) in &numbered_lines[1..] {
        if numbered.line_number <= *offset || numbered.line_number - 1 - offset != start_index {
            return Err(ReplaceMatchError {
                message: format!(
                    "Failed to reconcile numbered old-side evidence in {}: numbered lines do not describe one contiguous original span",
                    path.display()
                ),
                chunk_index: 0,
                is_end_of_file: false,
                target_anchor: None,
                blame_first_old_line: true,
            });
        }
    }

    if start_index < minimum_index {
        return Err(ReplaceMatchError {
            message: format!(
                "Failed to match numbered old-side evidence in {}: numbered span precedes the current chunk order",
                path.display()
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: target_anchor_at_line(
                original_lines,
                path,
                first_line.line_number,
                "numbered old-side evidence",
            ),
            blame_first_old_line: true,
        });
    }

    if start_index + pattern.len() > original_lines.len() {
        return Err(ReplaceMatchError {
            message: format!(
                "Failed to match numbered old-side evidence in {}: numbered span exceeds the target file length",
                path.display()
            ),
            chunk_index: 0,
            is_end_of_file: false,
            target_anchor: target_anchor_at_line(
                original_lines,
                path,
                first_line.line_number,
                "numbered old-side evidence",
            ),
            blame_first_old_line: true,
        });
    }

    for (offset, line) in pattern.iter().enumerate() {
        let actual_index = start_index + offset;
        let actual_line = &original_lines[actual_index];
        if !lines_match_authored_evidence(actual_line, &line.visible) {
            return Err(ReplaceMatchError {
                message: format!(
                    "Failed to match numbered old-side evidence in {} at line {}:\n{}",
                    path.display(),
                    actual_index + 1,
                    line.visible
                ),
                chunk_index: 0,
                is_end_of_file: false,
                target_anchor: Some(ApplyPatchTargetAnchor {
                    path: path.to_path_buf(),
                    line_number: actual_index + 1,
                    column_number: 1,
                    label: Some("numbered evidence mismatch".to_string()),
                    excerpt: Some(actual_line.clone()),
                }),
                blame_first_old_line: true,
            });
        }
    }

    Ok(start_index)
}

fn pattern_matches_at_index(
    original_lines: &[String],
    pattern: &[ParsedChunkLine],
    start_index: usize,
) -> bool {
    if start_index + pattern.len() > original_lines.len() {
        return false;
    }

    pattern.iter().enumerate().all(|(offset, line)| {
        lines_match_authored_evidence(&original_lines[start_index + offset], &line.visible)
    })
}

fn exact_old_side_match_at_index<'a>(
    original_lines: &[String],
    pattern: &'a [ParsedChunkLine],
    new_slice: &'a [String],
    start_index: usize,
) -> Option<(&'a [ParsedChunkLine], &'a [String], usize)> {
    if pattern_matches_at_index(original_lines, pattern, start_index) {
        return Some((pattern, new_slice, start_index));
    }

    if pattern.last().is_some_and(|line| line.visible.is_empty()) {
        let trimmed_pattern = &pattern[..pattern.len() - 1];
        let trimmed_new_slice = if new_slice.last().is_some_and(String::is_empty) {
            &new_slice[..new_slice.len() - 1]
        } else {
            new_slice
        };
        if pattern_matches_at_index(original_lines, trimmed_pattern, start_index) {
            return Some((trimmed_pattern, trimmed_new_slice, start_index));
        }
    }

    None
}

fn expected_lines_match_error(
    path: &Path,
    pattern_text: &[String],
    chunk_index: usize,
    is_end_of_file: bool,
    matched_context_anchor: Option<ApplyPatchTargetAnchor>,
) -> ReplaceMatchError {
    ReplaceMatchError {
        message: format!(
            "Failed to find expected lines in {}:\n{}",
            path.display(),
            pattern_text.join("\n")
        ),
        chunk_index,
        is_end_of_file,
        target_anchor: matched_context_anchor,
        blame_first_old_line: true,
    }
}

pub(crate) fn compute_replacements(
    original_lines: &[String],
    path: &Path,
    chunks: &[UpdateFileChunk],
    mode: NumberedEvidenceMode,
) -> Result<Vec<(usize, usize, Vec<String>)>, ReplaceMatchError> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for (chunk_index, chunk) in chunks.iter().enumerate() {
        let prepared = prepare_chunk_lines(chunk, mode).map_err(|message| ReplaceMatchError {
            message: format!(
                "Failed to prepare numbered old-side evidence in {}: {message}",
                path.display()
            ),
            chunk_index,
            is_end_of_file: false,
            target_anchor: None,
            blame_first_old_line: false,
        })?;
        let mut matched_context_anchor = None;
        let mut matched_numbered_context = None;
        let mut matched_context_index = None;
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(numbered_context) = parse_numbered_old_side_evidence(ctx_line) {
                match match_numbered_context_anchor(
                    original_lines,
                    path,
                    &numbered_context,
                    line_index,
                ) {
                    Ok((idx, anchor)) => {
                        matched_numbered_context = Some(numbered_context);
                        matched_context_index = Some(idx);
                        line_index = idx + 1;
                        matched_context_anchor = Some(anchor);
                    }
                    Err(mut err) => {
                        err.chunk_index = chunk_index;
                        return Err(err);
                    }
                }
            } else if let Some(idx) = seek_sequence::seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                false,
            ) {
                line_index = idx + 1;
                matched_context_anchor = Some(ApplyPatchTargetAnchor {
                    path: path.to_path_buf(),
                    line_number: idx + 1,
                    column_number: 1,
                    label: Some("matched context".to_string()),
                    excerpt: original_lines.get(idx).cloned(),
                });
            } else {
                return Err(ReplaceMatchError {
                    message: format!(
                        "Failed to find context '{}' in {}",
                        ctx_line,
                        path.display()
                    ),
                    chunk_index,
                    is_end_of_file: false,
                    target_anchor: None,
                    blame_first_old_line: false,
                });
            }
        }

        if prepared.old_lines.is_empty() {
            let insertion_idx = if original_lines.last().is_some_and(String::is_empty) {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push((insertion_idx, 0, prepared.new_lines.clone()));
            continue;
        }

        let mut pattern: &[ParsedChunkLine] = &prepared.old_lines;
        let mut new_slice: &[String] = &prepared.new_lines;
        let mut pattern_text = pattern
            .iter()
            .map(|line| line.visible.clone())
            .collect::<Vec<_>>();
        let duplicate_first_old_side_start = matched_numbered_context
            .as_ref()
            .zip(matched_context_index)
            .and_then(|(numbered_context, context_index)| {
                pattern.first().and_then(|first_line| {
                    lines_match_authored_evidence(&numbered_context.text, &first_line.visible)
                        .then_some(context_index)
                })
            });
        let mut found = if let Some(start_index) = duplicate_first_old_side_start {
            Some(
                exact_old_side_match_at_index(original_lines, pattern, new_slice, start_index)
                    .map(
                        |(matched_pattern, matched_new_slice, matched_start_index)| {
                            pattern = matched_pattern;
                            new_slice = matched_new_slice;
                            matched_start_index
                        },
                    )
                    .ok_or_else(|| {
                        expected_lines_match_error(
                            path,
                            &pattern_text,
                            chunk_index,
                            chunk.is_end_of_file,
                            matched_context_anchor.clone(),
                        )
                    }),
            )
        } else if pattern.iter().any(|line| line.numbered.is_some()) {
            Some(find_numbered_old_side_match(
                original_lines,
                path,
                pattern,
                line_index,
            ))
        } else {
            Some(
                seek_sequence::seek_sequence(
                    original_lines,
                    &pattern_text,
                    line_index,
                    chunk.is_end_of_file,
                )
                .ok_or_else(|| {
                    expected_lines_match_error(
                        path,
                        &pattern_text,
                        chunk_index,
                        chunk.is_end_of_file,
                        matched_context_anchor.clone(),
                    )
                }),
            )
        };

        if duplicate_first_old_side_start.is_none()
            && found.as_ref().is_some_and(|result| result.is_err())
            && pattern.last().is_some_and(|line| line.visible.is_empty())
        {
            pattern = &pattern[..pattern.len() - 1];
            pattern_text = pattern
                .iter()
                .map(|line| line.visible.clone())
                .collect::<Vec<_>>();
            if new_slice.last().is_some_and(String::is_empty) {
                new_slice = &new_slice[..new_slice.len() - 1];
            }

            found = if pattern.iter().any(|line| line.numbered.is_some()) {
                Some(find_numbered_old_side_match(
                    original_lines,
                    path,
                    pattern,
                    line_index,
                ))
            } else {
                Some(
                    seek_sequence::seek_sequence(
                        original_lines,
                        &pattern_text,
                        line_index,
                        chunk.is_end_of_file,
                    )
                    .ok_or_else(|| {
                        expected_lines_match_error(
                            path,
                            &pattern_text,
                            chunk_index,
                            chunk.is_end_of_file,
                            matched_context_anchor.clone(),
                        )
                    }),
                )
            };
        }

        let start_idx = match found.expect("pattern match result should exist") {
            Ok(start_idx) => start_idx,
            Err(mut err) => {
                err.chunk_index = chunk_index;
                if err.target_anchor.is_none() {
                    err.target_anchor = matched_context_anchor;
                }
                return Err(err);
            }
        };
        replacements.push((start_idx, pattern.len(), new_slice.to_vec()));
        line_index = start_idx + pattern.len();
    }

    replacements.sort_by(|(lhs_idx, _, _), (rhs_idx, _, _)| lhs_idx.cmp(rhs_idx));

    Ok(replacements)
}
