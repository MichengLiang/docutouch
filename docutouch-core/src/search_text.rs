use crate::path_display::{display_path, format_scope};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const SEARCH_TEXT_PREVIEW_MAX_FILES: usize = 8;
const SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchTextView {
    #[default]
    Preview,
    Full,
}

#[derive(Debug)]
struct SearchTextGroup {
    path: String,
    matches: Vec<SearchTextMatch>,
    total_hits: usize,
}

#[derive(Debug)]
struct SearchTextMatch {
    line_number: usize,
    text: String,
    hit_count: usize,
}

pub async fn search_text(
    query: &str,
    search_paths: &[PathBuf],
    rg_args_text: &str,
    view: SearchTextView,
    display_base_dir: Option<&Path>,
) -> Result<String, String> {
    let rg_args = parse_search_text_rg_args(rg_args_text)?;
    let output = run_search_text_rg(query, search_paths, &rg_args).await?;
    let (groups, total_matches) = parse_search_text_output(&output, display_base_dir)?;
    Ok(format_search_text_result(
        query,
        &format_scope(search_paths, display_base_dir),
        rg_args_text,
        view,
        &groups,
        total_matches,
    ))
}

fn parse_search_text_rg_args(rg_args: &str) -> Result<Vec<String>, String> {
    let trimmed = rg_args.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parts =
        shlex::split(trimmed).ok_or_else(|| "rg_args contains invalid quoting".to_string())?;
    for part in &parts {
        match part.as_str() {
            "--json"
            | "--heading"
            | "--no-heading"
            | "--color"
            | "-n"
            | "--line-number"
            | "-N"
            | "--no-line-number"
            | "-c"
            | "--count"
            | "--count-matches"
            | "-l"
            | "--files-with-matches"
            | "--files-without-match"
            | "--files"
            | "--type-list"
            | "--replace"
            | "-A"
            | "-B"
            | "-C"
            | "--context"
            | "--before-context"
            | "--after-context" => {
                return Err(format!(
                    "rg_args cannot use render-shaping flag `{part}` in search_text; use search-behavior flags such as `-F`, `-i`, `-g`, or `-P`, or use raw `rg` in the terminal for unrestricted output shaping"
                ));
            }
            _ => {}
        }
    }
    Ok(parts)
}

async fn run_search_text_rg(
    query: &str,
    search_paths: &[PathBuf],
    rg_args: &[String],
) -> Result<String, String> {
    let mut command = tokio::process::Command::new("rg");
    command
        .arg("--json")
        .arg("--line-number")
        .arg("--color")
        .arg("never")
        .arg("--no-heading");
    for arg in rg_args {
        command.arg(arg);
    }
    command.arg("--").arg(query);
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
        if value.get("type").and_then(Value::as_str) != Some("match") {
            continue;
        }
        let data = value
            .get("data")
            .ok_or_else(|| "rg JSON output missing `data` for a match event".to_string())?;
        let raw_path = extract_rg_text(data.get("path"))
            .ok_or_else(|| "rg JSON output missing match path".to_string())?;
        let line_text = extract_rg_text(data.get("lines"))
            .ok_or_else(|| "rg JSON output missing match lines".to_string())?;
        let line_number = data
            .get("line_number")
            .and_then(Value::as_u64)
            .ok_or_else(|| "rg JSON output missing match line_number".to_string())?
            as usize;
        let display_path = display_path(display_base_dir, Path::new(&raw_path));
        let index = if let Some(index) = group_index_by_path.get(&display_path).copied() {
            index
        } else {
            let index = groups.len();
            groups.push(SearchTextGroup {
                path: display_path.clone(),
                matches: Vec::new(),
                total_hits: 0,
            });
            group_index_by_path.insert(display_path, index);
            index
        };
        let hit_count = data
            .get("submatches")
            .and_then(Value::as_array)
            .map(Vec::len)
            .filter(|count| *count > 0)
            .unwrap_or(1);
        groups[index].matches.push(SearchTextMatch {
            line_number,
            text: trim_rg_line_text(&line_text),
            hit_count,
        });
        groups[index].total_hits += hit_count;
        total_matches += hit_count;
    }

    groups.sort_by(|lhs, rhs| {
        rhs.matches
            .len()
            .cmp(&lhs.matches.len())
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

fn format_search_text_result(
    query: &str,
    scope: &str,
    rg_args: &str,
    view: SearchTextView,
    groups: &[SearchTextGroup],
    total_matches: usize,
) -> String {
    let matched_lines = groups
        .iter()
        .map(|group| group.matches.len())
        .sum::<usize>();
    let mut lines = vec![
        format!("search_text[{}]:", view.label()),
        format!("query: {query}"),
        format!("scope: {scope}"),
        format!("files: {}", groups.len()),
        format!("matched_lines: {matched_lines}"),
        format!("matches: {total_matches}"),
    ];
    if !rg_args.trim().is_empty() {
        lines.push(format!("rg_args: {}", rg_args.trim()));
    }
    if view == SearchTextView::Preview {
        let rendered_files = groups.len().min(SEARCH_TEXT_PREVIEW_MAX_FILES);
        let rendered_lines = groups
            .iter()
            .take(rendered_files)
            .map(|group| {
                group
                    .matches
                    .len()
                    .min(SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE)
            })
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
    let rendered_files = match view {
        SearchTextView::Preview => groups.len().min(SEARCH_TEXT_PREVIEW_MAX_FILES),
        SearchTextView::Full => groups.len(),
    };
    let mut rendered_lines_total = 0usize;
    for (index, group) in groups.iter().take(rendered_files).enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        let line_word = if group.matches.len() == 1 {
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
            group.matches.len(),
            line_word,
            group.total_hits,
            hit_word
        ));
        let rendered_entries = match view {
            SearchTextView::Preview => group
                .matches
                .iter()
                .take(SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE)
                .collect::<Vec<_>>(),
            SearchTextView::Full => group.matches.iter().collect::<Vec<_>>(),
        };
        let line_number_width = rendered_entries
            .iter()
            .map(|entry| entry.line_number)
            .max()
            .unwrap_or_default()
            .to_string()
            .len()
            .max(1);
        rendered_lines_total += rendered_entries.len();
        for entry in rendered_entries {
            let line_number = entry.line_number;
            if entry.hit_count > 1 {
                lines.push(format!(
                    "  {line_number:>width$} | {}  [{} hits]",
                    entry.text,
                    entry.hit_count,
                    width = line_number_width,
                ));
            } else {
                lines.push(format!(
                    "  {line_number:>width$} | {}",
                    entry.text,
                    width = line_number_width
                ));
            }
        }
        if view == SearchTextView::Preview
            && group.matches.len() > SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE
        {
            lines.push(format!(
                "  note: {} more matched lines in this file",
                group.matches.len() - SEARCH_TEXT_PREVIEW_MAX_LINES_PER_FILE
            ));
        }
    }

    if view == SearchTextView::Preview {
        let omitted_files = groups.len().saturating_sub(rendered_files);
        let omitted_lines = matched_lines.saturating_sub(rendered_lines_total);
        if omitted_files > 0 || omitted_lines > 0 {
            lines.push(String::new());
            lines.push("omitted:".to_string());
            if omitted_files > 0 {
                lines.push(format!("- {omitted_files} more files not shown"));
            }
            if omitted_lines > 0 {
                lines.push(format!("- {omitted_lines} more matched lines not shown"));
            }
        }
    }

    lines.join("\n")
}

impl SearchTextView {
    fn label(self) -> &'static str {
        match self {
            SearchTextView::Preview => "preview",
            SearchTextView::Full => "full",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_renders_explicit_omission_accounting() {
        let groups = vec![SearchTextGroup {
            path: "src/noisy.txt".to_string(),
            matches: (1..=5)
                .map(|line_number| SearchTextMatch {
                    line_number,
                    text: "alpha".to_string(),
                    hit_count: 1,
                })
                .collect(),
            total_hits: 5,
        }];
        let rendered =
            format_search_text_result("alpha", "src", "", SearchTextView::Preview, &groups, 5);
        assert!(rendered.contains("rendered_files: 1"));
        assert!(rendered.contains("rendered_lines: 3"));
        assert!(rendered.contains("note: 2 more matched lines in this file"));
        assert!(rendered.contains("- 2 more matched lines not shown"));
    }

    #[test]
    fn aligns_line_numbers_within_each_file_group() {
        let groups = vec![
            SearchTextGroup {
                path: "src/one.txt".to_string(),
                matches: vec![
                    SearchTextMatch {
                        line_number: 9,
                        text: "alpha".to_string(),
                        hit_count: 1,
                    },
                    SearchTextMatch {
                        line_number: 10,
                        text: "beta".to_string(),
                        hit_count: 1,
                    },
                ],
                total_hits: 2,
            },
            SearchTextGroup {
                path: "src/two.txt".to_string(),
                matches: vec![SearchTextMatch {
                    line_number: 100,
                    text: "gamma".to_string(),
                    hit_count: 1,
                }],
                total_hits: 1,
            },
        ];

        let rendered = format_search_text_result("a", "src", "", SearchTextView::Full, &groups, 3);
        assert!(rendered.contains("   9 | alpha"));
        assert!(rendered.contains("  10 | beta"));
        assert!(!rendered.contains("    9 | alpha"));
    }

    #[test]
    fn preview_alignment_is_based_on_rendered_entries_only() {
        let groups = vec![SearchTextGroup {
            path: "src/noisy.txt".to_string(),
            matches: vec![
                SearchTextMatch {
                    line_number: 9,
                    text: "alpha".to_string(),
                    hit_count: 1,
                },
                SearchTextMatch {
                    line_number: 10,
                    text: "alpha".to_string(),
                    hit_count: 1,
                },
                SearchTextMatch {
                    line_number: 11,
                    text: "alpha".to_string(),
                    hit_count: 1,
                },
                SearchTextMatch {
                    line_number: 100,
                    text: "alpha".to_string(),
                    hit_count: 1,
                },
            ],
            total_hits: 4,
        }];

        let rendered =
            format_search_text_result("alpha", "src", "", SearchTextView::Preview, &groups, 4);
        assert!(rendered.contains("rendered_lines: 3"));
        assert!(rendered.contains("   9 | alpha"));
        assert!(rendered.contains("  10 | alpha"));
        assert!(rendered.contains("  11 | alpha"));
        assert!(!rendered.contains("100 | alpha"));
        assert!(!rendered.contains("    9 | alpha"));
    }
}
