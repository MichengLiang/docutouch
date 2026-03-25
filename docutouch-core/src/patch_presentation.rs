use crate::patch_runtime::{
    ApplyOutcome, ApplyOutcomeAffected, ApplyOutcomeFailedUnit, DiagnosticTargetAnchor,
    FailureDetails,
};
use crate::path_display::display_path;
use crate::presentation_shared::{
    DiagnosticSourceExcerpt, diagnostic_excerpt_gutter_line,
    diagnostic_location as render_diagnostic_location,
    diagnostic_source_excerpt as render_diagnostic_source_excerpt,
    render_affected_change_lines as render_shared_affected_change_lines, render_target_detail_line,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub use crate::patch_runtime::formal_patch_path;

#[derive(Clone, Debug)]
pub struct PatchPresentationContext {
    pub runtime_base_dir: PathBuf,
    pub display_base_dir: Option<PathBuf>,
}

pub fn format_patch_outcome(
    patch: &str,
    context: &PatchPresentationContext,
    outcome: &ApplyOutcome,
) -> Result<String, String> {
    if outcome.ok {
        return Ok(format_patch_success_message(
            outcome,
            context.display_base_dir.as_deref(),
        ));
    }

    let error = outcome
        .error
        .as_ref()
        .ok_or_else(|| "Patch failed without diagnostics payload".to_string())?;
    let patch_path = outcome
        .patch_source
        .as_ref()
        .map(|reference| PathBuf::from(&reference.path))
        .or_else(|| error.details.patch_file.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("<patch>"));
    Err(format_patch_failure_message(
        context.display_base_dir.as_deref(),
        patch,
        outcome,
        &error.details,
        &error.summary,
        &patch_path,
    ))
}

fn format_patch_success_message(outcome: &ApplyOutcome, display_base_dir: Option<&Path>) -> String {
    let mut lines = vec!["Success. Updated the following files:".to_string()];
    lines.extend(render_affected_change_lines(
        &outcome.affected,
        display_base_dir,
        "",
    ));
    if outcome.file_paths.is_empty() {
        lines.push("(no file changes)".to_string());
    }

    if !outcome.warnings.is_empty() {
        lines.push(String::new());
    }
    for (index, warning) in outcome.warnings.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("warning[{}]: {}", warning.code, warning.summary));
        lines.push(format!(
            "  --> {}",
            display_path(display_base_dir, Path::new(&warning.target_path))
        ));
        if let Some(help) = &warning.help {
            lines.push(format!("  = help: {help}"));
        }
    }

    lines.join("\n")
}

fn render_affected_change_lines(
    affected: &ApplyOutcomeAffected,
    display_base_dir: Option<&Path>,
    prefix: &str,
) -> Vec<String> {
    render_shared_affected_change_lines(
        display_base_dir,
        prefix,
        affected.added.iter().map(|path| Path::new(path)),
        affected.modified.iter().map(|path| Path::new(path)),
        affected.deleted.iter().map(|path| Path::new(path)),
    )
}

fn format_patch_failure_message(
    display_base_dir: Option<&Path>,
    patch: &str,
    outcome: &ApplyOutcome,
    details: &FailureDetails,
    summary: &str,
    patch_path: &Path,
) -> String {
    let human_summary = human_patch_summary(details, summary);
    let location = diagnostic_location(display_base_dir, details, patch_path);
    let rendered_patch_path = display_path(display_base_dir, patch_path);
    let render_failed_unit_details = !should_compact_single_full_failure(outcome, details);
    let mut seen_help_keys = if render_failed_unit_details {
        rendered_failed_unit_help_keys(&outcome.failed_units)
    } else {
        HashSet::new()
    };
    let mut lines = vec![diagnostic_headline(details)];
    lines.push(format!("  --> {location}"));
    if let Some(source_excerpt) = diagnostic_source_excerpt(patch, details) {
        lines.push(diagnostic_excerpt_gutter_line(source_excerpt.width));
        lines.extend(source_excerpt.lines);
        lines.push(diagnostic_excerpt_gutter_line(source_excerpt.width));
    }
    if let Some(target_line) = render_target_detail_line(
        display_base_dir,
        "   ",
        details.target_path.as_deref().map(Path::new),
        details
            .target_anchor
            .as_ref()
            .map(|anchor| Path::new(&anchor.path)),
    ) {
        lines.push(target_line);
    }
    lines.extend(diagnostic_target_anchor_lines(display_base_dir, details));
    if location != rendered_patch_path {
        lines.push(format!("   = patch: {rendered_patch_path}"));
    }
    if let Some(action_index) = details.action_index {
        lines.push(format!("   = action: {action_index}"));
    }
    if let Some(hunk_index) = details.hunk_index {
        lines.push(format!("   = hunk: {hunk_index}"));
    }
    let committed_change_lines =
        render_affected_change_lines(&outcome.affected, display_base_dir, "");
    if !committed_change_lines.is_empty() {
        lines.push(String::new());
        lines.push("   = committed changes:".to_string());
        lines.extend(render_numbered_lines(&committed_change_lines, "     "));
    }
    if render_failed_unit_details && !outcome.failed_units.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "   = failed file groups: {}",
            outcome.failed_units.len()
        ));
        for (index, failure) in outcome.failed_units.iter().enumerate() {
            lines.push(String::new());
            lines.push(format!("   = failed_group[{}]:", index + 1));
            lines.push(format!("     error[{}]: {}", failure.code, failure.summary));
            if let Some(target_path) = &failure.target_path {
                lines.push(format!(
                    "     --> {}",
                    display_path(display_base_dir, Path::new(target_path))
                ));
            }
            if let Some(patch_pointer) =
                failed_unit_patch_pointer(display_base_dir, patch_path, failure)
            {
                lines.push(format!("     = patch: {patch_pointer}"));
            }
            if let Some(action_index) = failure.action_index {
                lines.push(format!("     = action: {action_index}"));
            }
            if let Some(hunk_index) = failure.hunk_index {
                lines.push(format!("     = hunk: {hunk_index}"));
            }
            let attempted =
                render_affected_change_lines(&failure.attempted, display_base_dir, "       ");
            if !attempted.is_empty() {
                lines.push("     = attempted changes:".to_string());
                lines.extend(attempted);
            }
            lines.push("     = caused by:".to_string());
            lines.extend(indented_block_lines(&failure.message, "       "));
            if let Some(help) = &failure.help {
                lines.push(format!("     = help: {help}"));
            }
        }
    }
    for help in diagnostic_help_messages(details)
        .into_iter()
        .chain(compact_failed_unit_help_messages(outcome, details))
    {
        if seen_help_keys.insert(normalized_help_key(&help)) {
            lines.push(format!("   = help: {help}"));
        }
    }
    lines.push(String::new());
    lines.push("caused by:".to_string());
    for line in human_summary.lines() {
        lines.push(format!("  {line}"));
    }
    lines.join("\n")
}

fn should_compact_single_full_failure(outcome: &ApplyOutcome, details: &FailureDetails) -> bool {
    outcome.failed_units.len() == 1 && details.error_code.as_deref() != Some("PARTIAL_UNIT_FAILURE")
}

fn normalized_help_key(help: &str) -> String {
    help.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn rendered_failed_unit_help_keys(failed_units: &[ApplyOutcomeFailedUnit]) -> HashSet<String> {
    failed_units
        .iter()
        .filter_map(|failure| failure.help.as_deref())
        .map(normalized_help_key)
        .collect()
}

fn compact_failed_unit_help_messages(
    outcome: &ApplyOutcome,
    details: &FailureDetails,
) -> Vec<String> {
    if !should_compact_single_full_failure(outcome, details) {
        return Vec::new();
    }
    if !diagnostic_help_messages(details).is_empty() {
        return Vec::new();
    }
    outcome
        .failed_units
        .first()
        .and_then(|failure| failure.help.as_ref())
        .into_iter()
        .cloned()
        .collect()
}

fn diagnostic_headline(details: &FailureDetails) -> String {
    match details.error_code.as_deref() {
        Some("PARTIAL_UNIT_FAILURE") => {
            "error[PARTIAL_UNIT_FAILURE]: patch partially applied".to_string()
        }
        Some("OUTER_EMPTY_PATCH") => "error[OUTER_EMPTY_PATCH]: patch cannot be empty".to_string(),
        Some("OUTER_INVALID_PATCH") => "error[OUTER_INVALID_PATCH]: invalid patch".to_string(),
        Some("OUTER_INVALID_ADD_LINE") => {
            "error[OUTER_INVALID_ADD_LINE]: Add File block is malformed".to_string()
        }
        Some("OUTER_INVALID_HUNK") => "error[OUTER_INVALID_HUNK]: malformed patch hunk".to_string(),
        Some("OUTER_INVALID_LINE") => {
            "error[OUTER_INVALID_LINE]: update hunk contains an invalid line".to_string()
        }
        Some("MATCH_INVALID_CONTEXT") => {
            "error[MATCH_INVALID_CONTEXT]: patch context did not match target file".to_string()
        }
        Some("MATCH_INVALID_EOF_CONTEXT") => {
            "error[MATCH_INVALID_EOF_CONTEXT]: patch EOF context did not match target file"
                .to_string()
        }
        Some("UPDATE_TARGET_MISSING") => {
            "error[UPDATE_TARGET_MISSING]: Update File targeted a missing path".to_string()
        }
        Some("DELETE_TARGET_MISSING") => {
            "error[DELETE_TARGET_MISSING]: Delete File targeted a missing path".to_string()
        }
        Some("TARGET_READ_ERROR") => {
            "error[TARGET_READ_ERROR]: target path could not be read".to_string()
        }
        Some("TARGET_WRITE_ERROR") => {
            "error[TARGET_WRITE_ERROR]: patch could not be written to the target path".to_string()
        }
        Some(code) => format!("error[{code}]: patch could not be applied"),
        None => "error: patch could not be applied".to_string(),
    }
}

fn diagnostic_location(
    display_base_dir: Option<&Path>,
    details: &FailureDetails,
    patch_path: &Path,
) -> String {
    render_diagnostic_location(
        display_base_dir,
        patch_path,
        details.source_line,
        details.source_column,
        details.target_path.as_deref().map(Path::new),
    )
}

fn diagnostic_source_excerpt(
    patch: &str,
    details: &FailureDetails,
) -> Option<DiagnosticSourceExcerpt> {
    render_diagnostic_source_excerpt(patch, details.source_line, details.source_column, 0)
}

fn diagnostic_target_anchor_lines(
    display_base_dir: Option<&Path>,
    details: &FailureDetails,
) -> Vec<String> {
    anchor_lines(display_base_dir, details.target_anchor.as_ref(), "   ")
}

fn anchor_lines(
    display_base_dir: Option<&Path>,
    anchor: Option<&DiagnosticTargetAnchor>,
    prefix: &str,
) -> Vec<String> {
    let Some(anchor) = anchor else {
        return Vec::new();
    };
    let rendered_path = display_path(display_base_dir, Path::new(&anchor.path));
    let mut lines = vec![format!(
        "{prefix}= target anchor: {}:{}:{}",
        rendered_path, anchor.line_number, anchor.column_number
    )];
    if let Some(excerpt) = &anchor.excerpt {
        let width = anchor.line_number.to_string().len();
        let caret_padding = " ".repeat(anchor.column_number.saturating_sub(1));
        lines.push(format!(
            "{prefix}  {line:>width$} | {excerpt}",
            line = anchor.line_number,
            width = width
        ));
        let mut caret_line = format!("{caret_padding}^");
        if let Some(label) = &anchor.label {
            caret_line.push(' ');
            caret_line.push_str(label);
        }
        lines.push(format!(
            "{prefix}  {:>width$} | {caret_line}",
            "",
            width = width
        ));
    }
    lines
}

fn render_numbered_lines(lines: &[String], prefix: &str) -> Vec<String> {
    let width = lines.len().max(1).to_string().len();
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            format!(
                "{prefix}[{number:0width$}] {line}",
                number = index + 1,
                width = width
            )
        })
        .collect()
}

fn indented_block_lines(text: &str, prefix: &str) -> Vec<String> {
    text.lines().map(|line| format!("{prefix}{line}")).collect()
}

fn failed_unit_patch_pointer(
    display_base_dir: Option<&Path>,
    patch_path: &Path,
    failure: &ApplyOutcomeFailedUnit,
) -> Option<String> {
    let source_line = failure.source_line?;
    let source_column = failure.source_column.unwrap_or(1);
    Some(format!(
        "{}:{source_line}:{source_column}",
        display_path(display_base_dir, patch_path)
    ))
}

fn diagnostic_help_messages(details: &FailureDetails) -> Vec<String> {
    match details.error_code.as_deref() {
        Some("PARTIAL_UNIT_FAILURE") => vec![
            "re-read committed files and retry only the failing groups".to_string(),
            "do not reapply committed groups unchanged".to_string(),
        ],
        Some("UPDATE_TARGET_MISSING") => {
            vec!["create the file first or use Add File if you intend to create it".to_string()]
        }
        Some("DELETE_TARGET_MISSING") => vec![
            "re-read the workspace and remove the delete if the file is already gone".to_string(),
        ],
        Some("TARGET_READ_ERROR") | Some("TARGET_WRITE_ERROR") => {
            vec!["repair the target path permissions or filesystem state and retry".to_string()]
        }
        Some("MATCH_INVALID_CONTEXT") | Some("MATCH_INVALID_EOF_CONTEXT") => {
            vec!["re-read the target file and regenerate the patch with fresh context".to_string()]
        }
        Some("OUTER_EMPTY_PATCH") => {
            vec!["provide a complete patch envelope before retrying".to_string()]
        }
        Some("OUTER_INVALID_PATCH") => vec!["repair the patch shell before retrying".to_string()],
        Some("OUTER_INVALID_ADD_LINE") => {
            vec!["prefix each Add File content line with '+' before retrying".to_string()]
        }
        Some("OUTER_INVALID_HUNK") => vec!["repair the malformed hunk before retrying".to_string()],
        Some("OUTER_INVALID_LINE") => {
            vec!["prefix each update line with ' ', '+', or '-' before retrying".to_string()]
        }
        _ => details.fix_hint.iter().cloned().collect(),
    }
}

fn human_patch_summary(details: &FailureDetails, summary: &str) -> String {
    match details.error_code.as_deref() {
        Some("OUTER_INVALID_ADD_LINE") => {
            "Add File block requires lines prefixed with '+'".to_string()
        }
        Some("OUTER_INVALID_LINE") => {
            "Update File block contains an invalid line prefix".to_string()
        }
        _ => summary.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch_runtime::{
        ApplyOutcome, ApplyOutcomeAffected, apply_patch_program, apply_patch_program_with_source,
    };
    use std::fs;

    #[test]
    fn patch_success_preserves_no_op_line() {
        let outcome = ApplyOutcome {
            ok: true,
            patch_source: None,
            affected: ApplyOutcomeAffected::default(),
            file_paths: Vec::new(),
            warnings: Vec::new(),
            failed_units: Vec::new(),
            error: None,
        };
        let rendered = format_patch_success_message(&outcome, None);
        assert_eq!(
            rendered,
            "Success. Updated the following files:\n(no file changes)"
        );
    }

    #[test]
    fn patch_failure_persists_failed_patch_source_for_inline_patch() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join("app.py"), "value = 1\n").expect("seed file");
        let patch = "*** Begin Patch\n*** Update File: app.py\n@@ 1 | value = 1\n-missing = 1\n+value = 2\n*** End Patch\n";
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let outcome = apply_patch_program(patch, &context.runtime_base_dir);
        let message =
            format_patch_outcome(patch, &context, &outcome).expect_err("patch should fail");

        assert!(message.contains("error[MATCH_INVALID_CONTEXT]"));
        assert!(!message.contains("--> <patch>:4:1"));
        assert!(message.contains("4 | -missing = 1"));
        assert_eq!(
            message
                .matches("re-read the target file and regenerate the patch with fresh context")
                .count(),
            1
        );
        assert!(!message.contains("Patch context could not be matched against the target file"));
        assert!(!message.contains("failed file groups:"));
        assert!(message.contains(".docutouch"));
        assert!(temp.path().join(".docutouch").exists());
    }

    #[test]
    fn patch_failure_renders_explicit_patch_file_hint_from_runtime() {
        let temp = tempfile::tempdir().expect("tempdir");
        let patch_path = temp.path().join("input.patch");
        fs::write(temp.path().join("app.py"), "value = 1\n").expect("seed file");
        let patch = "*** Begin Patch\n*** Update File: app.py\n@@ 1 | value = 1\n-missing = 1\n+value = 2\n*** End Patch\n";
        fs::write(&patch_path, patch).expect("seed patch file");
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let outcome =
            apply_patch_program_with_source(patch, &context.runtime_base_dir, Some(&patch_path));
        let message =
            format_patch_outcome(patch, &context, &outcome).expect_err("patch should fail");

        assert!(message.contains("  --> input.patch:4:1"));
        assert!(message.contains("   = patch: input.patch"));
        assert!(!message.contains(".docutouch/failed-patches/"));
    }

    #[test]
    fn empty_patch_renders_structured_failure_without_fake_span() {
        let temp = tempfile::tempdir().expect("tempdir");
        let patch = "";
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let outcome = apply_patch_program(patch, &context.runtime_base_dir);
        let message =
            format_patch_outcome(patch, &context, &outcome).expect_err("patch should fail");

        assert!(message.contains("error[OUTER_EMPTY_PATCH]: patch cannot be empty"));
        assert!(message.contains(".docutouch"));
        assert!(!message.contains("  --> <patch>"));
        assert!(message.contains("provide a complete patch envelope before retrying"));
        assert!(message.contains("\ncaused by:\n  patch cannot be empty"));
        assert!(!message.contains("| ^"));
        assert!(!message.contains("failed file groups:"));
    }

    #[test]
    fn patch_failure_prefers_runtime_owned_persisted_patch_source_over_embedded_hint() {
        let temp = tempfile::tempdir().expect("tempdir");
        let patch = "*** Patch File: missing.patch";
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let outcome = apply_patch_program(patch, &context.runtime_base_dir);
        let message = format_patch_outcome(patch, &context, &outcome)
            .expect_err("patch should fail with persisted fallback");

        assert!(message.contains(".docutouch"));
        assert!(!message.contains("--> missing.patch"));
        assert!(!message.contains("= patch: missing.patch"));
    }

    #[test]
    fn patch_failure_renders_embedded_patch_file_from_runtime() {
        let temp = tempfile::tempdir().expect("tempdir");
        let patch_path = temp.path().join("embedded.patch");
        fs::write(&patch_path, "*** Begin Patch\n*** End Patch\n").expect("seed patch file");
        let patch = "*** Patch File: embedded.patch";
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let outcome = apply_patch_program(patch, &context.runtime_base_dir);
        let message =
            format_patch_outcome(patch, &context, &outcome).expect_err("patch should fail");

        assert!(message.contains("  --> embedded.patch"));
        assert!(message.contains("   = help: repair the patch shell before retrying"));
        assert!(!message.contains(".docutouch/failed-patches/"));
    }

    fn assert_excerpt_gutter_aligned(message: &str, excerpt_line: &str) {
        let lines = message.lines().collect::<Vec<_>>();
        let excerpt_index = lines
            .iter()
            .position(|line| *line == excerpt_line)
            .unwrap_or_else(|| panic!("expected excerpt line `{excerpt_line}`"));

        let excerpt_bar = lines[excerpt_index]
            .find('|')
            .unwrap_or_else(|| panic!("expected `|` in excerpt line `{excerpt_line}`"));
        let gutter_bar = lines[excerpt_index - 1]
            .find('|')
            .expect("expected excerpt gutter above");
        let caret_bar = lines[excerpt_index + 1]
            .find('|')
            .expect("expected caret line after excerpt line");
        let gutter_after_bar = lines[excerpt_index + 2]
            .find('|')
            .expect("expected excerpt gutter below");

        assert_eq!(gutter_bar, excerpt_bar);
        assert_eq!(caret_bar, excerpt_bar);
        assert_eq!(gutter_after_bar, excerpt_bar);
    }

    #[test]
    fn patch_failure_renders_dynamic_excerpt_gutters_aligned_to_line_number_width() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join("app.py"), "value = 1\n").expect("seed file");
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let patch_one_digit = "*** Begin Patch\n*** Update File: app.py\n@@ 1 | value = 1\n-missing = 1\n+value = 2\n*** End Patch\n";
        let outcome = apply_patch_program(patch_one_digit, &context.runtime_base_dir);
        let message = format_patch_outcome(patch_one_digit, &context, &outcome)
            .expect_err("patch should fail");
        assert_excerpt_gutter_aligned(&message, "4 | -missing = 1");

        let patch_two_digit = concat!(
            "*** Begin Patch\n",
            "*** Add File: a.txt\n",
            "+ok\n",
            "*** Add File: b.txt\n",
            "+ok\n",
            "*** Add File: c.txt\n",
            "+ok\n",
            "*** Add File: d.txt\n",
            "+ok\n",
            "*** Add File: e.txt\n",
            "broken line\n",
            "*** End Patch\n",
        );
        let outcome = apply_patch_program(patch_two_digit, &context.runtime_base_dir);
        let message = format_patch_outcome(patch_two_digit, &context, &outcome)
            .expect_err("patch should fail");
        assert_excerpt_gutter_aligned(&message, "11 | broken line");
    }

    #[test]
    fn partial_failure_renders_full_committed_list_without_omission_prose() {
        let temp = tempfile::tempdir().expect("tempdir");
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };

        let mut patch = String::from("*** Begin Patch\n");
        for index in 0..10 {
            patch.push_str(&format!(
                "*** Add File: created-{index}.txt\n+hello {index}\n"
            ));
        }
        patch.push_str("*** Update File: missing.txt\n@@ 1 | old\n-old\n+new\n*** End Patch\n");

        let outcome = apply_patch_program(&patch, &context.runtime_base_dir);
        let message =
            format_patch_outcome(&patch, &context, &outcome).expect_err("patch should fail");

        assert!(message.contains("committed changes:"));
        assert!(!message.contains("showing 8 of 10"));
        assert!(!message.contains("... and 2 more committed changes"));
        for index in 0..10 {
            assert!(
                message.contains(&format!("A created-{index}.txt")),
                "missing committed path created-{index}.txt in:\n{message}"
            );
        }
    }

    #[test]
    fn partial_failure_indents_multiline_failed_group_causes() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join("a.txt"), "value = 1\n").expect("seed a");
        fs::write(temp.path().join("b.txt"), "value = 1\n").expect("seed b");
        let context = PatchPresentationContext {
            runtime_base_dir: temp.path().to_path_buf(),
            display_base_dir: Some(temp.path().to_path_buf()),
        };
        let patch = concat!(
            "*** Begin Patch\n",
            "*** Add File: created.txt\n",
            "+hello\n",
            "*** Update File: a.txt\n",
            "@@ 1 | value = 1\n",
            "-missing a\n",
            "+value = 2\n",
            "*** Update File: b.txt\n",
            "@@ 1 | value = 1\n",
            "-missing b\n",
            "+value = 3\n",
            "*** End Patch\n",
        );

        let outcome = apply_patch_program(patch, &context.runtime_base_dir);
        let message =
            format_patch_outcome(patch, &context, &outcome).expect_err("patch should fail");
        let lines = message.lines().collect::<Vec<_>>();

        assert!(message.contains("   = failed_group[1]:"));
        assert!(
            lines
                .iter()
                .filter(|line| line.starts_with("     = patch: .docutouch/failed-patches/"))
                .count()
                == 2,
            "expected one per-group patch pointer for each mismatch block in:\n{message}"
        );

        assert!(
            !lines.iter().any(|line| *line == "missing a"),
            "multiline cause leaked out of indentation:\n{message}"
        );
        assert!(
            !lines.iter().any(|line| *line == "missing b"),
            "multiline cause leaked out of indentation:\n{message}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.trim() == "missing a" && line.starts_with("       ")),
            "expected indented failed-group cause evidence for missing a:\n{message}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.trim() == "missing b" && line.starts_with("       ")),
            "expected indented failed-group cause evidence for missing b:\n{message}"
        );
    }
}
