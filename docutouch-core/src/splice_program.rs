use crate::splice_selection::{
    SelectionBlock, SelectionDiagnosticCode, SelectionParseError, SelectionSide,
    parse_selection_block_with_locations,
};
use std::fmt;

const BEGIN_SPLICE: &str = "*** Begin Splice";
const END_SPLICE: &str = "*** End Splice";
const COPY_FROM_FILE: &str = "*** Copy From File: ";
const MOVE_FROM_FILE: &str = "*** Move From File: ";
const DELETE_SPAN_FROM_FILE: &str = "*** Delete Span From File: ";
const APPEND_TO_FILE: &str = "*** Append To File: ";
const INSERT_BEFORE_IN_FILE: &str = "*** Insert Before In File: ";
const INSERT_AFTER_IN_FILE: &str = "*** Insert After In File: ";
const REPLACE_IN_FILE: &str = "*** Replace In File: ";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceProgram {
    pub actions: Vec<SpliceAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpliceAction {
    Transfer(TransferAction),
    Delete(DeleteAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferSourceKind {
    Copy,
    Move,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferAction {
    pub authored_header_line: usize,
    pub source_kind: TransferSourceKind,
    pub source_path: String,
    pub source_selection: SelectionBlock,
    pub target: TargetAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteAction {
    pub authored_header_line: usize,
    pub source_path: String,
    pub source_selection: SelectionBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetAction {
    Append {
        path: String,
    },
    InsertBefore {
        path: String,
        selection: SelectionBlock,
    },
    InsertAfter {
        path: String,
        selection: SelectionBlock,
    },
    Replace {
        path: String,
        selection: SelectionBlock,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceProgramParseError {
    code: String,
    message: String,
    source_line: Option<usize>,
    source_column: Option<usize>,
}

impl SpliceProgramParseError {
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

    fn with_selection_error(
        context: &str,
        selection_header_line: usize,
        error: SelectionParseError,
    ) -> Self {
        let code = match error.code() {
            SelectionDiagnosticCode::Truncated => "SPLICE_SELECTION_TRUNCATED",
            SelectionDiagnosticCode::Invalid if context.contains("source") => {
                "SPLICE_SOURCE_SELECTION_INVALID"
            }
            SelectionDiagnosticCode::Invalid => "SPLICE_TARGET_SELECTION_INVALID",
        };
        let source_line = error
            .item_index()
            .map(|item_index| selection_header_line + 1 + item_index)
            .unwrap_or(selection_header_line);
        Self::at_line(code, source_line, format!("{context}: {}", error.message()))
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn source_line(&self) -> Option<usize> {
        self.source_line
    }

    pub fn source_column(&self) -> Option<usize> {
        self.source_column
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SpliceProgramParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SpliceProgramParseError {}

pub fn parse_splice_program(input: &str) -> Result<SpliceProgram, SpliceProgramParseError> {
    let lines = input.lines().collect::<Vec<_>>();
    if lines.first().copied() != Some(BEGIN_SPLICE) {
        return Err(SpliceProgramParseError::at_line(
            "SPLICE_PROGRAM_INVALID",
            1,
            "splice program must begin with `*** Begin Splice`",
        ));
    }

    let mut cursor = 1usize;
    let mut actions = Vec::new();
    while cursor < lines.len() {
        if lines[cursor] == END_SPLICE {
            cursor += 1;
            if cursor != lines.len() {
                return Err(SpliceProgramParseError::at_line(
                    "SPLICE_PROGRAM_INVALID",
                    cursor + 1,
                    "splice program must not contain trailing content after `*** End Splice`",
                ));
            }
            if actions.is_empty() {
                return Err(SpliceProgramParseError::at_line(
                    "SPLICE_PROGRAM_INVALID",
                    1,
                    "splice program must contain at least one action",
                ));
            }
            return Ok(SpliceProgram { actions });
        }

        let (action, next_cursor) = parse_action(&lines, cursor)?;
        actions.push(action);
        cursor = next_cursor;
    }

    Err(SpliceProgramParseError::new(
        "SPLICE_PROGRAM_INVALID",
        "splice program must end with `*** End Splice`",
    ))
}

pub fn extract_splice_paths(
    input: &str,
) -> Result<Vec<std::path::PathBuf>, SpliceProgramParseError> {
    let program = parse_splice_program(input)?;
    let mut paths = Vec::new();
    for action in program.actions {
        match action {
            SpliceAction::Delete(action) => paths.push(action.source_path.into()),
            SpliceAction::Transfer(action) => {
                paths.push(action.source_path.into());
                match action.target {
                    TargetAction::Append { path }
                    | TargetAction::InsertBefore { path, .. }
                    | TargetAction::InsertAfter { path, .. }
                    | TargetAction::Replace { path, .. } => paths.push(path.into()),
                }
            }
        }
    }
    Ok(paths)
}

fn parse_action(
    lines: &[&str],
    start: usize,
) -> Result<(SpliceAction, usize), SpliceProgramParseError> {
    let header = lines[start];
    if let Some(path) = header.strip_prefix(COPY_FROM_FILE) {
        return parse_transfer_action(lines, start, TransferSourceKind::Copy, path.to_string());
    }
    if let Some(path) = header.strip_prefix(MOVE_FROM_FILE) {
        return parse_transfer_action(lines, start, TransferSourceKind::Move, path.to_string());
    }
    if let Some(path) = header.strip_prefix(DELETE_SPAN_FROM_FILE) {
        let (selection, next_cursor) = parse_selection_after_header(
            lines,
            start + 1,
            SelectionSide::Source,
            "invalid source selection",
        )?;
        return Ok((
            SpliceAction::Delete(DeleteAction {
                authored_header_line: start + 1,
                source_path: path.to_string(),
                source_selection: selection,
            }),
            next_cursor,
        ));
    }

    Err(SpliceProgramParseError::at_line(
        "SPLICE_PROGRAM_INVALID",
        start + 1,
        format!("unsupported splice action header: {header}"),
    ))
}

fn parse_transfer_action(
    lines: &[&str],
    header_index: usize,
    source_kind: TransferSourceKind,
    source_path: String,
) -> Result<(SpliceAction, usize), SpliceProgramParseError> {
    let (source_selection, mut next_cursor) = parse_selection_after_header(
        lines,
        header_index + 1,
        SelectionSide::Source,
        "invalid source selection",
    )?;

    let target_header = lines.get(next_cursor).copied().ok_or_else(|| {
        SpliceProgramParseError::at_line(
            "SPLICE_PROGRAM_INVALID",
            header_index + 1,
            "transfer action is missing a target clause",
        )
    })?;

    let target = if let Some(path) = target_header.strip_prefix(APPEND_TO_FILE) {
        next_cursor += 1;
        TargetAction::Append {
            path: path.to_string(),
        }
    } else if let Some(path) = target_header.strip_prefix(INSERT_BEFORE_IN_FILE) {
        let (selection, cursor_after_target) = parse_selection_after_header(
            lines,
            next_cursor + 1,
            SelectionSide::Target,
            "invalid target selection",
        )?;
        next_cursor = cursor_after_target;
        TargetAction::InsertBefore {
            path: path.to_string(),
            selection,
        }
    } else if let Some(path) = target_header.strip_prefix(INSERT_AFTER_IN_FILE) {
        let (selection, cursor_after_target) = parse_selection_after_header(
            lines,
            next_cursor + 1,
            SelectionSide::Target,
            "invalid target selection",
        )?;
        next_cursor = cursor_after_target;
        TargetAction::InsertAfter {
            path: path.to_string(),
            selection,
        }
    } else if let Some(path) = target_header.strip_prefix(REPLACE_IN_FILE) {
        let (selection, cursor_after_target) = parse_selection_after_header(
            lines,
            next_cursor + 1,
            SelectionSide::Target,
            "invalid target selection",
        )?;
        next_cursor = cursor_after_target;
        TargetAction::Replace {
            path: path.to_string(),
            selection,
        }
    } else {
        return Err(SpliceProgramParseError::at_line(
            "SPLICE_PROGRAM_INVALID",
            next_cursor + 1,
            "transfer action must end with a supported target header",
        ));
    };

    Ok((
        SpliceAction::Transfer(TransferAction {
            authored_header_line: header_index + 1,
            source_kind,
            source_path,
            source_selection,
            target,
        }),
        next_cursor,
    ))
}

fn parse_selection_after_header(
    lines: &[&str],
    cursor: usize,
    side: SelectionSide,
    context: &str,
) -> Result<(SelectionBlock, usize), SpliceProgramParseError> {
    if lines.get(cursor).copied() != Some("@@") {
        return Err(SpliceProgramParseError::at_line(
            if context.contains("source") {
                "SPLICE_SOURCE_SELECTION_INVALID"
            } else {
                "SPLICE_TARGET_SELECTION_INVALID"
            },
            cursor + 1,
            format!("{context}: selection must begin with `@@`"),
        ));
    }
    let mut body_end = cursor + 1;
    while body_end < lines.len() && !is_control_line(lines[body_end]) {
        body_end += 1;
    }
    let body_lines = &lines[cursor + 1..body_end];
    let selection = parse_selection_block_with_locations(side, body_lines, Some(cursor + 2))
        .map_err(|error| {
            SpliceProgramParseError::with_selection_error(context, cursor + 1, error)
        })?;
    Ok((selection, body_end))
}

fn is_control_line(line: &str) -> bool {
    line == END_SPLICE || line.starts_with("*** ")
}
