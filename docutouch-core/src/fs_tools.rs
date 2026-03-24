use ignore::gitignore::{Gitignore, GitignoreBuilder};
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
    pub timestamp_fields: Vec<TimestampField>,
}

#[derive(Clone, Debug)]
pub struct DirectoryListResult {
    pub tree: String,
    pub dir_count: usize,
    pub file_count: usize,
    pub filtered_hidden_count: usize,
    pub filtered_gitignored_count: usize,
    pub filtered_both_count: usize,
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
        let filtered_total =
            self.filtered_hidden_count + self.filtered_gitignored_count + self.filtered_both_count;
        if filtered_total == 0 {
            return format!("{}\n{}", self.tree, stats);
        }

        format!(
            "{}\n{}\nfiltered: {} entries ({} hidden, {} gitignored, {} both)",
            self.tree,
            stats,
            filtered_total,
            self.filtered_hidden_count,
            self.filtered_gitignored_count,
            self.filtered_both_count,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ReadFileResult {
    pub content: String,
    pub file_path: String,
    pub start_line: usize,
    pub line_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ReadFileOptions {
    pub line_range: Option<(usize, usize)>,
    pub show_line_numbers: bool,
}

#[derive(Clone, Debug)]
pub struct ReadFileSampledViewOptions {
    pub sample_step: usize,
    pub sample_lines: usize,
    pub max_chars: Option<usize>,
}

pub fn normalize_sampled_view_options(
    sample_step: Option<usize>,
    sample_lines: Option<usize>,
    max_chars: Option<usize>,
) -> Result<Option<ReadFileSampledViewOptions>, String> {
    const DEFAULT_SAMPLE_STEP: usize = 5;
    const DEFAULT_SAMPLE_LINES: usize = 2;

    if sample_step.is_none() && sample_lines.is_none() && max_chars.is_none() {
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
        max_chars,
    }))
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
    let repo_root = find_git_repo_root(dir_path);
    let mut matcher_cache: HashMap<PathBuf, Vec<GitIgnoreMatcher>> = HashMap::new();
    let mut lines = vec![format!("{}/", display_dir_name(dir_path))];
    let mut counts = Counts::default();

    walk_directory(
        dir_path,
        "",
        1,
        max_depth,
        &options,
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
    let sliced = match options.line_range {
        Some((start, end)) => slice_content_by_line_range(
            &content,
            start,
            end,
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
    let rendered =
        render_read_file_content(&sliced, options.show_line_numbers, sampled_view.as_ref());

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
}

fn walk_directory(
    current_path: &Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    options: &DirectoryListOptions,
    repo_root: Option<&Path>,
    matcher_cache: &mut HashMap<PathBuf, Vec<GitIgnoreMatcher>>,
    lines: &mut Vec<String>,
    counts: &mut Counts,
) {
    if depth > max_depth {
        return;
    }

    let mut children = match fs::read_dir(current_path) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>(),
        Err(_) => return,
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
        visible_children.push(child);
    }

    for (index, child) in visible_children.iter().enumerate() {
        let is_last = index + 1 == visible_children.len();
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };
        let name = child
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        if child.is_dir() {
            lines.push(format!("{}{}{}/", prefix, connector, name));
            counts.dir_count += 1;
            if depth < max_depth {
                walk_directory(
                    child,
                    &format!("{}{}", prefix, child_prefix),
                    depth + 1,
                    max_depth,
                    options,
                    repo_root,
                    matcher_cache,
                    lines,
                    counts,
                );
            }
            continue;
        }

        let metadata = format_file_metadata(child, options);
        lines.push(format!("{}{}{} ({})", prefix, connector, name, metadata));
        counts.file_count += 1;
    }
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
    if options.max_chars == Some(0) {
        return Err("sampled view requires max_chars >= 1".to_string());
    }
    Ok(())
}

fn render_read_file_content(
    sliced: &SlicedContent,
    show_line_numbers: bool,
    sampled_view: Option<&ReadFileSampledViewOptions>,
) -> String {
    match sampled_view {
        Some(sampled_view) => render_sampled_view(
            &sliced.content,
            sliced.start_line,
            show_line_numbers,
            sampled_view,
        ),
        None if show_line_numbers => render_with_line_numbers(&sliced.content, sliced.start_line),
        None => sliced.content.clone(),
    }
}

fn render_with_line_numbers(content: &str, start_line: usize) -> String {
    if content.is_empty() {
        return String::new();
    }
    let line_count = content.split_inclusive('\n').count();
    let line_number_width = (start_line + line_count - 1).to_string().len();
    content
        .split_inclusive('\n')
        .enumerate()
        .map(|(offset, line)| {
            format!(
                "{:>width$} | {}",
                start_line + offset,
                line,
                width = line_number_width
            )
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
        for index in block_start..block_end {
            let rendered_line = truncate_sampled_line(lines[index], options.max_chars);
            if show_line_numbers {
                rendered.push_str(&format!(
                    "{:>width$} | {}",
                    start_line + index,
                    rendered_line,
                    width = line_number_width
                ));
            } else {
                rendered.push_str(&rendered_line);
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

fn truncate_sampled_line(line: &str, max_chars: Option<usize>) -> String {
    let Some(max_chars) = max_chars else {
        return line.to_string();
    };
    let (body, newline) = split_line_ending(line);
    let body_len = body.chars().count();
    if body_len <= max_chars {
        return line.to_string();
    }

    let visible = body.chars().take(max_chars).collect::<String>();
    let omitted = body_len - max_chars;
    format!("{visible}...[{omitted} chars omitted]{newline}")
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(stripped) = line.strip_suffix("\r\n") {
        (stripped, "\r\n")
    } else if let Some(stripped) = line.strip_suffix('\n') {
        (stripped, "\n")
    } else {
        (line, "")
    }
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

fn display_dir_name(dir_path: &Path) -> String {
    dir_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| dir_path.display().to_string())
}
