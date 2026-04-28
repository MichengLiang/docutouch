use crate::path_display::{display_path, format_scope};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;

const DEFAULT_LIMIT: usize = 8;
const DEFAULT_MATCHES_PER_GROUP: usize = 3;
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
    file: String,
    line: usize,
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
        match options.mode {
            StructuralSearchMode::Find => self.find(options, StructuralSearchMode::Find).await,
            StructuralSearchMode::RuleTest => {
                self.find(options, StructuralSearchMode::RuleTest).await
            }
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
        if options.limit == Some(0) {
            return Ok(format_parameter_error(mode, "limit must be greater than 0"));
        }

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
        Ok(format_find_result(
            qid,
            mode,
            &options,
            &language,
            &groups,
            status,
            &parse_gaps,
        ))
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
        Ok(format_around_result(qid, &target, &group))
    }

    async fn explain_ast(&mut self, options: StructuralSearchOptions) -> Result<String, String> {
        let target = match self.resolve_target(StructuralSearchMode::ExplainAst, &options) {
            Ok(target) => target,
            Err(message) => return Ok(message),
        };
        let qid = self.register(StructuralSearchMode::ExplainAst, Vec::new());
        Ok(format!(
            "structural_search[explain_ast] q{qid}\nsource: {}:{}\nlanguage: {}\n\n[1] local tree\n  {}:{}      {}\n  node kind: line_context\n  local tree: selected line within nearest parsed syntax context\n\ncandidate pattern hints\n- use the visible call or item text as a starter pattern\n- replace variable subexpressions with metavariables when needed\n\nnext:\n- rule_test with a candidate pattern\n- find with a narrower pattern\n",
            target.item.file,
            target.item.line,
            target.item.language,
            target.item.file,
            target.item.line,
            target.item.text
        ))
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
            Err(message) => return Err(format_parameter_error(mode, &message)),
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
                file,
                line: line_number,
                text: trim_text(line),
                lines: trim_text(line),
                language,
                captures: CaptureSummary::default(),
            },
        })
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

fn normalize_matches(
    raw_matches: Vec<RawAstGrepMatch>,
    options: &StructuralSearchOptions,
) -> Vec<StructuralMatch> {
    raw_matches
        .into_iter()
        .filter(|raw| options.include_tests || !is_test_path(&raw.file))
        .map(|raw| {
            let path =
                display_path_with_base(Path::new(&raw.file), options.display_base_dir.as_deref());
            StructuralMatch {
                file: path,
                line: raw.range.start.line + 1,
                text: trim_text(&raw.text),
                lines: trim_text(&raw.lines),
                language: raw.language,
                captures: normalize_captures(raw.meta_variables.unwrap_or_default()),
            }
        })
        .collect()
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
    for item in matches {
        if is_test_path(&item.file) {
            tests.push(item);
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
    groups
}

fn format_find_result(
    qid: usize,
    mode: StructuralSearchMode,
    options: &StructuralSearchOptions,
    language: &str,
    groups: &[StructuralSearchGroup],
    status: Option<&str>,
    parse_gaps: &[String],
) -> String {
    let mut out = String::new();
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
    let display_limit = match options.view {
        StructuralSearchView::Preview => options.limit.unwrap_or(DEFAULT_LIMIT),
        StructuralSearchView::Full => usize::MAX,
    };
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
) -> String {
    let item = &target.item;
    let mut out = String::new();
    out.push_str(&format!("structural_search[around] q{qid}\n"));
    if let (Some(source_query), Some(source_group)) = (target.source_query, target.source_group) {
        out.push_str(&format!(
            "from: q{source_query}.[{source_group}] {}\n",
            group.title
        ));
    }
    out.push_str(&format!("source: {}:{}\n\n", item.file, item.line));
    out.push_str("Enclosing\n");
    out.push_str("  item: nearest enclosing syntax item\n\n");
    out.push_str("Node\n");
    out.push_str(&format!("  language: {}\n", item.language));
    out.push_str("  kind: line_context\n");
    out.push_str(&format!("  text: {}\n\n", item.text));
    out.push_str("Siblings\n");
    out.push_str("  summary: sibling extraction is backend-dependent\n\n");
    out.push_str("Children\n");
    out.push_str("  summary: direct child extraction is backend-dependent\n\n");
    out.push_str("Captures\n");
    append_capture_lines(&mut out, &item.captures, "  ");
    out.push_str("\nnext:\n");
    out.push_str("- read_file around the source line\n");
    out
}

fn append_capture_lines(out: &mut String, captures: &CaptureSummary, indent: &str) {
    let mut wrote = false;
    for (name, value) in &captures.single {
        out.push_str(&format!("{indent}{name} = {value}\n"));
        wrote = true;
    }
    for (name, values) in &captures.multi {
        out.push_str(&format!("{indent}{name} = {}\n", values.join(", ")));
        wrote = true;
    }
    for (name, value) in &captures.transformed {
        out.push_str(&format!("{indent}{name} = {value}\n"));
        wrote = true;
    }
    if !wrote {
        out.push_str(&format!("{indent}none\n"));
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

fn append_rule_summary(out: &mut String, rule: &Value) {
    out.push_str(&format!("rule: {}\n", summarize_rule(rule)));
    for relation in summarize_relations(rule) {
        out.push_str(&format!("relation: {relation}\n"));
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
    let inferred = infer_languages_from_paths(&options.search_paths)
        .into_iter()
        .collect::<Vec<_>>();
    match inferred.as_slice() {
        [language] => Ok(language.clone()),
        [] => Err(format_unsupported_language(mode, "<unrecognized>")),
        _ => Err(format_ambiguous_language(&inferred)),
    }
}

fn infer_languages_from_paths(paths: &[PathBuf]) -> BTreeSet<String> {
    let mut languages = BTreeSet::new();
    for path in paths {
        collect_languages(path, &mut languages);
    }
    languages
}

fn collect_languages(path: &Path, languages: &mut BTreeSet<String>) {
    if path.is_file() {
        if let Some(language) = infer_language_from_path(path) {
            languages.insert(language);
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
            languages.insert(language);
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

fn format_ambiguous_language(languages: &[String]) -> String {
    let mut out = String::new();
    out.push_str("structural_search[find]\nstatus: ambiguous-language\n\n");
    out.push_str("Language could not be selected from the current scope.\n\n");
    out.push_str("candidates:\n");
    for (index, language) in languages.iter().enumerate() {
        out.push_str(&format!("[{}] {language}\n", index + 1));
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
        StructuralSearchView::Full => usize::MAX,
    }
}

fn matches_per_group(options: &StructuralSearchOptions) -> usize {
    match options.view {
        StructuralSearchView::Preview => DEFAULT_MATCHES_PER_GROUP,
        StructuralSearchView::Full => usize::MAX,
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

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unprintable rule>".to_string())
}

fn parse_path_line(source: &str) -> Result<(String, usize), String> {
    let (path, line) = source
        .rsplit_once(':')
        .ok_or_else(|| "query must use path:line".to_string())?;
    let line = line
        .parse()
        .map_err(|_| "query line must be a positive integer".to_string())?;
    Ok((path.to_string(), line))
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
