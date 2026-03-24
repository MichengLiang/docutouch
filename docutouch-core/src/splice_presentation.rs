use crate::path_display::display_path;
use crate::presentation_shared::{
    diagnostic_excerpt_gutter_line, diagnostic_location as render_diagnostic_location,
    diagnostic_source_excerpt as render_diagnostic_source_excerpt,
    render_affected_change_lines as render_shared_affected_change_lines, render_target_detail_line,
};
use crate::splice_runtime::{
    SpliceDiagnosticTargetAnchor, SpliceRuntimeError, SpliceRuntimeOutcome,
};
use codex_apply_patch::AffectedPaths;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SplicePresentationContext {
    pub display_base_dir: Option<PathBuf>,
    pub splice_source: Option<PathBuf>,
}

pub fn format_splice_result(
    splice: &str,
    context: &SplicePresentationContext,
    outcome: Result<&SpliceRuntimeOutcome, &SpliceRuntimeError>,
) -> Result<String, String> {
    match outcome {
        Ok(outcome) => Ok(format_splice_success(
            &outcome.affected,
            context.display_base_dir.as_deref(),
        )),
        Err(error) => Err(format_splice_failure(splice, context, error)),
    }
}

fn format_splice_success(affected: &AffectedPaths, display_base_dir: Option<&Path>) -> String {
    let mut lines = vec!["Success. Updated the following files:".to_string()];
    lines.extend(render_affected_lines(affected, display_base_dir, ""));
    if affected.added.is_empty() && affected.modified.is_empty() && affected.deleted.is_empty() {
        lines.push("(no file changes)".to_string());
    }
    lines.join("\n")
}

fn format_splice_failure(
    splice: &str,
    context: &SplicePresentationContext,
    error: &SpliceRuntimeError,
) -> String {
    let details = error.details();
    let source_path = context
        .splice_source
        .clone()
        .unwrap_or_else(|| PathBuf::from("<splice>"));
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
        render_diagnostic_source_excerpt(splice, details.source_line, details.source_column, 1)
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
    if let Some(action_index) = details.action_index {
        lines.push(format!("   = action: {action_index}"));
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
    target_anchor: Option<&SpliceDiagnosticTargetAnchor>,
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
        "SPLICE_PROGRAM_INVALID" => "invalid splice program".to_string(),
        "SPLICE_SOURCE_SELECTION_INVALID" => "source selection did not resolve".to_string(),
        "SPLICE_TARGET_SELECTION_INVALID" => "target selection did not resolve".to_string(),
        "SPLICE_SELECTION_TRUNCATED" => {
            "selection contains forbidden truncation or omission syntax".to_string()
        }
        "SPLICE_SOURCE_STATE_INVALID" => "source state is invalid for this action".to_string(),
        "SPLICE_TARGET_STATE_INVALID" => "target state is invalid for this action".to_string(),
        "SPLICE_OVERLAP_ILLEGAL" => "same-file anchored overlap is illegal".to_string(),
        "SPLICE_WRITE_ERROR" => "splice could not be written to disk".to_string(),
        "SPLICE_PARTIAL_UNIT_FAILURE" => "splice partially applied".to_string(),
        _ => fallback.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::splice_runtime::apply_splice_program;
    use std::fs;

    #[test]
    fn splice_success_preserves_no_op_line() {
        let outcome = SpliceRuntimeOutcome {
            affected: AffectedPaths::default(),
        };

        let rendered = format_splice_success(&outcome.affected, None);
        assert_eq!(
            rendered,
            "Success. Updated the following files:\n(no file changes)"
        );
    }

    #[test]
    fn splice_failure_renders_caret_and_dedupes_target_path_when_anchor_matches() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join("source.txt"), "alpha\n").expect("write source");
        let splice = concat!(
            "*** Begin Splice\n",
            "*** Copy From File: source.txt\n",
            "@@\n",
            "1 | alpha\n",
            "*** Insert Before In File: missing.txt\n",
            "@@\n",
            "1 | alpha\n",
            "*** End Splice\n",
        );
        let context = SplicePresentationContext {
            display_base_dir: Some(temp.path().to_path_buf()),
            splice_source: Some(temp.path().join("input.splice")),
        };

        let outcome = apply_splice_program(splice, temp.path());
        let message = format_splice_result(splice, &context, outcome.as_ref())
            .expect_err("splice should fail");
        let lines = message.lines().collect::<Vec<_>>();
        let excerpt_index = lines
            .iter()
            .position(|line| *line == "6 | @@")
            .expect("expected failing selection header excerpt");

        assert!(message.contains("error[SPLICE_TARGET_STATE_INVALID]"));
        assert!(message.contains("  --> input.splice:6:1"));
        assert_eq!(
            lines[excerpt_index - 1].find('|'),
            lines[excerpt_index].find('|'),
            "excerpt gutter should align above failing source line"
        );
        assert_eq!(
            lines[excerpt_index + 1].find('|'),
            lines[excerpt_index].find('|'),
            "caret line should align with excerpt gutter"
        );
        assert_eq!(
            lines[excerpt_index + 2].find('|'),
            lines[excerpt_index].find('|'),
            "excerpt gutter should align below failing source line"
        );
        assert!(message.contains("5 | *** Insert Before In File: missing.txt"));
        assert!(message.contains("  | ^"));
        assert!(message.contains("   = target_anchor: missing.txt:1:1"));
        assert!(message.contains("   = target_excerpt: alpha"));
        assert!(!message.contains("   = target: missing.txt"));
    }
}
