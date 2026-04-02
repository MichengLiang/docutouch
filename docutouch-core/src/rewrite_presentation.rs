use crate::path_display::display_path;
use crate::presentation_shared::{
    diagnostic_excerpt_gutter_line, diagnostic_location as render_diagnostic_location,
    diagnostic_source_excerpt as render_diagnostic_source_excerpt,
    render_affected_change_lines as render_shared_affected_change_lines, render_target_detail_line,
};
use crate::rewrite_runtime::{
    RewriteDiagnosticTargetAnchor, RewriteRuntimeError, RewriteRuntimeOutcome, RewriteWarning,
};
use codex_apply_patch::AffectedPaths;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct RewritePresentationContext {
    pub display_base_dir: Option<PathBuf>,
    pub rewrite_source: Option<PathBuf>,
}

pub fn format_rewrite_result(
    rewrite: &str,
    context: &RewritePresentationContext,
    outcome: Result<&RewriteRuntimeOutcome, &RewriteRuntimeError>,
) -> Result<String, String> {
    match outcome {
        Ok(outcome) => Ok(format_rewrite_success(
            &outcome.affected,
            &outcome.warnings,
            context.display_base_dir.as_deref(),
        )),
        Err(error) => Err(format_rewrite_failure(rewrite, context, error)),
    }
}

fn format_rewrite_success(
    affected: &AffectedPaths,
    warnings: &[RewriteWarning],
    display_base_dir: Option<&Path>,
) -> String {
    let mut lines = vec!["Success. Updated the following files:".to_string()];
    lines.extend(render_affected_lines(affected, display_base_dir, ""));
    if affected.added.is_empty() && affected.modified.is_empty() && affected.deleted.is_empty() {
        lines.push("(no file changes)".to_string());
    }
    if !warnings.is_empty() {
        lines.push(String::new());
        lines.push("Warnings:".to_string());
        for warning in warnings {
            lines.push(format!(
                "- {} [{}]",
                render_warning_summary(warning, display_base_dir),
                warning.code
            ));
        }
    }
    lines.join("\n")
}

fn format_rewrite_failure(
    rewrite: &str,
    context: &RewritePresentationContext,
    error: &RewriteRuntimeError,
) -> String {
    let details = error.details();
    let source_path = context
        .rewrite_source
        .clone()
        .unwrap_or_else(|| PathBuf::from("<rewrite>"));
    let display_base_dir = context.display_base_dir.as_deref();
    let location = render_diagnostic_location(
        display_base_dir,
        &source_path,
        details.source_line,
        details.source_column,
        details.target_path.as_deref(),
    );

    let mut lines = vec![format!(
        "error[{}]: {}",
        details.error_code,
        diagnostic_summary(details.error_code.as_str(), error.message())
    )];
    lines.push(format!("  --> {location}"));
    if let Some(source_excerpt) =
        render_diagnostic_source_excerpt(rewrite, details.source_line, details.source_column, 1)
    {
        lines.push(diagnostic_excerpt_gutter_line(source_excerpt.width));
        lines.extend(source_excerpt.lines);
        lines.push(diagnostic_excerpt_gutter_line(source_excerpt.width));
    }
    if let Some(target_line) = render_target_detail_line(
        display_base_dir,
        "   ",
        details.target_path.as_deref(),
        details
            .target_anchor
            .as_ref()
            .map(|anchor| anchor.path.as_path()),
    ) {
        lines.push(target_line);
    }
    lines.extend(render_target_anchor(
        display_base_dir,
        details.target_anchor.as_ref(),
    ));
    if let Some(operation_index) = details.operation_index {
        lines.push(format!("   = operation: {operation_index}"));
    }
    if !error.warnings().is_empty() {
        lines.push(String::new());
        lines.push("   = warnings before failure:".to_string());
        for warning in error.warnings() {
            lines.push(format!("     - {} [{}]", warning.summary, warning.code));
        }
    }
    if !error.affected().added.is_empty()
        || !error.affected().modified.is_empty()
        || !error.affected().deleted.is_empty()
    {
        lines.push(String::new());
        lines.push("   = committed changes:".to_string());
        lines.extend(render_affected_lines(
            error.affected(),
            display_base_dir,
            "     ",
        ));
    }
    if !error.failed_units().is_empty() {
        lines.push(String::new());
        lines.push(format!("   = failed units: {}", error.failed_units().len()));
        for (index, failed_unit) in error.failed_units().iter().enumerate() {
            lines.push(String::new());
            lines.push(format!("   = failed_unit[{}]:", index + 1));
            lines.push(format!(
                "     error[{}]: {}",
                failed_unit.code, failed_unit.summary
            ));
            if !failed_unit.committed.added.is_empty()
                || !failed_unit.committed.modified.is_empty()
                || !failed_unit.committed.deleted.is_empty()
            {
                lines.push("     = committed during failed unit:".to_string());
                lines.extend(render_affected_lines(
                    &failed_unit.committed,
                    display_base_dir,
                    "       ",
                ));
            }
            if !failed_unit.attempted.added.is_empty()
                || !failed_unit.attempted.modified.is_empty()
                || !failed_unit.attempted.deleted.is_empty()
            {
                lines.push("     = attempted changes:".to_string());
                lines.extend(render_affected_lines(
                    &failed_unit.attempted,
                    display_base_dir,
                    "       ",
                ));
            }
            if let Some(help) = &failed_unit.help {
                lines.push(format!("     = help: {help}"));
            }
        }
    }
    if let Some(fix_hint) = &details.fix_hint {
        lines.push(format!("   = help: {fix_hint}"));
    }
    lines.push(String::new());
    lines.push("caused by:".to_string());
    lines.push(format!("  {}", error.message()));
    lines.join("\n")
}

fn render_affected_lines(
    affected: &AffectedPaths,
    display_base_dir: Option<&Path>,
    prefix: &str,
) -> Vec<String> {
    render_shared_affected_change_lines(
        display_base_dir,
        prefix,
        affected.added.iter(),
        affected.modified.iter(),
        affected.deleted.iter(),
    )
}

fn render_target_anchor(
    display_base_dir: Option<&Path>,
    target_anchor: Option<&RewriteDiagnosticTargetAnchor>,
) -> Vec<String> {
    let Some(target_anchor) = target_anchor else {
        return Vec::new();
    };
    let mut lines = vec![format!(
        "   = target_anchor: {}:{}:{}",
        display_path(display_base_dir, &target_anchor.path),
        target_anchor.line_number,
        target_anchor.column_number
    )];
    if let Some(excerpt) = &target_anchor.excerpt {
        lines.push(format!("   = target_excerpt: {excerpt}"));
    }
    lines
}

fn diagnostic_summary(code: &str, fallback: &str) -> String {
    match code {
        "REWRITE_PROGRAM_INVALID" => "invalid rewrite program".to_string(),
        "REWRITE_SELECTION_INVALID" => "rewrite selection did not resolve".to_string(),
        "REWRITE_SELECTION_TRUNCATED" => {
            "rewrite selection contains forbidden truncation or omission syntax".to_string()
        }
        "REWRITE_SELECTION_OVERLAP" => {
            "rewrite selections overlap against one original snapshot".to_string()
        }
        "REWRITE_TARGET_STATE_INVALID" => "target state is invalid for this rewrite".to_string(),
        "REWRITE_WRITE_ERROR" => "rewrite could not be written to disk".to_string(),
        "REWRITE_PARTIAL_UNIT_FAILURE" => "rewrite partially applied".to_string(),
        _ => fallback.to_string(),
    }
}

fn render_warning_summary(warning: &RewriteWarning, display_base_dir: Option<&Path>) -> String {
    let target_path = display_path(display_base_dir, &warning.target_path);
    warning
        .summary
        .replace(&warning.target_path.display().to_string(), &target_path)
}
