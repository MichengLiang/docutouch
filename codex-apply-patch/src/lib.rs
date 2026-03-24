mod invocation;
mod mutation_support;
mod parser;
mod seek_sequence;
mod standalone_executable;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
pub use mutation_support::{
    AffectedPaths, ByteFileChange, ByteFileCommitError, ByteFileCommitOperation,
    MissingAfterBehavior, PathIdentityKey, ResolvedPath, RuntimePathMap, RuntimePathState,
    build_connected_path_groups, commit_byte_changes_atomically, diff_affected_paths,
    extend_affected_paths, normalize_patch_path, path_identity_key, resolve_runtime_path,
};
pub use parser::Hunk;
pub use parser::ParseError;
use parser::ParseError::*;
pub use parser::PatchActionSource;
pub use parser::PatchChunkSource;
pub use parser::PatchSourceMap;
pub use parser::UpdateFileChunk;
pub use parser::parse_patch;
pub use parser::parse_patch_with_source_map;
use similar::TextDiff;
use thiserror::Error;

pub use invocation::maybe_parse_apply_patch_verified;
pub use standalone_executable::main;

use crate::invocation::ExtractHeredocError;
use crate::mutation_support::{StagedPathMap, StagedPathState, commit_path_for_key};

/// Detailed instructions for gpt-4.1 on how to use the `apply_patch` tool.
pub const APPLY_PATCH_TOOL_INSTRUCTIONS: &str = include_str!("../apply_patch_tool_instructions.md");

/// Special argv[1] flag used when the Codex executable self-invokes to run the
/// internal `apply_patch` path.
///
/// Although this constant lives in `codex-apply-patch` (to avoid forcing
/// `codex-arg0` to depend on `codex-core`), it is part of the "codex core"
/// process-invocation contract between the apply-patch runtime and the arg0
/// dispatcher.
pub const CODEX_CORE_APPLY_PATCH_ARG1: &str = "--codex-run-as-apply-patch";

#[derive(Debug, Error, PartialEq)]
pub enum ApplyPatchError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    IoError(#[from] IoError),
    /// Error that occurs while computing replacements when applying patch chunks
    #[error("{0}")]
    ComputeReplacements(String),
    #[error("{0}")]
    ExecutionError(String),
    #[error("{0}")]
    PartialApply(String),
    /// A raw patch body was provided without an explicit `apply_patch` invocation.
    #[error(
        "patch detected without explicit call to apply_patch. Rerun as [\"apply_patch\", \"<patch>\"]"
    )]
    ImplicitInvocation,
}

impl From<std::io::Error> for ApplyPatchError {
    fn from(err: std::io::Error) -> Self {
        ApplyPatchError::IoError(IoError {
            context: "I/O error".to_string(),
            source: err,
        })
    }
}

impl From<&std::io::Error> for ApplyPatchError {
    fn from(err: &std::io::Error) -> Self {
        ApplyPatchError::IoError(IoError {
            context: "I/O error".to_string(),
            source: std::io::Error::new(err.kind(), err.to_string()),
        })
    }
}

#[derive(Debug, Error)]
#[error("{context}: {source}")]
pub struct IoError {
    context: String,
    #[source]
    source: std::io::Error,
}

impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context && self.source.to_string() == other.source.to_string()
    }
}

/// Both the raw PATCH argument to `apply_patch` as well as the PATCH argument
/// parsed into hunks.
#[derive(Debug, PartialEq)]
pub struct ApplyPatchArgs {
    pub patch: String,
    pub hunks: Vec<Hunk>,
    pub workdir: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum ApplyPatchFileChange {
    Add {
        content: String,
    },
    Delete {
        content: String,
    },
    Update {
        unified_diff: String,
        move_path: Option<PathBuf>,
        /// new_content that will result after the unified_diff is applied.
        new_content: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyPatchStatus {
    FullSuccess,
    PartialSuccess,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedUnit {
    pub touched_paths: Vec<PathBuf>,
    pub attempted: AttemptedPaths,
    pub code: String,
    pub summary: String,
    pub target_path: Option<PathBuf>,
    pub action_index: Option<usize>,
    pub hunk_index: Option<usize>,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub target_anchor: Option<ApplyPatchTargetAnchor>,
    pub help: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPatchTargetAnchor {
    pub path: PathBuf,
    pub line_number: usize,
    pub column_number: usize,
    pub label: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AttemptedPaths {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPatchWarning {
    pub code: String,
    pub summary: String,
    pub target_path: PathBuf,
    pub help: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPatchReport {
    pub status: ApplyPatchStatus,
    pub affected: AffectedPaths,
    pub committed_units: usize,
    pub warnings: Vec<ApplyPatchWarning>,
    pub failed_units: Vec<FailedUnit>,
}

#[derive(Debug, Clone)]
struct UnitFailure {
    code: String,
    summary: String,
    target_path: Option<PathBuf>,
    action_index: Option<usize>,
    hunk_index: Option<usize>,
    source_line: Option<usize>,
    source_column: Option<usize>,
    target_anchor: Option<ApplyPatchTargetAnchor>,
    help: Option<String>,
    message: String,
}

#[derive(Debug, Clone, Copy)]
struct SourceLocation {
    line_number: usize,
    column_number: usize,
}

#[derive(Debug, Clone, Copy)]
struct PathDiagnosticRef {
    action_index: usize,
    hunk_index: Option<usize>,
    source_location: Option<SourceLocation>,
}

#[derive(Debug, PartialEq)]
pub enum MaybeApplyPatchVerified {
    /// `argv` corresponded to an `apply_patch` invocation, and these are the
    /// resulting proposed file changes.
    Body(ApplyPatchAction),
    /// `argv` could not be parsed to determine whether it corresponds to an
    /// `apply_patch` invocation.
    ShellParseError(ExtractHeredocError),
    /// `argv` corresponded to an `apply_patch` invocation, but it could not
    /// be fulfilled due to the specified error.
    CorrectnessError(ApplyPatchError),
    /// `argv` decidedly did not correspond to an `apply_patch` invocation.
    NotApplyPatch,
}

/// ApplyPatchAction is the result of parsing an `apply_patch` command. By
/// construction, all paths should be absolute paths.
#[derive(Debug, PartialEq)]
pub struct ApplyPatchAction {
    changes: HashMap<PathBuf, ApplyPatchFileChange>,

    /// The raw patch argument that can be used with `apply_patch` as an exec
    /// call. i.e., if the original arg was parsed in "lenient" mode with a
    /// heredoc, this should be the value without the heredoc wrapper.
    pub patch: String,

    /// The working directory that was used to resolve relative paths in the patch.
    pub cwd: PathBuf,
}

impl ApplyPatchAction {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns the changes that would be made by applying the patch.
    pub fn changes(&self) -> &HashMap<PathBuf, ApplyPatchFileChange> {
        &self.changes
    }

    /// Should be used exclusively for testing. (Not worth the overhead of
    /// creating a feature flag for this.)
    pub fn new_add_for_test(path: &Path, content: String) -> Self {
        if !path.is_absolute() {
            panic!("path must be absolute");
        }

        #[expect(clippy::expect_used)]
        let filename = path
            .file_name()
            .expect("path should not be empty")
            .to_string_lossy();
        let patch = format!(
            r#"*** Begin Patch
*** Update File: {filename}
@@
+ {content}
*** End Patch"#,
        );
        let changes = HashMap::from([(path.to_path_buf(), ApplyPatchFileChange::Add { content })]);
        #[expect(clippy::expect_used)]
        Self {
            changes,
            cwd: path
                .parent()
                .expect("path should have parent")
                .to_path_buf(),
            patch,
        }
    }
}

/// Applies the patch and prints the result to stdout/stderr.
pub fn apply_patch(
    patch: &str,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> Result<(), ApplyPatchError> {
    let cwd = std::env::current_dir().map_err(ApplyPatchError::from)?;
    let report = match apply_patch_in_dir(patch, &cwd) {
        Ok(report) => report,
        Err(e) => {
            match &e {
                ApplyPatchError::ParseError(InvalidPatchError(message)) => {
                    writeln!(stderr, "Invalid patch: {message}").map_err(ApplyPatchError::from)?;
                }
                ApplyPatchError::ParseError(InvalidHunkError {
                    message,
                    line_number,
                }) => {
                    writeln!(
                        stderr,
                        "Invalid patch hunk on line {line_number}: {message}"
                    )
                    .map_err(ApplyPatchError::from)?;
                }
                ApplyPatchError::ExecutionError(message) => {
                    writeln!(stderr, "{message}").map_err(ApplyPatchError::from)?;
                }
                ApplyPatchError::PartialApply(message) => {
                    writeln!(stderr, "{message}").map_err(ApplyPatchError::from)?;
                }
                ApplyPatchError::IoError(err) => {
                    writeln!(stderr, "{err}").map_err(ApplyPatchError::from)?;
                }
                ApplyPatchError::ComputeReplacements(message) => {
                    writeln!(stderr, "{message}").map_err(ApplyPatchError::from)?;
                }
                ApplyPatchError::ImplicitInvocation => {
                    writeln!(stderr, "{e}").map_err(ApplyPatchError::from)?;
                }
            }
            return Err(e);
        }
    };

    match report.status {
        ApplyPatchStatus::FullSuccess => {
            let display = relativize_affected_paths(&report.affected, &cwd);
            print_summary(&display, stdout).map_err(ApplyPatchError::from)?;
            print_warnings(&relativize_warnings(&report.warnings, &cwd), stdout)
                .map_err(ApplyPatchError::from)?;
            Ok(())
        }
        ApplyPatchStatus::PartialSuccess => {
            let message = report
                .failed_units
                .first()
                .map(|failure| failure.message.clone())
                .unwrap_or_else(|| "Patch partially applied.".to_string());
            Err(ApplyPatchError::PartialApply(message))
        }
        ApplyPatchStatus::Failure => {
            let message = report
                .failed_units
                .first()
                .map(|failure| failure.message.clone())
                .unwrap_or_else(|| "Patch failed.".to_string());
            Err(ApplyPatchError::ExecutionError(message))
        }
    }
}

pub fn apply_patch_in_dir(patch: &str, cwd: &Path) -> Result<ApplyPatchReport, ApplyPatchError> {
    let (parsed, source_map) = parse_patch_with_source_map(patch)?;
    apply_hunks_in_dir(&parsed.hunks, cwd, Some(&source_map))
}

/// Applies hunks and continues to update stdout/stderr
pub fn apply_hunks(
    hunks: &[Hunk],
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> Result<(), ApplyPatchError> {
    let cwd = std::env::current_dir().map_err(ApplyPatchError::from)?;
    match apply_hunks_in_dir(hunks, &cwd, None)? {
        ApplyPatchReport {
            status: ApplyPatchStatus::FullSuccess,
            affected,
            warnings,
            ..
        } => {
            let display = relativize_affected_paths(&affected, &cwd);
            print_summary(&display, stdout).map_err(ApplyPatchError::from)?;
            print_warnings(&relativize_warnings(&warnings, &cwd), stdout)
                .map_err(ApplyPatchError::from)?;
            Ok(())
        }
        ApplyPatchReport {
            status: ApplyPatchStatus::PartialSuccess,
            failed_units,
            ..
        } => {
            let message = failed_units
                .first()
                .map(|failure| failure.message.clone())
                .unwrap_or_else(|| "Patch partially applied.".to_string());
            writeln!(stderr, "{message}").map_err(ApplyPatchError::from)?;
            Err(ApplyPatchError::PartialApply(message))
        }
        ApplyPatchReport {
            status: ApplyPatchStatus::Failure,
            failed_units,
            ..
        } => {
            let message = failed_units
                .first()
                .map(|failure| failure.message.clone())
                .unwrap_or_else(|| "Patch failed.".to_string());
            writeln!(stderr, "{message}").map_err(ApplyPatchError::from)?;
            Err(ApplyPatchError::ExecutionError(message))
        }
    }
}

/// Applies each parsed patch hunk to the filesystem.
/// Returns an error if any of the changes could not be applied.
/// Apply the hunks to the filesystem, returning which files were added, modified, or deleted.
/// Returns an error if the patch could not be applied.
#[allow(dead_code)]
fn apply_hunks_to_files(hunks: &[Hunk]) -> anyhow::Result<AffectedPaths> {
    let cwd = std::env::current_dir()?;
    let report = apply_hunks_in_dir(hunks, &cwd, None).map_err(anyhow::Error::from)?;
    match report.status {
        ApplyPatchStatus::FullSuccess => Ok(report.affected),
        ApplyPatchStatus::PartialSuccess | ApplyPatchStatus::Failure => {
            let message = report
                .failed_units
                .first()
                .map(|failure| failure.message.clone())
                .unwrap_or_else(|| "Patch failed.".to_string());
            anyhow::bail!(message)
        }
    }
}

#[derive(Debug, Clone)]
enum ResolvedHunk {
    AddFile {
        action_index: usize,
        path: ResolvedPath,
        contents: String,
        action_source: Option<SourceLocation>,
    },
    DeleteFile {
        action_index: usize,
        path: ResolvedPath,
        action_source: Option<SourceLocation>,
    },
    UpdateFile {
        action_index: usize,
        path: ResolvedPath,
        move_path: Option<ResolvedPath>,
        chunks: Vec<UpdateFileChunk>,
        action_source: Option<SourceLocation>,
        move_source: Option<SourceLocation>,
        chunk_sources: Vec<PatchChunkSource>,
    },
}

impl ResolvedHunk {
    fn touched_paths(&self) -> Vec<ResolvedPath> {
        match self {
            ResolvedHunk::AddFile { path, .. } => vec![path.clone()],
            ResolvedHunk::DeleteFile { path, .. } => vec![path.clone()],
            ResolvedHunk::UpdateFile {
                path, move_path, ..
            } => {
                let mut paths = vec![path.clone()];
                if let Some(dest) = move_path
                    && dest.key != path.key
                {
                    paths.push(dest.clone());
                }
                paths
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CommitUnit {
    hunks: Vec<ResolvedHunk>,
    touched_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct PlannedUnit {
    after: StagedPathMap,
    commit_paths: Vec<PathBuf>,
    affected: AffectedPaths,
    warnings: Vec<ApplyPatchWarning>,
    path_diagnostics: HashMap<PathIdentityKey, PathDiagnosticRef>,
}

fn apply_hunks_in_dir(
    hunks: &[Hunk],
    cwd: &Path,
    source_map: Option<&PatchSourceMap>,
) -> Result<ApplyPatchReport, ApplyPatchError> {
    if hunks.is_empty() {
        return Ok(ApplyPatchReport {
            status: ApplyPatchStatus::FullSuccess,
            affected: AffectedPaths::default(),
            committed_units: 0,
            warnings: Vec::new(),
            failed_units: Vec::new(),
        });
    }

    let resolved = resolve_hunks(hunks, cwd, source_map);
    let units = build_commit_units(&resolved);
    let total_units = units.len();
    let mut affected = AffectedPaths::default();
    let mut warnings = Vec::new();
    let mut failed_units = Vec::new();

    for unit in units {
        let attempted = intended_affected_paths(&unit);
        match plan_commit_unit(&unit) {
            Ok(plan) => match commit_unit_atomically(&plan) {
                Ok(group_affected) => {
                    extend_affected_paths(&mut affected, group_affected);
                    warnings.extend(plan.warnings.clone());
                }
                Err(err) => failed_units.push(FailedUnit {
                    touched_paths: unit.touched_paths.clone(),
                    attempted: attempted.clone(),
                    code: err.code,
                    summary: err.summary,
                    target_path: err.target_path,
                    action_index: err.action_index,
                    hunk_index: err.hunk_index,
                    source_line: err.source_line,
                    source_column: err.source_column,
                    target_anchor: err.target_anchor,
                    help: err.help,
                    message: err.message,
                }),
            },
            Err(err) => failed_units.push(FailedUnit {
                touched_paths: unit.touched_paths.clone(),
                attempted,
                code: err.code,
                summary: err.summary,
                target_path: err.target_path,
                action_index: err.action_index,
                hunk_index: err.hunk_index,
                source_line: err.source_line,
                source_column: err.source_column,
                target_anchor: err.target_anchor,
                help: err.help,
                message: err.message,
            }),
        }
    }

    let status = if failed_units.is_empty() {
        ApplyPatchStatus::FullSuccess
    } else if affected.is_empty() {
        ApplyPatchStatus::Failure
    } else {
        ApplyPatchStatus::PartialSuccess
    };

    Ok(ApplyPatchReport {
        status,
        affected,
        committed_units: total_units.saturating_sub(failed_units.len()),
        warnings,
        failed_units,
    })
}

fn resolve_hunks(
    hunks: &[Hunk],
    cwd: &Path,
    source_map: Option<&PatchSourceMap>,
) -> Vec<ResolvedHunk> {
    hunks
        .iter()
        .enumerate()
        .map(|(index, hunk)| {
            let action_source = source_map
                .and_then(|map| map.actions.get(index))
                .map(source_location_from_action);
            match hunk {
                Hunk::AddFile { path, contents } => ResolvedHunk::AddFile {
                    action_index: index + 1,
                    path: resolve_patch_path(cwd, path),
                    contents: contents.clone(),
                    action_source,
                },
                Hunk::DeleteFile { path } => ResolvedHunk::DeleteFile {
                    action_index: index + 1,
                    path: resolve_patch_path(cwd, path),
                    action_source,
                },
                Hunk::UpdateFile {
                    path,
                    move_path,
                    chunks,
                } => ResolvedHunk::UpdateFile {
                    action_index: index + 1,
                    path: resolve_patch_path(cwd, path),
                    move_path: move_path.as_ref().map(|dest| resolve_patch_path(cwd, dest)),
                    chunks: chunks.clone(),
                    action_source,
                    move_source: source_map
                        .and_then(|map| map.actions.get(index))
                        .and_then(source_location_from_move),
                    chunk_sources: source_map
                        .and_then(|map| map.actions.get(index))
                        .map(|action| action.chunks.clone())
                        .unwrap_or_default(),
                },
            }
        })
        .collect()
}

fn source_location_from_action(action: &PatchActionSource) -> SourceLocation {
    SourceLocation {
        line_number: action.line_number,
        column_number: action.column_number,
    }
}

fn source_location_from_move(action: &PatchActionSource) -> Option<SourceLocation> {
    Some(SourceLocation {
        line_number: action.move_line_number?,
        column_number: action.move_column_number.unwrap_or(1),
    })
}

fn source_location_from_chunk(chunk: &PatchChunkSource) -> SourceLocation {
    SourceLocation {
        line_number: chunk.line_number,
        column_number: chunk.column_number,
    }
}

fn source_location_from_first_old_line(chunk: &PatchChunkSource) -> Option<SourceLocation> {
    Some(SourceLocation {
        line_number: chunk.first_old_line_number?,
        column_number: chunk.first_old_line_column.unwrap_or(1),
    })
}

fn source_location_from_first_removed_line(chunk: &PatchChunkSource) -> Option<SourceLocation> {
    Some(SourceLocation {
        line_number: chunk.first_removed_line_number?,
        column_number: chunk.first_removed_line_column.unwrap_or(1),
    })
}

fn source_location_parts(location: Option<SourceLocation>) -> (Option<usize>, Option<usize>) {
    (
        location.map(|loc| loc.line_number),
        location.map(|loc| loc.column_number),
    )
}

fn resolve_patch_path(cwd: &Path, path: &Path) -> ResolvedPath {
    let (actual_path, key) = resolve_runtime_path(cwd, path);
    ResolvedPath { actual_path, key }
}

fn build_commit_units(hunks: &[ResolvedHunk]) -> Vec<CommitUnit> {
    if hunks.is_empty() {
        return Vec::new();
    }

    build_connected_path_groups(
        &hunks
            .iter()
            .map(ResolvedHunk::touched_paths)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|group| CommitUnit {
        hunks: group
            .item_indices
            .iter()
            .map(|index| hunks[*index].clone())
            .collect(),
        touched_paths: group
            .touched_paths
            .into_iter()
            .map(|path| path.actual_path)
            .collect(),
    })
    .collect()
}

fn plan_commit_unit(unit: &CommitUnit) -> Result<PlannedUnit, UnitFailure> {
    let mut before: StagedPathMap = HashMap::new();
    let mut working: StagedPathMap = HashMap::new();
    let mut move_pairs: Vec<(PathIdentityKey, PathIdentityKey, PathBuf)> = Vec::new();
    let mut warnings = Vec::new();
    let mut path_diagnostics = HashMap::new();

    for hunk in &unit.hunks {
        match hunk {
            ResolvedHunk::AddFile {
                action_index,
                path,
                contents,
                action_source,
            } => {
                record_path_diagnostic(
                    &mut path_diagnostics,
                    path,
                    *action_index,
                    None,
                    *action_source,
                );
                if load_file_state(path, &mut before, &working)
                    .map_err(|err| {
                        unit_failure_for_read_error(path, *action_index, *action_source, err)
                    })?
                    .is_some()
                {
                    warnings.push(ApplyPatchWarning {
                        code: "ADD_REPLACED_EXISTING_FILE".to_string(),
                        summary: "Add File targeted an existing file and replaced its contents"
                            .to_string(),
                        target_path: path.actual_path.clone(),
                        help: Some("prefer Update File when editing an existing file".to_string()),
                    });
                }
                set_working_state(&mut working, path, Some(contents.clone()));
            }
            ResolvedHunk::DeleteFile {
                action_index,
                path,
                action_source,
            } => {
                record_path_diagnostic(
                    &mut path_diagnostics,
                    path,
                    *action_index,
                    None,
                    *action_source,
                );
                let current = load_file_state(path, &mut before, &working).map_err(|err| {
                    unit_failure_for_read_error(path, *action_index, *action_source, err)
                })?;
                if current.is_none() {
                    let (source_line, source_column) = source_location_parts(*action_source);
                    return Err(UnitFailure {
                        code: "DELETE_TARGET_MISSING".to_string(),
                        summary: "Delete File targeted a path that does not exist".to_string(),
                        target_path: Some(path.actual_path.clone()),
                        action_index: Some(*action_index),
                        hunk_index: None,
                        source_line,
                        source_column,
                        target_anchor: None,
                        help: Some("re-read the workspace and remove the delete if the file is already gone".to_string()),
                        message: format!(
                            "Failed to delete file {}: No such file or directory (os error 2)",
                            path.actual_path.display()
                        ),
                    });
                }
                set_working_state(&mut working, path, None);
            }
            ResolvedHunk::UpdateFile {
                action_index,
                path,
                move_path,
                chunks,
                action_source,
                move_source,
                chunk_sources,
            } => {
                record_path_diagnostic(
                    &mut path_diagnostics,
                    path,
                    *action_index,
                    None,
                    *action_source,
                );
                let current = load_file_state(path, &mut before, &working).map_err(|err| {
                    unit_failure_for_read_error(path, *action_index, *action_source, err)
                })?;
                let Some(current) = current else {
                    let (source_line, source_column) = source_location_parts(*action_source);
                    return Err(UnitFailure {
                        code: "UPDATE_TARGET_MISSING".to_string(),
                        summary: "Update File targeted a path that does not exist".to_string(),
                        target_path: Some(path.actual_path.clone()),
                        action_index: Some(*action_index),
                        hunk_index: None,
                        source_line,
                        source_column,
                        target_anchor: None,
                        help: Some(
                            "create the file first or use Add File if you intend to create it"
                                .to_string(),
                        ),
                        message: format!(
                            "Failed to read file to update {}: No such file or directory (os error 2)",
                            path.actual_path.display()
                        ),
                    });
                };
                let new_contents = apply_update_file_to_content_with_diagnostics(
                    &current,
                    &path.actual_path,
                    chunks,
                )
                .map_err(|err| {
                    unit_failure_for_match_error(
                        path,
                        *action_index,
                        *action_source,
                        chunk_sources.get(err.chunk_index).copied(),
                        err,
                    )
                })?;
                if let Some(dest) = move_path {
                    let dest_current =
                        load_file_state(dest, &mut before, &working).map_err(|err| {
                            unit_failure_for_read_error(
                                dest,
                                *action_index,
                                move_source.or(*action_source),
                                err,
                            )
                        })?;
                    if dest.key == path.key {
                        set_working_state(&mut working, path, Some(new_contents));
                    } else {
                        record_path_diagnostic(
                            &mut path_diagnostics,
                            dest,
                            *action_index,
                            None,
                            move_source.or(*action_source),
                        );
                        if dest_current.is_some() {
                            warnings.push(ApplyPatchWarning {
                                code: "MOVE_REPLACED_EXISTING_DESTINATION".to_string(),
                                summary: "Move to targeted an existing file path and replaced the destination contents"
                                    .to_string(),
                                target_path: dest.actual_path.clone(),
                                help: Some(
                                    "prefer a fresh destination path when renaming".to_string(),
                                ),
                            });
                        }
                        move_pairs.push((
                            path.key.clone(),
                            dest.key.clone(),
                            dest.actual_path.clone(),
                        ));
                        set_working_state(&mut working, dest, Some(new_contents));
                        set_working_state(&mut working, path, None);
                    }
                } else {
                    set_working_state(&mut working, path, Some(new_contents));
                }
            }
        }
    }

    let mut commit_paths = before
        .keys()
        .chain(working.keys())
        .filter_map(|key| commit_path_for_key(key, &before, &working))
        .collect::<Vec<_>>();
    commit_paths.sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    commit_paths.dedup();
    commit_paths.retain(|path| {
        let key = path_identity_key(path);
        let before_value = before
            .get(&key)
            .map(|state| state.contents.clone())
            .unwrap_or(None);
        let after_value = working
            .get(&key)
            .map(|state| state.contents.clone())
            .unwrap_or_else(|| before_value.clone());
        before_value != after_value
    });

    let mut affected =
        diff_affected_paths(&before, &working, MissingAfterBehavior::TreatAsUnchanged);
    for (source_key, dest_key, dest_path) in move_pairs {
        affected
            .deleted
            .retain(|path| path_identity_key(path) != source_key);
        affected
            .added
            .retain(|path| path_identity_key(path) != dest_key);
        if !affected
            .modified
            .iter()
            .any(|path| path_identity_key(path) == dest_key)
        {
            affected.modified.push(dest_path);
        }
    }
    affected
        .added
        .sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    affected
        .modified
        .sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    affected
        .deleted
        .sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    Ok(PlannedUnit {
        after: working,
        commit_paths,
        affected,
        warnings,
        path_diagnostics,
    })
}

fn intended_affected_paths(unit: &CommitUnit) -> AttemptedPaths {
    let mut attempted = AttemptedPaths::default();
    for hunk in &unit.hunks {
        match hunk {
            ResolvedHunk::AddFile { path, .. } => push_unique_path(&mut attempted.added, path),
            ResolvedHunk::DeleteFile { path, .. } => push_unique_path(&mut attempted.deleted, path),
            ResolvedHunk::UpdateFile {
                path, move_path, ..
            } => {
                if let Some(dest) = move_path
                    && dest.key != path.key
                {
                    push_unique_path(&mut attempted.modified, dest);
                } else {
                    push_unique_path(&mut attempted.modified, path);
                }
            }
        }
    }
    attempted
        .added
        .sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    attempted
        .modified
        .sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    attempted
        .deleted
        .sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    attempted
}

fn push_unique_path(target: &mut Vec<PathBuf>, path: &ResolvedPath) {
    if !target
        .iter()
        .any(|existing| path_identity_key(existing) == path.key)
    {
        target.push(path.actual_path.clone());
    }
}

fn record_path_diagnostic(
    diagnostics: &mut HashMap<PathIdentityKey, PathDiagnosticRef>,
    path: &ResolvedPath,
    action_index: usize,
    hunk_index: Option<usize>,
    source_location: Option<SourceLocation>,
) {
    diagnostics.insert(
        path.key.clone(),
        PathDiagnosticRef {
            action_index,
            hunk_index,
            source_location,
        },
    );
}

fn unit_failure_for_read_error(
    path: &ResolvedPath,
    action_index: usize,
    action_source: Option<SourceLocation>,
    err: ApplyPatchError,
) -> UnitFailure {
    let (source_line, source_column) = source_location_parts(action_source);
    UnitFailure {
        code: "TARGET_READ_ERROR".to_string(),
        summary: "failed to read the target path while planning the patch".to_string(),
        target_path: Some(path.actual_path.clone()),
        action_index: Some(action_index),
        hunk_index: None,
        source_line,
        source_column,
        target_anchor: None,
        help: Some("repair the target path permissions or filesystem state and retry".to_string()),
        message: err.to_string(),
    }
}

fn unit_failure_for_match_error(
    path: &ResolvedPath,
    action_index: usize,
    action_source: Option<SourceLocation>,
    chunk_source: Option<PatchChunkSource>,
    err: ReplaceMatchError,
) -> UnitFailure {
    let code = if err.is_end_of_file {
        "MATCH_INVALID_EOF_CONTEXT"
    } else {
        "MATCH_INVALID_CONTEXT"
    };
    let summary = if err.is_end_of_file {
        "patch EOF context did not match target file"
    } else {
        "patch context did not match target file"
    };
    let chunk_location = chunk_source.and_then(|chunk| {
        if err.blame_first_old_line {
            source_location_from_first_removed_line(&chunk)
                .or_else(|| source_location_from_first_old_line(&chunk))
                .or_else(|| Some(source_location_from_chunk(&chunk)))
        } else {
            Some(source_location_from_chunk(&chunk))
        }
    });
    let (source_line, source_column) = source_location_parts(chunk_location.or(action_source));
    UnitFailure {
        code: code.to_string(),
        summary: summary.to_string(),
        target_path: Some(path.actual_path.clone()),
        action_index: Some(action_index),
        hunk_index: Some(err.chunk_index + 1),
        source_line,
        source_column,
        target_anchor: err.target_anchor,
        help: Some(
            "re-read the target file and regenerate the patch with fresh context".to_string(),
        ),
        message: err.message,
    }
}

fn unit_failure_for_write_error(
    path: &Path,
    diagnostic_ref: Option<PathDiagnosticRef>,
    message: String,
) -> UnitFailure {
    let (source_line, source_column) =
        source_location_parts(diagnostic_ref.and_then(|value| value.source_location));
    UnitFailure {
        code: "TARGET_WRITE_ERROR".to_string(),
        summary: "patch could not be written to the target path".to_string(),
        target_path: Some(path.to_path_buf()),
        action_index: diagnostic_ref.map(|value| value.action_index),
        hunk_index: diagnostic_ref.and_then(|value| value.hunk_index),
        source_line,
        source_column,
        target_anchor: None,
        help: Some("repair the target path or parent directories and retry".to_string()),
        message,
    }
}

fn load_file_state(
    path: &ResolvedPath,
    before: &mut StagedPathMap,
    working: &StagedPathMap,
) -> Result<Option<String>, ApplyPatchError> {
    if let Some(state) = working.get(&path.key) {
        return Ok(state.contents.clone());
    }
    if let Some(state) = before.get(&path.key) {
        return Ok(state.contents.clone());
    }

    let state = match std::fs::read_to_string(&path.actual_path) {
        Ok(contents) => Some(contents),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(ApplyPatchError::IoError(IoError {
                context: format!("Failed to read {}", path.actual_path.display()),
                source: err,
            }));
        }
    };
    before.insert(
        path.key.clone(),
        StagedPathState {
            actual_path: path.actual_path.clone(),
            contents: state.clone(),
        },
    );
    Ok(state)
}

fn set_working_state(working: &mut StagedPathMap, path: &ResolvedPath, contents: Option<String>) {
    working.insert(
        path.key.clone(),
        StagedPathState {
            actual_path: path.actual_path.clone(),
            contents,
        },
    );
}

fn relativize_affected_paths(affected: &AffectedPaths, cwd: &Path) -> AffectedPaths {
    fn relativize(path: &Path, cwd: &Path) -> PathBuf {
        path.strip_prefix(cwd)
            .map_or_else(|_| path.to_path_buf(), PathBuf::from)
    }

    AffectedPaths {
        added: affected
            .added
            .iter()
            .map(|path| relativize(path, cwd))
            .collect(),
        modified: affected
            .modified
            .iter()
            .map(|path| relativize(path, cwd))
            .collect(),
        deleted: affected
            .deleted
            .iter()
            .map(|path| relativize(path, cwd))
            .collect(),
    }
}

fn relativize_warnings(warnings: &[ApplyPatchWarning], cwd: &Path) -> Vec<ApplyPatchWarning> {
    warnings
        .iter()
        .map(|warning| ApplyPatchWarning {
            code: warning.code.clone(),
            summary: warning.summary.clone(),
            target_path: warning
                .target_path
                .strip_prefix(cwd)
                .map_or_else(|_| warning.target_path.clone(), PathBuf::from),
            help: warning.help.clone(),
        })
        .collect()
}

fn commit_unit_atomically(plan: &PlannedUnit) -> Result<AffectedPaths, UnitFailure> {
    let mut changed_paths = plan.commit_paths.clone();
    changed_paths.sort();
    changed_paths.dedup();

    if changed_paths.is_empty() {
        return Ok(AffectedPaths::default());
    }

    let changes = changed_paths
        .iter()
        .map(|path| {
            let key = path_identity_key(path);
            ByteFileChange {
                key: key.clone(),
                actual_path: path.clone(),
                after: plan.after.get(&key).and_then(|state| {
                    state
                        .contents
                        .as_ref()
                        .map(|value| value.as_bytes().to_vec())
                }),
            }
        })
        .collect::<Vec<_>>();

    commit_byte_changes_atomically(&changes).map_err(|err| {
        let diagnostic_ref = plan.path_diagnostics.get(&err.key).copied();
        unit_failure_for_write_error(
            &err.actual_path,
            diagnostic_ref,
            format_patch_write_error_message(&err),
        )
    })?;

    Ok(plan.affected.clone())
}

fn format_patch_write_error_message(error: &ByteFileCommitError) -> String {
    match error.operation {
        ByteFileCommitOperation::ReadMetadata => {
            format!(
                "Failed to read metadata for {}: {}",
                error.actual_path.display(),
                error.error
            )
        }
        ByteFileCommitOperation::CreateParent => {
            format!(
                "Failed to create parent directories for {}: {}",
                error.actual_path.display(),
                error.error
            )
        }
        ByteFileCommitOperation::BackupExisting
        | ByteFileCommitOperation::WriteTemp
        | ByteFileCommitOperation::InstallTarget => {
            format!(
                "Failed to write file {}: {}",
                error.actual_path.display(),
                error.error
            )
        }
    }
}

struct AppliedPatch {
    original_contents: String,
    new_contents: String,
}

#[derive(Debug, Clone)]
struct ReplaceMatchError {
    message: String,
    chunk_index: usize,
    is_end_of_file: bool,
    target_anchor: Option<ApplyPatchTargetAnchor>,
    blame_first_old_line: bool,
}

pub fn apply_update_file_to_content(
    original_contents: &str,
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> std::result::Result<String, ApplyPatchError> {
    apply_update_file_to_content_with_diagnostics(original_contents, path, chunks)
        .map_err(|err| ApplyPatchError::ComputeReplacements(err.message))
}

fn apply_update_file_to_content_with_diagnostics(
    original_contents: &str,
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<String, ReplaceMatchError> {
    let original_file_lines = parse_file_lines(original_contents);
    let original_lines = original_file_lines
        .iter()
        .map(|line| line.text.clone())
        .collect::<Vec<_>>();
    let replacements = compute_replacements(&original_lines, path, chunks)?;
    let new_lines = apply_replacements_preserving_file_format(&original_file_lines, &replacements);
    Ok(render_file_lines(&new_lines))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileLine {
    text: String,
    ending: Option<LineEnding>,
}

fn parse_file_lines(contents: &str) -> Vec<FileLine> {
    contents
        .split_inclusive('\n')
        .map(|segment| {
            if let Some(text) = segment.strip_suffix("\r\n") {
                FileLine {
                    text: text.to_string(),
                    ending: Some(LineEnding::Crlf),
                }
            } else if let Some(text) = segment.strip_suffix('\n') {
                FileLine {
                    text: text.to_string(),
                    ending: Some(LineEnding::Lf),
                }
            } else {
                FileLine {
                    text: segment.to_string(),
                    ending: None,
                }
            }
        })
        .collect()
}

fn render_file_lines(lines: &[FileLine]) -> String {
    let mut rendered = String::new();
    for line in lines {
        rendered.push_str(&line.text);
        if let Some(ending) = line.ending {
            rendered.push_str(ending.as_str());
        }
    }
    rendered
}

fn apply_replacements_preserving_file_format(
    original_lines: &[FileLine],
    replacements: &[(usize, usize, Vec<String>)],
) -> Vec<FileLine> {
    let dominant_ending = dominant_line_ending(original_lines);
    let original_had_trailing_newline = original_lines
        .last()
        .is_some_and(|line| line.ending.is_some());
    let mut lines = original_lines.to_vec();

    for (start_idx, old_len, new_segment) in replacements.iter().rev() {
        let start_idx = *start_idx;
        let old_len = *old_len;
        let replacement_ending =
            replacement_line_ending(original_lines, start_idx, old_len, dominant_ending);
        let replacement_reaches_eof = start_idx + old_len >= original_lines.len();

        if old_len == 0
            && start_idx == original_lines.len()
            && !new_segment.is_empty()
            && start_idx > 0
            && lines
                .get(start_idx - 1)
                .is_some_and(|line| line.ending.is_none())
        {
            if let Some(previous_line) = lines.get_mut(start_idx - 1) {
                previous_line.ending = Some(replacement_ending);
            }
        }

        for _ in 0..old_len {
            if start_idx < lines.len() {
                lines.remove(start_idx);
            }
        }

        let inserted_lines = new_segment
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let ending = if index + 1 < new_segment.len() {
                    Some(replacement_ending)
                } else if replacement_reaches_eof && !original_had_trailing_newline {
                    None
                } else {
                    Some(replacement_ending)
                };
                FileLine {
                    text: line.clone(),
                    ending,
                }
            })
            .collect::<Vec<_>>();

        for (offset, line) in inserted_lines.into_iter().enumerate() {
            lines.insert(start_idx + offset, line);
        }
    }

    lines
}

fn replacement_line_ending(
    original_lines: &[FileLine],
    start_idx: usize,
    old_len: usize,
    dominant_ending: LineEnding,
) -> LineEnding {
    original_lines[start_idx..start_idx.saturating_add(old_len).min(original_lines.len())]
        .iter()
        .find_map(|line| line.ending)
        .or_else(|| {
            start_idx
                .checked_sub(1)
                .and_then(|index| original_lines.get(index))
                .and_then(|line| line.ending)
        })
        .or_else(|| {
            original_lines
                .get(start_idx.saturating_add(old_len))
                .and_then(|line| line.ending)
        })
        .unwrap_or(dominant_ending)
}

fn dominant_line_ending(lines: &[FileLine]) -> LineEnding {
    let mut first_seen = None;
    let mut lf_count = 0usize;
    let mut crlf_count = 0usize;

    for line in lines {
        match line.ending {
            Some(LineEnding::Lf) => {
                first_seen.get_or_insert(LineEnding::Lf);
                lf_count += 1;
            }
            Some(LineEnding::Crlf) => {
                first_seen.get_or_insert(LineEnding::Crlf);
                crlf_count += 1;
            }
            None => {}
        }
    }

    if crlf_count > lf_count {
        LineEnding::Crlf
    } else if lf_count > crlf_count {
        LineEnding::Lf
    } else {
        first_seen.unwrap_or(LineEnding::Lf)
    }
}

/// Return *only* the new file contents (joined into a single `String`) after
/// applying the chunks to the file at `path`.
fn derive_new_contents_from_chunks(
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> std::result::Result<AppliedPatch, ApplyPatchError> {
    let original_contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            return Err(ApplyPatchError::IoError(IoError {
                context: format!("Failed to read file to update {}", path.display()),
                source: err,
            }));
        }
    };

    let new_contents = apply_update_file_to_content(&original_contents, path, chunks)?;
    Ok(AppliedPatch {
        original_contents,
        new_contents,
    })
}

/// Compute a list of replacements needed to transform `original_lines` into the
/// new lines, given the patch `chunks`. Each replacement is returned as
/// `(start_index, old_len, new_lines)`.
fn compute_replacements(
    original_lines: &[String],
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<Vec<(usize, usize, Vec<String>)>, ReplaceMatchError> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for (chunk_index, chunk) in chunks.iter().enumerate() {
        let mut matched_context_anchor = None;
        // If a chunk has a `change_context`, we use seek_sequence to find it, then
        // adjust our `line_index` to continue from there.
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(idx) = seek_sequence::seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                false,
            ) {
                line_index = idx + 1;
                matched_context_anchor = Some(ApplyPatchTargetAnchor {
                    path: path.to_path_buf(),
                    line_number: idx + 1,
                    column_number: 1,
                    label: Some("matched context".to_string()),
                    excerpt: original_lines.get(idx).cloned(),
                });
            } else {
                return Err(ReplaceMatchError {
                    message: format!(
                        "Failed to find context '{}' in {}",
                        ctx_line,
                        path.display()
                    ),
                    chunk_index,
                    is_end_of_file: false,
                    target_anchor: None,
                    blame_first_old_line: false,
                });
            }
        }

        if chunk.old_lines.is_empty() {
            // Pure addition (no old lines). We'll add them at the end or just
            // before the final empty line if one exists.
            let insertion_idx = if original_lines.last().is_some_and(String::is_empty) {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push((insertion_idx, 0, chunk.new_lines.clone()));
            continue;
        }

        // Otherwise, try to match the existing lines in the file with the old lines
        // from the chunk. If found, schedule that region for replacement.
        // Attempt to locate the `old_lines` verbatim within the file.  In many
        // real‑world diffs the last element of `old_lines` is an *empty* string
        // representing the terminating newline of the region being replaced.
        // This sentinel is not present in `original_lines` because we strip the
        // trailing empty slice emitted by `split('\n')`.  If a direct search
        // fails and the pattern ends with an empty string, retry without that
        // final element so that modifications touching the end‑of‑file can be
        // located reliably.

        let mut pattern: &[String] = &chunk.old_lines;
        let mut found =
            seek_sequence::seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);

        let mut new_slice: &[String] = &chunk.new_lines;

        if found.is_none() && pattern.last().is_some_and(String::is_empty) {
            // Retry without the trailing empty line which represents the final
            // newline in the file.
            pattern = &pattern[..pattern.len() - 1];
            if new_slice.last().is_some_and(String::is_empty) {
                new_slice = &new_slice[..new_slice.len() - 1];
            }

            found = seek_sequence::seek_sequence(
                original_lines,
                pattern,
                line_index,
                chunk.is_end_of_file,
            );
        }

        if let Some(start_idx) = found {
            replacements.push((start_idx, pattern.len(), new_slice.to_vec()));
            line_index = start_idx + pattern.len();
        } else {
            return Err(ReplaceMatchError {
                message: format!(
                    "Failed to find expected lines in {}:\n{}",
                    path.display(),
                    chunk.old_lines.join("\n"),
                ),
                chunk_index,
                is_end_of_file: chunk.is_end_of_file,
                target_anchor: matched_context_anchor,
                blame_first_old_line: true,
            });
        }
    }

    replacements.sort_by(|(lhs_idx, _, _), (rhs_idx, _, _)| lhs_idx.cmp(rhs_idx));

    Ok(replacements)
}

/// Intended result of a file update for apply_patch.
#[derive(Debug, Eq, PartialEq)]
pub struct ApplyPatchFileUpdate {
    unified_diff: String,
    content: String,
}

pub fn unified_diff_from_chunks(
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> std::result::Result<ApplyPatchFileUpdate, ApplyPatchError> {
    unified_diff_from_chunks_with_context(path, chunks, 1)
}

pub fn unified_diff_from_chunks_with_context(
    path: &Path,
    chunks: &[UpdateFileChunk],
    context: usize,
) -> std::result::Result<ApplyPatchFileUpdate, ApplyPatchError> {
    let AppliedPatch {
        original_contents,
        new_contents,
    } = derive_new_contents_from_chunks(path, chunks)?;
    let text_diff = TextDiff::from_lines(&original_contents, &new_contents);
    let unified_diff = text_diff.unified_diff().context_radius(context).to_string();
    Ok(ApplyPatchFileUpdate {
        unified_diff,
        content: new_contents,
    })
}

/// Print the summary of changes in git-style format.
/// Write a summary of changes to the given writer.
pub fn print_summary(
    affected: &AffectedPaths,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    writeln!(out, "Success. Updated the following files:")?;
    for path in &affected.added {
        writeln!(out, "A {}", path.display())?;
    }
    for path in &affected.modified {
        writeln!(out, "M {}", path.display())?;
    }
    for path in &affected.deleted {
        writeln!(out, "D {}", path.display())?;
    }
    if affected.is_empty() {
        writeln!(out, "(no file changes)")?;
    }
    Ok(())
}

fn print_warnings(
    warnings: &[ApplyPatchWarning],
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    if warnings.is_empty() {
        return Ok(());
    }

    writeln!(out)?;
    for (index, warning) in warnings.iter().enumerate() {
        if index > 0 {
            writeln!(out)?;
        }
        writeln!(out, "warning[{}]: {}", warning.code, warning.summary)?;
        writeln!(out, "  --> {}", warning.target_path.display())?;
        if let Some(help) = &warning.help {
            writeln!(out, "  = help: {help}")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::string::ToString;
    use tempfile::tempdir;

    /// Helper to construct a patch with the given body.
    fn wrap_patch(body: &str) -> String {
        format!("*** Begin Patch\n{body}\n*** End Patch")
    }

    #[test]
    fn test_add_file_hunk_creates_file_with_contents() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("add.txt");
        let patch = wrap_patch(&format!(
            r#"*** Add File: {}
+ab
+cd"#,
            path.display()
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        // Verify expected stdout and stderr outputs.
        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        let expected_out = format!(
            "Success. Updated the following files:\nA {}\n",
            path.display()
        );
        assert_eq!(stdout_str, expected_out);
        assert_eq!(stderr_str, "");
        let contents = fs::read_to_string(path).unwrap();
        assert_eq!(contents, "ab\ncd\n");
    }

    #[test]
    fn test_delete_file_hunk_removes_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("del.txt");
        fs::write(&path, "x").unwrap();
        let patch = wrap_patch(&format!("*** Delete File: {}", path.display()));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        let expected_out = format!(
            "Success. Updated the following files:\nD {}\n",
            path.display()
        );
        assert_eq!(stdout_str, expected_out);
        assert_eq!(stderr_str, "");
        assert!(!path.exists());
    }

    #[test]
    fn test_update_file_hunk_modifies_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("update.txt");
        fs::write(&path, "foo\nbar\n").unwrap();
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
 foo
-bar
+baz"#,
            path.display()
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        // Validate modified file contents and expected stdout/stderr.
        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        let expected_out = format!(
            "Success. Updated the following files:\nM {}\n",
            path.display()
        );
        assert_eq!(stdout_str, expected_out);
        assert_eq!(stderr_str, "");
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "foo\nbaz\n");
    }

    #[test]
    fn test_update_file_hunk_can_move_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.txt");
        let dest = dir.path().join("dst.txt");
        fs::write(&src, "line\n").unwrap();
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
*** Move to: {}
@@
-line
+line2"#,
            src.display(),
            dest.display()
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        // Validate move semantics and expected stdout/stderr.
        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        let expected_out = format!(
            "Success. Updated the following files:\nM {}\n",
            dest.display()
        );
        assert_eq!(stdout_str, expected_out);
        assert_eq!(stderr_str, "");
        assert!(!src.exists());
        let contents = fs::read_to_string(&dest).unwrap();
        assert_eq!(contents, "line2\n");
    }

    /// Verify that a single `Update File` hunk with multiple change chunks can update different
    /// parts of a file and that the file is listed only once in the summary.
    #[test]
    fn test_multiple_update_chunks_apply_to_single_file() {
        // Start with a file containing four lines.
        let dir = tempdir().unwrap();
        let path = dir.path().join("multi.txt");
        fs::write(&path, "foo\nbar\nbaz\nqux\n").unwrap();
        // Construct an update patch with two separate change chunks.
        // The first chunk uses the line `foo` as context and transforms `bar` into `BAR`.
        // The second chunk uses `baz` as context and transforms `qux` into `QUX`.
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
 foo
-bar
+BAR
@@
 baz
-qux
+QUX"#,
            path.display()
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        let expected_out = format!(
            "Success. Updated the following files:\nM {}\n",
            path.display()
        );
        assert_eq!(stdout_str, expected_out);
        assert_eq!(stderr_str, "");
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "foo\nBAR\nbaz\nQUX\n");
    }

    /// A more involved `Update File` hunk that exercises additions, deletions and
    /// replacements in separate chunks that appear in non‑adjacent parts of the
    /// file.  Verifies that all edits are applied and that the summary lists the
    /// file only once.
    #[test]
    fn test_update_file_hunk_interleaved_changes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("interleaved.txt");

        // Original file: six numbered lines.
        fs::write(&path, "a\nb\nc\nd\ne\nf\n").unwrap();

        // Patch performs:
        //  • Replace `b` → `B`
        //  • Replace `e` → `E` (using surrounding context)
        //  • Append new line `g` at the end‑of‑file
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
 a
-b
+B
@@
 c
 d
-e
+E
@@
 f
+g
*** End of File"#,
            path.display()
        ));

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();

        let stdout_str = String::from_utf8(stdout).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();

        let expected_out = format!(
            "Success. Updated the following files:\nM {}\n",
            path.display()
        );
        assert_eq!(stdout_str, expected_out);
        assert_eq!(stderr_str, "");

        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "a\nB\nc\nd\nE\nf\ng\n");
    }

    #[test]
    fn test_pure_addition_chunk_followed_by_removal() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("panic.txt");
        fs::write(&path, "line1\nline2\nline3\n").unwrap();
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
+after-context
+second-line
@@
 line1
-line2
-line3
+line2-replacement"#,
            path.display()
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        let contents = fs::read_to_string(path).unwrap();
        assert_eq!(
            contents,
            "line1\nline2-replacement\nafter-context\nsecond-line\n"
        );
    }

    /// Ensure that patches authored with ASCII characters can update lines that
    /// contain typographic Unicode punctuation (e.g. EN DASH, NON-BREAKING
    /// HYPHEN). Historically `git apply` succeeds in such scenarios but our
    /// internal matcher failed requiring an exact byte-for-byte match.  The
    /// fuzzy-matching pass that normalises common punctuation should now bridge
    /// the gap.
    #[test]
    fn test_update_line_with_unicode_dash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("unicode.py");

        // Original line contains EN DASH (\u{2013}) and NON-BREAKING HYPHEN (\u{2011}).
        let original = "import asyncio  # local import \u{2013} avoids top\u{2011}level dep\n";
        std::fs::write(&path, original).unwrap();

        // Patch uses plain ASCII dash / hyphen.
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
-import asyncio  # local import - avoids top-level dep
+import asyncio  # HELLO"#,
            path.display()
        ));

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();

        // File should now contain the replaced comment.
        let expected = "import asyncio  # HELLO\n";
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, expected);

        // Ensure success summary lists the file as modified.
        let stdout_str = String::from_utf8(stdout).unwrap();
        let expected_out = format!(
            "Success. Updated the following files:\nM {}\n",
            path.display()
        );
        assert_eq!(stdout_str, expected_out);

        // No stderr expected.
        assert_eq!(String::from_utf8(stderr).unwrap(), "");
    }

    #[test]
    fn test_unified_diff() {
        // Start with a file containing four lines.
        let dir = tempdir().unwrap();
        let path = dir.path().join("multi.txt");
        fs::write(&path, "foo\nbar\nbaz\nqux\n").unwrap();
        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
 foo
-bar
+BAR
@@
 baz
-qux
+QUX"#,
            path.display()
        ));
        let patch = parse_patch(&patch).unwrap();

        let update_file_chunks = match patch.hunks.as_slice() {
            [Hunk::UpdateFile { chunks, .. }] => chunks,
            _ => panic!("Expected a single UpdateFile hunk"),
        };
        let diff = unified_diff_from_chunks(&path, update_file_chunks).unwrap();
        let expected_diff = r#"@@ -1,4 +1,4 @@
 foo
-bar
+BAR
 baz
-qux
+QUX
"#;
        let expected = ApplyPatchFileUpdate {
            unified_diff: expected_diff.to_string(),
            content: "foo\nBAR\nbaz\nQUX\n".to_string(),
        };
        assert_eq!(expected, diff);
    }

    #[test]
    fn test_unified_diff_first_line_replacement() {
        // Replace the very first line of the file.
        let dir = tempdir().unwrap();
        let path = dir.path().join("first.txt");
        fs::write(&path, "foo\nbar\nbaz\n").unwrap();

        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
-foo
+FOO
 bar
"#,
            path.display()
        ));

        let patch = parse_patch(&patch).unwrap();
        let chunks = match patch.hunks.as_slice() {
            [Hunk::UpdateFile { chunks, .. }] => chunks,
            _ => panic!("Expected a single UpdateFile hunk"),
        };

        let diff = unified_diff_from_chunks(&path, chunks).unwrap();
        let expected_diff = r#"@@ -1,2 +1,2 @@
-foo
+FOO
 bar
"#;
        let expected = ApplyPatchFileUpdate {
            unified_diff: expected_diff.to_string(),
            content: "FOO\nbar\nbaz\n".to_string(),
        };
        assert_eq!(expected, diff);
    }

    #[test]
    fn test_unified_diff_last_line_replacement() {
        // Replace the very last line of the file.
        let dir = tempdir().unwrap();
        let path = dir.path().join("last.txt");
        fs::write(&path, "foo\nbar\nbaz\n").unwrap();

        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
 foo
 bar
-baz
+BAZ
"#,
            path.display()
        ));

        let patch = parse_patch(&patch).unwrap();
        let chunks = match patch.hunks.as_slice() {
            [Hunk::UpdateFile { chunks, .. }] => chunks,
            _ => panic!("Expected a single UpdateFile hunk"),
        };

        let diff = unified_diff_from_chunks(&path, chunks).unwrap();
        let expected_diff = r#"@@ -2,2 +2,2 @@
 bar
-baz
+BAZ
"#;
        let expected = ApplyPatchFileUpdate {
            unified_diff: expected_diff.to_string(),
            content: "foo\nbar\nBAZ\n".to_string(),
        };
        assert_eq!(expected, diff);
    }

    #[test]
    fn test_unified_diff_insert_at_eof() {
        // Insert a new line at end‑of‑file.
        let dir = tempdir().unwrap();
        let path = dir.path().join("insert.txt");
        fs::write(&path, "foo\nbar\nbaz\n").unwrap();

        let patch = wrap_patch(&format!(
            r#"*** Update File: {}
@@
+quux
*** End of File
"#,
            path.display()
        ));

        let patch = parse_patch(&patch).unwrap();
        let chunks = match patch.hunks.as_slice() {
            [Hunk::UpdateFile { chunks, .. }] => chunks,
            _ => panic!("Expected a single UpdateFile hunk"),
        };

        let diff = unified_diff_from_chunks(&path, chunks).unwrap();
        let expected_diff = r#"@@ -3 +3,2 @@
 baz
+quux
"#;
        let expected = ApplyPatchFileUpdate {
            unified_diff: expected_diff.to_string(),
            content: "foo\nbar\nbaz\nquux\n".to_string(),
        };
        assert_eq!(expected, diff);
    }

    #[test]
    fn test_unified_diff_interleaved_changes() {
        // Original file with six lines.
        let dir = tempdir().unwrap();
        let path = dir.path().join("interleaved.txt");
        fs::write(&path, "a\nb\nc\nd\ne\nf\n").unwrap();

        // Patch replaces two separate lines and appends a new one at EOF using
        // three distinct chunks.
        let patch_body = format!(
            r#"*** Update File: {}
@@
 a
-b
+B
@@
 d
-e
+E
@@
 f
+g
*** End of File"#,
            path.display()
        );
        let patch = wrap_patch(&patch_body);

        // Extract chunks then build the unified diff.
        let parsed = parse_patch(&patch).unwrap();
        let chunks = match parsed.hunks.as_slice() {
            [Hunk::UpdateFile { chunks, .. }] => chunks,
            _ => panic!("Expected a single UpdateFile hunk"),
        };

        let diff = unified_diff_from_chunks(&path, chunks).unwrap();

        let expected_diff = r#"@@ -1,6 +1,7 @@
 a
-b
+B
 c
 d
-e
+E
 f
+g
"#;

        let expected = ApplyPatchFileUpdate {
            unified_diff: expected_diff.to_string(),
            content: "a\nB\nc\nd\nE\nf\ng\n".to_string(),
        };

        assert_eq!(expected, diff);

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        apply_patch(&patch, &mut stdout, &mut stderr).unwrap();
        let contents = fs::read_to_string(path).unwrap();
        assert_eq!(
            contents,
            r#"a
B
c
d
E
f
g
"#
        );
    }

    #[test]
    fn test_apply_patch_fails_on_write_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("readonly.txt");
        fs::write(&path, "before\n").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&path, perms).unwrap();

        let patch = wrap_patch(&format!(
            "*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
            path.display()
        ));

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = apply_patch(&patch, &mut stdout, &mut stderr);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_patch_in_dir_reports_partial_success_for_independent_units() {
        let dir = tempdir().unwrap();
        let created = dir.path().join("created.txt");
        let patch = wrap_patch(
            "*** Add File: created.txt\n+hello\n*** Update File: missing.txt\n@@\n-old\n+new",
        );

        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();
        assert_eq!(report.status, ApplyPatchStatus::PartialSuccess);
        assert_eq!(report.committed_units, 1);
        assert_eq!(fs::read_to_string(&created).unwrap(), "hello\n");
        assert_eq!(report.affected.added, vec![created]);
        assert_eq!(report.failed_units.len(), 1);
        assert_eq!(report.failed_units[0].code, "UPDATE_TARGET_MISSING");
        assert_eq!(report.failed_units[0].action_index, Some(2));
        assert_eq!(report.failed_units[0].source_line, Some(4));
        assert_eq!(report.failed_units[0].source_column, Some(1));
        assert!(
            report.failed_units[0]
                .message
                .contains("Failed to read file to update")
        );
    }

    #[test]
    fn test_apply_patch_in_dir_rolls_back_failed_move_unit() {
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("src");
        let bad_parent = dir.path().join("blocked");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("name.txt"), "from\n").unwrap();
        fs::write(&bad_parent, "not a directory\n").unwrap();

        let patch = wrap_patch(&format!(
            "*** Update File: {}\n*** Move to: {}\n@@\n-from\n+new",
            source_dir.join("name.txt").display(),
            bad_parent.join("dir").join("name.txt").display()
        ));

        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();
        assert_eq!(report.status, ApplyPatchStatus::Failure);
        assert_eq!(report.failed_units[0].code, "TARGET_WRITE_ERROR");
        assert_eq!(report.failed_units[0].action_index, Some(1));
        assert_eq!(report.failed_units[0].source_line, Some(3));
        assert_eq!(report.failed_units[0].source_column, Some(1));
        assert_eq!(
            fs::read_to_string(source_dir.join("name.txt")).unwrap(),
            "from\n"
        );
        assert!(!bad_parent.join("dir").join("name.txt").exists());
    }

    #[test]
    fn test_apply_patch_in_dir_keeps_duplicate_path_updates_in_one_unit() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("dupe.txt");
        fs::write(&path, "a\nb\n").unwrap();
        let patch = wrap_patch(&format!(
            "*** Update File: {}\n@@\n-a\n+x\n*** Update File: {}\n@@\n-b\n+y",
            path.display(),
            path.display()
        ));

        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();
        assert_eq!(report.status, ApplyPatchStatus::FullSuccess);
        assert_eq!(fs::read_to_string(&path).unwrap(), "x\ny\n");
    }

    #[test]
    fn test_apply_patch_in_dir_warns_when_add_replaces_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("existing.txt");
        fs::write(&path, "old\n").unwrap();

        let patch = wrap_patch("*** Add File: existing.txt\n+new");
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::FullSuccess);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.warnings[0].code, "ADD_REPLACED_EXISTING_FILE");
        assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
    }

    #[test]
    fn test_apply_patch_in_dir_warns_when_move_replaces_existing_destination() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("from.txt");
        let dest = dir.path().join("to.txt");
        fs::write(&source, "from\n").unwrap();
        fs::write(&dest, "dest\n").unwrap();

        let patch = wrap_patch("*** Update File: from.txt\n*** Move to: to.txt\n@@\n-from\n+new");
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::FullSuccess);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(
            report.warnings[0].code,
            "MOVE_REPLACED_EXISTING_DESTINATION"
        );
        assert_eq!(fs::read_to_string(&dest).unwrap(), "new\n");
    }

    #[test]
    fn test_apply_patch_in_dir_elides_noop_update_without_touching_timestamp() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("noop.txt");
        fs::write(&path, "same\n").unwrap();
        let before = fs::metadata(&path).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1200));

        let patch = wrap_patch("*** Update File: noop.txt\n@@\n-same\n+same");
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();
        let after = fs::metadata(&path).unwrap().modified().unwrap();

        assert_eq!(report.status, ApplyPatchStatus::FullSuccess);
        assert!(report.affected.added.is_empty());
        assert!(report.affected.modified.is_empty());
        assert!(report.affected.deleted.is_empty());
        assert_eq!(fs::read_to_string(&path).unwrap(), "same\n");
        assert_eq!(before, after);
    }

    #[test]
    fn test_apply_patch_in_dir_reports_chunk_source_for_context_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app.txt");
        fs::write(&path, "value = 1\n").unwrap();

        let patch = wrap_patch("*** Update File: app.txt\n@@\n-missing = 1\n+value = 2");
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::Failure);
        assert_eq!(report.failed_units[0].code, "MATCH_INVALID_CONTEXT");
        assert_eq!(report.failed_units[0].action_index, Some(1));
        assert_eq!(report.failed_units[0].hunk_index, Some(1));
        assert_eq!(report.failed_units[0].source_line, Some(4));
        assert_eq!(report.failed_units[0].source_column, Some(1));
    }

    #[test]
    fn test_apply_patch_in_dir_reports_target_anchor_for_context_guided_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("app.txt");
        fs::write(&path, "fn handler():\n    value = 1\n").unwrap();

        let patch = wrap_patch(
            "*** Update File: app.txt\n@@ fn handler():\n-    missing = 1\n+    value = 2",
        );
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::Failure);
        assert_eq!(report.failed_units[0].code, "MATCH_INVALID_CONTEXT");
        let anchor = report.failed_units[0]
            .target_anchor
            .as_ref()
            .expect("target anchor");
        assert_eq!(anchor.path, path);
        assert_eq!(anchor.line_number, 1);
        assert_eq!(anchor.column_number, 1);
        assert_eq!(anchor.label.as_deref(), Some("matched context"));
        assert_eq!(anchor.excerpt.as_deref(), Some("fn handler():"));
    }

    #[test]
    fn test_apply_patch_in_dir_groups_normalized_alias_paths_atomically() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        let path = dir.path().join("item.txt");
        fs::write(&path, "a\nb\n").unwrap();

        let patch = wrap_patch(
            "*** Update File: sub/../item.txt\n@@\n-a\n+x\n*** Update File: item.txt\n@@\n-c\n+z",
        );
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::Failure);
        assert_eq!(report.failed_units[0].code, "MATCH_INVALID_CONTEXT");
        assert_eq!(report.failed_units[0].action_index, Some(2));
        assert_eq!(report.failed_units[0].hunk_index, Some(1));
        assert_eq!(fs::read_to_string(&path).unwrap(), "a\nb\n");
    }

    #[test]
    fn test_apply_patch_in_dir_treats_same_path_move_alias_as_in_place_update() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        let path = dir.path().join("note.txt");
        fs::write(&path, "from\n").unwrap();

        let patch =
            wrap_patch("*** Update File: note.txt\n*** Move to: sub/../note.txt\n@@\n-from\n+new");
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::FullSuccess);
        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
    }

    #[test]
    fn test_apply_update_file_preserves_crlf_when_updating_existing_file() {
        let path = Path::new("crlf.txt");
        let chunks = vec![UpdateFileChunk {
            change_context: None,
            old_lines: vec!["b".to_string()],
            new_lines: vec!["x".to_string()],
            is_end_of_file: false,
        }];

        let updated = apply_update_file_to_content("a\r\nb\r\nc\r\n", path, &chunks).unwrap();

        assert_eq!(updated, "a\r\nx\r\nc\r\n");
    }

    #[test]
    fn test_apply_update_file_preserves_eof_without_newline_when_replacing_last_line() {
        let path = Path::new("no_newline.txt");
        let chunks = vec![UpdateFileChunk {
            change_context: None,
            old_lines: vec!["no newline at end".to_string()],
            new_lines: vec!["first line".to_string(), "second line".to_string()],
            is_end_of_file: false,
        }];

        let updated = apply_update_file_to_content("no newline at end", path, &chunks).unwrap();

        assert_eq!(updated, "first line\nsecond line");
    }

    #[test]
    fn test_apply_update_file_uses_existing_style_for_added_lines_at_eof() {
        let path = Path::new("append.txt");
        let chunks = vec![UpdateFileChunk {
            change_context: None,
            old_lines: vec![],
            new_lines: vec!["added".to_string()],
            is_end_of_file: false,
        }];

        let updated = apply_update_file_to_content("head\r\nbody", path, &chunks).unwrap();

        assert_eq!(updated, "head\r\nbody\r\nadded");
    }

    #[cfg(windows)]
    #[test]
    fn test_apply_patch_in_dir_groups_case_alias_paths_atomically_on_windows() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("Name.txt");
        fs::write(&path, "a\nb\n").unwrap();

        let patch = wrap_patch(
            "*** Update File: Name.txt\n@@\n-a\n+x\n*** Update File: name.txt\n@@\n-c\n+z",
        );
        let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

        assert_eq!(report.status, ApplyPatchStatus::Failure);
        assert_eq!(report.failed_units[0].code, "MATCH_INVALID_CONTEXT");
        assert_eq!(fs::read_to_string(&path).unwrap(), "a\nb\n");
    }
}
