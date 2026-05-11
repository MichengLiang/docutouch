use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::types::{Types, TypesBuilder};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use time::UtcOffset;
use time::format_description::well_known::Rfc3339;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampField {
    Created,
    Modified,
}

#[derive(Clone, Debug, Default)]
pub struct DirectoryListOptions {
    pub max_depth: usize,
    pub show_hidden: bool,
    pub include_gitignored: bool,
    pub file_types: Vec<String>,
    pub file_types_not: Vec<String>,
    pub timestamp_fields: Vec<TimestampField>,
    pub root_display: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DirectoryListResult {
    pub tree: String,
    pub dir_count: usize,
    pub file_count: usize,
    pub filtered_hidden_count: usize,
    pub filtered_gitignored_count: usize,
    pub filtered_both_count: usize,
    pub filtered_type_count: usize,
    pub warnings: Vec<String>,
}

impl DirectoryListResult {
    pub fn display(&self) -> String {
        let dir_word = if self.dir_count == 1 {
            "directory"
        } else {
            "directories"
        };
        let file_word = if self.file_count == 1 {
            "file"
        } else {
            "files"
        };
        let stats = format!(
            "{} {}, {} {}",
            self.dir_count, dir_word, self.file_count, file_word
        );
        let filtered_total = self.filtered_hidden_count
            + self.filtered_gitignored_count
            + self.filtered_both_count
            + self.filtered_type_count;
        if filtered_total == 0 {
            let mut display = format!("{}\n{}", self.tree, stats);
            append_warnings(&mut display, &self.warnings);
            return display;
        }

        let mut filtered_parts = vec![
            format!("{} hidden", self.filtered_hidden_count),
            format!("{} gitignored", self.filtered_gitignored_count),
            format!("{} both", self.filtered_both_count),
        ];
        if self.filtered_type_count > 0 {
            filtered_parts.push(format!("{} type", self.filtered_type_count));
        }

        let mut display = format!(
            "{}\n{}\nfiltered: {} entries ({})",
            self.tree,
            stats,
            filtered_total,
            filtered_parts.join(", "),
        );
        append_warnings(&mut display, &self.warnings);
        display
    }
}

fn append_warnings(display: &mut String, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    display.push_str("\nwarnings:");
    for warning in warnings {
        display.push_str("\n- ");
        display.push_str(warning);
    }
}

#[derive(Clone, Debug)]
pub struct ReadFileResult {
    pub content: String,
    pub file_path: String,
    pub start_line: usize,
    pub line_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadFileLineRange {
    Closed {
        start: usize,
        end: usize,
    },
    SliceLike {
        start: Option<i64>,
        stop: Option<i64>,
    },
}

impl From<(usize, usize)> for ReadFileLineRange {
    fn from((start, end): (usize, usize)) -> Self {
        Self::Closed { start, end }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReadFileOptions {
    pub line_range: Option<ReadFileLineRange>,
    pub show_line_numbers: bool,
}

#[derive(Clone, Debug)]
pub struct ReadFileSampledViewOptions {
    pub sample_step: usize,
    pub sample_lines: usize,
}

pub fn normalize_sampled_view_options(
    sample_step: Option<usize>,
    sample_lines: Option<usize>,
) -> Result<Option<ReadFileSampledViewOptions>, String> {
    const DEFAULT_SAMPLE_STEP: usize = 5;
    const DEFAULT_SAMPLE_LINES: usize = 2;

    if sample_step.is_none() && sample_lines.is_none() {
        return Ok(None);
    }

    let sample_step = sample_step
        .unwrap_or_else(|| DEFAULT_SAMPLE_STEP.max(sample_lines.unwrap_or(0).saturating_add(1)));
    let sample_lines = sample_lines.unwrap_or_else(|| {
        if sample_step > 1 {
            DEFAULT_SAMPLE_LINES.min(sample_step - 1)
        } else {
            DEFAULT_SAMPLE_LINES
        }
    });

    Ok(Some(ReadFileSampledViewOptions {
        sample_step,
        sample_lines,
    }))
}

pub fn parse_read_file_line_range_text(text: &str) -> Result<ReadFileLineRange, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("line_range cannot be empty".to_string());
    }
    let normalized = trimmed.trim_matches(&['[', ']'][..]).trim();

    if normalized.contains(':') {
        return parse_slice_like_line_range_text(normalized);
    }
    if normalized.contains(',') {
        return parse_closed_line_range_text(normalized, ',');
    }
    if is_legacy_hyphen_closed_range(normalized) {
        return parse_closed_line_range_text(normalized, '-');
    }

    Err("line_range must use `start:stop`; this slice-like form supports omitted bounds and negative tail offsets such as `:50`, `50:`, `-50:`, or `:-1`".to_string())
}

#[derive(Clone, Debug)]
struct GitIgnoreMatcher {
    base_dir: PathBuf,
    spec: Gitignore,
}

pub fn list_directory(
    dir_path: &Path,
    options: DirectoryListOptions,
) -> std::io::Result<DirectoryListResult> {
    if !dir_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path does not exist: {}", dir_path.display()),
        ));
    }
    if !dir_path.is_dir() {
        return Err(std::io::Error::other(format!(
            "Path is not a directory: {}",
            dir_path.display()
        )));
    }

    let max_depth = if options.max_depth == 0 {
        3
    } else {
        options.max_depth
    };
    let type_filter = build_type_filter(&options)?;
    let repo_root = find_git_repo_root(dir_path);
    let mut matcher_cache: HashMap<PathBuf, Vec<GitIgnoreMatcher>> = HashMap::new();
    let mut lines = vec![format!("{}/", display_root_dir_name(dir_path, &options))];
    let mut counts = Counts::default();

    walk_directory(
        dir_path,
        "",
        1,
        max_depth,
        &options,
        type_filter.matcher.as_ref(),
        repo_root.as_deref(),
        &mut matcher_cache,
        &mut lines,
        &mut counts,
    );

    Ok(DirectoryListResult {
        tree: lines.join("\n"),
        dir_count: counts.dir_count,
        file_count: counts.file_count,
        filtered_hidden_count: counts.filtered_hidden_count,
        filtered_gitignored_count: counts.filtered_gitignored_count,
        filtered_both_count: counts.filtered_both_count,
        filtered_type_count: counts.filtered_type_count,
        warnings: type_filter.warnings,
    })
}

pub fn read_file(file_path: &Path, options: ReadFileOptions) -> std::io::Result<ReadFileResult> {
    read_file_with_sampled_view(file_path, options, None)
}

pub fn read_file_with_sampled_view(
    file_path: &Path,
    options: ReadFileOptions,
    sampled_view: Option<ReadFileSampledViewOptions>,
) -> std::io::Result<ReadFileResult> {
    if !file_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File does not exist: {}", file_path.display()),
        ));
    }
    if file_path.is_dir() {
        return Err(std::io::Error::other(format!(
            "Path is a directory, not a file: {}",
            file_path.display()
        )));
    }

    let content = fs::read_to_string(file_path)?;
    let sliced = match options.line_range.as_ref() {
        Some(line_range) => slice_content_by_line_range(
            &content,
            line_range,
            file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
        )
        .map_err(std::io::Error::other)?,
        None => {
            let line_count = count_content_lines(&content);
            SlicedContent {
                content,
                start_line: if line_count == 0 { 0 } else { 1 },
                line_count,
            }
        }
    };
    if let Some(sampled_view) = sampled_view.as_ref() {
        validate_sampled_view_options(sampled_view).map_err(std::io::Error::other)?;
    }
    let rendered = render_read_file_content(&sliced, &options, sampled_view.as_ref());

    Ok(ReadFileResult {
        content: rendered,
        file_path: file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        start_line: sliced.start_line,
        line_count: sliced.line_count,
    })
}

#[derive(Default)]
struct Counts {
    dir_count: usize,
    file_count: usize,
    filtered_hidden_count: usize,
    filtered_gitignored_count: usize,
    filtered_both_count: usize,
    filtered_type_count: usize,
}

#[allow(clippy::too_many_arguments)]
fn walk_directory(
    current_path: &Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    options: &DirectoryListOptions,
    type_matcher: Option<&Types>,
    repo_root: Option<&Path>,
    matcher_cache: &mut HashMap<PathBuf, Vec<GitIgnoreMatcher>>,
    lines: &mut Vec<String>,
    counts: &mut Counts,
) -> bool {
    if depth > max_depth {
        return false;
    }

    let mut children = match fs::read_dir(current_path) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>(),
        Err(_) => return false,
    };
    children.sort_by(|lhs, rhs| {
        let lhs_is_dir = lhs.is_dir();
        let rhs_is_dir = rhs.is_dir();
        (
            !lhs_is_dir,
            lhs.file_name()
                .map(|name| name.to_string_lossy().to_lowercase()),
        )
            .cmp(&(
                !rhs_is_dir,
                rhs.file_name()
                    .map(|name| name.to_string_lossy().to_lowercase()),
            ))
    });

    let active_matchers = if repo_root.is_some() {
        get_active_matchers(current_path, repo_root, matcher_cache)
    } else {
        Vec::new()
    };

    let mut visible_children = Vec::new();
    for child in children {
        let is_hidden = child
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false);
        let is_gitignored = is_gitignored(&child, child.is_dir(), &active_matchers);

        let hidden_filtered = is_hidden && !options.show_hidden;
        let gitignored_filtered = is_gitignored && !options.include_gitignored;
        if hidden_filtered || gitignored_filtered {
            if hidden_filtered && gitignored_filtered {
                counts.filtered_both_count += 1;
            } else if hidden_filtered {
                counts.filtered_hidden_count += 1;
            } else {
                counts.filtered_gitignored_count += 1;
            }
            continue;
        }
        if child.is_file() && is_type_filtered(&child, type_matcher) {
            counts.filtered_type_count += 1;
            continue;
        }
        visible_children.push(child);
    }

    let mut rendered_children = Vec::new();
    for child in &visible_children {
        let name = child
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        if child.is_dir() {
            let mut child_lines = Vec::new();
            let child_has_visible = if depth < max_depth {
                walk_directory(
                    child,
                    "",
                    depth + 1,
                    max_depth,
                    options,
                    type_matcher,
                    repo_root,
                    matcher_cache,
                    &mut child_lines,
                    counts,
                )
            } else {
                true
            };
            if type_matcher.is_some() && !child_has_visible {
                continue;
            }
            rendered_children.push(RenderedChild {
                name: format!("{name}/"),
                metadata: None,
                child_lines,
            });
            counts.dir_count += 1;
            continue;
        }

        let metadata = format_file_metadata(child, options);
        rendered_children.push(RenderedChild {
            name: name.to_string(),
            metadata: Some(metadata),
            child_lines: Vec::new(),
        });
        counts.file_count += 1;
    }

    for (index, child) in rendered_children.iter().enumerate() {
        let is_last = index + 1 == rendered_children.len();
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };
        match child.metadata.as_ref() {
            Some(metadata) => lines.push(format!(
                "{}{}{} ({})",
                prefix, connector, child.name, metadata
            )),
            None => {
                lines.push(format!("{}{}{}", prefix, connector, child.name));
                for child_line in &child.child_lines {
                    lines.push(format!("{}{}{}", prefix, child_prefix, child_line));
                }
            }
        }
    }

    !rendered_children.is_empty()
}

struct RenderedChild {
    name: String,
    metadata: Option<String>,
    child_lines: Vec<String>,
}

struct TypeFilter {
    matcher: Option<Types>,
    warnings: Vec<String>,
}

fn build_type_filter(options: &DirectoryListOptions) -> std::io::Result<TypeFilter> {
    if options.file_types.is_empty() && options.file_types_not.is_empty() {
        return Ok(TypeFilter {
            matcher: None,
            warnings: Vec::new(),
        });
    }

    let mut builder = TypesBuilder::new();
    builder.add_defaults();
    let mut selected_any = false;
    let mut warnings = Vec::new();
    for file_type in &options.file_types {
        let name = parse_file_type_name("file_types", file_type)?;
        if is_known_file_type(name, true) {
            builder.select(name);
            selected_any = true;
        } else {
            warnings.push(format!(
                "file_types ignored unknown ripgrep/ignore file type '{name}'"
            ));
        }
    }
    for file_type in &options.file_types_not {
        let name = parse_file_type_name("file_types_not", file_type)?;
        if is_known_file_type(name, false) {
            builder.negate(name);
            selected_any = true;
        } else {
            warnings.push(format!(
                "file_types_not ignored unknown ripgrep/ignore file type '{name}'"
            ));
        }
    }
    if !selected_any {
        warnings.push(
            "no valid file type filters remained; type filtering was disabled for this call"
                .to_string(),
        );
        return Ok(TypeFilter {
            matcher: None,
            warnings,
        });
    }

    let matcher = builder
        .build()
        .map(Some)
        .map_err(|err| std::io::Error::other(format!("invalid file type filter: {err}")))?;
    Ok(TypeFilter { matcher, warnings })
}

fn parse_file_type_name<'a>(field_name: &str, value: &'a str) -> std::io::Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(std::io::Error::other(format!(
            "{field_name} cannot contain an empty file type"
        )));
    }
    Ok(trimmed)
}

fn is_known_file_type(name: &str, include: bool) -> bool {
    let mut builder = TypesBuilder::new();
    builder.add_defaults();
    if include {
        builder.select(name);
    } else {
        builder.negate(name);
    }
    builder.build().is_ok()
}

fn is_type_filtered(path: &Path, type_matcher: Option<&Types>) -> bool {
    let Some(type_matcher) = type_matcher else {
        return false;
    };
    matches!(type_matcher.matched(path, false), Match::Ignore(_))
}

fn get_active_matchers(
    current_path: &Path,
    repo_root: Option<&Path>,
    matcher_cache: &mut HashMap<PathBuf, Vec<GitIgnoreMatcher>>,
) -> Vec<GitIgnoreMatcher> {
    let Some(repo_root) = repo_root else {
        return Vec::new();
    };

    if let Some(matchers) = matcher_cache.get(current_path) {
        return matchers.clone();
    }

    let parent_matchers = if current_path == repo_root {
        Vec::new()
    } else {
        get_active_matchers(
            current_path.parent().unwrap_or(repo_root),
            Some(repo_root),
            matcher_cache,
        )
    };

    let mut matchers = parent_matchers;
    let gitignore_path = current_path.join(".gitignore");
    if let Some(spec) = load_gitignore_spec(&gitignore_path) {
        matchers.push(GitIgnoreMatcher {
            base_dir: current_path.to_path_buf(),
            spec,
        });
    }

    matcher_cache.insert(current_path.to_path_buf(), matchers.clone());
    matchers
}

fn load_gitignore_spec(gitignore_path: &Path) -> Option<Gitignore> {
    if !gitignore_path.is_file() {
        return None;
    }

    let mut builder = GitignoreBuilder::new(gitignore_path.parent()?);
    builder.add(gitignore_path);
    builder.build().ok()
}

fn is_gitignored(path: &Path, is_dir: bool, matchers: &[GitIgnoreMatcher]) -> bool {
    let mut ignored = false;
    for matcher in matchers {
        let Ok(relative) = path.strip_prefix(&matcher.base_dir) else {
            continue;
        };
        let matched = matcher.spec.matched(relative, is_dir);
        if matched.is_ignore() {
            ignored = true;
        } else if matched.is_whitelist() {
            ignored = false;
        }
    }
    ignored
}

fn count_utf8_lines(file_path: &Path) -> Option<usize> {
    fs::read_to_string(file_path)
        .ok()
        .map(|content| content.lines().count())
}

fn find_git_repo_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.canonicalize().ok()?;
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        let parent = current.parent()?.to_path_buf();
        if parent == current {
            return None;
        }
        current = parent;
    }
}

fn format_size(size_bytes: u64) -> String {
    if size_bytes < 1024 {
        return format!("{} B", size_bytes);
    }
    if size_bytes < 1024 * 1024 {
        return format!("{:.1} KB", size_bytes as f64 / 1024.0);
    }
    format!("{:.1} MB", size_bytes as f64 / (1024.0 * 1024.0))
}

#[derive(Clone, Debug)]
struct SlicedContent {
    content: String,
    start_line: usize,
    line_count: usize,
}

fn slice_content_by_line_range(
    content: &str,
    line_range: &ReadFileLineRange,
    display_path: &str,
) -> Result<SlicedContent, String> {
    match line_range {
        ReadFileLineRange::Closed { start, end } => {
            slice_content_by_closed_line_range(content, *start, *end, display_path)
        }
        ReadFileLineRange::SliceLike { start, stop } => {
            slice_content_by_slice_like_range(content, *start, *stop)
        }
    }
}

fn slice_content_by_closed_line_range(
    content: &str,
    start: usize,
    end: usize,
    display_path: &str,
) -> Result<SlicedContent, String> {
    if start == 0 || end == 0 {
        return Err("line_range must use positive 1-indexed line numbers".to_string());
    }
    if start > end {
        return Err(format!("line_range start {} must be <= end {}", start, end));
    }

    let lines = content.split_inclusive('\n').collect::<Vec<_>>();
    let total_lines = lines.len();
    if total_lines == 0 {
        return Ok(SlicedContent {
            content: String::new(),
            start_line: 0,
            line_count: 0,
        });
    }
    if start > total_lines {
        return Err(format!(
            "line_range start {} exceeds total lines {} for '{}'",
            start, total_lines, display_path
        ));
    }

    let effective_end = end.min(total_lines);
    Ok(SlicedContent {
        content: lines[start - 1..effective_end].concat(),
        start_line: start,
        line_count: effective_end - start + 1,
    })
}

fn slice_content_by_slice_like_range(
    content: &str,
    start: Option<i64>,
    stop: Option<i64>,
) -> Result<SlicedContent, String> {
    let lines = content.split_inclusive('\n').collect::<Vec<_>>();
    let total_lines = lines.len() as i64;
    if total_lines == 0 {
        return Ok(SlicedContent {
            content: String::new(),
            start_line: 0,
            line_count: 0,
        });
    }

    let resolved_start = resolve_slice_like_start(start, total_lines).max(1);
    let resolved_stop = resolve_slice_like_stop(stop, total_lines).min(total_lines);
    if resolved_start > resolved_stop || resolved_start > total_lines || resolved_stop < 1 {
        return Ok(SlicedContent {
            content: String::new(),
            start_line: 0,
            line_count: 0,
        });
    }

    let start_index = (resolved_start - 1) as usize;
    let end_index = resolved_stop as usize;
    Ok(SlicedContent {
        content: lines[start_index..end_index].concat(),
        start_line: resolved_start as usize,
        line_count: (resolved_stop - resolved_start + 1) as usize,
    })
}

fn resolve_slice_like_start(start: Option<i64>, total_lines: i64) -> i64 {
    match start {
        None => 1,
        Some(value) if value > 0 => value,
        Some(value) => total_lines + value + 1,
    }
}

fn resolve_slice_like_stop(stop: Option<i64>, total_lines: i64) -> i64 {
    match stop {
        None => total_lines,
        Some(value) if value > 0 => value,
        Some(value) => total_lines + value,
    }
}

fn parse_slice_like_line_range_text(text: &str) -> Result<ReadFileLineRange, String> {
    if text.matches(':').count() != 1 {
        return Err("line_range slice form supports exactly one `:` and does not support step; use sample_step/sample_lines for sampled inspection".to_string());
    }
    let (start_text, stop_text) = text
        .split_once(':')
        .expect("validated single-colon slice form");
    let start = parse_slice_like_endpoint(start_text)?;
    let stop = parse_slice_like_endpoint(stop_text)?;
    Ok(ReadFileLineRange::SliceLike { start, stop })
}

fn parse_slice_like_endpoint(text: &str) -> Result<Option<i64>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let value = trimmed.parse::<i64>().map_err(|_| {
        "line_range must use `start:stop`; this slice-like form supports omitted bounds and negative tail offsets such as `:50`, `50:`, `-50:`, or `:-1`".to_string()
    })?;
    if value == 0 {
        return Err("line_range slice endpoints must not be 0; use positive 1-indexed lines or negative tail offsets".to_string());
    }
    Ok(Some(value))
}

fn parse_closed_line_range_text(text: &str, delimiter: char) -> Result<ReadFileLineRange, String> {
    let parts = text
        .split(delimiter)
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("line_range must use `start:stop`; this slice-like form supports omitted bounds and negative tail offsets such as `:50`, `50:`, `-50:`, or `:-1`".to_string());
    }
    let start = parts[0]
        .parse::<usize>()
        .map_err(|_| "line_range closed ranges must use positive integers".to_string())?;
    let end = parts[1]
        .parse::<usize>()
        .map_err(|_| "line_range closed ranges must use positive integers".to_string())?;
    Ok(ReadFileLineRange::Closed { start, end })
}

fn is_legacy_hyphen_closed_range(text: &str) -> bool {
    let mut parts = text.split('-').map(str::trim);
    let Some(start) = parts.next() else {
        return false;
    };
    let Some(end) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && !start.is_empty()
        && !end.is_empty()
        && start.chars().all(|ch| ch.is_ascii_digit())
        && end.chars().all(|ch| ch.is_ascii_digit())
}

fn count_content_lines(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.split_inclusive('\n').count()
    }
}

fn validate_sampled_view_options(options: &ReadFileSampledViewOptions) -> Result<(), String> {
    if options.sample_step == 0 || options.sample_lines == 0 {
        return Err(
            "sampled view requires positive integers for sample_step and sample_lines".to_string(),
        );
    }
    if options.sample_lines >= options.sample_step {
        return Err("sampled view requires 1 <= sample_lines < sample_step".to_string());
    }
    Ok(())
}

fn render_read_file_content(
    sliced: &SlicedContent,
    options: &ReadFileOptions,
    sampled_view: Option<&ReadFileSampledViewOptions>,
) -> String {
    match sampled_view {
        Some(sampled_view) => render_sampled_view(
            &sliced.content,
            sliced.start_line,
            options.show_line_numbers,
            sampled_view,
        ),
        None => render_exact_view(
            &sliced.content,
            sliced.start_line,
            options.show_line_numbers,
        ),
    }
}

fn render_exact_view(content: &str, start_line: usize, show_line_numbers: bool) -> String {
    if content.is_empty() {
        return String::new();
    }

    let lines = content.split_inclusive('\n').collect::<Vec<_>>();
    let line_number_width = if show_line_numbers {
        (start_line + lines.len() - 1).to_string().len()
    } else {
        0
    };

    lines
        .into_iter()
        .enumerate()
        .map(|(offset, line)| {
            if show_line_numbers {
                format!(
                    "{:>width$} | {}",
                    start_line + offset,
                    line,
                    width = line_number_width
                )
            } else {
                line.to_string()
            }
        })
        .collect()
}

fn render_sampled_view(
    content: &str,
    start_line: usize,
    show_line_numbers: bool,
    options: &ReadFileSampledViewOptions,
) -> String {
    if content.is_empty() {
        return String::new();
    }

    let lines = content.split_inclusive('\n').collect::<Vec<_>>();
    let line_number_width = if show_line_numbers {
        (start_line + lines.len() - 1).to_string().len()
    } else {
        0
    };
    let mut rendered = String::new();
    let mut block_start = 0;

    while block_start < lines.len() {
        let block_end = (block_start + options.sample_lines).min(lines.len());
        for (index, line) in lines.iter().enumerate().take(block_end).skip(block_start) {
            if show_line_numbers {
                rendered.push_str(&format!(
                    "{:>width$} | {}",
                    start_line + index,
                    line,
                    width = line_number_width
                ));
            } else {
                rendered.push_str(line);
            }
        }

        let window_end = (block_start + options.sample_step).min(lines.len());
        if block_end < window_end {
            if window_end < lines.len() {
                rendered.push_str("...\n");
            } else {
                rendered.push_str("...");
            }
        }

        block_start += options.sample_step;
    }

    rendered
}
fn format_file_metadata(file_path: &Path, options: &DirectoryListOptions) -> String {
    let metadata = match file_path.metadata() {
        Ok(metadata) => metadata,
        Err(_) => return "size unavailable, lines unavailable".to_string(),
    };

    let mut parts = vec![format_size(metadata.len())];
    parts.push(match count_utf8_lines(file_path) {
        Some(line_count) => {
            let line_word = if line_count == 1 { "line" } else { "lines" };
            format!("{line_count} {line_word}")
        }
        None => "lines unavailable".to_string(),
    });

    for field in &options.timestamp_fields {
        match field {
            TimestampField::Created => parts.push(format!(
                "created={}",
                format_system_time(metadata.created().ok())
            )),
            TimestampField::Modified => parts.push(format!(
                "modified={}",
                format_system_time(metadata.modified().ok())
            )),
        }
    }

    parts.join(", ")
}

fn format_system_time(value: Option<std::time::SystemTime>) -> String {
    let Some(value) = value else {
        return "unavailable".to_string();
    };
    let Ok(local_offset) = UtcOffset::current_local_offset() else {
        return OffsetDateTime::from(value)
            .format(&Rfc3339)
            .unwrap_or_else(|_| "unavailable".to_string());
    };
    OffsetDateTime::from(value)
        .to_offset(local_offset)
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unavailable".to_string())
}

fn display_root_dir_name(dir_path: &Path, options: &DirectoryListOptions) -> String {
    options
        .root_display
        .as_deref()
        .map(trim_trailing_path_separators)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| display_dir_name(dir_path))
}

fn display_dir_name(dir_path: &Path) -> String {
    dir_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| dir_path.display().to_string())
}

fn trim_trailing_path_separators(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() { path } else { trimmed }
}
