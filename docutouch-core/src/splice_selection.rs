use std::fmt;

const SOURCE_OMISSION_TOKEN: &str = "... source lines omitted ...";
const TARGET_OMISSION_TOKEN: &str = "... target lines omitted ...";
const REWRITE_OMISSION_TOKEN: &str = "... lines omitted ...";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionSide {
    Source,
    Target,
    Rewrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionDiagnosticCode {
    Invalid,
    Truncated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionLine {
    pub line_number: usize,
    pub visible_content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionItem {
    Line(SelectionLine),
    Omission,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionBlock {
    pub side: SelectionSide,
    pub items: Vec<SelectionItem>,
    pub authored_header_line: Option<usize>,
    pub authored_item_lines: Vec<Option<usize>>,
}

impl SelectionBlock {
    pub fn first_numbered_authored_line(&self) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .find_map(|(index, item)| match item {
                SelectionItem::Line(_) => self.authored_item_lines.get(index).copied().flatten(),
                SelectionItem::Omission => None,
            })
    }

    pub fn authored_line_for_item_index(&self, item_index: usize) -> Option<usize> {
        self.authored_item_lines.get(item_index).copied().flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSelection {
    pub side: SelectionSide,
    pub start_line: usize,
    pub end_line: usize,
    pub lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedSelectionOffsets {
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SelectionResolutionRange {
    pub start_line: usize,
    pub end_line: usize,
    pub first_item_index: usize,
    pub last_item_index: usize,
}

#[derive(Clone, Copy, Debug)]
struct NumberedSelectionItem<'a> {
    item_index: usize,
    line: &'a SelectionLine,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionParseError {
    message: String,
    code: SelectionDiagnosticCode,
    item_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionResolveError {
    message: String,
    code: SelectionDiagnosticCode,
    item_index: Option<usize>,
}

impl SelectionDiagnosticCode {
    pub fn splice_code_for_side(self, side: SelectionSide) -> &'static str {
        match self {
            SelectionDiagnosticCode::Invalid => match side {
                SelectionSide::Source => "SPLICE_SOURCE_SELECTION_INVALID",
                SelectionSide::Target => "SPLICE_TARGET_SELECTION_INVALID",
                SelectionSide::Rewrite => "REWRITE_SELECTION_INVALID",
            },
            SelectionDiagnosticCode::Truncated => match side {
                SelectionSide::Source | SelectionSide::Target => "SPLICE_SELECTION_TRUNCATED",
                SelectionSide::Rewrite => "REWRITE_SELECTION_TRUNCATED",
            },
        }
    }

    pub fn rewrite_code(self) -> &'static str {
        match self {
            SelectionDiagnosticCode::Invalid => "REWRITE_SELECTION_INVALID",
            SelectionDiagnosticCode::Truncated => "REWRITE_SELECTION_TRUNCATED",
        }
    }
}

impl SelectionParseError {
    fn new(
        code: SelectionDiagnosticCode,
        message: impl Into<String>,
        item_index: Option<usize>,
    ) -> Self {
        Self {
            message: message.into(),
            code,
            item_index,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> SelectionDiagnosticCode {
        self.code
    }

    pub fn item_index(&self) -> Option<usize> {
        self.item_index
    }
}

impl SelectionResolveError {
    pub(crate) fn new(
        code: SelectionDiagnosticCode,
        message: impl Into<String>,
        item_index: Option<usize>,
    ) -> Self {
        Self {
            message: message.into(),
            code,
            item_index,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> SelectionDiagnosticCode {
        self.code
    }

    pub fn item_index(&self) -> Option<usize> {
        self.item_index
    }
}

impl fmt::Display for SelectionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SelectionParseError {}

impl fmt::Display for SelectionResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SelectionResolveError {}

pub fn parse_selection_block(
    side: SelectionSide,
    body_lines: &[&str],
) -> Result<SelectionBlock, SelectionParseError> {
    parse_selection_block_with_locations(side, body_lines, None)
}

pub(crate) fn parse_selection_block_with_locations(
    side: SelectionSide,
    body_lines: &[&str],
    body_start_line: Option<usize>,
) -> Result<SelectionBlock, SelectionParseError> {
    if body_lines.is_empty() {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection body must contain at least one numbered line",
            None,
        ));
    }

    let mut items = Vec::with_capacity(body_lines.len());
    let mut authored_item_lines = Vec::with_capacity(body_lines.len());
    let expected_omission = omission_token(side);
    for (item_index, raw_line) in body_lines.iter().enumerate() {
        authored_item_lines.push(body_start_line.map(|line| line + item_index));
        if *raw_line == expected_omission {
            items.push(SelectionItem::Omission);
            continue;
        }
        if omission_tokens()
            .iter()
            .copied()
            .any(|token| token != expected_omission && *raw_line == token)
        {
            return Err(SelectionParseError::new(
                SelectionDiagnosticCode::Truncated,
                format!(
                    "selection uses the wrong omission token for the {:?} side",
                    side
                ),
                Some(item_index),
            ));
        }
        if raw_line.starts_with("...") {
            return Err(SelectionParseError::new(
                SelectionDiagnosticCode::Truncated,
                "selection contains a malformed omission token",
                Some(item_index),
            ));
        }
        if contains_horizontal_truncation_marker(raw_line) {
            return Err(SelectionParseError::new(
                SelectionDiagnosticCode::Truncated,
                "selection contains a forbidden horizontal truncation marker",
                Some(item_index),
            ));
        }
        items.push(SelectionItem::Line(parse_selection_line(
            raw_line, item_index,
        )?));
    }

    validate_selection_items(&items)?;

    Ok(SelectionBlock {
        side,
        items,
        authored_header_line: body_start_line.map(|line| line.saturating_sub(1)),
        authored_item_lines,
    })
}

pub fn resolve_selection_block(
    block: &SelectionBlock,
    file_text: &str,
) -> Result<ResolvedSelection, SelectionResolveError> {
    let actual_lines = normalize_file_lines(file_text);
    let range = validate_selection_resolution(block, "target text", |line| {
        actual_lines
            .get(line.line_number.saturating_sub(1))
            .map(|actual| *actual == line.visible_content.as_str())
    })?;

    let resolved_lines = actual_lines
        .get(range.start_line - 1..range.end_line)
        .ok_or_else(|| {
            SelectionResolveError::new(
                SelectionDiagnosticCode::Invalid,
                format!(
                    "selection interval {}..={} extends past the target text",
                    range.start_line, range.end_line
                ),
                Some(range.first_item_index),
            )
        })?
        .iter()
        .map(|line| (*line).to_string())
        .collect::<Vec<_>>();

    Ok(ResolvedSelection {
        side: block.side,
        start_line: range.start_line,
        end_line: range.end_line,
        lines: resolved_lines,
    })
}

pub(crate) fn validate_selection_resolution<F>(
    block: &SelectionBlock,
    text_label: &str,
    mut matches_line: F,
) -> Result<SelectionResolutionRange, SelectionResolveError>
where
    F: FnMut(&SelectionLine) -> Option<bool>,
{
    let numbered_lines = numbered_selection_items(block)?;
    let first = numbered_lines[0];
    let last = *numbered_lines
        .last()
        .expect("numbered selection items must contain at least one line");

    for numbered in &numbered_lines {
        match matches_line(numbered.line) {
            Some(true) => {}
            Some(false) => {
                return Err(SelectionResolveError::new(
                    SelectionDiagnosticCode::Invalid,
                    selection_line_mismatch_message(numbered.line.line_number, text_label),
                    Some(numbered.item_index),
                ));
            }
            None => {
                return Err(SelectionResolveError::new(
                    SelectionDiagnosticCode::Invalid,
                    selection_line_missing_message(numbered.line.line_number, text_label),
                    Some(numbered.item_index),
                ));
            }
        }
    }

    Ok(SelectionResolutionRange {
        start_line: first.line.line_number,
        end_line: last.line.line_number,
        first_item_index: first.item_index,
        last_item_index: last.item_index,
    })
}

pub fn resolve_selection_offsets(
    block: &SelectionBlock,
    file_bytes: &[u8],
    text_label: &str,
) -> Result<ResolvedSelectionOffsets, SelectionResolveError> {
    let lines = split_file_lines(file_bytes);
    let range = validate_selection_resolution(block, text_label, |line| {
        lines
            .get(line.line_number.saturating_sub(1))
            .map(|actual| actual.body == line.visible_content.as_bytes())
    })?;

    let start_offset = lines
        .get(range.start_line - 1)
        .ok_or_else(|| {
            SelectionResolveError::new(
                SelectionDiagnosticCode::Invalid,
                format!(
                    "selection line {} does not exist in the {text_label}",
                    range.start_line
                ),
                Some(range.first_item_index),
            )
        })?
        .start;
    let end_offset = lines
        .get(range.end_line - 1)
        .ok_or_else(|| {
            SelectionResolveError::new(
                SelectionDiagnosticCode::Invalid,
                format!(
                    "selection line {} does not exist in the {text_label}",
                    range.end_line
                ),
                Some(range.last_item_index),
            )
        })?
        .end;

    Ok(ResolvedSelectionOffsets {
        start_offset,
        end_offset,
    })
}

fn parse_selection_line(
    raw_line: &str,
    item_index: usize,
) -> Result<SelectionLine, SelectionParseError> {
    let (number_text, visible_content) = raw_line.split_once(" | ").ok_or_else(|| {
        SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection line must use the exact `N | content` delimiter",
            Some(item_index),
        )
    })?;

    if number_text.is_empty() || !number_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection line number must be a positive integer",
            Some(item_index),
        ));
    }
    if number_text.starts_with('0') {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection line number must not use a leading zero",
            Some(item_index),
        ));
    }

    let line_number = number_text.parse::<usize>().map_err(|_| {
        SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection line number must be a valid positive integer",
            Some(item_index),
        )
    })?;
    if line_number == 0 {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection line number must be a positive integer",
            Some(item_index),
        ));
    }

    Ok(SelectionLine {
        line_number,
        visible_content: visible_content.to_string(),
    })
}

fn validate_selection_items(items: &[SelectionItem]) -> Result<(), SelectionParseError> {
    let line_count = items
        .iter()
        .filter(|item| matches!(item, SelectionItem::Line(_)))
        .count();
    if line_count == 0 {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Invalid,
            "selection body must contain at least one numbered line",
            None,
        ));
    }

    if matches!(items.first(), Some(SelectionItem::Omission)) {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Truncated,
            "omission token must be surrounded by numbered lines",
            Some(0),
        ));
    }
    if matches!(items.last(), Some(SelectionItem::Omission)) {
        return Err(SelectionParseError::new(
            SelectionDiagnosticCode::Truncated,
            "omission token must be surrounded by numbered lines",
            Some(items.len() - 1),
        ));
    }

    let mut previous_line_number = None;
    let mut previous_was_omission = false;
    let mut last_number_before_gap = None;
    for (item_index, item) in items.iter().enumerate() {
        match item {
            SelectionItem::Line(line) => {
                if let Some(previous) = previous_line_number {
                    if previous_was_omission {
                        if line.line_number <= previous + 1 {
                            return Err(SelectionParseError::new(
                                SelectionDiagnosticCode::Truncated,
                                "omission token must denote at least one omitted line",
                                Some(item_index),
                            ));
                        }
                    } else if line.line_number != previous + 1 {
                        return Err(SelectionParseError::new(
                            SelectionDiagnosticCode::Invalid,
                            "non-contiguous numbered lines require an omission token",
                            Some(item_index),
                        ));
                    }
                }
                if let Some(last_seen) = last_number_before_gap
                    && line.line_number <= last_seen
                {
                    return Err(SelectionParseError::new(
                        SelectionDiagnosticCode::Invalid,
                        "selection line numbers must be strictly increasing",
                        Some(item_index),
                    ));
                }
                previous_line_number = Some(line.line_number);
                previous_was_omission = false;
            }
            SelectionItem::Omission => {
                if previous_was_omission {
                    return Err(SelectionParseError::new(
                        SelectionDiagnosticCode::Truncated,
                        "selection cannot contain consecutive omission tokens",
                        Some(item_index),
                    ));
                }
                last_number_before_gap = previous_line_number;
                previous_was_omission = true;
            }
        }
    }

    Ok(())
}

fn numbered_selection_items(
    block: &SelectionBlock,
) -> Result<Vec<NumberedSelectionItem<'_>>, SelectionResolveError> {
    let numbered_lines = block
        .items
        .iter()
        .enumerate()
        .filter_map(|(item_index, item)| match item {
            SelectionItem::Line(line) => Some(NumberedSelectionItem { item_index, line }),
            SelectionItem::Omission => None,
        })
        .collect::<Vec<_>>();
    if numbered_lines.is_empty() {
        return Err(SelectionResolveError::new(
            SelectionDiagnosticCode::Invalid,
            "selection must contain at least one numbered line",
            None,
        ));
    }
    Ok(numbered_lines)
}

fn normalize_file_lines(file_text: &str) -> Vec<&str> {
    file_text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect()
}

fn omission_token(side: SelectionSide) -> &'static str {
    match side {
        SelectionSide::Source => SOURCE_OMISSION_TOKEN,
        SelectionSide::Target => TARGET_OMISSION_TOKEN,
        SelectionSide::Rewrite => REWRITE_OMISSION_TOKEN,
    }
}

fn omission_tokens() -> [&'static str; 3] {
    [
        SOURCE_OMISSION_TOKEN,
        TARGET_OMISSION_TOKEN,
        REWRITE_OMISSION_TOKEN,
    ]
}

fn contains_horizontal_truncation_marker(raw_line: &str) -> bool {
    raw_line.contains("...[") && raw_line.contains(" chars omitted]")
}

#[derive(Clone, Debug)]
struct ByteLineSpan {
    body: Vec<u8>,
    start: usize,
    end: usize,
}

fn split_file_lines(file_bytes: &[u8]) -> Vec<ByteLineSpan> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, byte) in file_bytes.iter().enumerate() {
        if *byte == b'\n' {
            let body_end = if index > start && file_bytes[index - 1] == b'\r' {
                index - 1
            } else {
                index
            };
            lines.push(ByteLineSpan {
                body: file_bytes[start..body_end].to_vec(),
                start,
                end: index + 1,
            });
            start = index + 1;
        }
    }
    if start < file_bytes.len() {
        lines.push(ByteLineSpan {
            body: file_bytes[start..].to_vec(),
            start,
            end: file_bytes.len(),
        });
    }
    lines
}

fn selection_line_missing_message(line_number: usize, text_label: &str) -> String {
    format!(
        "selection line {} does not exist in the {text_label}",
        line_number
    )
}

fn selection_line_mismatch_message(line_number: usize, text_label: &str) -> String {
    format!(
        "selection line {} does not match the {text_label}",
        line_number
    )
}
