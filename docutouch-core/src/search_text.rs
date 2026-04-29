use crate::path_display::{display_path, format_scope};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const SEARCH_TEXT_PREVIEW_MAX_FILES: usize = 8;
const SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE: usize = 3;
const SEARCH_TEXT_PREVIEW_MAX_CONTEXT_LINES_PER_FILE: usize = 6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchTextView {
    #[default]
    Preview,
    Full,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchTextQueryMode {
    #[default]
    Auto,
    Literal,
    Regex,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchTextOutputMode {
    #[default]
    Auto,
    Grouped,
    GroupedContext,
    Counts,
    Files,
    RawText,
    RawJson,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchTextSurfaceKind {
    StructuredText,
    RawText,
    RawJson,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchTextResult {
    pub content: String,
    pub surface_kind: SearchTextSurfaceKind,
}

#[derive(Debug)]
struct SearchTextGroup {
    path: String,
    entries: Vec<SearchTextEntry>,
    matched_lines: usize,
    total_hits: usize,
}

#[derive(Debug)]
struct SearchTextEntry {
    line_number: usize,
    text: String,
    kind: SearchTextEntryKind,
    hit_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchTextEntryKind {
    Match,
    Context,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QueryInterpretation {
    Regex,
    Literal,
    LiteralFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CountSurfaceKind {
    MatchedLines,
    Matches,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FilesSurfaceKind {
    WithMatches,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolvedOutputMode {
    Grouped,
    GroupedContext { before: usize, after: usize },
    Counts(CountSurfaceKind),
    Files(FilesSurfaceKind),
    RawText,
    RawJson,
}

impl ResolvedOutputMode {
    fn surface_kind(self) -> SearchTextSurfaceKind {
        match self {
            Self::RawText => SearchTextSurfaceKind::RawText,
            Self::RawJson => SearchTextSurfaceKind::RawJson,
            Self::Grouped | Self::GroupedContext { .. } | Self::Counts(_) | Self::Files(_) => {
                SearchTextSurfaceKind::StructuredText
            }
        }
    }

    fn is_raw(self) -> bool {
        matches!(self, Self::RawText | Self::RawJson)
    }

    fn label(self, view: SearchTextView) -> &'static str {
        match self {
            Self::Grouped => view.label(),
            Self::GroupedContext { .. } => "grouped_context",
            Self::Counts(_) => "counts",
            Self::Files(_) => "files",
            Self::RawText => "raw_text",
            Self::RawJson => "raw_json",
        }
    }
}

#[derive(Debug)]
struct ParsedRgArgs {
    tokens: Vec<String>,
    has_json: bool,
    has_count: bool,
    has_count_matches: bool,
    has_files_with_matches: bool,
    has_files_without_match: bool,
    has_files: bool,
    has_heading: bool,
    has_replace: bool,
    has_type_list: bool,
    context_before: Option<usize>,
    context_after: Option<usize>,
    explicit_regex_requested: bool,
    queryless_hint: bool,
}

impl ParsedRgArgs {
    fn parse(rg_args: &str) -> Result<Self, String> {
        let trimmed = rg_args.trim();
        if trimmed.is_empty() {
            return Ok(Self {
                tokens: Vec::new(),
                has_json: false,
                has_count: false,
                has_count_matches: false,
                has_files_with_matches: false,
                has_files_without_match: false,
                has_files: false,
                has_heading: false,
                has_replace: false,
                has_type_list: false,
                context_before: None,
                context_after: None,
                explicit_regex_requested: false,
                queryless_hint: false,
            });
        }

        let tokens =
            shlex::split(trimmed).ok_or_else(|| "rg_args contains invalid quoting".to_string())?;
        let mut parsed = Self {
            tokens,
            has_json: false,
            has_count: false,
            has_count_matches: false,
            has_files_with_matches: false,
            has_files_without_match: false,
            has_files: false,
            has_heading: false,
            has_replace: false,
            has_type_list: false,
            context_before: None,
            context_after: None,
            explicit_regex_requested: false,
            queryless_hint: false,
        };

        let mut index = 0usize;
        while index < parsed.tokens.len() {
            let token = parsed.tokens[index].as_str();
            match token {
                "--json" => parsed.has_json = true,
                "-c" | "--count" => parsed.has_count = true,
                "--count-matches" => parsed.has_count_matches = true,
                "-l" | "--files-with-matches" => parsed.has_files_with_matches = true,
                "--files-without-match" => parsed.has_files_without_match = true,
                "--files" => {
                    parsed.has_files = true;
                    parsed.queryless_hint = true;
                }
                "--heading" => parsed.has_heading = true,
                "--replace" => parsed.has_replace = true,
                "--type-list" => {
                    parsed.has_type_list = true;
                    parsed.queryless_hint = true;
                }
                "-P" | "--pcre2" | "-e" | "--regexp" | "-f" | "--file" => {
                    parsed.explicit_regex_requested = true;
                }
                "--engine" => {
                    parsed.explicit_regex_requested = true;
                    index += 1;
                }
                "-A" | "--after-context" => {
                    if let Some(value) = parse_following_usize(&parsed.tokens, index) {
                        parsed.context_after = Some(value);
                    }
                    index += 1;
                }
                "-B" | "--before-context" => {
                    if let Some(value) = parse_following_usize(&parsed.tokens, index) {
                        parsed.context_before = Some(value);
                    }
                    index += 1;
                }
                "-C" | "--context" => {
                    if let Some(value) = parse_following_usize(&parsed.tokens, index) {
                        parsed.context_before = Some(value);
                        parsed.context_after = Some(value);
                    }
                    index += 1;
                }
                "--color" => {
                    index += 1;
                }
                _ => {
                    if token.starts_with("--after-context=") {
                        if let Some(value) = parse_inline_usize(token) {
                            parsed.context_after = Some(value);
                        }
                    } else if token.starts_with("--before-context=") {
                        if let Some(value) = parse_inline_usize(token) {
                            parsed.context_before = Some(value);
                        }
                    } else if token.starts_with("--context=") {
                        if let Some(value) = parse_inline_usize(token) {
                            parsed.context_before = Some(value);
                            parsed.context_after = Some(value);
                        }
                    } else if token.starts_with("-A") && token.len() > 2 {
                        if let Some(value) = parse_short_inline_usize(token) {
                            parsed.context_after = Some(value);
                        }
                    } else if token.starts_with("-B") && token.len() > 2 {
                        if let Some(value) = parse_short_inline_usize(token) {
                            parsed.context_before = Some(value);
                        }
                    } else if token.starts_with("-C") && token.len() > 2 {
                        if let Some(value) = parse_short_inline_usize(token) {
                            parsed.context_before = Some(value);
                            parsed.context_after = Some(value);
                        }
                    } else if token.starts_with("--color=") {
                        // Absorbed by structured modes, preserved by raw modes.
                    } else if token.starts_with("--engine=") {
                        parsed.explicit_regex_requested = true;
                    }
                }
            }
            index += 1;
        }

        Ok(parsed)
    }

    fn context_requested(&self) -> bool {
        self.context_before.unwrap_or(0) > 0 || self.context_after.unwrap_or(0) > 0
    }

    fn has_output_conflict(&self) -> bool {
        let has_count = self.has_count || self.has_count_matches;
        let has_file_surface =
            self.has_files_with_matches || self.has_files_without_match || self.has_files;
        (has_file_surface || self.context_requested()) && has_count
            || (self.context_requested() && has_file_surface)
    }

    fn requires_raw_text_mode(&self) -> bool {
        self.has_heading || self.has_replace || self.has_type_list
    }

    fn json_event_stream_compatible(&self) -> bool {
        !(self.has_count
            || self.has_count_matches
            || self.has_files_with_matches
            || self.has_files_without_match
            || self.has_files
            || self.requires_raw_text_mode())
    }

    fn preferred_count_kind(&self) -> CountSurfaceKind {
        if self.has_count_matches {
            CountSurfaceKind::Matches
        } else {
            CountSurfaceKind::MatchedLines
        }
    }

    fn default_context_before(&self) -> usize {
        self.context_before.unwrap_or(0)
    }

    fn default_context_after(&self) -> usize {
        self.context_after.unwrap_or(0)
    }

    fn structured_tokens(&self, mode: ResolvedOutputMode) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut index = 0usize;
        while index < self.tokens.len() {
            let token = self.tokens[index].as_str();
            let mut skip_current = false;
            let mut skip_next = false;
            match token {
                "--json"
                | "-n"
                | "--line-number"
                | "-N"
                | "--no-line-number"
                | "--heading"
                | "--no-heading"
                | "-c"
                | "--count"
                | "--count-matches"
                | "-l"
                | "--files-with-matches"
                | "--files-without-match"
                | "--files" => {
                    skip_current = true;
                }
                "--type-list" | "--replace" => {
                    skip_current = true;
                    if token == "--replace" {
                        skip_next = true;
                    }
                }
                "--color" => {
                    if token_value_is_valid_color(self.tokens.get(index + 1).map(String::as_str)) {
                        skip_current = true;
                        skip_next = true;
                    }
                }
                "-A" | "-B" | "-C" | "--after-context" | "--before-context" | "--context" => {
                    if parse_following_usize(&self.tokens, index).is_some()
                        && !matches!(mode, ResolvedOutputMode::GroupedContext { .. })
                    {
                        skip_current = true;
                        skip_next = true;
                    }
                }
                _ => {
                    if token.starts_with("--after-context=")
                        || token.starts_with("--before-context=")
                        || token.starts_with("--context=")
                        || ((token.starts_with("-A")
                            || token.starts_with("-B")
                            || token.starts_with("-C"))
                            && token.len() > 2)
                    {
                        let parsed_ok = if token.starts_with("--") {
                            parse_inline_usize(token).is_some()
                        } else {
                            parse_short_inline_usize(token).is_some()
                        };
                        if parsed_ok && !matches!(mode, ResolvedOutputMode::GroupedContext { .. }) {
                            skip_current = true;
                        }
                    } else if token.starts_with("--color=") && inline_color_value_is_valid(token) {
                        skip_current = true;
                    }
                }
            }
            if !skip_current {
                tokens.push(self.tokens[index].clone());
            }
            index += 1;
            if skip_next && index < self.tokens.len() {
                index += 1;
            }
        }
        tokens
    }

    fn raw_tokens(&self, mode: ResolvedOutputMode) -> Vec<String> {
        let mut tokens = self.tokens.clone();
        if mode == ResolvedOutputMode::RawJson && !self.has_json {
            tokens.push("--json".to_string());
        }
        tokens
    }
}

pub async fn search_text(
    query: &str,
    search_paths: &[PathBuf],
    rg_args_text: &str,
    query_mode: SearchTextQueryMode,
    output_mode: SearchTextOutputMode,
    view: SearchTextView,
    display_base_dir: Option<&Path>,
) -> Result<SearchTextResult, String> {
    let parsed = ParsedRgArgs::parse(rg_args_text)?;
    let resolved_mode = resolve_output_mode(output_mode, &parsed);
    if query.trim().is_empty() && !query_can_be_empty(&parsed, resolved_mode) {
        return Err("query cannot be empty unless rg_args requests a queryless raw rg mode such as `--files` or `--type-list`".to_string());
    }

    let execution = execute_search(query, search_paths, &parsed, query_mode, resolved_mode).await?;
    if execution.mode.is_raw() {
        return Ok(SearchTextResult {
            content: execution.output,
            surface_kind: execution.mode.surface_kind(),
        });
    }

    let (groups, total_matches) = parse_search_text_output(&execution.output, display_base_dir)?;
    let content = match execution.mode {
        ResolvedOutputMode::Grouped => format_grouped_result(
            query,
            &format_scope(search_paths, display_base_dir),
            rg_args_text,
            view,
            &groups,
            total_matches,
            execution.interpretation,
        ),
        ResolvedOutputMode::GroupedContext { before, after } => format_grouped_context_result(
            query,
            &format_scope(search_paths, display_base_dir),
            rg_args_text,
            view,
            &groups,
            total_matches,
            execution.interpretation,
            before,
            after,
        ),
        ResolvedOutputMode::Counts(kind) => format_counts_result(
            query,
            &format_scope(search_paths, display_base_dir),
            rg_args_text,
            &groups,
            total_matches,
            execution.interpretation,
            kind,
        ),
        ResolvedOutputMode::Files(kind) => format_files_result(
            query,
            &format_scope(search_paths, display_base_dir),
            rg_args_text,
            &groups,
            execution.interpretation,
            kind,
        ),
        ResolvedOutputMode::RawText | ResolvedOutputMode::RawJson => unreachable!(),
    };

    Ok(SearchTextResult {
        content,
        surface_kind: SearchTextSurfaceKind::StructuredText,
    })
}

struct SearchExecution {
    output: String,
    interpretation: QueryInterpretation,
    mode: ResolvedOutputMode,
}

async fn execute_search(
    query: &str,
    search_paths: &[PathBuf],
    parsed: &ParsedRgArgs,
    query_mode: SearchTextQueryMode,
    resolved_mode: ResolvedOutputMode,
) -> Result<SearchExecution, String> {
    match query_mode {
        SearchTextQueryMode::Literal => {
            let output =
                run_search_text_rg(query, search_paths, parsed, resolved_mode, true).await?;
            Ok(SearchExecution {
                output,
                interpretation: QueryInterpretation::Literal,
                mode: resolved_mode,
            })
        }
        SearchTextQueryMode::Regex => {
            let output = run_search_text_rg(query, search_paths, parsed, resolved_mode, false)
                .await
                .map_err(|error| map_regex_error(query, error))?;
            Ok(SearchExecution {
                output,
                interpretation: QueryInterpretation::Regex,
                mode: resolved_mode,
            })
        }
        SearchTextQueryMode::Auto => {
            match run_search_text_rg(query, search_paths, parsed, resolved_mode, false).await {
                Ok(output) => Ok(SearchExecution {
                    output,
                    interpretation: QueryInterpretation::Regex,
                    mode: resolved_mode,
                }),
                Err(error)
                    if is_regex_parse_error(&error)
                        && !parsed.explicit_regex_requested
                        && !query.is_empty() =>
                {
                    let output =
                        run_search_text_rg(query, search_paths, parsed, resolved_mode, true)
                            .await?;
                    Ok(SearchExecution {
                        output,
                        interpretation: QueryInterpretation::LiteralFallback,
                        mode: resolved_mode,
                    })
                }
                Err(error) => Err(map_regex_error(query, error)),
            }
        }
    }
}

fn resolve_output_mode(
    requested: SearchTextOutputMode,
    parsed: &ParsedRgArgs,
) -> ResolvedOutputMode {
    if requested == SearchTextOutputMode::RawText {
        return ResolvedOutputMode::RawText;
    }
    if requested == SearchTextOutputMode::RawJson {
        return if parsed.json_event_stream_compatible() {
            ResolvedOutputMode::RawJson
        } else {
            ResolvedOutputMode::RawText
        };
    }
    if parsed.has_json {
        return if parsed.json_event_stream_compatible() {
            ResolvedOutputMode::RawJson
        } else {
            ResolvedOutputMode::RawText
        };
    }
    if parsed.has_output_conflict() {
        return ResolvedOutputMode::RawText;
    }
    if parsed.requires_raw_text_mode() || parsed.has_files_without_match || parsed.has_files {
        return ResolvedOutputMode::RawText;
    }

    match requested {
        SearchTextOutputMode::Auto => {
            if parsed.has_count || parsed.has_count_matches {
                ResolvedOutputMode::Counts(parsed.preferred_count_kind())
            } else if parsed.has_files_with_matches {
                ResolvedOutputMode::Files(FilesSurfaceKind::WithMatches)
            } else if parsed.context_requested() {
                ResolvedOutputMode::GroupedContext {
                    before: parsed.default_context_before(),
                    after: parsed.default_context_after(),
                }
            } else {
                ResolvedOutputMode::Grouped
            }
        }
        SearchTextOutputMode::Grouped => {
            if parsed.has_count || parsed.has_count_matches || parsed.has_files_with_matches {
                ResolvedOutputMode::RawText
            } else if parsed.context_requested() {
                ResolvedOutputMode::GroupedContext {
                    before: parsed.default_context_before(),
                    after: parsed.default_context_after(),
                }
            } else {
                ResolvedOutputMode::Grouped
            }
        }
        SearchTextOutputMode::GroupedContext => {
            if parsed.has_count || parsed.has_count_matches || parsed.has_files_with_matches {
                ResolvedOutputMode::RawText
            } else {
                ResolvedOutputMode::GroupedContext {
                    before: parsed.default_context_before(),
                    after: parsed.default_context_after(),
                }
            }
        }
        SearchTextOutputMode::Counts => {
            if parsed.has_files_with_matches {
                ResolvedOutputMode::RawText
            } else {
                ResolvedOutputMode::Counts(parsed.preferred_count_kind())
            }
        }
        SearchTextOutputMode::Files => {
            if parsed.has_count || parsed.has_count_matches {
                ResolvedOutputMode::RawText
            } else {
                ResolvedOutputMode::Files(FilesSurfaceKind::WithMatches)
            }
        }
        SearchTextOutputMode::RawText => ResolvedOutputMode::RawText,
        SearchTextOutputMode::RawJson => ResolvedOutputMode::RawText,
    }
}

fn query_can_be_empty(parsed: &ParsedRgArgs, mode: ResolvedOutputMode) -> bool {
    mode.is_raw() && parsed.queryless_hint
}

async fn run_search_text_rg(
    query: &str,
    search_paths: &[PathBuf],
    parsed: &ParsedRgArgs,
    mode: ResolvedOutputMode,
    literal_query: bool,
) -> Result<String, String> {
    let mut command = tokio::process::Command::new("rg");
    if mode.is_raw() {
        for arg in parsed.raw_tokens(mode) {
            command.arg(arg);
        }
    } else {
        command
            .arg("--json")
            .arg("--line-number")
            .arg("--color")
            .arg("never")
            .arg("--no-heading");
        for arg in parsed.structured_tokens(mode) {
            command.arg(arg);
        }
    }

    if literal_query {
        command.arg("-F");
    }
    if !query.is_empty() {
        command.arg("--").arg(query);
    }
    for search_path in search_paths {
        command.arg(search_path);
    }

    let output = command
        .output()
        .await
        .map_err(|err| format!("Failed to run rg: {err}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    match output.status.code() {
        Some(0) | Some(1) => Ok(stdout),
        _ if !stderr.is_empty() => Err(format!("rg failed: {stderr}")),
        _ => Err(format!("rg failed with status {}", output.status)),
    }
}

fn parse_search_text_output(
    output: &str,
    display_base_dir: Option<&Path>,
) -> Result<(Vec<SearchTextGroup>, usize), String> {
    let mut groups = Vec::new();
    let mut group_index_by_path = HashMap::new();
    let mut total_matches = 0usize;

    for raw_line in output.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(raw_line)
            .map_err(|err| format!("Failed to parse rg JSON output: {err}"))?;
        let event_type = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if event_type != "match" && event_type != "context" {
            continue;
        }
        let data = value
            .get("data")
            .ok_or_else(|| format!("rg JSON output missing `data` for a {event_type} event"))?;
        let raw_path = extract_rg_text(data.get("path"))
            .ok_or_else(|| "rg JSON output missing event path".to_string())?;
        let line_text = extract_rg_text(data.get("lines"))
            .ok_or_else(|| "rg JSON output missing event lines".to_string())?;
        let line_number = data
            .get("line_number")
            .and_then(Value::as_u64)
            .ok_or_else(|| "rg JSON output missing event line_number".to_string())?
            as usize;
        let display_path = display_path(display_base_dir, Path::new(&raw_path));
        let index = if let Some(index) = group_index_by_path.get(&display_path).copied() {
            index
        } else {
            let index = groups.len();
            groups.push(SearchTextGroup {
                path: display_path.clone(),
                entries: Vec::new(),
                matched_lines: 0,
                total_hits: 0,
            });
            group_index_by_path.insert(display_path, index);
            index
        };
        let (kind, hit_count) = if event_type == "match" {
            let hit_count = data
                .get("submatches")
                .and_then(Value::as_array)
                .map(Vec::len)
                .filter(|count| *count > 0)
                .unwrap_or(1);
            groups[index].matched_lines += 1;
            groups[index].total_hits += hit_count;
            total_matches += hit_count;
            (SearchTextEntryKind::Match, hit_count)
        } else {
            (SearchTextEntryKind::Context, 0)
        };
        groups[index].entries.push(SearchTextEntry {
            line_number,
            text: trim_rg_line_text(&line_text),
            kind,
            hit_count,
        });
    }

    groups.sort_by(|lhs, rhs| {
        rhs.matched_lines
            .cmp(&lhs.matched_lines)
            .then_with(|| rhs.total_hits.cmp(&lhs.total_hits))
            .then_with(|| lhs.path.cmp(&rhs.path))
    });
    Ok((groups, total_matches))
}

fn extract_rg_text(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if value.get("bytes").is_some() {
        return Some("<binary>".to_string());
    }
    None
}

fn trim_rg_line_text(text: &str) -> String {
    text.trim_end_matches(&['\r', '\n'][..]).to_string()
}

fn format_grouped_result(
    query: &str,
    scope: &str,
    rg_args: &str,
    view: SearchTextView,
    groups: &[SearchTextGroup],
    total_matches: usize,
    interpretation: QueryInterpretation,
) -> String {
    format_grouped_like_result(
        ResolvedOutputMode::Grouped,
        query,
        scope,
        rg_args,
        view,
        groups,
        total_matches,
        interpretation,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn format_grouped_context_result(
    query: &str,
    scope: &str,
    rg_args: &str,
    view: SearchTextView,
    groups: &[SearchTextGroup],
    total_matches: usize,
    interpretation: QueryInterpretation,
    before: usize,
    after: usize,
) -> String {
    format_grouped_like_result(
        ResolvedOutputMode::GroupedContext { before, after },
        query,
        scope,
        rg_args,
        view,
        groups,
        total_matches,
        interpretation,
        Some((before, after)),
    )
}

#[allow(clippy::too_many_arguments)]
fn format_grouped_like_result(
    mode: ResolvedOutputMode,
    query: &str,
    scope: &str,
    rg_args: &str,
    view: SearchTextView,
    groups: &[SearchTextGroup],
    total_matches: usize,
    interpretation: QueryInterpretation,
    context_window: Option<(usize, usize)>,
) -> String {
    let matched_lines = groups
        .iter()
        .map(|group| group.matched_lines)
        .sum::<usize>();
    let mut lines = vec![format!("search_text[{}]:", mode.label(view))];
    if !query.is_empty() {
        lines.push(format!("query: {query}"));
    }
    lines.push(format!("scope: {scope}"));
    lines.push(format!("files: {}", groups.len()));
    lines.push(format!("matched_lines: {matched_lines}"));
    lines.push(format!("matches: {total_matches}"));
    if let Some((before, after)) = context_window {
        lines.push(format!("context: before={before} after={after}"));
    }
    if let Some(label) = interpretation_label(interpretation) {
        lines.push(format!("query_interpretation: {label}"));
    }
    if !rg_args.trim().is_empty() {
        lines.push(format!("rg_args: {}", rg_args.trim()));
    }

    let per_file_limit = match mode {
        ResolvedOutputMode::Grouped => SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE,
        ResolvedOutputMode::GroupedContext { .. } => SEARCH_TEXT_PREVIEW_MAX_CONTEXT_LINES_PER_FILE,
        _ => 0,
    };

    if view == SearchTextView::Preview {
        let rendered_files = groups.len().min(SEARCH_TEXT_PREVIEW_MAX_FILES);
        let rendered_lines = groups
            .iter()
            .take(rendered_files)
            .map(|group| group.entries.len().min(per_file_limit))
            .sum::<usize>();
        lines.push(format!("rendered_files: {rendered_files}"));
        lines.push(format!("rendered_lines: {rendered_lines}"));
    }
    if groups.is_empty() {
        lines.push(String::new());
        lines.push("(no matches)".to_string());
        return lines.join("\n");
    }

    lines.push(String::new());
    let rendered_files = if view == SearchTextView::Preview {
        groups.len().min(SEARCH_TEXT_PREVIEW_MAX_FILES)
    } else {
        groups.len()
    };
    let mut rendered_entries_total = 0usize;

    for (index, group) in groups.iter().take(rendered_files).enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        let line_word = if group.matched_lines == 1 {
            "line"
        } else {
            "lines"
        };
        let hit_word = if group.total_hits == 1 {
            "match"
        } else {
            "matches"
        };
        lines.push(format!(
            "[{}] {} ({} {}, {} {})",
            index + 1,
            group.path,
            group.matched_lines,
            line_word,
            group.total_hits,
            hit_word
        ));
        let rendered_entries = if view == SearchTextView::Preview {
            group
                .entries
                .iter()
                .take(per_file_limit)
                .collect::<Vec<_>>()
        } else {
            group.entries.iter().collect::<Vec<_>>()
        };
        let line_number_width = rendered_entries
            .iter()
            .map(|entry| entry.line_number)
            .max()
            .unwrap_or_default()
            .to_string()
            .len()
            .max(1);
        rendered_entries_total += rendered_entries.len();
        for entry in rendered_entries {
            render_group_entry(
                &mut lines,
                entry,
                line_number_width,
                matches!(mode, ResolvedOutputMode::GroupedContext { .. }),
            );
        }
        if view == SearchTextView::Preview && group.entries.len() > per_file_limit {
            lines.push(format!(
                "  note: {} more rendered lines in this file",
                group.entries.len() - per_file_limit
            ));
        }
    }

    if view == SearchTextView::Preview {
        let omitted_files = groups.len().saturating_sub(rendered_files);
        let omitted_lines = groups
            .iter()
            .map(|group| group.entries.len())
            .sum::<usize>()
            .saturating_sub(rendered_entries_total);
        if omitted_files > 0 || omitted_lines > 0 {
            lines.push(String::new());
            lines.push("omitted:".to_string());
            if omitted_files > 0 {
                lines.push(format!("- {omitted_files} more files not shown"));
            }
            if omitted_lines > 0 {
                lines.push(format!("- {omitted_lines} more rendered lines not shown"));
            }
        }
    }

    lines.join("\n")
}

fn render_group_entry(
    lines: &mut Vec<String>,
    entry: &SearchTextEntry,
    width: usize,
    render_context_marker: bool,
) {
    let line_number = entry.line_number;
    let prefix = if render_context_marker {
        match entry.kind {
            SearchTextEntryKind::Match => "M",
            SearchTextEntryKind::Context => "C",
        }
    } else {
        ""
    };
    let line_prefix = if render_context_marker {
        format!("  {prefix} {line_number:>width$} | ", width = width)
    } else {
        format!("  {line_number:>width$} | ", width = width)
    };
    if entry.hit_count > 1 {
        lines.push(format!(
            "{line_prefix}{}  [{} hits]",
            entry.text, entry.hit_count
        ));
    } else {
        lines.push(format!("{line_prefix}{}", entry.text));
    }
}

fn format_counts_result(
    query: &str,
    scope: &str,
    rg_args: &str,
    groups: &[SearchTextGroup],
    total_matches: usize,
    interpretation: QueryInterpretation,
    count_kind: CountSurfaceKind,
) -> String {
    let matched_lines = groups
        .iter()
        .map(|group| group.matched_lines)
        .sum::<usize>();
    let mut lines = vec!["search_text[counts]:".to_string()];
    if !query.is_empty() {
        lines.push(format!("query: {query}"));
    }
    lines.push(format!("scope: {scope}"));
    lines.push(format!("files: {}", groups.len()));
    lines.push(format!("matched_lines: {matched_lines}"));
    lines.push(format!("matches: {total_matches}"));
    lines.push(format!(
        "count_mode: {}",
        match count_kind {
            CountSurfaceKind::MatchedLines => "matched_lines",
            CountSurfaceKind::Matches => "matches",
        }
    ));
    if let Some(label) = interpretation_label(interpretation) {
        lines.push(format!("query_interpretation: {label}"));
    }
    if !rg_args.trim().is_empty() {
        lines.push(format!("rg_args: {}", rg_args.trim()));
    }
    lines.push(String::new());
    if groups.is_empty() {
        lines.push("(no matches)".to_string());
        return lines.join("\n");
    }
    for (index, group) in groups.iter().enumerate() {
        let value = match count_kind {
            CountSurfaceKind::MatchedLines => group.matched_lines,
            CountSurfaceKind::Matches => group.total_hits,
        };
        let unit = match count_kind {
            CountSurfaceKind::MatchedLines => {
                if value == 1 {
                    "matched line"
                } else {
                    "matched lines"
                }
            }
            CountSurfaceKind::Matches => {
                if value == 1 {
                    "match"
                } else {
                    "matches"
                }
            }
        };
        lines.push(format!(
            "[{}] {} | {} {}",
            index + 1,
            group.path,
            value,
            unit
        ));
    }
    lines.join("\n")
}

fn format_files_result(
    query: &str,
    scope: &str,
    rg_args: &str,
    groups: &[SearchTextGroup],
    interpretation: QueryInterpretation,
    file_kind: FilesSurfaceKind,
) -> String {
    let mut lines = vec!["search_text[files]:".to_string()];
    if !query.is_empty() {
        lines.push(format!("query: {query}"));
    }
    lines.push(format!("scope: {scope}"));
    lines.push(format!("files: {}", groups.len()));
    lines.push(format!(
        "mode: {}",
        match file_kind {
            FilesSurfaceKind::WithMatches => "files_with_matches",
        }
    ));
    if let Some(label) = interpretation_label(interpretation) {
        lines.push(format!("query_interpretation: {label}"));
    }
    if !rg_args.trim().is_empty() {
        lines.push(format!("rg_args: {}", rg_args.trim()));
    }
    lines.push(String::new());
    if groups.is_empty() {
        lines.push("(no matches)".to_string());
        return lines.join("\n");
    }
    for (index, group) in groups.iter().enumerate() {
        lines.push(format!("[{}] {}", index + 1, group.path));
    }
    lines.join("\n")
}

fn interpretation_label(interpretation: QueryInterpretation) -> Option<&'static str> {
    match interpretation {
        QueryInterpretation::Regex => None,
        QueryInterpretation::Literal => Some("literal"),
        QueryInterpretation::LiteralFallback => Some("literal_fallback"),
    }
}

impl SearchTextView {
    fn label(self) -> &'static str {
        match self {
            SearchTextView::Preview => "preview",
            SearchTextView::Full => "full",
        }
    }
}

fn is_regex_parse_error(error: &str) -> bool {
    error.contains("regex parse error")
        || error.contains("the literal \"\\n\" is not allowed in a regex")
}

fn map_regex_error(query: &str, error: String) -> String {
    if is_regex_parse_error(&error) {
        format!(
            "query was interpreted as a ripgrep regex and failed to parse. If you meant to search the literal text `{query}`, retry with `query_mode: \"literal\"` or leave `query_mode` at `auto`. Raw rg error: {error}"
        )
    } else {
        error
    }
}

fn parse_following_usize(tokens: &[String], index: usize) -> Option<usize> {
    tokens
        .get(index + 1)
        .and_then(|value| value.parse::<usize>().ok())
}

fn parse_inline_usize(token: &str) -> Option<usize> {
    token
        .split_once('=')
        .and_then(|(_, value)| value.parse::<usize>().ok())
}

fn parse_short_inline_usize(token: &str) -> Option<usize> {
    token[2..].parse::<usize>().ok()
}

fn token_value_is_valid_color(value: Option<&str>) -> bool {
    matches!(value, Some("never" | "auto" | "always" | "ansi"))
}

fn inline_color_value_is_valid(token: &str) -> bool {
    token
        .split_once('=')
        .map(|(_, value)| matches!(value, "never" | "auto" | "always" | "ansi"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_mode_auto_prefers_grouped_context_for_context_flags() {
        let parsed = ParsedRgArgs::parse("-C2 -g '*.rs'").expect("parse rg args");
        assert_eq!(
            resolve_output_mode(SearchTextOutputMode::Auto, &parsed),
            ResolvedOutputMode::GroupedContext {
                before: 2,
                after: 2,
            }
        );
    }

    #[test]
    fn output_mode_auto_prefers_counts_for_count_matches() {
        let parsed = ParsedRgArgs::parse("--count-matches").expect("parse rg args");
        assert_eq!(
            resolve_output_mode(SearchTextOutputMode::Auto, &parsed),
            ResolvedOutputMode::Counts(CountSurfaceKind::Matches)
        );
    }

    #[test]
    fn output_mode_auto_falls_back_to_raw_text_on_conflicts() {
        let parsed = ParsedRgArgs::parse("-C2 --count").expect("parse rg args");
        assert_eq!(
            resolve_output_mode(SearchTextOutputMode::Auto, &parsed),
            ResolvedOutputMode::RawText
        );
    }

    #[test]
    fn output_mode_auto_does_not_force_raw_json_for_non_event_json_combinations() {
        let parsed = ParsedRgArgs::parse("--json --count-matches").expect("parse rg args");
        assert_eq!(
            resolve_output_mode(SearchTextOutputMode::Auto, &parsed),
            ResolvedOutputMode::RawText
        );
    }

    #[test]
    fn explicit_raw_text_is_not_overridden_by_json_flag() {
        let parsed = ParsedRgArgs::parse("--json -l").expect("parse rg args");
        assert_eq!(
            resolve_output_mode(SearchTextOutputMode::RawText, &parsed),
            ResolvedOutputMode::RawText
        );
    }

    #[test]
    fn invalid_context_values_are_not_treated_as_valid_context_requests() {
        let parsed = ParsedRgArgs::parse("-C nope").expect("parse rg args");
        assert!(!parsed.context_requested());
        assert!(
            parsed
                .structured_tokens(ResolvedOutputMode::Grouped)
                .contains(&"-C".to_string())
        );
        assert!(
            parsed
                .structured_tokens(ResolvedOutputMode::Grouped)
                .contains(&"nope".to_string())
        );
    }

    #[test]
    fn parses_compact_context_flags() {
        let parsed = ParsedRgArgs::parse("-C2 --before-context=3").expect("parse rg args");
        assert_eq!(parsed.context_before, Some(3));
        assert_eq!(parsed.context_after, Some(2));
    }

    #[test]
    fn preview_renders_explicit_omission_accounting() {
        let groups = vec![SearchTextGroup {
            path: "src/noisy.txt".to_string(),
            entries: (1..=5)
                .map(|line_number| SearchTextEntry {
                    line_number,
                    text: "alpha".to_string(),
                    kind: SearchTextEntryKind::Match,
                    hit_count: 1,
                })
                .collect(),
            matched_lines: 5,
            total_hits: 5,
        }];
        let rendered = format_grouped_result(
            "alpha",
            "src",
            "",
            SearchTextView::Preview,
            &groups,
            5,
            QueryInterpretation::Regex,
        );
        assert!(rendered.contains("rendered_files: 1"));
        assert!(rendered.contains("rendered_lines: 3"));
        assert!(rendered.contains("note: 2 more rendered lines in this file"));
        assert!(rendered.contains("- 2 more rendered lines not shown"));
    }

    #[test]
    fn grouped_context_marks_match_and_context_lines() {
        let groups = vec![SearchTextGroup {
            path: "src/noisy.txt".to_string(),
            entries: vec![
                SearchTextEntry {
                    line_number: 9,
                    text: "alpha".to_string(),
                    kind: SearchTextEntryKind::Match,
                    hit_count: 1,
                },
                SearchTextEntry {
                    line_number: 10,
                    text: "beta".to_string(),
                    kind: SearchTextEntryKind::Context,
                    hit_count: 0,
                },
            ],
            matched_lines: 1,
            total_hits: 1,
        }];
        let rendered = format_grouped_context_result(
            "alpha",
            "src",
            "-C1",
            SearchTextView::Full,
            &groups,
            1,
            QueryInterpretation::LiteralFallback,
            1,
            1,
        );
        assert!(rendered.contains("search_text[grouped_context]:"));
        assert!(rendered.contains("query_interpretation: literal_fallback"));
        assert!(rendered.contains("M  9 | alpha") || rendered.contains("M   9 | alpha"));
        assert!(rendered.contains("C 10 | beta") || rendered.contains("C  10 | beta"));
    }

    #[test]
    fn files_surface_is_compact() {
        let groups = vec![SearchTextGroup {
            path: "src/one.txt".to_string(),
            entries: vec![SearchTextEntry {
                line_number: 1,
                text: "alpha".to_string(),
                kind: SearchTextEntryKind::Match,
                hit_count: 1,
            }],
            matched_lines: 1,
            total_hits: 1,
        }];
        let rendered = format_files_result(
            "alpha",
            "src",
            "-l",
            &groups,
            QueryInterpretation::Regex,
            FilesSurfaceKind::WithMatches,
        );
        assert!(rendered.contains("search_text[files]:"));
        assert!(rendered.contains("mode: files_with_matches"));
        assert!(rendered.contains("[1] src/one.txt"));
    }

    #[test]
    fn counts_surface_can_render_matches() {
        let groups = vec![SearchTextGroup {
            path: "src/one.txt".to_string(),
            entries: vec![SearchTextEntry {
                line_number: 1,
                text: "alpha alpha".to_string(),
                kind: SearchTextEntryKind::Match,
                hit_count: 2,
            }],
            matched_lines: 1,
            total_hits: 2,
        }];
        let rendered = format_counts_result(
            "alpha",
            "src",
            "--count-matches",
            &groups,
            2,
            QueryInterpretation::Regex,
            CountSurfaceKind::Matches,
        );
        assert!(rendered.contains("count_mode: matches"));
        assert!(rendered.contains("[1] src/one.txt | 2 matches"));
    }
}
