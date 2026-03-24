use crate::path_display::display_path;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticSourceExcerpt {
    pub width: usize,
    pub lines: Vec<String>,
}

pub(crate) fn render_affected_change_lines<
    PAdded,
    PModified,
    PDeleted,
    IAdded,
    IModified,
    IDeleted,
>(
    display_base_dir: Option<&Path>,
    prefix: &str,
    added: IAdded,
    modified: IModified,
    deleted: IDeleted,
) -> Vec<String>
where
    IAdded: IntoIterator<Item = PAdded>,
    IModified: IntoIterator<Item = PModified>,
    IDeleted: IntoIterator<Item = PDeleted>,
    PAdded: AsRef<Path>,
    PModified: AsRef<Path>,
    PDeleted: AsRef<Path>,
{
    let mut lines = Vec::new();
    extend_tagged_paths(&mut lines, display_base_dir, prefix, "A", added);
    extend_tagged_paths(&mut lines, display_base_dir, prefix, "M", modified);
    extend_tagged_paths(&mut lines, display_base_dir, prefix, "D", deleted);
    lines
}

pub(crate) fn diagnostic_location(
    display_base_dir: Option<&Path>,
    source_path: &Path,
    source_line: Option<usize>,
    source_column: Option<usize>,
    target_path: Option<&Path>,
) -> String {
    if let Some(source_line) = source_line {
        return format!(
            "{}:{source_line}:{}",
            display_path(display_base_dir, source_path),
            source_column.unwrap_or(1)
        );
    }
    if let Some(target_path) = target_path {
        return display_path(display_base_dir, target_path);
    }
    display_path(display_base_dir, source_path)
}

pub(crate) fn diagnostic_excerpt_gutter_line(width: usize) -> String {
    format!("{:>width$} |", "", width = width)
}

pub(crate) fn diagnostic_source_excerpt(
    source: &str,
    source_line: Option<usize>,
    source_column: Option<usize>,
    context_before: usize,
) -> Option<DiagnosticSourceExcerpt> {
    let source_line = source_line?;
    let all_lines = source.lines().collect::<Vec<_>>();
    if all_lines.is_empty() || source_line == 0 || source_line > all_lines.len() {
        return None;
    }

    let start_line = source_line.saturating_sub(context_before).max(1);
    let width = source_line.to_string().len();
    let mut lines = Vec::new();
    for line_number in start_line..=source_line {
        lines.push(format!(
            "{line_number:>width$} | {}",
            all_lines[line_number - 1],
            width = width
        ));
    }
    let caret_padding = " ".repeat(source_column.unwrap_or(1).saturating_sub(1));
    lines.push(format!("{:>width$} | {caret_padding}^", "", width = width));

    Some(DiagnosticSourceExcerpt { width, lines })
}

pub(crate) fn render_target_detail_line(
    display_base_dir: Option<&Path>,
    prefix: &str,
    target_path: Option<&Path>,
    anchor_path: Option<&Path>,
) -> Option<String> {
    let target_path = target_path?;
    let rendered_target_path = display_path(display_base_dir, target_path);
    if anchor_path.map(|path| display_path(display_base_dir, path))
        == Some(rendered_target_path.clone())
    {
        return None;
    }

    Some(format!("{prefix}= target: {rendered_target_path}"))
}

fn extend_tagged_paths<P, I>(
    lines: &mut Vec<String>,
    display_base_dir: Option<&Path>,
    prefix: &str,
    tag: &str,
    paths: I,
) where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    for path in paths {
        lines.push(format!(
            "{prefix}{tag} {}",
            display_path(display_base_dir, path.as_ref())
        ));
    }
}
