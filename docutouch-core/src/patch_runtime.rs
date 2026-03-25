use codex_apply_patch::{
    ApplyPatchError as OfficialApplyPatchError, ApplyPatchStatus,
    ApplyPatchTargetAnchor as OfficialApplyPatchTargetAnchor,
    ApplyPatchWarning as OfficialApplyPatchWarning, FailedUnit, Hunk,
    ParseError as OfficialParseError, apply_patch_in_dir, parse_patch,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FAILED_PATCH_SOURCE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchSourceKind {
    Hint,
    Embedded,
    Persisted,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PatchSourceReference {
    pub path: String,
    pub kind: PatchSourceKind,
}

impl PatchSourceReference {
    fn new(path: PathBuf, kind: PatchSourceKind) -> Self {
        Self {
            path: path.to_string_lossy().into_owned(),
            kind,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct FailureDetails {
    pub phase: Option<String>,
    pub category: Option<String>,
    pub error_code: Option<String>,
    pub patch_file: Option<String>,
    pub target_path: Option<String>,
    pub action_index: Option<usize>,
    pub hunk_index: Option<usize>,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub target_anchor: Option<DiagnosticTargetAnchor>,
    pub fix_hint: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FailurePayload {
    pub kind: String,
    pub summary: String,
    pub details: FailureDetails,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApplyOutcomeWarning {
    pub code: String,
    pub summary: String,
    pub target_path: String,
    pub phase: Option<String>,
    pub category: Option<String>,
    pub help: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticTargetAnchor {
    pub path: String,
    pub line_number: usize,
    pub column_number: usize,
    pub label: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ApplyOutcomeAffected {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ApplyOutcomeFailedUnit {
    pub phase: Option<String>,
    pub category: Option<String>,
    pub touched_paths: Vec<String>,
    pub attempted: ApplyOutcomeAffected,
    pub code: String,
    pub summary: String,
    pub target_path: Option<String>,
    pub action_index: Option<usize>,
    pub hunk_index: Option<usize>,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub target_anchor: Option<DiagnosticTargetAnchor>,
    pub help: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApplyOutcome {
    pub ok: bool,
    pub patch_source: Option<PatchSourceReference>,
    pub affected: ApplyOutcomeAffected,
    pub file_paths: Vec<String>,
    pub warnings: Vec<ApplyOutcomeWarning>,
    pub failed_units: Vec<ApplyOutcomeFailedUnit>,
    pub error: Option<FailurePayload>,
}

impl ApplyOutcome {
    fn ok(affected: ApplyOutcomeAffected, warnings: Vec<ApplyOutcomeWarning>) -> Self {
        Self {
            ok: true,
            patch_source: None,
            file_paths: affected_file_paths(&affected),
            affected,
            warnings,
            failed_units: Vec::new(),
            error: None,
        }
    }

    fn err(
        summary: String,
        details: FailureDetails,
        affected: ApplyOutcomeAffected,
        file_paths: Vec<String>,
        warnings: Vec<ApplyOutcomeWarning>,
        failed_units: Vec<ApplyOutcomeFailedUnit>,
    ) -> Self {
        Self {
            ok: false,
            patch_source: None,
            affected,
            file_paths,
            warnings,
            failed_units,
            error: Some(FailurePayload {
                kind: "apply".to_string(),
                summary,
                details,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatchWorkspaceRequirement {
    NeedsWorkspace,
    AbsoluteOnly { anchor_dir: PathBuf },
    Unanchored,
}

pub fn apply_patch_program(program: &str, base_dir: &Path) -> ApplyOutcome {
    apply_patch_program_with_source(program, base_dir, None)
}

pub fn apply_patch_program_with_source(
    program: &str,
    base_dir: &Path,
    patch_source_path_hint: Option<&Path>,
) -> ApplyOutcome {
    let mut patch_source =
        primary_patch_source_reference(program, base_dir, patch_source_path_hint);
    let mut outcome = apply_patch_program_inner(program, base_dir);
    if !outcome.ok && patch_source.is_none() {
        patch_source = persist_failed_patch_source(program, base_dir)
            .map(|path| PatchSourceReference::new(path, PatchSourceKind::Persisted));
    }
    if let Some(reference) = patch_source.clone() {
        if let Some(error) = outcome.error.as_mut() {
            error.details.patch_file = Some(reference.path.clone());
        }
    }
    outcome.patch_source = patch_source;
    outcome
}

fn primary_patch_source_reference(
    program: &str,
    base_dir: &Path,
    patch_source_path_hint: Option<&Path>,
) -> Option<PatchSourceReference> {
    patch_source_path_hint
        .map(Path::to_path_buf)
        .filter(|path| is_readable_file(path))
        .map(|path| PatchSourceReference::new(path, PatchSourceKind::Hint))
        .or_else(|| {
            formal_patch_path(program, base_dir)
                .filter(|path| is_readable_file(path))
                .map(|path| PatchSourceReference::new(path, PatchSourceKind::Embedded))
        })
}

fn is_readable_file(path: &Path) -> bool {
    path.is_file() && fs::File::open(path).is_ok()
}

fn apply_patch_program_inner(program: &str, base_dir: &Path) -> ApplyOutcome {
    if program.trim().is_empty() {
        return ApplyOutcome::err(
            "patch cannot be empty".to_string(),
            FailureDetails {
                phase: Some("parse".to_string()),
                category: Some("outer_format".to_string()),
                error_code: Some("OUTER_EMPTY_PATCH".to_string()),
                fix_hint: Some("provide a complete patch envelope before retrying".to_string()),
                ..FailureDetails::default()
            },
            ApplyOutcomeAffected::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }
    match apply_patch_in_dir(program, base_dir) {
        Ok(report) => {
            let affected = affected_from_report(&report.affected);
            let file_paths = affected_file_paths(&affected);
            let warnings = outcome_warnings(&report.warnings);
            let failed_units = outcome_failed_units(&report.failed_units);
            match report.status {
                ApplyPatchStatus::FullSuccess => ApplyOutcome::ok(affected, warnings),
                ApplyPatchStatus::PartialSuccess => {
                    let summary = partial_summary(&report.failed_units, report.committed_units);
                    let details = failure_details_from_failed_units(
                        "PARTIAL_UNIT_FAILURE",
                        &report.failed_units,
                        Some("some file groups were committed while others failed"),
                    );
                    ApplyOutcome::err(
                        summary,
                        details,
                        affected,
                        file_paths,
                        warnings,
                        failed_units,
                    )
                }
                ApplyPatchStatus::Failure => {
                    let summary = first_failure_summary(&report.failed_units);
                    let details = failure_details_from_failed_units(
                        "UNIT_FAILURE",
                        &report.failed_units,
                        Some("repair the failing file group and retry the same patch"),
                    );
                    ApplyOutcome::err(
                        summary,
                        details,
                        affected,
                        file_paths,
                        warnings,
                        failed_units,
                    )
                }
            }
        }
        Err(err) => match err {
            OfficialApplyPatchError::ParseError(parse_error) => {
                let (summary, details) = map_parse_error(parse_error);
                ApplyOutcome::err(
                    summary,
                    details,
                    ApplyOutcomeAffected::default(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
            other => {
                let summary = other.to_string();
                let details = FailureDetails {
                    phase: Some("apply".to_string()),
                    category: Some("apply".to_string()),
                    error_code: Some("PATCH_EXECUTION_ERROR".to_string()),
                    fix_hint: Some(
                        "repair the failing file group and retry the same patch".to_string(),
                    ),
                    ..FailureDetails::default()
                };
                ApplyOutcome::err(
                    summary,
                    details,
                    ApplyOutcomeAffected::default(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
        },
    }
}

pub fn formal_patch_path(program: &str, base_dir: &Path) -> Option<PathBuf> {
    let trimmed = program.trim();
    let prefix = "*** Patch File: ";
    if !trimmed.starts_with(prefix) {
        return None;
    }
    let lines = trimmed.lines().collect::<Vec<_>>();
    if lines.len() != 1 {
        return None;
    }
    let raw_path = lines[0].strip_prefix(prefix)?.trim();
    if raw_path.is_empty() {
        return None;
    }
    Some(resolve_patch_path(base_dir, raw_path))
}

pub fn extract_patch_paths(program: &str) -> Vec<PathBuf> {
    let Ok(parsed) = parse_patch(program) else {
        return Vec::new();
    };

    parsed
        .hunks
        .iter()
        .flat_map(hunk_paths)
        .map(Path::to_path_buf)
        .collect()
}

pub fn patch_workspace_requirement(program: &str) -> PatchWorkspaceRequirement {
    let mut anchor_dir = None;

    for path in extract_patch_paths(program) {
        if path.is_absolute() {
            if anchor_dir.is_none() {
                anchor_dir = Some(path_anchor_dir(&path));
            }
            continue;
        }
        return PatchWorkspaceRequirement::NeedsWorkspace;
    }

    anchor_dir
        .map(|anchor_dir| PatchWorkspaceRequirement::AbsoluteOnly { anchor_dir })
        .unwrap_or(PatchWorkspaceRequirement::Unanchored)
}

fn resolve_patch_path(base_dir: &Path, raw_path: &str) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}

fn persist_failed_patch_source(program: &str, base_dir: &Path) -> Option<PathBuf> {
    if base_dir.as_os_str().is_empty() || !base_dir.is_absolute() {
        return None;
    }
    let dir = base_dir.join(".docutouch").join("failed-patches");
    fs::create_dir_all(&dir).ok()?;
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    let sequence = FAILED_PATCH_SOURCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("{millis:013}-{sequence}.patch"));
    fs::write(&path, program).ok()?;
    Some(path)
}

fn hunk_paths(hunk: &Hunk) -> Vec<&Path> {
    match hunk {
        Hunk::AddFile { path, .. } => vec![path.as_path()],
        Hunk::DeleteFile { path } => vec![path.as_path()],
        Hunk::UpdateFile {
            path, move_path, ..
        } => {
            let mut paths = vec![path.as_path()];
            if let Some(move_path) = move_path.as_ref() {
                paths.push(move_path.as_path());
            }
            paths
        }
    }
}

fn path_anchor_dir(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf())
}

fn affected_from_report(affected: &codex_apply_patch::AffectedPaths) -> ApplyOutcomeAffected {
    ApplyOutcomeAffected {
        added: affected
            .added
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        modified: affected
            .modified
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        deleted: affected
            .deleted
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
    }
}

fn affected_file_paths(affected: &ApplyOutcomeAffected) -> Vec<String> {
    let mut paths = affected
        .added
        .iter()
        .chain(affected.modified.iter())
        .chain(affected.deleted.iter())
        .cloned()
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn outcome_warnings(warnings: &[OfficialApplyPatchWarning]) -> Vec<ApplyOutcomeWarning> {
    warnings
        .iter()
        .map(|warning| ApplyOutcomeWarning {
            code: warning.code.clone(),
            summary: warning.summary.clone(),
            target_path: warning.target_path.to_string_lossy().into_owned(),
            phase: warning_phase(&warning.code).map(ToOwned::to_owned),
            category: warning_category(&warning.code).map(ToOwned::to_owned),
            help: warning.help.clone(),
        })
        .collect()
}

fn outcome_failed_units(failed_units: &[FailedUnit]) -> Vec<ApplyOutcomeFailedUnit> {
    failed_units
        .iter()
        .map(|failure| {
            let (phase, category) = diagnostic_taxonomy_for_code(&failure.code);
            ApplyOutcomeFailedUnit {
                phase: Some(phase.to_string()),
                category: Some(category.to_string()),
                touched_paths: failure
                    .touched_paths
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect(),
                attempted: ApplyOutcomeAffected {
                    added: failure
                        .attempted
                        .added
                        .iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect(),
                    modified: failure
                        .attempted
                        .modified
                        .iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect(),
                    deleted: failure
                        .attempted
                        .deleted
                        .iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect(),
                },
                code: failure.code.clone(),
                summary: failure.summary.clone(),
                target_path: failure
                    .target_path
                    .as_ref()
                    .map(|path| path.to_string_lossy().into_owned()),
                action_index: failure.action_index,
                hunk_index: failure.hunk_index,
                source_line: failure.source_line,
                source_column: failure.source_column,
                target_anchor: failure.target_anchor.as_ref().map(diagnostic_target_anchor),
                help: failure.help.clone(),
                message: failure.message.clone(),
            }
        })
        .collect()
}

fn partial_summary(failed_units: &[FailedUnit], committed_units: usize) -> String {
    let committed = committed_units;
    let failed = failed_units.len();
    format!(
        "Patch partially applied.\n{} committed {} succeeded.\n{}.",
        committed,
        pluralize(committed, "file group", "file groups"),
        failed_summary_line(failed),
    )
}

fn first_failure_summary(failed_units: &[FailedUnit]) -> String {
    failed_units
        .first()
        .map(|failure| failure.message.clone())
        .unwrap_or_else(|| "Patch failed.".to_string())
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn failed_summary_line(count: usize) -> String {
    if count == 1 {
        "1 failed file group requires repair".to_string()
    } else {
        format!("{count} failed file groups require repair")
    }
}

fn failure_details_from_failed_units(
    error_code: &str,
    failed_units: &[FailedUnit],
    fix_hint: Option<&str>,
) -> FailureDetails {
    let primary = failed_units.first();
    let target_path = primary
        .and_then(|failure| {
            failure
                .target_path
                .as_ref()
                .or_else(|| failure.touched_paths.first())
        })
        .map(|path| path.to_string_lossy().into_owned());
    let resolved_error_code = if error_code == "UNIT_FAILURE" {
        primary
            .map(|failure| failure.code.clone())
            .unwrap_or_else(|| error_code.to_string())
    } else {
        error_code.to_string()
    };
    let (phase, category) = diagnostic_taxonomy_for_code(&resolved_error_code);
    FailureDetails {
        phase: Some(phase.to_string()),
        category: Some(category.to_string()),
        error_code: Some(resolved_error_code),
        target_path,
        action_index: primary.and_then(|failure| failure.action_index),
        hunk_index: primary.and_then(|failure| failure.hunk_index),
        source_line: primary.and_then(|failure| failure.source_line),
        source_column: primary.and_then(|failure| failure.source_column),
        target_anchor: primary
            .and_then(|failure| failure.target_anchor.as_ref())
            .map(diagnostic_target_anchor),
        fix_hint: fix_hint.map(ToOwned::to_owned),
        ..FailureDetails::default()
    }
}

fn map_parse_error(parse_error: OfficialParseError) -> (String, FailureDetails) {
    match parse_error {
        OfficialParseError::InvalidPatchError(summary) => (
            summary,
            FailureDetails {
                phase: Some("parse".to_string()),
                category: Some("outer_format".to_string()),
                error_code: Some("OUTER_INVALID_PATCH".to_string()),
                fix_hint: Some("repair the patch shell before retrying".to_string()),
                ..FailureDetails::default()
            },
        ),
        OfficialParseError::InvalidHunkError {
            message,
            line_number,
        } => (
            message.clone(),
            FailureDetails {
                phase: Some("parse".to_string()),
                category: Some("outer_format".to_string()),
                error_code: Some(classify_invalid_hunk_message(&message).to_string()),
                source_line: Some(line_number),
                source_column: Some(1),
                fix_hint: Some(invalid_hunk_fix_hint(&message).to_string()),
                ..FailureDetails::default()
            },
        ),
    }
}

fn diagnostic_target_anchor(anchor: &OfficialApplyPatchTargetAnchor) -> DiagnosticTargetAnchor {
    DiagnosticTargetAnchor {
        path: anchor.path.to_string_lossy().into_owned(),
        line_number: anchor.line_number,
        column_number: anchor.column_number,
        label: anchor.label.clone(),
        excerpt: anchor.excerpt.clone(),
    }
}

fn diagnostic_taxonomy_for_code(code: &str) -> (&'static str, &'static str) {
    match code {
        "PARTIAL_UNIT_FAILURE" => ("report", "partial_apply"),
        "OUTER_EMPTY_PATCH"
        | "OUTER_INVALID_PATCH"
        | "OUTER_INVALID_ADD_LINE"
        | "OUTER_INVALID_HUNK"
        | "OUTER_INVALID_LINE" => ("parse", "outer_format"),
        "MATCH_INVALID_CONTEXT" | "MATCH_INVALID_EOF_CONTEXT" => ("plan", "context_match"),
        "UPDATE_TARGET_MISSING" | "DELETE_TARGET_MISSING" => ("plan", "target_state"),
        "TARGET_READ_ERROR" => ("plan", "filesystem"),
        "TARGET_WRITE_ERROR" => ("commit", "filesystem"),
        _ => ("apply", "apply"),
    }
}

fn warning_phase(code: &str) -> Option<&'static str> {
    match code {
        "ADD_REPLACED_EXISTING_FILE" | "MOVE_REPLACED_EXISTING_DESTINATION" => Some("plan"),
        _ => None,
    }
}

fn warning_category(code: &str) -> Option<&'static str> {
    match code {
        "ADD_REPLACED_EXISTING_FILE" | "MOVE_REPLACED_EXISTING_DESTINATION" => {
            Some("compatibility")
        }
        _ => None,
    }
}

fn classify_invalid_hunk_message(message: &str) -> &'static str {
    if message == "Add File block requires lines prefixed with '+'" {
        "OUTER_INVALID_ADD_LINE"
    } else if message.starts_with("Unexpected line found in update hunk:") {
        "OUTER_INVALID_LINE"
    } else {
        "OUTER_INVALID_HUNK"
    }
}

fn invalid_hunk_fix_hint(message: &str) -> &'static str {
    match classify_invalid_hunk_message(message) {
        "OUTER_INVALID_ADD_LINE" => "prefix each Add File content line with '+' before retrying",
        "OUTER_INVALID_LINE" => "prefix each update line with ' ', '+', or '-' before retrying",
        _ => "repair the malformed hunk before retrying",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn apply_patch_program_reports_full_success() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("app.txt");
        fs::write(&target, "old\n").unwrap();

        let outcome = apply_patch_program(
            &format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-old\n+new\n*** End Patch",
                target.display()
            ),
            dir.path(),
        );

        assert!(outcome.ok);
        assert_eq!(fs::read_to_string(target).unwrap(), "new\n");
        assert!(outcome.warnings.is_empty());
        assert_eq!(outcome.affected.modified.len(), 1);
    }

    #[test]
    fn apply_patch_program_reports_partial_success_with_committed_files() {
        let dir = tempdir().unwrap();
        let created = dir.path().join("created.txt");

        let outcome = apply_patch_program(
            "*** Begin Patch\n*** Add File: created.txt\n+hello\n*** Update File: missing.txt\n@@ 1 | old\n-old\n+new\n*** End Patch",
            dir.path(),
        );

        assert!(!outcome.ok);
        assert_eq!(fs::read_to_string(&created).unwrap(), "hello\n");
        assert_eq!(
            outcome.file_paths,
            vec![created.to_string_lossy().into_owned()]
        );
        assert_eq!(outcome.affected.added.len(), 1);
        let error = outcome.error.expect("error payload");
        assert!(error.summary.contains("Patch partially applied"));
        assert_eq!(
            error.details.error_code.as_deref(),
            Some("PARTIAL_UNIT_FAILURE")
        );
        assert_eq!(error.details.phase.as_deref(), Some("report"));
        assert_eq!(error.details.category.as_deref(), Some("partial_apply"));
        assert_eq!(error.details.source_line, Some(4));
        assert_eq!(error.details.source_column, Some(1));
        assert_eq!(outcome.failed_units[0].source_line, Some(4));
        assert_eq!(outcome.failed_units[0].source_column, Some(1));
    }

    #[test]
    fn apply_patch_program_reports_execution_failure_source_location_for_context_mismatch() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("app.txt");
        fs::write(&target, "value = 1\n").unwrap();

        let outcome = apply_patch_program(
            "*** Begin Patch\n*** Update File: app.txt\n@@ 1 | value = 1\n-missing = 1\n+value = 2\n*** End Patch",
            dir.path(),
        );

        assert!(!outcome.ok);
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.error_code.as_deref(),
            Some("MATCH_INVALID_CONTEXT")
        );
        assert_eq!(error.details.phase.as_deref(), Some("plan"));
        assert_eq!(error.details.category.as_deref(), Some("context_match"));
        assert_eq!(error.details.source_line, Some(4));
        assert_eq!(error.details.source_column, Some(1));
        assert_eq!(outcome.failed_units[0].hunk_index, Some(1));
        assert_eq!(outcome.failed_units[0].source_line, Some(4));
        assert_eq!(outcome.failed_units[0].source_column, Some(1));
    }

    #[test]
    fn apply_patch_program_prefers_first_removed_line_when_context_precedes_mismatch() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("app.txt");
        fs::write(&target, "context\nother\nvalue = 1\n").unwrap();

        let outcome = apply_patch_program(
            "*** Begin Patch\n*** Update File: app.txt\n@@\n context\n other\n-missing = 1\n+value = 2\n*** End Patch",
            dir.path(),
        );

        assert!(!outcome.ok);
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.error_code.as_deref(),
            Some("MATCH_INVALID_CONTEXT")
        );
        assert_eq!(error.details.source_line, Some(6));
        assert_eq!(error.details.source_column, Some(1));
        assert_eq!(outcome.failed_units[0].source_line, Some(6));
        assert_eq!(outcome.failed_units[0].source_column, Some(1));
    }

    #[test]
    fn apply_patch_program_reports_target_anchor_for_context_guided_mismatch() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("app.txt");
        fs::write(&target, "fn handler():\n    value = 1\n").unwrap();

        let outcome = apply_patch_program(
            "*** Begin Patch\n*** Update File: app.txt\n@@ 1 | fn handler():\n-    missing = 1\n+    value = 2\n*** End Patch",
            dir.path(),
        );

        assert!(!outcome.ok);
        let error = outcome.error.expect("error payload");
        let anchor = error.details.target_anchor.as_ref().expect("target anchor");
        assert!(anchor.path.ends_with("app.txt"));
        assert_eq!(anchor.line_number, 1);
        assert_eq!(anchor.column_number, 1);
        assert!(anchor.label.as_ref().is_some_and(|label| !label.is_empty()));
        assert_eq!(anchor.excerpt.as_deref(), Some("fn handler():"));
        assert_eq!(outcome.failed_units[0].touched_paths.len(), 1);
        assert!(outcome.failed_units[0].touched_paths[0].ends_with("app.txt"));
    }

    #[test]
    fn apply_patch_program_reports_warning_when_add_replaces_existing_file() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("app.txt");
        fs::write(&target, "old\n").unwrap();

        let outcome = apply_patch_program(
            "*** Begin Patch\n*** Add File: app.txt\n+new\n*** End Patch",
            dir.path(),
        );

        assert!(outcome.ok);
        assert_eq!(fs::read_to_string(&target).unwrap(), "new\n");
        assert_eq!(outcome.warnings.len(), 1);
        assert_eq!(
            outcome.warnings[0].code,
            "ADD_REPLACED_EXISTING_FILE".to_string()
        );
        assert_eq!(outcome.warnings[0].phase.as_deref(), Some("plan"));
        assert_eq!(
            outcome.warnings[0].category.as_deref(),
            Some("compatibility")
        );
    }

    #[test]
    fn apply_patch_program_reports_warning_when_move_replaces_existing_destination() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("from.txt");
        let dest = dir.path().join("to.txt");
        fs::write(&source, "from\n").unwrap();
        fs::write(&dest, "dest\n").unwrap();

        let outcome = apply_patch_program(
            "*** Begin Patch\n*** Update File: from.txt\n*** Move to: to.txt\n@@\n-from\n+new\n*** End Patch",
            dir.path(),
        );

        assert!(outcome.ok);
        assert_eq!(fs::read_to_string(&dest).unwrap(), "new\n");
        assert_eq!(outcome.warnings.len(), 1);
        assert_eq!(
            outcome.warnings[0].code,
            "MOVE_REPLACED_EXISTING_DESTINATION".to_string()
        );
    }

    #[test]
    fn extract_patch_paths_collects_update_and_move_targets() {
        let paths = extract_patch_paths(
            "*** Begin Patch\n*** Update File: from.txt\n*** Move to: to.txt\n@@ 1 | old\n-old\n+new\n*** Add File: created.txt\n+hello\n*** End Patch",
        );

        assert_eq!(
            paths,
            vec![
                PathBuf::from("from.txt"),
                PathBuf::from("to.txt"),
                PathBuf::from("created.txt"),
            ]
        );
    }

    #[test]
    fn patch_workspace_requirement_requires_workspace_for_relative_paths() {
        let requirement = patch_workspace_requirement(
            "*** Begin Patch\n*** Add File: notes.txt\n+hello\n*** End Patch",
        );

        assert_eq!(requirement, PatchWorkspaceRequirement::NeedsWorkspace);
    }

    #[test]
    fn patch_workspace_requirement_uses_absolute_anchor_for_absolute_paths() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("notes.txt");
        let requirement = patch_workspace_requirement(&format!(
            "*** Begin Patch\n*** Add File: {}\n+hello\n*** End Patch",
            target.display()
        ));

        assert_eq!(
            requirement,
            PatchWorkspaceRequirement::AbsoluteOnly {
                anchor_dir: dir.path().to_path_buf()
            }
        );
    }

    #[test]
    fn patch_workspace_requirement_is_unanchored_when_patch_does_not_parse() {
        let requirement = patch_workspace_requirement("*** Begin Patch\n");

        assert_eq!(requirement, PatchWorkspaceRequirement::Unanchored);
    }

    #[test]
    fn apply_patch_program_persists_failed_patch_source_when_embedded_patch_file_is_unreadable() {
        let dir = tempdir().unwrap();

        let outcome =
            apply_patch_program_with_source("*** Patch File: missing.patch", dir.path(), None);

        assert!(!outcome.ok);
        assert_eq!(
            outcome
                .patch_source
                .as_ref()
                .map(|reference| reference.kind),
            Some(PatchSourceKind::Persisted)
        );
        let error = outcome.error.expect("error payload");
        let patch_file = error.details.patch_file.expect("patch file");
        assert!(patch_file.contains(".docutouch"));
        assert!(Path::new(&patch_file).is_file());
    }

    #[test]
    fn apply_patch_program_uses_explicit_hint_as_primary_patch_source() {
        let dir = tempdir().unwrap();
        let hint_path = dir.path().join("cli-input.patch");
        let embedded_path = dir.path().join("embedded.patch");
        fs::write(&hint_path, "*** Begin Patch\n*** End Patch\n").unwrap();
        fs::write(&embedded_path, "*** Begin Patch\n*** End Patch\n").unwrap();

        let outcome = apply_patch_program_with_source(
            "*** Patch File: embedded.patch",
            dir.path(),
            Some(&hint_path),
        );

        assert!(!outcome.ok);
        let patch_source = outcome.patch_source.expect("patch source");
        assert_eq!(patch_source.kind, PatchSourceKind::Hint);
        assert_eq!(patch_source.path, hint_path.to_string_lossy());
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.patch_file.as_deref(),
            Some(patch_source.path.as_str())
        );
    }

    #[test]
    fn apply_patch_program_uses_embedded_patch_file_when_it_exists() {
        let dir = tempdir().unwrap();
        let embedded_path = dir.path().join("embedded.patch");
        fs::write(&embedded_path, "*** Begin Patch\n*** End Patch\n").unwrap();

        let outcome =
            apply_patch_program_with_source("*** Patch File: embedded.patch", dir.path(), None);

        assert!(!outcome.ok);
        let patch_source = outcome.patch_source.expect("patch source");
        assert_eq!(patch_source.kind, PatchSourceKind::Embedded);
        assert_eq!(patch_source.path, embedded_path.to_string_lossy());
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.patch_file.as_deref(),
            Some(patch_source.path.as_str())
        );
    }

    #[test]
    fn apply_patch_program_falls_back_to_embedded_patch_file_when_hint_path_is_unreadable() {
        let dir = tempdir().unwrap();
        let hint_path = dir.path().join("missing.patch");
        let embedded_path = dir.path().join("embedded.patch");
        fs::write(&embedded_path, "*** Begin Patch\n*** End Patch\n").unwrap();

        let outcome = apply_patch_program_with_source(
            "*** Patch File: embedded.patch",
            dir.path(),
            Some(&hint_path),
        );

        assert!(!outcome.ok);
        let patch_source = outcome.patch_source.expect("patch source");
        assert_eq!(patch_source.kind, PatchSourceKind::Embedded);
        assert_eq!(patch_source.path, embedded_path.to_string_lossy());
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.patch_file.as_deref(),
            Some(patch_source.path.as_str())
        );
    }

    #[test]
    fn apply_patch_program_persists_failed_patch_source_when_hint_path_is_unreadable_and_no_embedded_source_exists()
     {
        let dir = tempdir().unwrap();
        let hint_path = dir.path().join("missing.patch");

        let outcome = apply_patch_program_with_source(
            "*** Patch File: also-missing.patch",
            dir.path(),
            Some(&hint_path),
        );

        assert!(!outcome.ok);
        let patch_source = outcome.patch_source.expect("patch source");
        assert_eq!(patch_source.kind, PatchSourceKind::Persisted);
        assert!(patch_source.path.contains(".docutouch"));
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.patch_file.as_deref(),
            Some(patch_source.path.as_str())
        );
    }

    #[test]
    fn apply_patch_program_rolls_back_failed_move_group() {
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("src");
        let blocked = dir.path().join("blocked");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("name.txt"), "from\n").unwrap();
        fs::write(&blocked, "not a directory\n").unwrap();

        let outcome = apply_patch_program(
            &format!(
                "*** Begin Patch\n*** Update File: {}\n*** Move to: {}\n@@\n-from\n+new\n*** End Patch",
                source_dir.join("name.txt").display(),
                blocked.join("dir").join("name.txt").display()
            ),
            dir.path(),
        );

        assert!(!outcome.ok);
        let error = outcome.error.expect("error payload");
        assert_eq!(
            error.details.error_code.as_deref(),
            Some("TARGET_WRITE_ERROR")
        );
        assert_eq!(error.details.phase.as_deref(), Some("commit"));
        assert_eq!(error.details.category.as_deref(), Some("filesystem"));
        assert_eq!(error.details.source_line, Some(3));
        assert_eq!(error.details.source_column, Some(1));
        assert_eq!(
            fs::read_to_string(source_dir.join("name.txt")).unwrap(),
            "from\n"
        );
        assert!(!blocked.join("dir").join("name.txt").exists());
    }
}
