use crate::path_display::{display_path, format_scope};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};
use tokio::process::Command;

const DEFAULT_LIMIT: usize = 8;
const DEFAULT_MATCHES_PER_GROUP: usize = 3;
const MAX_DISPLAY_LIMIT: usize = 64;
const MAX_CAPTURE_LINES: usize = 3;
const MAX_CAPTURE_WIDTH: usize = 120;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StructuralSearchMode {
    #[default]
    Find,
    Expand,
    Around,
    ExplainAst,
    RuleTest,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StructuralSearchView {
    #[default]
    Preview,
    Full,
}

#[derive(Clone, Debug)]
pub struct StructuralSearchOptions {
    pub mode: StructuralSearchMode,
    pub pattern: Option<String>,
    pub rule: Option<Value>,
    pub query: Option<String>,
    pub reference: Option<String>,
    pub search_paths: Vec<PathBuf>,
    pub display_base_dir: Option<PathBuf>,
    pub language: Option<String>,
    pub include_tests: bool,
    pub context: Vec<String>,
    pub limit: Option<usize>,
    pub view: StructuralSearchView,
}

#[derive(Debug, Default)]
pub struct StructuralSearchSession {
    next_query_number: usize,
    recent_query: Option<usize>,
    registry: HashMap<usize, RegisteredResult>,
}

#[derive(Clone, Debug)]
struct RegisteredResult {
    mode: StructuralSearchMode,
    groups: Vec<StructuralSearchGroup>,
}

#[derive(Clone, Debug)]
struct StructuralSearchGroup {
    title: String,
    matches: Vec<StructuralMatch>,
}

#[derive(Clone, Debug)]
struct StructuralMatch {
    source_path: Option<PathBuf>,
    file: String,
    line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    text: String,
    lines: String,
    language: String,
    captures: CaptureSummary,
}

#[derive(Clone, Debug)]
struct ResolvedTarget {
    source_query: Option<usize>,
    source_group: Option<usize>,
    title: String,
    item: StructuralMatch,
}

impl ResolvedTarget {
    fn source_label(&self) -> String {
        if let (Some(query), Some(group)) = (self.source_query, self.source_group) {
            format!("q{query}.[{group}] {}:{}", self.item.file, self.item.line)
        } else {
            format!("{}:{}", self.item.file, self.item.line)
        }
    }
}

#[derive(Clone, Debug, Default)]
struct CaptureSummary {
    single: Vec<(String, String)>,
    multi: Vec<(String, Vec<String>)>,
    transformed: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAstGrepMatch {
    text: String,
    range: RawRange,
    file: String,
    lines: String,
    language: String,
    #[serde(default)]
    meta_variables: Option<RawMetaVariables>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRange {
    start: RawPosition,
    end: RawPosition,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPosition {
    line: usize,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawMetaVariables {
    #[serde(default)]
    single: HashMap<String, RawMatchNode>,
    #[serde(default)]
    multi: HashMap<String, Vec<RawMatchNode>>,
    #[serde(default)]
    transformed: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMatchNode {
    text: String,
}

struct AstGrepRun {
    matches: Vec<RawAstGrepMatch>,
    diagnostics: String,
}

impl StructuralSearchSession {
    pub async fn search(&mut self, options: StructuralSearchOptions) -> Result<String, String> {
        if let Some(message) = validate_common_parameters(&options) {
            return Ok(format_parameter_error(options.mode, &message));
        }
        match options.mode {
            StructuralSearchMode::Find => self.find(options, StructuralSearchMode::Find).await,
            StructuralSearchMode::RuleTest => self.rule_test(options).await,
            StructuralSearchMode::Expand => self.expand(options),
            StructuralSearchMode::Around => self.around(options),
            StructuralSearchMode::ExplainAst => self.explain_ast(options).await,
        }
    }

    async fn find(
        &mut self,
        mut options: StructuralSearchOptions,
        mode: StructuralSearchMode,
    ) -> Result<String, String> {
        match normalize_rule_input(options.rule.take()) {
            Ok(rule) => options.rule = rule,
            Err(message) => return Ok(format_parameter_error(mode, &message)),
        }

        if let Some(rule) = &options.rule
            && contains_unsupported_edit_field(rule)
        {
            return Ok(format_unsupported_rule_field(mode, rule));
        }

        if options.search_paths.is_empty() {
            return Ok(format_workspace_required(mode, &options));
        }
        if let Some(message) = validate_search_paths(mode, &options) {
            return Ok(message);
        }
        let language = match resolve_language(mode, &options) {
            Ok(language) => language,
            Err(message) => return Ok(message),
        };
        let parse_gaps = collect_parse_gaps(
            &options.search_paths,
            &language,
            options.include_tests,
            options.display_base_dir.as_deref(),
        );
        let run = match run_ast_grep(&options, &language).await {
            Ok(run) => run,
            Err(message) => return Ok(format_unavailable(mode, &message)),
        };
        if pattern_has_error_node(&run.diagnostics) {
            return Ok(format_invalid_pattern(mode, &options, &language));
        }
        let matches = normalize_matches(run.matches, &options);
        let groups = group_matches(matches);
        let qid = self.register(mode, groups.clone());
        let status = match (!parse_gaps.is_empty(), mode, groups.is_empty()) {
            (true, _, _) => Some("parse-partial"),
            (_, _, true) => Some("no-matches"),
            (_, StructuralSearchMode::RuleTest, false) => Some("matched"),
            _ => None,
        };
        Ok(format_find_result(FindResultSurface {
            qid,
            mode,
            options: &options,
            language: &language,
            groups: &groups,
            status,
            parse_gaps: &parse_gaps,
            test_source: None,
        }))
    }

    async fn rule_test(&mut self, options: StructuralSearchOptions) -> Result<String, String> {
        let mut options = options;
        match normalize_rule_input(options.rule.take()) {
            Ok(rule) => options.rule = rule,
            Err(message) => {
                return Ok(format_parameter_error(
                    StructuralSearchMode::RuleTest,
                    &message,
                ));
            }
        }

        if let Some(rule) = &options.rule
            && contains_unsupported_edit_field(rule)
        {
            return Ok(format_unsupported_rule_field(
                StructuralSearchMode::RuleTest,
                rule,
            ));
        }

        if let Some(target) = self.resolve_rule_test_target(&options) {
            return self.rule_test_target(options, target).await;
        }

        self.find(options, StructuralSearchMode::RuleTest).await
    }

    async fn rule_test_target(
        &mut self,
        options: StructuralSearchOptions,
        target: ResolvedTarget,
    ) -> Result<String, String> {
        let language = normalize_language(&target.item.language);
        if !is_supported_language(&language) {
            return Ok(format_unsupported_language(
                StructuralSearchMode::RuleTest,
                &language,
            ));
        }
        let source = if target.item.lines.trim().is_empty() {
            target.item.text.clone()
        } else {
            target.item.lines.clone()
        };
        let run = match run_ast_grep_stdin(&options, &language, &source).await {
            Ok(run) => run,
            Err(message) => {
                return Ok(format_unavailable(StructuralSearchMode::RuleTest, &message));
            }
        };
        if pattern_has_error_node(&run.diagnostics) {
            return Ok(format_invalid_pattern(
                StructuralSearchMode::RuleTest,
                &options,
                &language,
            ));
        }
        let matches = normalize_stdin_matches(run.matches, &target.item);
        let groups = group_matches(matches);
        let qid = self.register(StructuralSearchMode::RuleTest, groups.clone());
        let status = if groups.is_empty() {
            Some("no-matches")
        } else {
            Some("matched")
        };
        Ok(format_find_result(FindResultSurface {
            qid,
            mode: StructuralSearchMode::RuleTest,
            options: &options,
            language: &language,
            groups: &groups,
            status,
            parse_gaps: &[],
            test_source: Some(&target.source_label()),
        }))
    }

    fn expand(&mut self, options: StructuralSearchOptions) -> Result<String, String> {
        let Some(reference) = options.reference.as_deref() else {
            return Ok(format_invalid_ref(
                StructuralSearchMode::Expand,
                "<missing>",
            ));
        };
        let Some((query_id, group_index)) = self.resolve_ref(reference) else {
            return Ok(format_invalid_ref(StructuralSearchMode::Expand, reference));
        };
        let Some(group) = self
            .registry
            .get(&query_id)
            .and_then(|entry| entry.groups.get(group_index - 1))
            .cloned()
        else {
            return Ok(format_invalid_ref(StructuralSearchMode::Expand, reference));
        };
        if group.matches.is_empty() {
            return Ok(format_invalid_ref(StructuralSearchMode::Expand, reference));
        }
        let qid = self.register(
            StructuralSearchMode::Expand,
            vec![StructuralSearchGroup {
                title: group.title.clone(),
                matches: group.matches.clone(),
            }],
        );
        Ok(format_expand_result(
            qid,
            query_id,
            group_index,
            &group,
            &options,
        ))
    }

    fn around(&mut self, options: StructuralSearchOptions) -> Result<String, String> {
        let target = match self.resolve_target(StructuralSearchMode::Around, &options) {
            Ok(target) => target,
            Err(message) => return Ok(message),
        };
        let group = StructuralSearchGroup {
            title: target.title.clone(),
            matches: vec![target.item.clone()],
        };
        let qid = self.register(StructuralSearchMode::Around, vec![group.clone()]);
        Ok(format_around_result(qid, &target, &group, &options))
    }

    async fn explain_ast(&mut self, options: StructuralSearchOptions) -> Result<String, String> {
        let target = match self.resolve_target(StructuralSearchMode::ExplainAst, &options) {
            Ok(target) => target,
            Err(message) => return Ok(message),
        };
        let qid = self.register(StructuralSearchMode::ExplainAst, Vec::new());
        Ok(format_explain_ast_result(qid, &target))
    }

    fn register(
        &mut self,
        mode: StructuralSearchMode,
        groups: Vec<StructuralSearchGroup>,
    ) -> usize {
        let qid = if self.next_query_number == 0 {
            1
        } else {
            self.next_query_number
        };
        self.next_query_number = qid + 1;
        self.recent_query = Some(qid);
        self.registry.insert(qid, RegisteredResult { mode, groups });
        qid
    }

    fn resolve_ref(&self, reference: &str) -> Option<(usize, usize)> {
        if let Some(rest) = reference.strip_prefix('q') {
            let (query, group) = rest.split_once('.')?;
            let query_id = query.parse().ok()?;
            let group_index = group.parse().ok()?;
            return Some((query_id, group_index));
        }
        let group_index = reference.parse().ok()?;
        let query_id = self.recent_query?;
        let entry = self.registry.get(&query_id)?;
        if entry.mode == StructuralSearchMode::ExplainAst {
            return None;
        }
        Some((query_id, group_index))
    }

    fn resolve_target(
        &self,
        mode: StructuralSearchMode,
        options: &StructuralSearchOptions,
    ) -> Result<ResolvedTarget, String> {
        if let Some(reference) = options.reference.as_deref() {
            let Some((query_id, group_index)) = self.resolve_ref(reference) else {
                return Err(format_invalid_ref(mode, reference));
            };
            let Some(group) = self
                .registry
                .get(&query_id)
                .and_then(|entry| entry.groups.get(group_index - 1))
                .cloned()
            else {
                return Err(format_invalid_ref(mode, reference));
            };
            let Some(item) = group.matches.first().cloned() else {
                return Err(format_invalid_ref(mode, reference));
            };
            return Ok(ResolvedTarget {
                source_query: Some(query_id),
                source_group: Some(group_index),
                title: group.title,
                item,
            });
        }

        let Some(source) = options.query.as_deref() else {
            return Err(format_invalid_ref(mode, "<missing>"));
        };
        let (path_text, line_number) = match parse_path_line(source) {
            Ok(parsed) => parsed,
            Err(path_line_error) => match parse_line_number(source) {
                Ok(line) => {
                    let Some(path) = single_search_file(options) else {
                        return Err(format_parameter_error(mode, &path_line_error));
                    };
                    (path.to_string_lossy().to_string(), line)
                }
                Err(_) => return Err(format_parameter_error(mode, &path_line_error)),
            },
        };
        let path = resolve_query_path(options, &path_text);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => return Err(format_scope_error(mode, &path, &error.to_string())),
        };
        let Some(line) = content.lines().nth(line_number.saturating_sub(1)) else {
            return Err(format_scope_error(
                mode,
                &path,
                &format!("line {line_number} is outside the file"),
            ));
        };
        let file = display_path_with_base(&path, options.display_base_dir.as_deref());
        let language = options
            .language
            .as_deref()
            .map(normalize_language)
            .or_else(|| infer_language_from_path(&path))
            .unwrap_or_else(|| "unknown".to_string());
        Ok(ResolvedTarget {
            source_query: None,
            source_group: None,
            title: "path:line target".to_string(),
            item: StructuralMatch {
                source_path: Some(path),
                file,
                line: line_number,
                start_column: first_non_whitespace_column(line),
                end_line: line_number,
                end_column: line.chars().count() + 1,
                text: trim_text(line),
                lines: trim_text(line),
                language,
                captures: CaptureSummary::default(),
            },
        })
    }

    fn resolve_rule_test_target(
        &self,
        options: &StructuralSearchOptions,
    ) -> Option<ResolvedTarget> {
        if options.reference.is_some() {
            return self
                .resolve_target(StructuralSearchMode::RuleTest, options)
                .ok();
        }
        let query = options.query.as_deref()?;
        if parse_path_line(query).is_ok() || parse_line_number(query).is_ok() {
            return self
                .resolve_target(StructuralSearchMode::RuleTest, options)
                .ok();
        }
        None
    }
}

async fn run_ast_grep(
    options: &StructuralSearchOptions,
    language: &str,
) -> Result<AstGrepRun, String> {
    let mut command = Command::new("ast-grep");
    if let Some(rule) = &options.rule {
        command.arg("scan");
        command.arg("--inline-rules").arg(rule.to_string());
    } else {
        let Some(pattern) = &options.pattern else {
            return Err("pattern or rule is required".to_string());
        };
        command.arg("run");
        command.arg("-p").arg(pattern);
        command.arg("-l").arg(language);
    }
    command.arg("--json=compact");
    for path in &options.search_paths {
        command.arg(path);
    }

    let output = command
        .output()
        .await
        .map_err(|error| format!("failed to run ast-grep: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() && stdout.trim() != "[]" {
        let message = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(if message.is_empty() {
            "ast-grep failed".to_string()
        } else {
            message
        });
    }
    let matches = serde_json::from_str(stdout.trim())
        .map_err(|error| format!("failed to parse ast-grep JSON: {error}"))?;
    Ok(AstGrepRun {
        matches,
        diagnostics: stderr.to_string(),
    })
}

async fn run_ast_grep_stdin(
    options: &StructuralSearchOptions,
    language: &str,
    source: &str,
) -> Result<AstGrepRun, String> {
    let mut command = StdCommand::new("ast-grep");
    if let Some(rule) = &options.rule {
        command.arg("scan");
        command.arg("--inline-rules").arg(rule.to_string());
    } else {
        let Some(pattern) = &options.pattern else {
            return Err("pattern or rule is required".to_string());
        };
        command.arg("run");
        command.arg("-p").arg(pattern);
        command.arg("-l").arg(language);
    }
    command.arg("--stdin");
    command.arg("--json=compact");
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to run ast-grep: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(source.as_bytes())
            .map_err(|error| format!("failed to write source to ast-grep: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to run ast-grep: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && stdout.trim() != "[]" {
        let message = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(if message.is_empty() {
            "ast-grep failed".to_string()
        } else {
            message
        });
    }
    let matches = serde_json::from_str(stdout.trim())
        .map_err(|error| format!("failed to parse ast-grep JSON: {error}"))?;
    Ok(AstGrepRun {
        matches,
        diagnostics: stderr.to_string(),
    })
}

fn normalize_matches(
    raw_matches: Vec<RawAstGrepMatch>,
    options: &StructuralSearchOptions,
) -> Vec<StructuralMatch> {
    raw_matches
        .into_iter()
        .filter(|raw| options.include_tests || !is_test_path(&raw.file))
        .map(|raw| {
            let raw_path = PathBuf::from(&raw.file);
            let path = display_path_with_base(&raw_path, options.display_base_dir.as_deref());
            let (start_column, end_line, end_column) = display_range(&raw);
            StructuralMatch {
                source_path: Some(raw_path),
                file: path,
                line: raw.range.start.line + 1,
                start_column,
                end_line,
                end_column,
                text: trim_text(&raw.text),
                lines: trim_text(&raw.lines),
                language: raw.language,
                captures: normalize_captures(raw.meta_variables.unwrap_or_default()),
            }
        })
        .collect()
}

fn normalize_stdin_matches(
    raw_matches: Vec<RawAstGrepMatch>,
    target: &StructuralMatch,
) -> Vec<StructuralMatch> {
    raw_matches
        .into_iter()
        .map(|raw| {
            let (start_column, end_line, end_column) = display_range(&raw);
            StructuralMatch {
                source_path: target.source_path.clone(),
                file: target.file.clone(),
                line: target.line + raw.range.start.line,
                start_column,
                end_line: target.line + end_line - 1,
                end_column,
                text: trim_text(&raw.text),
                lines: trim_text(&raw.lines),
                language: target.language.clone(),
                captures: normalize_captures(raw.meta_variables.unwrap_or_default()),
            }
        })
        .collect()
}

fn display_range(raw: &RawAstGrepMatch) -> (usize, usize, usize) {
    let mut lines = raw.lines.lines();
    let first_line = lines.next().unwrap_or(raw.text.as_str());
    let last_line = raw.lines.lines().last().unwrap_or(first_line);
    let start_column = first_non_whitespace_column(first_line);
    let end_line = raw.range.end.line + 1;
    let end_column = last_line.chars().count() + 1;
    (start_column, end_line, end_column)
}

fn normalize_captures(raw: RawMetaVariables) -> CaptureSummary {
    let mut summary = CaptureSummary::default();
    let mut single: Vec<_> = raw.single.into_iter().collect();
    single.sort_by(|a, b| a.0.cmp(&b.0));
    summary.single = single
        .into_iter()
        .map(|(name, node)| (format!("${name}"), trim_capture(&node.text)))
        .collect();

    let mut multi: Vec<_> = raw.multi.into_iter().collect();
    multi.sort_by(|a, b| a.0.cmp(&b.0));
    summary.multi = multi
        .into_iter()
        .map(|(name, nodes)| {
            (
                format!("$$${name}"),
                nodes
                    .into_iter()
                    .map(|node| trim_capture(&node.text))
                    .filter(|text| text != ",")
                    .collect(),
            )
        })
        .collect();

    let mut transformed: Vec<_> = raw.transformed.into_iter().collect();
    transformed.sort_by(|a, b| a.0.cmp(&b.0));
    summary.transformed = transformed
        .into_iter()
        .map(|(name, value)| (format!("${name}"), trim_capture(&value)))
        .collect();
    summary
}

fn group_matches(matches: Vec<StructuralMatch>) -> Vec<StructuralSearchGroup> {
    let mut production = Vec::new();
    let mut tests = Vec::new();
    let mut fixtures = Vec::new();
    let mut generated = Vec::new();
    for item in matches {
        if is_test_path(&item.file) {
            tests.push(item);
        } else if is_fixture_path(&item.file) {
            fixtures.push(item);
        } else if is_generated_path(&item.file) {
            generated.push(item);
        } else {
            production.push(item);
        }
    }
    let mut groups = Vec::new();
    if !production.is_empty() {
        groups.push(StructuralSearchGroup {
            title: "production matches".to_string(),
            matches: production,
        });
    }
    if !tests.is_empty() {
        groups.push(StructuralSearchGroup {
            title: "test matches".to_string(),
            matches: tests,
        });
    }
    if !fixtures.is_empty() {
        groups.push(StructuralSearchGroup {
            title: "fixture matches".to_string(),
            matches: fixtures,
        });
    }
    if !generated.is_empty() {
        groups.push(StructuralSearchGroup {
            title: "generated matches".to_string(),
            matches: generated,
        });
    }
    groups
}

struct FindResultSurface<'a> {
    qid: usize,
    mode: StructuralSearchMode,
    options: &'a StructuralSearchOptions,
    language: &'a str,
    groups: &'a [StructuralSearchGroup],
    status: Option<&'a str>,
    parse_gaps: &'a [String],
    test_source: Option<&'a str>,
}

fn format_find_result(surface: FindResultSurface<'_>) -> String {
    let mut out = String::new();
    let FindResultSurface {
        qid,
        mode,
        options,
        language,
        groups,
        status,
        parse_gaps,
        test_source,
    } = surface;
    out.push_str(&format!("structural_search[{}] q{qid}\n", mode_label(mode)));
    if let Some(query) = &options.query {
        out.push_str(&format!("query: {}\n", query.trim()));
    }
    if let Some(pattern) = &options.pattern {
        out.push_str(&format!("pattern: {}\n", pattern.trim()));
    }
    if let Some(rule) = &options.rule {
        append_rule_summary(&mut out, rule);
    }
    if let Some(test_source) = test_source {
        out.push_str(&format!("test source: {test_source}\n"));
    }
    out.push_str(&format!("language: {language}\n"));
    out.push_str(&format!(
        "scope: {}; tests {}\n",
        format_scope(&options.search_paths, options.display_base_dir.as_deref()),
        if options.include_tests {
            "included"
        } else {
            "excluded"
        }
    ));
    if let Some(status) = status {
        out.push_str(&format!("status: {status}\n"));
        if status == "no-matches" {
            out.push('\n');
            out.push_str("No structural matches found in selected scope.\n\n");
            out.push_str("next:\n");
            out.push_str("- broaden path\n");
            out.push_str("- use search_text for literal candidates\n");
            return out;
        }
    }

    let total_matches: usize = groups.iter().map(|group| group.matches.len()).sum();
    let display_limit = display_limit(options);
    let displayed_groups = groups.len().min(display_limit);
    let displayed_matches: usize = groups
        .iter()
        .take(displayed_groups)
        .map(|group| group.matches.len().min(DEFAULT_MATCHES_PER_GROUP))
        .sum();
    out.push_str(&format!(
        "matches: {displayed_groups} groups, {displayed_matches} displayed matches, {total_matches} known matches\n\n"
    ));
    if !parse_gaps.is_empty() {
        out.push_str("Missing coverage:\n");
        out.push_str(&format!(
            "- {} files could not be parsed as {language}\n",
            parse_gaps.len()
        ));
        for path in parse_gaps.iter().take(3) {
            out.push_str(&format!("  - {path}\n"));
        }
        if parse_gaps.len() > 3 {
            out.push_str(&format!(
                "  - {} more files omitted\n",
                parse_gaps.len() - 3
            ));
        }
        out.push('\n');
    }

    for (index, group) in groups.iter().take(displayed_groups).enumerate() {
        out.push_str(&format!("[{}] {}\n", index + 1, group.title));
        for item in group.matches.iter().take(matches_per_group(options)) {
            out.push_str(&format!(
                "  {}:{}      {}\n",
                item.file, item.line, item.text
            ));
        }
        if let Some(captures) = group_capture_summary(group) {
            out.push_str(&format!("  captures: {captures}\n"));
        }
        if group.matches.len() > matches_per_group(options) {
            out.push_str(&format!(
                "  note: {} more matches not shown in this group\n",
                group.matches.len() - matches_per_group(options)
            ));
        }
        out.push('\n');
    }

    out.push_str("omitted:\n");
    if groups.len() > displayed_groups {
        out.push_str(&format!(
            "- {} groups not shown\n",
            groups.len() - displayed_groups
        ));
    }
    let hidden_matches = total_matches.saturating_sub(displayed_matches);
    if hidden_matches > 0 {
        out.push_str(&format!("- {hidden_matches} matches not shown\n"));
    }
    if groups.len() <= displayed_groups && hidden_matches == 0 {
        out.push_str("- none\n");
    }

    out.push_str("\nnext:\n");
    out.push_str("- expand 1       show matches and captures for the first group\n");
    out.push_str("- around 1       show local structure for the first group\n");
    out
}

fn format_expand_result(
    qid: usize,
    source_query: usize,
    source_group: usize,
    group: &StructuralSearchGroup,
    options: &StructuralSearchOptions,
) -> String {
    let mut out = String::new();
    let total_matches = group.matches.len();
    let display_limit = display_limit(options);
    let displayed_matches = total_matches.min(display_limit);
    out.push_str(&format!("structural_search[expand] q{qid}\n"));
    out.push_str(&format!(
        "from: q{source_query}.[{source_group}] {}\n",
        group.title
    ));
    out.push_str(&format!(
        "matches: {displayed_matches} displayed, {total_matches} total\n\n",
    ));
    for (index, item) in group.matches.iter().take(displayed_matches).enumerate() {
        out.push_str(&format!("[{}] {}:{}\n", index + 1, item.file, item.line));
        out.push_str(&format!("  text: {}\n", item.text));
        out.push_str(&format!("  context: {}\n", item.lines));
        out.push_str("  captures:\n");
        append_capture_lines(&mut out, &item.captures, "    ");
        out.push('\n');
    }
    out.push_str("omitted:\n");
    if total_matches > displayed_matches {
        out.push_str(&format!(
            "- {} matches not shown\n",
            total_matches - displayed_matches
        ));
    } else {
        out.push_str("- none\n");
    }
    out.push('\n');
    out.push_str("next:\n");
    out.push_str("- around 1       show local structure for the first match\n");
    out
}

fn format_around_result(
    qid: usize,
    target: &ResolvedTarget,
    group: &StructuralSearchGroup,
    options: &StructuralSearchOptions,
) -> String {
    let item = &target.item;
    let context = LocalSyntaxContext::build(item);
    let requested = requested_contexts(&options.context);
    let mut out = String::new();
    out.push_str(&format!("structural_search[around] q{qid}\n"));
    if let (Some(source_query), Some(source_group)) = (target.source_query, target.source_group) {
        out.push_str(&format!(
            "from: q{source_query}.[{source_group}] {}\n",
            group.title
        ));
    }
    out.push_str(&format!("source: {}:{}\n\n", item.file, item.line));
    if requested.contains("enclosing") {
        out.push_str("Enclosing\n");
        out.push_str(&format!("  item: {}\n\n", context.enclosing));
    }
    if requested.contains("node") || requested.contains("node_tree") {
        out.push_str("Node\n");
        out.push_str(&format!("  language: {}\n", item.language));
        out.push_str(&format!("  kind: {}\n", context.node_kind));
        out.push_str(&format!("  range: {}\n", context.range));
        out.push_str(&format!("  text: {}\n\n", item.text));
    }
    if requested.contains("siblings") {
        out.push_str("Siblings\n");
        out.push_str(&format!("  previous: {}\n", context.previous));
        out.push_str(&format!("  next: {}\n\n", context.next));
    }
    if requested.contains("children") || requested.contains("node_tree") {
        out.push_str("Children\n");
        out.push_str(&format!("  {}\n\n", context.children));
    }
    if requested.contains("captures") {
        out.push_str("Captures\n");
        append_capture_lines(&mut out, &item.captures, "  ");
    }
    out.push_str("\nnext:\n");
    out.push_str("- read_file around the source line\n");
    out
}

fn format_explain_ast_result(qid: usize, target: &ResolvedTarget) -> String {
    let item = &target.item;
    let context = LocalSyntaxContext::build(item);
    format!(
        "structural_search[explain_ast] q{qid}\nsource: {}:{}\nlanguage: {}\n\n[1] local tree\n  {}\n  node kind: {}\n  range: {}\n  local tree:\n    {}\n    {} {}\n\ncandidate pattern hints\n- {}\n- replace variable subexpressions with metavariables when needed\n\nnext:\n- rule_test with a candidate pattern\n- find with a narrower pattern\n",
        item.file,
        item.line,
        item.language,
        target.source_label(),
        context.node_kind,
        context.range,
        context.enclosing,
        context.node_kind,
        item.text,
        context.pattern_hint,
    )
}

struct LocalSyntaxContext {
    node_kind: String,
    range: String,
    enclosing: String,
    previous: String,
    next: String,
    children: String,
    pattern_hint: String,
}

impl LocalSyntaxContext {
    fn build(item: &StructuralMatch) -> Self {
        let lines = read_source_lines(item);
        let selected_index = item.line.saturating_sub(1);
        let selected_line = lines
            .get(selected_index)
            .map(String::as_str)
            .unwrap_or(item.lines.as_str());
        let node_kind = infer_node_kind(&item.language, &item.text);
        let enclosing = infer_enclosing(&item.language, &lines, selected_index);
        let previous = nearest_non_empty_line_before(&lines, selected_index);
        let next = nearest_non_empty_line_after(&lines, selected_index);
        let children = infer_children(&node_kind, &item.text);
        let pattern_hint = infer_pattern_hint(&node_kind, &item.text);
        let start_column = if item.start_column > 0 {
            item.start_column
        } else {
            first_non_whitespace_column(selected_line)
        };
        let end_line = item.end_line.max(item.line);
        let end_column = if item.end_column > 0 {
            item.end_column
        } else {
            selected_line.chars().count() + 1
        };
        Self {
            node_kind,
            range: format!("{}:{start_column}-{end_line}:{end_column}", item.line),
            enclosing,
            previous,
            next,
            children,
            pattern_hint,
        }
    }
}

fn append_capture_lines(out: &mut String, captures: &CaptureSummary, indent: &str) {
    let mut written = 0;
    let total = captures.single.len() + captures.multi.len() + captures.transformed.len();
    for (name, value) in captures
        .single
        .iter()
        .take(MAX_CAPTURE_LINES.saturating_sub(written))
    {
        out.push_str(&format!("{indent}{name} = {value}\n"));
        written += 1;
    }
    for (name, values) in captures
        .multi
        .iter()
        .take(MAX_CAPTURE_LINES.saturating_sub(written))
    {
        out.push_str(&format!("{indent}{name} = {}\n", values.join(", ")));
        written += 1;
    }
    for (name, value) in captures
        .transformed
        .iter()
        .take(MAX_CAPTURE_LINES.saturating_sub(written))
    {
        out.push_str(&format!("{indent}{name} = {value}\n"));
        written += 1;
    }
    if written == 0 {
        out.push_str(&format!("{indent}none\n"));
    } else if total > written {
        out.push_str(&format!("{indent}{} captures not shown\n", total - written));
    }
}

fn group_capture_summary(group: &StructuralSearchGroup) -> Option<String> {
    let first = group.matches.first()?;
    let mut parts = Vec::new();
    for (name, value) in &first.captures.single {
        parts.push(format!("{name} = {value}"));
    }
    for (name, values) in &first.captures.multi {
        parts.push(format!("{name} has {} observed nodes", values.len()));
    }
    for (name, value) in &first.captures.transformed {
        parts.push(format!("{name} = {value}"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

fn requested_contexts(context: &[String]) -> BTreeSet<&'static str> {
    let mut requested = BTreeSet::new();
    if context.is_empty() {
        requested.extend(["enclosing", "node", "siblings", "children", "captures"]);
        return requested;
    }
    for item in context {
        match item.as_str() {
            "captures" => {
                requested.insert("captures");
            }
            "enclosing" => {
                requested.insert("enclosing");
            }
            "siblings" => {
                requested.insert("siblings");
            }
            "children" => {
                requested.insert("children");
                requested.insert("node");
            }
            "node" => {
                requested.insert("node");
            }
            "node_tree" => {
                requested.insert("node_tree");
                requested.insert("node");
                requested.insert("children");
            }
            _ => {}
        }
    }
    requested
}

fn read_source_lines(item: &StructuralMatch) -> Vec<String> {
    item.source_path
        .as_ref()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|content| content.lines().map(ToString::to_string).collect())
        .unwrap_or_else(|| item.lines.lines().map(ToString::to_string).collect())
}

fn infer_node_kind(language: &str, text: &str) -> String {
    let trimmed = text.trim();
    match normalize_language(language).as_str() {
        "rust" => {
            if trimmed.starts_with("impl ") {
                "impl_item".to_string()
            } else if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                "function_item".to_string()
            } else if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
                "mod_item".to_string()
            } else if trimmed.contains("=>") {
                "match_arm".to_string()
            } else if looks_like_call(trimmed) {
                "call_expression".to_string()
            } else {
                "syntax_node".to_string()
            }
        }
        "typescript" | "javascript" => {
            if trimmed.starts_with("function ") {
                "function_declaration".to_string()
            } else if looks_like_call(trimmed) {
                "call_expression".to_string()
            } else {
                "syntax_node".to_string()
            }
        }
        "python" => {
            if trimmed.starts_with("def ") {
                "function_definition".to_string()
            } else if looks_like_call(trimmed) {
                "call".to_string()
            } else {
                "syntax_node".to_string()
            }
        }
        _ => "syntax_node".to_string(),
    }
}

fn infer_enclosing(language: &str, lines: &[String], selected_index: usize) -> String {
    let normalized = normalize_language(language);
    for line in lines.iter().take(selected_index + 1).rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match normalized.as_str() {
            "rust" => {
                if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                    return format!("function_item {}", strip_after(trimmed, '{'));
                }
                if trimmed.starts_with("impl ") {
                    return format!("impl_item {}", strip_after(trimmed, '{'));
                }
                if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
                    return format!("mod_item {}", strip_after(trimmed, '{'));
                }
            }
            "typescript" | "javascript" if trimmed.starts_with("function ") => {
                return format!("function_declaration {}", strip_after(trimmed, '{'));
            }
            "python" if trimmed.starts_with("def ") => {
                return format!("function_definition {}", strip_after(trimmed, ':'));
            }
            _ => {}
        }
    }
    "none".to_string()
}

fn nearest_non_empty_line_before(lines: &[String], selected_index: usize) -> String {
    lines
        .iter()
        .take(selected_index)
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && *line != "{" && *line != "}")
        .unwrap_or("none")
        .to_string()
}

fn nearest_non_empty_line_after(lines: &[String], selected_index: usize) -> String {
    lines
        .iter()
        .skip(selected_index + 1)
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && *line != "{" && *line != "}")
        .unwrap_or("none")
        .to_string()
}

fn infer_children(node_kind: &str, text: &str) -> String {
    if node_kind == "call_expression"
        && let Some(arguments) = call_arguments(text)
    {
        return format!("arguments: {arguments}");
    }
    "none".to_string()
}

fn infer_pattern_hint(node_kind: &str, text: &str) -> String {
    if node_kind == "call_expression"
        && let Some(name) = call_name(text)
    {
        return format!("{name}($$$ARGS)");
    }
    text.to_string()
}

fn append_rule_summary(out: &mut String, rule: &Value) {
    out.push_str(&format!("rule: {}\n", summarize_rule(rule)));
    for relation in summarize_relations(rule) {
        out.push_str(&format!("relation: {relation}\n"));
    }
    for fact in summarize_relation_facts(rule) {
        out.push_str(&format!("{fact}\n"));
    }
    if let Some(constraints) = rule.get("constraints").and_then(Value::as_object) {
        let mut names: Vec<_> = constraints.keys().cloned().collect();
        names.sort();
        for name in names {
            let summary = constraints
                .get(&name)
                .map(compact_json)
                .unwrap_or_else(|| "{}".to_string());
            out.push_str(&format!("constraints: ${name} restricted by {summary}\n"));
        }
    }
    if let Some(utils) = rule.get("utils").and_then(Value::as_object) {
        let mut names: Vec<_> = utils.keys().cloned().collect();
        names.sort();
        if !names.is_empty() {
            out.push_str(&format!("utils: {}\n", names.join(", ")));
        }
    }
}

fn summarize_relation_facts(rule: &Value) -> Vec<String> {
    let mut facts = BTreeSet::new();
    let body = rule.get("rule").unwrap_or(rule);
    collect_relation_facts(body, &mut facts);
    facts.into_iter().collect()
}

fn collect_relation_facts(value: &Value, facts: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(inside) = map.get("inside")
                && let Some(kind) = rule_kind(inside)
            {
                facts.insert(format!(
                    "context: {} inside {kind}",
                    rule_node_summary(value)
                ));
            }
            if let Some(has) = map.get("has") {
                facts.insert(format!(
                    "relation: {} has {}",
                    rule_node_summary(value),
                    rule_pattern_or_summary(has)
                ));
            }
            for value in map.values() {
                collect_relation_facts(value, facts);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_relation_facts(item, facts);
            }
        }
        _ => {}
    }
}

fn rule_kind(value: &Value) -> Option<String> {
    value
        .get("kind")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn rule_node_summary(value: &Value) -> String {
    if let Some(kind) = value.get("kind").and_then(Value::as_str) {
        return kind.to_string();
    }
    if let Some(pattern) = value.get("pattern").and_then(Value::as_str) {
        if looks_like_call(pattern) {
            return "call_expression".to_string();
        }
        return pattern.to_string();
    }
    summarize_rule(value)
}

fn rule_pattern_or_summary(value: &Value) -> String {
    value
        .get("pattern")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| rule_node_summary(value))
}

fn summarize_relations(rule: &Value) -> Vec<String> {
    let mut relations = BTreeSet::new();
    let body = rule.get("rule").unwrap_or(rule);
    collect_relation_parts(body, &mut relations);
    relations.into_iter().collect()
}

fn collect_relation_parts(value: &Value, relations: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(key.as_str(), "inside" | "has" | "precedes" | "follows") {
                    relations.insert(key.clone());
                }
                collect_relation_parts(value, relations);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_relation_parts(item, relations);
            }
        }
        _ => {}
    }
}

fn summarize_rule(rule: &Value) -> String {
    let mut parts = BTreeSet::new();
    let body = rule.get("rule").unwrap_or(rule);
    collect_rule_parts(body, &mut parts);
    if parts.is_empty() {
        "inline rule object".to_string()
    } else {
        parts.into_iter().collect::<Vec<_>>().join(" + ")
    }
}

fn collect_rule_parts(value: &Value, parts: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if is_rule_key(key) {
                    parts.insert(key.clone());
                }
                collect_rule_parts(value, parts);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_rule_parts(item, parts);
            }
        }
        _ => {}
    }
}

fn is_rule_key(key: &str) -> bool {
    matches!(
        key,
        "pattern"
            | "kind"
            | "regex"
            | "nthChild"
            | "range"
            | "inside"
            | "has"
            | "precedes"
            | "follows"
            | "all"
            | "any"
            | "not"
            | "matches"
    )
}

fn format_unsupported_rule_field(mode: StructuralSearchMode, rule: &Value) -> String {
    format!(
        "structural_search[{}]\nstatus: unsupported-rule-field\n\nProblem\n  Query-only structural_search does not accept edit fields in rule input.\n\nrule: {}\n\nnext:\n- remove edit fields and rerun the query\n",
        mode_label(mode),
        compact_json(rule)
    )
}

fn format_invalid_pattern(
    mode: StructuralSearchMode,
    options: &StructuralSearchOptions,
    language: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "structural_search[{}]\nstatus: invalid-pattern\nlanguage: {language}\n",
        mode_label(mode)
    ));
    if let Some(pattern) = &options.pattern {
        out.push_str(&format!("pattern: {}\n", pattern.trim()));
    }
    if let Some(rule) = &options.rule {
        append_rule_summary(&mut out, rule);
    }
    out.push_str(
        "\nProblem\n  Pattern cannot be parsed cleanly for the selected language.\n\nnext:\n- explain_ast on a representative line\n- simplify the pattern and retry\n",
    );
    out
}

fn format_unavailable(mode: StructuralSearchMode, message: &str) -> String {
    format!(
        "structural_search[{}]\nstatus: unavailable\n\nProblem\n  ast-grep backend could not complete the query: {message}\n\nnext:\n- verify ast-grep is installed\n- simplify the query and retry\n",
        mode_label(mode)
    )
}

fn format_workspace_required(
    mode: StructuralSearchMode,
    options: &StructuralSearchOptions,
) -> String {
    let query = options.query.as_deref().unwrap_or("<none>");
    format!(
        "structural_search[{}]\nstatus: workspace-required\nquery: {query}\n\nNo docutouch workspace is set and no resolvable path was provided.\n\nnext:\n- set docutouch workspace\n- call structural_search with path\n",
        mode_label(mode)
    )
}

fn format_scope_error(mode: StructuralSearchMode, path: &Path, message: &str) -> String {
    format!(
        "structural_search[{}]\nstatus: scope-error\npath: {}\n\nProblem\n  Selected scope is not readable: {message}\n\nnext:\n- choose an existing file or directory\n- narrow path to a readable source scope\n",
        mode_label(mode),
        path.display()
    )
}

fn format_parameter_error(mode: StructuralSearchMode, message: &str) -> String {
    format!(
        "structural_search[{}]\nstatus: parameter-error\n\nProblem\n  {message}\n\nnext:\n- correct the structural_search arguments\n",
        mode_label(mode)
    )
}

fn format_invalid_ref(mode: StructuralSearchMode, reference: &str) -> String {
    format!(
        "structural_search[{}]\nstatus: invalid-ref\nref: {reference}\n\nNo registered structural_search result matches this reference in the current MCP connection.\n\nnext:\n- use a visible qN.N reference from earlier structural_search output\n- run a new find query\n",
        mode_label(mode)
    )
}

fn validate_search_paths(
    mode: StructuralSearchMode,
    options: &StructuralSearchOptions,
) -> Option<String> {
    for path in &options.search_paths {
        if !path.exists() {
            return Some(format_scope_error(mode, path, "path does not exist"));
        }
        if path.is_dir() {
            if let Err(error) = fs::read_dir(path) {
                return Some(format_scope_error(mode, path, &error.to_string()));
            }
        } else if let Err(error) = fs::File::open(path) {
            return Some(format_scope_error(mode, path, &error.to_string()));
        }
    }
    None
}

fn validate_common_parameters(options: &StructuralSearchOptions) -> Option<String> {
    if options.limit == Some(0) {
        return Some("limit must be greater than 0".to_string());
    }
    if let Some(limit) = options.limit
        && limit > MAX_DISPLAY_LIMIT
    {
        return Some(format!("limit must be at most {MAX_DISPLAY_LIMIT}"));
    }
    for context in &options.context {
        if !matches!(
            context.as_str(),
            "captures" | "enclosing" | "siblings" | "children" | "node" | "node_tree"
        ) {
            return Some(format!("unsupported context: {context}"));
        }
    }
    None
}

fn resolve_language(
    mode: StructuralSearchMode,
    options: &StructuralSearchOptions,
) -> Result<String, String> {
    if let Some(language) = &options.language {
        let normalized = normalize_language(language);
        if !is_supported_language(&normalized) {
            return Err(format_unsupported_language(mode, &normalized));
        }
        return Ok(normalized);
    }
    let inferred = infer_language_counts_from_paths(&options.search_paths);
    if inferred.is_empty() {
        return Err(format_unsupported_language(mode, "<unrecognized>"));
    }
    if inferred.len() == 1 {
        return Ok(inferred.keys().next().expect("one language").clone());
    }
    Err(format_ambiguous_language(&inferred))
}

fn infer_language_counts_from_paths(paths: &[PathBuf]) -> BTreeMap<String, usize> {
    let mut languages = BTreeMap::new();
    for path in paths {
        collect_languages(path, &mut languages);
    }
    languages
}

fn collect_languages(path: &Path, languages: &mut BTreeMap<String, usize>) {
    if path.is_file() {
        if let Some(language) = infer_language_from_path(path) {
            *languages.entry(language).or_default() += 1;
        }
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let child = entry.path();
        if child.is_dir() {
            collect_languages(&child, languages);
        } else if let Some(language) = infer_language_from_path(&child) {
            *languages.entry(language).or_default() += 1;
        }
    }
}

fn collect_parse_gaps(
    paths: &[PathBuf],
    language: &str,
    include_tests: bool,
    display_base_dir: Option<&Path>,
) -> Vec<String> {
    let mut gaps = Vec::new();
    for path in paths {
        collect_parse_gap_paths(path, language, include_tests, display_base_dir, &mut gaps);
    }
    gaps.sort();
    gaps.dedup();
    gaps
}

fn collect_parse_gap_paths(
    path: &Path,
    language: &str,
    include_tests: bool,
    display_base_dir: Option<&Path>,
    gaps: &mut Vec<String>,
) {
    if path.is_file() {
        let display = display_path_with_base(path, display_base_dir);
        if !include_tests && is_test_path(&display) {
            return;
        }
        if let Some(file_language) = infer_language_from_path(path)
            && file_language != language
        {
            gaps.push(display);
        }
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_parse_gap_paths(
            &entry.path(),
            language,
            include_tests,
            display_base_dir,
            gaps,
        );
    }
}

fn infer_language_from_path(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match ext.as_str() {
        "rs" => Some("rust".to_string()),
        "ts" | "tsx" => Some("typescript".to_string()),
        "js" | "jsx" => Some("javascript".to_string()),
        "py" => Some("python".to_string()),
        _ => None,
    }
}

fn pattern_has_error_node(diagnostics: &str) -> bool {
    diagnostics.contains("Pattern contains an ERROR node")
}

fn is_supported_language(language: &str) -> bool {
    matches!(
        language,
        "rust" | "typescript" | "javascript" | "python" | "go" | "java" | "c" | "cpp" | "csharp"
    )
}

fn normalize_language(language: &str) -> String {
    match language.to_ascii_lowercase().as_str() {
        "rs" | "rust" => "rust".to_string(),
        "ts" | "tsx" | "typescript" => "typescript".to_string(),
        "js" | "jsx" | "javascript" => "javascript".to_string(),
        "py" | "python" => "python".to_string(),
        other => other.to_string(),
    }
}

fn format_ambiguous_language(languages: &BTreeMap<String, usize>) -> String {
    let mut out = String::new();
    out.push_str("structural_search[find]\nstatus: ambiguous-language\n\n");
    out.push_str("Language could not be selected from the current scope.\n\n");
    out.push_str("candidates:\n");
    for (index, (language, count)) in languages.iter().enumerate() {
        out.push_str(&format!("[{}] {language}: {count} files\n", index + 1));
    }
    out.push_str("\nnext:\n");
    out.push_str("- rerun with language set explicitly\n");
    out.push_str("- rerun with a narrower path\n");
    out
}

fn format_unsupported_language(mode: StructuralSearchMode, language: &str) -> String {
    format!(
        "structural_search[{}]\nstatus: unsupported-language\nlanguage: {language}\n\nLanguage is not supported by structural_search.\n\nnext:\n- rerun with a supported language\n- narrow path to files with an inferable language\n",
        mode_label(mode)
    )
}

fn contains_unsupported_edit_field(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            matches!(
                key.as_str(),
                "fix" | "rewrite" | "replacement" | "apply" | "autofix" | "transform"
            ) || contains_unsupported_edit_field(value)
        }),
        Value::Array(items) => items.iter().any(contains_unsupported_edit_field),
        _ => false,
    }
}

fn normalize_rule_input(rule: Option<Value>) -> Result<Option<Value>, String> {
    let Some(rule) = rule else {
        return Ok(None);
    };
    match rule {
        Value::Object(_) => Ok(Some(rule)),
        Value::String(text) => {
            let parsed: Value = serde_json::from_str(&text).map_err(|_| {
                "rule must be a JSON object; pass rule directly instead of YAML or an encoded JSON string".to_string()
            })?;
            if parsed.is_object() {
                Ok(Some(parsed))
            } else {
                Err("rule must be a JSON object; pass rule directly instead of an encoded JSON scalar or array".to_string())
            }
        }
        _ => Err(
            "rule must be a JSON object; pass rule directly instead of a string, array, or scalar"
                .to_string(),
        ),
    }
}

fn single_search_file(options: &StructuralSearchOptions) -> Option<&Path> {
    match options.search_paths.as_slice() {
        [path] if path.is_file() => Some(path.as_path()),
        _ => None,
    }
}

fn is_test_path(path: &str) -> bool {
    let lowered = path.replace('\\', "/").to_ascii_lowercase();
    lowered.contains("/tests/")
        || lowered.contains("/test/")
        || lowered.contains("/__tests__/")
        || lowered.ends_with("/tests.rs")
        || lowered.ends_with("/test.rs")
        || lowered.ends_with("_test.rs")
        || lowered.ends_with("_tests.rs")
        || lowered.ends_with("_spec.rs")
        || lowered.ends_with("_specs.rs")
        || lowered.contains(".test.")
        || lowered.contains(".spec.")
        || lowered
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("test_") && name.ends_with(".py"))
}

fn is_fixture_path(path: &str) -> bool {
    let lowered = path.replace('\\', "/").to_ascii_lowercase();
    lowered.contains("/fixtures/")
        || lowered.contains("/fixture/")
        || lowered.contains("/testdata/")
        || lowered.contains("/test-data/")
        || lowered.contains("_fixture.")
}

fn is_generated_path(path: &str) -> bool {
    let lowered = path.replace('\\', "/").to_ascii_lowercase();
    lowered.contains("/generated/")
        || lowered.contains("/gen/")
        || lowered.contains("/target/")
        || lowered.contains(".generated.")
        || lowered.ends_with("_generated.rs")
}

fn display_path_with_base(path: &Path, base: Option<&Path>) -> String {
    if let Some(base) = base {
        display_path(Some(base), path)
    } else {
        display_path(None, path)
    }
}

fn display_limit(options: &StructuralSearchOptions) -> usize {
    match options.view {
        StructuralSearchView::Preview => options.limit.unwrap_or(DEFAULT_LIMIT),
        StructuralSearchView::Full => MAX_DISPLAY_LIMIT,
    }
}

fn matches_per_group(options: &StructuralSearchOptions) -> usize {
    match options.view {
        StructuralSearchView::Preview => DEFAULT_MATCHES_PER_GROUP,
        StructuralSearchView::Full => MAX_DISPLAY_LIMIT,
    }
}

fn mode_label(mode: StructuralSearchMode) -> &'static str {
    match mode {
        StructuralSearchMode::Find => "find",
        StructuralSearchMode::Expand => "expand",
        StructuralSearchMode::Around => "around",
        StructuralSearchMode::ExplainAst => "explain_ast",
        StructuralSearchMode::RuleTest => "rule_test",
    }
}

fn trim_text(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    trim_capture(&collapsed)
}

fn trim_capture(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= MAX_CAPTURE_WIDTH {
        return trimmed.to_string();
    }
    let prefix: String = trimmed.chars().take(MAX_CAPTURE_WIDTH).collect();
    format!(
        "{prefix}...[{} chars omitted]",
        trimmed.chars().count() - MAX_CAPTURE_WIDTH
    )
}

fn strip_after(text: &str, delimiter: char) -> String {
    text.split(delimiter)
        .next()
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn looks_like_call(text: &str) -> bool {
    call_name(text).is_some()
}

fn call_name(text: &str) -> Option<String> {
    let before_paren = text.split_once('(')?.0.trim_end();
    let name = before_paren
        .rsplit(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == ':'))
        .next()?
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn call_arguments(text: &str) -> Option<String> {
    let start = text.find('(')?;
    let end = text.rfind(')')?;
    if end <= start {
        return None;
    }
    Some(text[start + 1..end].trim().to_string())
}

fn first_non_whitespace_column(line: &str) -> usize {
    line.chars()
        .position(|ch| !ch.is_whitespace())
        .map(|index| index + 1)
        .unwrap_or(1)
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unprintable rule>".to_string())
}

fn parse_path_line(source: &str) -> Result<(String, usize), String> {
    let (path, line) = source
        .rsplit_once(':')
        .ok_or_else(|| "query must use path:line".to_string())?;
    let line = parse_line_number(line)?;
    Ok((path.to_string(), line))
}

fn parse_line_number(source: &str) -> Result<usize, String> {
    let line = source
        .parse()
        .map_err(|_| "query line must be a positive integer".to_string())?;
    if line == 0 {
        return Err("query line must be a positive integer".to_string());
    }
    Ok(line)
}

fn resolve_query_path(options: &StructuralSearchOptions, path_text: &str) -> PathBuf {
    let path = PathBuf::from(path_text);
    if path.is_absolute() {
        path
    } else if let Some(base) = options.display_base_dir.as_ref() {
        base.join(path)
    } else {
        path
    }
}
