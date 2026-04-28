#![allow(clippy::result_large_err)]

use crate::line_boundary::{
    ends_with_line_break, needs_boundary_prefix, needs_boundary_suffix,
    normalize_payload_for_target_boundaries,
};
use crate::splice_program::{
    DeleteAction, SpliceAction, SpliceProgram, SpliceProgramParseError, TargetAction,
    TransferAction, TransferSourceKind, extract_splice_paths, parse_splice_program,
};
use crate::splice_selection::{
    ResolvedSelectionOffsets, SelectionBlock, SelectionItem, SelectionLine, SelectionResolveError,
    SelectionSide, resolve_selection_offsets,
};
use codex_apply_patch::{
    AffectedPaths, ByteFileChange, MissingAfterBehavior, PathIdentityKey, ResolvedPath,
    RuntimePathMap, RuntimePathState, build_connected_path_groups, commit_byte_changes_atomically,
    diff_affected_paths, extend_affected_paths, resolve_runtime_path,
};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceDiagnosticTargetAnchor {
    pub path: PathBuf,
    pub line_number: usize,
    pub column_number: usize,
    pub label: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceFailureDetails {
    pub error_code: String,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub action_index: Option<usize>,
    pub target_path: Option<PathBuf>,
    pub target_anchor: Option<SpliceDiagnosticTargetAnchor>,
    pub fix_hint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceFailedUnit {
    pub touched_paths: Vec<PathBuf>,
    pub attempted: AffectedPaths,
    pub committed: AffectedPaths,
    pub code: String,
    pub summary: String,
    pub target_path: Option<PathBuf>,
    pub action_index: Option<usize>,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub target_anchor: Option<SpliceDiagnosticTargetAnchor>,
    pub help: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceRuntimeError {
    summary: String,
    details: SpliceFailureDetails,
    affected: AffectedPaths,
    failed_units: Vec<SpliceFailedUnit>,
}

impl SpliceRuntimeError {
    fn from_parse_error(error: SpliceProgramParseError) -> Self {
        Self {
            summary: error.message().to_string(),
            details: SpliceFailureDetails {
                error_code: error.code().to_string(),
                source_line: error.source_line(),
                source_column: error.source_column(),
                action_index: None,
                target_path: None,
                target_anchor: None,
                fix_hint: Some(
                    "repair the splice envelope or selection syntax and retry".to_string(),
                ),
            },
            affected: AffectedPaths::default(),
            failed_units: Vec::new(),
        }
    }

    fn from_failed_units(affected: AffectedPaths, failed_units: Vec<SpliceFailedUnit>) -> Self {
        let primary = failed_units
            .first()
            .cloned()
            .expect("failed units must contain at least one entry");
        let has_committed = !affected.added.is_empty()
            || !affected.modified.is_empty()
            || !affected.deleted.is_empty();
        let (summary, details) = if has_committed {
            (
                format!(
                    "splice partially applied: {} {} committed, {} {} failed",
                    affected_path_count(&affected),
                    pluralize(affected_path_count(&affected), "path", "paths"),
                    failed_units.len(),
                    pluralize(failed_units.len(), "unit", "units")
                ),
                SpliceFailureDetails {
                    error_code: "SPLICE_PARTIAL_UNIT_FAILURE".to_string(),
                    source_line: primary.source_line,
                    source_column: primary.source_column,
                    action_index: primary.action_index,
                    target_path: primary.target_path.clone(),
                    target_anchor: primary.target_anchor.clone(),
                    fix_hint: Some(
                        "repair the failing splice unit and rerun the same splice program"
                            .to_string(),
                    ),
                },
            )
        } else {
            (
                primary.message.clone(),
                SpliceFailureDetails {
                    error_code: primary.code.clone(),
                    source_line: primary.source_line,
                    source_column: primary.source_column,
                    action_index: primary.action_index,
                    target_path: primary.target_path.clone(),
                    target_anchor: primary.target_anchor.clone(),
                    fix_hint: primary.help.clone(),
                },
            )
        };
        Self {
            summary,
            details,
            affected,
            failed_units,
        }
    }

    pub fn message(&self) -> &str {
        &self.summary
    }

    pub fn details(&self) -> &SpliceFailureDetails {
        &self.details
    }

    pub fn affected(&self) -> &AffectedPaths {
        &self.affected
    }

    pub fn failed_units(&self) -> &[SpliceFailedUnit] {
        &self.failed_units
    }
}

impl fmt::Display for SpliceRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary)
    }
}

impl std::error::Error for SpliceRuntimeError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceRuntimeOutcome {
    pub affected: AffectedPaths,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpliceWorkspaceRequirement {
    NeedsWorkspace,
    AbsoluteOnly { anchor_dir: PathBuf },
    Unanchored,
}

#[derive(Clone, Debug)]
struct FailureDraft {
    code: String,
    summary: String,
    action_index: usize,
    source_line: Option<usize>,
    source_column: Option<usize>,
    target_path: Option<PathBuf>,
    target_anchor: Option<SpliceDiagnosticTargetAnchor>,
    help: Option<String>,
    message: String,
}

type RuntimePath = ResolvedPath;

type PathState = RuntimePathState<Vec<u8>>;
type PathStateMap = RuntimePathMap<Vec<u8>>;

type ResolvedByteSelection = ResolvedSelectionOffsets;

#[derive(Clone, Debug)]
enum RuntimeTarget {
    Append {
        path: RuntimePath,
    },
    InsertBefore {
        path: RuntimePath,
        selection: ResolvedByteSelection,
    },
    InsertAfter {
        path: RuntimePath,
        selection: ResolvedByteSelection,
    },
    Replace {
        path: RuntimePath,
        selection: ResolvedByteSelection,
    },
}

#[derive(Clone, Debug)]
struct PlannedTransferAction {
    source_kind: TransferSourceKind,
    source_path: RuntimePath,
    source_selection: ResolvedByteSelection,
    transferred_bytes: Vec<u8>,
    target: RuntimeTarget,
    target_anchor: Option<SpliceDiagnosticTargetAnchor>,
}

#[derive(Clone, Debug)]
struct PlannedDeleteAction {
    source_path: RuntimePath,
    source_selection: ResolvedByteSelection,
}

#[derive(Clone, Debug)]
enum PlannedPathMutation {
    Delete {
        start: usize,
        end: usize,
        action_index: usize,
    },
    Insert {
        boundary: usize,
        bytes: Vec<u8>,
        action_index: usize,
    },
    Replace {
        start: usize,
        end: usize,
        bytes: Vec<u8>,
        action_index: usize,
    },
}

#[derive(Clone, Debug)]
struct PathMutationContext {
    action_index: usize,
    source_line: Option<usize>,
    source_column: Option<usize>,
    target_path: Option<PathBuf>,
    target_anchor: Option<SpliceDiagnosticTargetAnchor>,
}

#[derive(Clone, Debug)]
struct PlannedPathMutations {
    actual_path: PathBuf,
    mutations: Vec<PlannedPathMutation>,
    context: PathMutationContext,
}

type PlannedMutationMap = HashMap<PathIdentityKey, PlannedPathMutations>;

#[derive(Clone, Debug)]
struct ConnectedUnit {
    action_indices: Vec<usize>,
    touched_paths: Vec<RuntimePath>,
}

#[derive(Clone, Debug)]
struct UnitPlan {
    original_state: PathStateMap,
    working_state: PathStateMap,
    affected: AffectedPaths,
    path_contexts: HashMap<PathIdentityKey, PathMutationContext>,
}

#[derive(Clone, Debug)]
struct ComposeError {
    action_index: usize,
    message: String,
}

pub fn apply_splice_program(
    program: &str,
    base_dir: &Path,
) -> Result<SpliceRuntimeOutcome, SpliceRuntimeError> {
    let parsed = parse_splice_program(program).map_err(SpliceRuntimeError::from_parse_error)?;
    execute_splice_program(&parsed, base_dir)
}

pub fn splice_workspace_requirement(program: &str) -> SpliceWorkspaceRequirement {
    let mut anchor_dir = None;

    for path in extract_splice_paths(program).unwrap_or_default() {
        if path.is_absolute() {
            if anchor_dir.is_none() {
                anchor_dir = Some(path_anchor_dir(&path));
            }
            continue;
        }
        return SpliceWorkspaceRequirement::NeedsWorkspace;
    }

    anchor_dir
        .map(|anchor_dir| SpliceWorkspaceRequirement::AbsoluteOnly { anchor_dir })
        .unwrap_or(SpliceWorkspaceRequirement::Unanchored)
}

pub fn execute_splice_program(
    program: &SpliceProgram,
    base_dir: &Path,
) -> Result<SpliceRuntimeOutcome, SpliceRuntimeError> {
    let mut affected = AffectedPaths::default();
    let mut failed_units = Vec::new();

    for unit in build_connected_units(program, base_dir) {
        match plan_unit(program, base_dir, &unit) {
            Ok(plan) => match commit_unit(&unit, &plan) {
                Ok(unit_affected) => extend_affected_paths(&mut affected, unit_affected),
                Err(failure) => {
                    extend_affected_paths(&mut affected, failure.committed.clone());
                    failed_units.push(failure);
                }
            },
            Err(failure) => failed_units.push(failure),
        }
    }

    if failed_units.is_empty() {
        return Ok(SpliceRuntimeOutcome { affected });
    }

    Err(SpliceRuntimeError::from_failed_units(
        affected,
        failed_units,
    ))
}

fn build_connected_units(program: &SpliceProgram, base_dir: &Path) -> Vec<ConnectedUnit> {
    build_connected_path_groups(
        &program
            .actions
            .iter()
            .map(|action| touched_paths_for_action(action, base_dir))
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|group| ConnectedUnit {
        action_indices: group.item_indices,
        touched_paths: group.touched_paths,
    })
    .collect()
}

fn touched_paths_for_action(action: &SpliceAction, base_dir: &Path) -> Vec<RuntimePath> {
    match action {
        SpliceAction::Delete(action) => vec![resolve_program_path(base_dir, &action.source_path)],
        SpliceAction::Transfer(action) => {
            let mut paths = vec![resolve_program_path(base_dir, &action.source_path)];
            let target = match &action.target {
                TargetAction::Append { path }
                | TargetAction::InsertBefore { path, .. }
                | TargetAction::InsertAfter { path, .. }
                | TargetAction::Replace { path, .. } => resolve_program_path(base_dir, path),
            };
            if target.key != paths[0].key {
                paths.push(target);
            }
            paths
        }
    }
}

fn plan_unit(
    program: &SpliceProgram,
    base_dir: &Path,
    unit: &ConnectedUnit,
) -> Result<UnitPlan, SpliceFailedUnit> {
    let original_state = load_original_state(&unit.touched_paths).map_err(|draft| {
        failed_unit_from_draft(
            unit,
            draft,
            AffectedPaths::default(),
            AffectedPaths::default(),
        )
    })?;
    let mut planned_by_path: PlannedMutationMap = HashMap::new();

    for action_index in &unit.action_indices {
        let action = &program.actions[*action_index];
        let planning_result = match action {
            SpliceAction::Delete(action) => {
                plan_delete_action(*action_index, action, base_dir, &original_state).map(
                    |planned| {
                        add_mutation(
                            &mut planned_by_path,
                            &planned.source_path,
                            PlannedPathMutation::Delete {
                                start: planned.source_selection.start_offset,
                                end: planned.source_selection.end_offset,
                                action_index: *action_index,
                            },
                            delete_context(action),
                        );
                    },
                )
            }
            SpliceAction::Transfer(action) => {
                plan_transfer_action(*action_index, action, base_dir, &original_state).map(
                    |planned| {
                        append_planned_transfer_mutations(
                            *action_index,
                            action,
                            planned,
                            &mut planned_by_path,
                        )
                    },
                )
            }
        };

        if let Err(draft) = planning_result {
            let attempted = attempted_affected(&original_state, &planned_by_path);
            return Err(failed_unit_from_draft(
                unit,
                draft,
                attempted,
                AffectedPaths::default(),
            ));
        }
    }

    let working_state =
        compose_working_state(&original_state, &planned_by_path).map_err(|error| {
            let attempted = attempted_affected(&original_state, &planned_by_path);
            failed_unit_from_draft(
                unit,
                overlap_or_intermediate_failure(program, error),
                attempted,
                AffectedPaths::default(),
            )
        })?;
    let affected = diff_affected_paths(
        &original_state,
        &working_state,
        MissingAfterBehavior::TreatAsDeleted,
    );
    let path_contexts = planned_by_path
        .into_iter()
        .map(|(key, planned)| (key, planned.context))
        .collect::<HashMap<_, _>>();

    Ok(UnitPlan {
        original_state,
        working_state,
        affected,
        path_contexts,
    })
}

fn attempted_affected(
    original_state: &PathStateMap,
    planned_by_path: &PlannedMutationMap,
) -> AffectedPaths {
    compose_working_state(original_state, planned_by_path)
        .map(|working_state| {
            diff_affected_paths(
                original_state,
                &working_state,
                MissingAfterBehavior::TreatAsDeleted,
            )
        })
        .unwrap_or_default()
}

fn failed_unit_from_draft(
    unit: &ConnectedUnit,
    draft: FailureDraft,
    attempted: AffectedPaths,
    committed: AffectedPaths,
) -> SpliceFailedUnit {
    SpliceFailedUnit {
        touched_paths: unit_touched_paths(unit),
        attempted,
        committed,
        code: draft.code,
        summary: draft.summary,
        target_path: draft.target_path,
        action_index: Some(draft.action_index + 1),
        source_line: draft.source_line,
        source_column: draft.source_column,
        target_anchor: draft.target_anchor,
        help: draft.help,
        message: draft.message,
    }
}

fn unit_touched_paths(unit: &ConnectedUnit) -> Vec<PathBuf> {
    unit.touched_paths
        .iter()
        .map(|path| path.actual_path.clone())
        .collect()
}

fn overlap_or_intermediate_failure(program: &SpliceProgram, error: ComposeError) -> FailureDraft {
    let action = &program.actions[error.action_index];
    FailureDraft {
        code: "SPLICE_OVERLAP_ILLEGAL".to_string(),
        summary: "same-file anchored action is invalid against the original snapshot".to_string(),
        action_index: error.action_index,
        source_line: Some(action_header_line(action)),
        source_column: Some(1),
        target_path: transfer_target_path(action),
        target_anchor: transfer_target_anchor(action),
        help: Some(
            "re-read the same-file source and target ranges against one original snapshot"
                .to_string(),
        ),
        message: error.message,
    }
}

fn load_original_state(paths: &[RuntimePath]) -> Result<PathStateMap, FailureDraft> {
    let mut state = HashMap::new();
    for path in paths {
        if state.contains_key(&path.key) {
            continue;
        }
        let contents = match fs::read(&path.actual_path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(FailureDraft {
                    code: "SPLICE_SOURCE_STATE_INVALID".to_string(),
                    summary: format!(
                        "source path could not be read: {}",
                        path.actual_path.display()
                    ),
                    action_index: 0,
                    source_line: None,
                    source_column: Some(1),
                    target_path: Some(path.actual_path.clone()),
                    target_anchor: None,
                    help: Some(
                        "repair the referenced path and retry the same splice program".to_string(),
                    ),
                    message: format!("failed to read {}: {error}", path.actual_path.display()),
                });
            }
        };
        state.insert(
            path.key.clone(),
            PathState {
                actual_path: path.actual_path.clone(),
                contents,
            },
        );
    }
    Ok(state)
}

fn plan_delete_action(
    action_index: usize,
    action: &DeleteAction,
    base_dir: &Path,
    original_state: &PathStateMap,
) -> Result<PlannedDeleteAction, FailureDraft> {
    let source_path = resolve_program_path(base_dir, &action.source_path);
    let source_bytes = original_state
        .get(&source_path.key)
        .and_then(|state| state.contents.as_ref())
        .ok_or_else(|| {
            source_state_failure(
                action_index,
                action,
                &source_path.actual_path,
                "source file does not exist",
            )
        })?;
    let source_selection =
        resolve_byte_selection(&action.source_selection, source_bytes).map_err(|error| {
            selection_failure(
                action_index,
                action.authored_header_line,
                &action.source_selection,
                SelectionSide::Source,
                None,
                error,
            )
        })?;
    Ok(PlannedDeleteAction {
        source_path,
        source_selection,
    })
}

fn plan_transfer_action(
    action_index: usize,
    action: &TransferAction,
    base_dir: &Path,
    original_state: &PathStateMap,
) -> Result<PlannedTransferAction, FailureDraft> {
    let source_path = resolve_program_path(base_dir, &action.source_path);
    let source_bytes = original_state
        .get(&source_path.key)
        .and_then(|state| state.contents.as_ref())
        .ok_or_else(|| {
            source_state_failure(
                action_index,
                action,
                &source_path.actual_path,
                "source file does not exist",
            )
        })?;
    let source_selection =
        resolve_byte_selection(&action.source_selection, source_bytes).map_err(|error| {
            selection_failure(
                action_index,
                action.authored_header_line,
                &action.source_selection,
                SelectionSide::Source,
                None,
                error,
            )
        })?;
    let transferred_bytes =
        source_bytes[source_selection.start_offset..source_selection.end_offset].to_vec();

    let (target, target_anchor) = match &action.target {
        TargetAction::Append { path } => {
            let path = resolve_program_path(base_dir, path);
            (RuntimeTarget::Append { path: path.clone() }, None)
        }
        TargetAction::InsertBefore { path, selection } => {
            let path = resolve_program_path(base_dir, path);
            let target_bytes = original_state
                .get(&path.key)
                .and_then(|state| state.contents.as_ref())
                .ok_or_else(|| {
                    target_state_failure(
                        action_index,
                        action,
                        &path.actual_path,
                        selection,
                        "target file does not exist",
                    )
                })?;
            let resolved = resolve_byte_selection(selection, target_bytes).map_err(|error| {
                selection_failure(
                    action_index,
                    action.authored_header_line,
                    selection,
                    SelectionSide::Target,
                    Some(path.actual_path.clone()),
                    error,
                )
            })?;
            let anchor = selection_anchor(&path.actual_path, selection, "target anchor");
            (
                RuntimeTarget::InsertBefore {
                    path: path.clone(),
                    selection: resolved,
                },
                anchor,
            )
        }
        TargetAction::InsertAfter { path, selection } => {
            let path = resolve_program_path(base_dir, path);
            let target_bytes = original_state
                .get(&path.key)
                .and_then(|state| state.contents.as_ref())
                .ok_or_else(|| {
                    target_state_failure(
                        action_index,
                        action,
                        &path.actual_path,
                        selection,
                        "target file does not exist",
                    )
                })?;
            let resolved = resolve_byte_selection(selection, target_bytes).map_err(|error| {
                selection_failure(
                    action_index,
                    action.authored_header_line,
                    selection,
                    SelectionSide::Target,
                    Some(path.actual_path.clone()),
                    error,
                )
            })?;
            let anchor = selection_anchor(&path.actual_path, selection, "target anchor");
            (
                RuntimeTarget::InsertAfter {
                    path: path.clone(),
                    selection: resolved,
                },
                anchor,
            )
        }
        TargetAction::Replace { path, selection } => {
            let path = resolve_program_path(base_dir, path);
            let target_bytes = original_state
                .get(&path.key)
                .and_then(|state| state.contents.as_ref())
                .ok_or_else(|| {
                    target_state_failure(
                        action_index,
                        action,
                        &path.actual_path,
                        selection,
                        "target file does not exist",
                    )
                })?;
            let resolved = resolve_byte_selection(selection, target_bytes).map_err(|error| {
                selection_failure(
                    action_index,
                    action.authored_header_line,
                    selection,
                    SelectionSide::Target,
                    Some(path.actual_path.clone()),
                    error,
                )
            })?;
            let anchor = selection_anchor(&path.actual_path, selection, "target anchor");
            (
                RuntimeTarget::Replace {
                    path: path.clone(),
                    selection: resolved,
                },
                anchor,
            )
        }
    };

    reject_same_file_overlap(
        action_index,
        action,
        &source_path,
        &source_selection,
        &target,
    )?;

    let transferred_bytes = normalize_transfer_bytes_for_target_boundary(
        transferred_bytes,
        &source_selection,
        source_bytes.len(),
        &target,
        original_state,
    );

    Ok(PlannedTransferAction {
        source_kind: action.source_kind,
        source_path,
        source_selection,
        transferred_bytes,
        target,
        target_anchor,
    })
}

fn normalize_transfer_bytes_for_target_boundary(
    transferred_bytes: Vec<u8>,
    source_selection: &ResolvedByteSelection,
    source_file_len: usize,
    target: &RuntimeTarget,
    original_state: &PathStateMap,
) -> Vec<u8> {
    if transferred_bytes.is_empty()
        || source_selection.end_offset != source_file_len
        || ends_with_line_break(&transferred_bytes)
    {
        return transferred_bytes;
    }

    let target_path = target_runtime_path(target);
    let target_bytes = original_state
        .get(&target_path.key)
        .and_then(|state| state.contents.as_deref())
        .unwrap_or_default();
    let (needs_prefix, needs_suffix) = requires_line_boundary_normalization(target, target_bytes);
    if !needs_prefix && !needs_suffix {
        return transferred_bytes;
    }

    normalize_payload_for_target_boundaries(
        transferred_bytes,
        target_bytes,
        needs_prefix,
        needs_suffix,
    )
}

fn requires_line_boundary_normalization(
    target: &RuntimeTarget,
    target_bytes: &[u8],
) -> (bool, bool) {
    match target {
        RuntimeTarget::Append { .. } => {
            let boundary = target_bytes.len();
            (needs_boundary_prefix(target_bytes, boundary), false)
        }
        RuntimeTarget::InsertBefore { selection, .. } => (
            false,
            needs_boundary_suffix(target_bytes, selection.start_offset),
        ),
        RuntimeTarget::InsertAfter { selection, .. } => {
            let boundary = selection.end_offset;
            (
                needs_boundary_prefix(target_bytes, boundary),
                needs_boundary_suffix(target_bytes, boundary),
            )
        }
        RuntimeTarget::Replace { selection, .. } => (
            false,
            needs_boundary_suffix(target_bytes, selection.end_offset),
        ),
    }
}

fn selection_failure(
    action_index: usize,
    header_line: usize,
    selection: &SelectionBlock,
    side: SelectionSide,
    target_path: Option<PathBuf>,
    error: SelectionResolveError,
) -> FailureDraft {
    let source_line = error
        .item_index()
        .and_then(|item_index| selection.authored_line_for_item_index(item_index))
        .or(selection.authored_header_line)
        .or(Some(header_line));
    let target_anchor = target_path
        .as_ref()
        .and_then(|path| selection_anchor(path, selection, "target anchor"));
    let code = match side {
        SelectionSide::Source => "SPLICE_SOURCE_SELECTION_INVALID",
        SelectionSide::Target => "SPLICE_TARGET_SELECTION_INVALID",
        SelectionSide::Rewrite => unreachable!("splice runtime never uses rewrite selections"),
    }
    .to_string();
    let summary = match side {
        SelectionSide::Source => "source selection did not resolve against the current file text",
        SelectionSide::Target => "target selection did not resolve against the current file text",
        SelectionSide::Rewrite => unreachable!("splice runtime never uses rewrite selections"),
    }
    .to_string();
    FailureDraft {
        code,
        summary,
        action_index,
        source_line,
        source_column: Some(1),
        target_path,
        target_anchor,
        help: Some(
            "re-read the numbered excerpt against the current file text and regenerate the splice"
                .to_string(),
        ),
        message: match side {
            SelectionSide::Source => format!("invalid source selection: {}", error.message()),
            SelectionSide::Target => format!("invalid target selection: {}", error.message()),
            SelectionSide::Rewrite => {
                unreachable!("splice runtime never uses rewrite selections")
            }
        },
    }
}

fn source_state_failure(
    action_index: usize,
    action: &impl HasAuthoredHeaderLine,
    source_path: &Path,
    message: &str,
) -> FailureDraft {
    FailureDraft {
        code: "SPLICE_SOURCE_STATE_INVALID".to_string(),
        summary: message.to_string(),
        action_index,
        source_line: Some(action.authored_header_line()),
        source_column: Some(1),
        target_path: Some(source_path.to_path_buf()),
        target_anchor: None,
        help: Some(
            "repair the source path or selection and retry the same splice program".to_string(),
        ),
        message: format!("{}: {}", message, source_path.display()),
    }
}

fn target_state_failure(
    action_index: usize,
    action: &TransferAction,
    target_path: &Path,
    selection: &SelectionBlock,
    message: &str,
) -> FailureDraft {
    FailureDraft {
        code: "SPLICE_TARGET_STATE_INVALID".to_string(),
        summary: message.to_string(),
        action_index,
        source_line: selection
            .authored_header_line
            .or(Some(action.authored_header_line)),
        source_column: Some(1),
        target_path: Some(target_path.to_path_buf()),
        target_anchor: selection_anchor(target_path, selection, "target anchor"),
        help: Some(
            "repair the target path or target selection and retry the same splice program"
                .to_string(),
        ),
        message: format!("{}: {}", message, target_path.display()),
    }
}

fn reject_same_file_overlap(
    action_index: usize,
    action: &TransferAction,
    source_path: &RuntimePath,
    source_selection: &ResolvedByteSelection,
    target: &RuntimeTarget,
) -> Result<(), FailureDraft> {
    let (target_path, target_range, target_anchor) = match target {
        RuntimeTarget::Append { .. } => return Ok(()),
        RuntimeTarget::InsertBefore { path, selection }
        | RuntimeTarget::InsertAfter { path, selection }
        | RuntimeTarget::Replace { path, selection } => (
            path,
            (selection.start_offset, selection.end_offset),
            transfer_target_anchor(&SpliceAction::Transfer(action.clone())),
        ),
    };

    if target_path.key == source_path.key
        && ranges_overlap(
            (source_selection.start_offset, source_selection.end_offset),
            target_range,
        )
    {
        return Err(FailureDraft {
            code: "SPLICE_OVERLAP_ILLEGAL".to_string(),
            summary: "same-file anchored source and target ranges overlap".to_string(),
            action_index,
            source_line: Some(action.authored_header_line),
            source_column: Some(1),
            target_path: Some(target_path.actual_path.clone()),
            target_anchor,
            help: Some(
                "rewrite the same-file action so source and target ranges do not overlap"
                    .to_string(),
            ),
            message: "same-file anchored action is invalid when source and target ranges overlap"
                .to_string(),
        });
    }

    Ok(())
}

fn append_planned_transfer_mutations(
    action_index: usize,
    action: &TransferAction,
    planned: PlannedTransferAction,
    planned_by_path: &mut PlannedMutationMap,
) {
    let target_path = target_runtime_path(&planned.target).clone();
    let target_context = PathMutationContext {
        action_index,
        source_line: Some(action.authored_header_line),
        source_column: Some(1),
        target_path: Some(target_path.actual_path.clone()),
        target_anchor: planned.target_anchor.clone(),
    };
    match planned.target {
        RuntimeTarget::Append { .. } => add_mutation(
            planned_by_path,
            &target_path,
            PlannedPathMutation::Insert {
                boundary: usize::MAX,
                bytes: planned.transferred_bytes.clone(),
                action_index,
            },
            target_context.clone(),
        ),
        RuntimeTarget::InsertBefore { selection, .. } => add_mutation(
            planned_by_path,
            &target_path,
            PlannedPathMutation::Insert {
                boundary: selection.start_offset,
                bytes: planned.transferred_bytes.clone(),
                action_index,
            },
            target_context.clone(),
        ),
        RuntimeTarget::InsertAfter { selection, .. } => add_mutation(
            planned_by_path,
            &target_path,
            PlannedPathMutation::Insert {
                boundary: selection.end_offset,
                bytes: planned.transferred_bytes.clone(),
                action_index,
            },
            target_context.clone(),
        ),
        RuntimeTarget::Replace { selection, .. } => add_mutation(
            planned_by_path,
            &target_path,
            PlannedPathMutation::Replace {
                start: selection.start_offset,
                end: selection.end_offset,
                bytes: planned.transferred_bytes.clone(),
                action_index,
            },
            target_context.clone(),
        ),
    }

    if matches!(planned.source_kind, TransferSourceKind::Move) {
        add_mutation(
            planned_by_path,
            &planned.source_path,
            PlannedPathMutation::Delete {
                start: planned.source_selection.start_offset,
                end: planned.source_selection.end_offset,
                action_index,
            },
            target_context,
        );
    }
}

fn add_mutation(
    planned_by_path: &mut PlannedMutationMap,
    path: &RuntimePath,
    mutation: PlannedPathMutation,
    context: PathMutationContext,
) {
    planned_by_path
        .entry(path.key.clone())
        .and_modify(|planned| {
            planned.mutations.push(mutation.clone());
            if context.action_index < planned.context.action_index {
                planned.context = context.clone();
            }
        })
        .or_insert_with(|| PlannedPathMutations {
            actual_path: path.actual_path.clone(),
            mutations: vec![mutation],
            context,
        });
}

fn target_runtime_path(target: &RuntimeTarget) -> &RuntimePath {
    match target {
        RuntimeTarget::Append { path }
        | RuntimeTarget::InsertBefore { path, .. }
        | RuntimeTarget::InsertAfter { path, .. }
        | RuntimeTarget::Replace { path, .. } => path,
    }
}

fn delete_context(action: &DeleteAction) -> PathMutationContext {
    PathMutationContext {
        action_index: 0,
        source_line: Some(action.authored_header_line),
        source_column: Some(1),
        target_path: None,
        target_anchor: None,
    }
}

fn resolve_program_path(base_dir: &Path, raw_path: &str) -> RuntimePath {
    let raw = PathBuf::from(raw_path);
    let (actual_path, key) = resolve_runtime_path(base_dir, &raw);
    RuntimePath { key, actual_path }
}

fn path_anchor_dir(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf())
}

fn resolve_byte_selection(
    block: &SelectionBlock,
    file_bytes: &[u8],
) -> Result<ResolvedByteSelection, SelectionResolveError> {
    resolve_selection_offsets(block, file_bytes, "file text")
}

fn compose_working_state(
    original_state: &PathStateMap,
    planned_by_path: &PlannedMutationMap,
) -> Result<PathStateMap, ComposeError> {
    let mut working_state = original_state.clone();
    for (key, planned) in planned_by_path {
        let original_bytes = working_state
            .get(key)
            .and_then(|state| state.contents.clone())
            .unwrap_or_default();
        let composed = compose_file_bytes(&original_bytes, &planned.mutations)?;
        working_state.insert(
            key.clone(),
            PathState {
                actual_path: planned.actual_path.clone(),
                contents: Some(composed),
            },
        );
    }
    Ok(working_state)
}

fn compose_file_bytes(
    original: &[u8],
    mutations: &[PlannedPathMutation],
) -> Result<Vec<u8>, ComposeError> {
    #[derive(Clone)]
    struct RangeMutation {
        start: usize,
        end: usize,
        replacement: Vec<u8>,
        action_index: usize,
    }

    let eof = original.len();
    let mut inserts: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    let mut ranges: Vec<RangeMutation> = Vec::new();
    for mutation in mutations {
        match mutation {
            PlannedPathMutation::Delete {
                start,
                end,
                action_index,
            } => ranges.push(RangeMutation {
                start: *start,
                end: *end,
                replacement: Vec::new(),
                action_index: *action_index,
            }),
            PlannedPathMutation::Insert {
                boundary,
                bytes,
                action_index,
            } => {
                let resolved_boundary = if *boundary == usize::MAX {
                    eof
                } else {
                    *boundary
                };
                inserts.push((resolved_boundary, *action_index, bytes.clone()));
            }
            PlannedPathMutation::Replace {
                start,
                end,
                bytes,
                action_index,
            } => ranges.push(RangeMutation {
                start: *start,
                end: *end,
                replacement: bytes.clone(),
                action_index: *action_index,
            }),
        }
    }

    ranges.sort_by_key(|range| (range.start, range.end, range.action_index));
    inserts.sort_by_key(|(boundary, action_index, _)| (*boundary, *action_index));

    for window in ranges.windows(2) {
        let left = &window[0];
        let right = &window[1];
        if right.start < left.end {
            return Err(ComposeError {
                action_index: right.action_index,
                message:
                    "runtime rejected overlapping same-file mutations planned against one original snapshot"
                        .to_string(),
            });
        }
    }

    for (boundary, action_index, _) in &inserts {
        if ranges
            .iter()
            .any(|range| *boundary > range.start && *boundary < range.end)
        {
            return Err(ComposeError {
                action_index: *action_index,
                message: "runtime rejected an insertion boundary that falls inside another planned same-file mutation".to_string(),
            });
        }
    }

    let mut result = Vec::new();
    let mut cursor = 0usize;
    let mut insert_index = 0usize;

    for range in &ranges {
        while insert_index < inserts.len() && inserts[insert_index].0 < range.start {
            let (boundary, _, bytes) = &inserts[insert_index];
            result.extend_from_slice(&original[cursor..*boundary]);
            cursor = *boundary;
            result.extend_from_slice(bytes);
            insert_index += 1;
        }

        result.extend_from_slice(&original[cursor..range.start]);
        while insert_index < inserts.len() && inserts[insert_index].0 == range.start {
            result.extend_from_slice(&inserts[insert_index].2);
            insert_index += 1;
        }
        result.extend_from_slice(&range.replacement);
        cursor = range.end;
    }

    while insert_index < inserts.len() {
        let (boundary, _, bytes) = &inserts[insert_index];
        result.extend_from_slice(&original[cursor..*boundary]);
        cursor = *boundary;
        result.extend_from_slice(bytes);
        insert_index += 1;
    }

    result.extend_from_slice(&original[cursor..]);
    Ok(result)
}

fn commit_unit(unit: &ConnectedUnit, plan: &UnitPlan) -> Result<AffectedPaths, SpliceFailedUnit> {
    let changed_paths = collect_changed_paths(plan);
    let byte_changes = changed_paths
        .iter()
        .map(|changed| ByteFileChange {
            key: changed.key.clone(),
            actual_path: changed.actual_path.clone(),
            after: changed.after.clone(),
        })
        .collect::<Vec<_>>();

    if let Err(error) = commit_byte_changes_atomically(&byte_changes) {
        let changed = changed_paths
            .iter()
            .find(|changed| changed.key == error.key)
            .expect("failed path must belong to changed set");
        let actual_state = reload_changed_state(&changed_paths);
        let committed =
            diff_affected_paths_subset(&plan.original_state, &actual_state, &changed_paths);
        return Err(failed_unit_from_draft(
            unit,
            FailureDraft {
                code: "SPLICE_WRITE_ERROR".to_string(),
                summary: format!("failed to write {}", error.actual_path.display()),
                action_index: changed.context.action_index,
                source_line: changed.context.source_line,
                source_column: changed.context.source_column,
                target_path: changed
                    .context
                    .target_path
                    .clone()
                    .or_else(|| Some(error.actual_path.clone())),
                target_anchor: changed.context.target_anchor.clone(),
                help: Some(
                    "repair the failing filesystem path and rerun the same splice program"
                        .to_string(),
                ),
                message: format!(
                    "failed to write {}: {}",
                    error.actual_path.display(),
                    error.error
                ),
            },
            plan.affected.clone(),
            committed,
        ));
    }

    Ok(plan.affected.clone())
}

#[derive(Clone, Debug)]
struct ChangedPath {
    key: PathIdentityKey,
    actual_path: PathBuf,
    after: Option<Vec<u8>>,
    context: PathMutationContext,
}

fn collect_changed_paths(plan: &UnitPlan) -> Vec<ChangedPath> {
    let mut keys = plan
        .original_state
        .keys()
        .chain(plan.working_state.keys())
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();

    let mut changed_paths = Vec::new();
    for key in keys {
        let before = plan
            .original_state
            .get(&key)
            .and_then(|state| state.contents.clone());
        let after = plan
            .working_state
            .get(&key)
            .and_then(|state| state.contents.clone());
        if before == after {
            continue;
        }
        let actual_path = plan
            .working_state
            .get(&key)
            .map(|state| state.actual_path.clone())
            .or_else(|| {
                plan.original_state
                    .get(&key)
                    .map(|state| state.actual_path.clone())
            })
            .expect("changed path must exist in original or working state");
        let context = plan
            .path_contexts
            .get(&key)
            .cloned()
            .unwrap_or(PathMutationContext {
                action_index: 0,
                source_line: None,
                source_column: Some(1),
                target_path: Some(actual_path.clone()),
                target_anchor: None,
            });
        changed_paths.push(ChangedPath {
            key,
            actual_path,
            after,
            context,
        });
    }
    changed_paths
}

fn reload_changed_state(changed_paths: &[ChangedPath]) -> PathStateMap {
    let mut state = HashMap::new();
    for changed in changed_paths {
        let contents = match fs::read(&changed.actual_path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(_) => changed.after.clone(),
        };
        state.insert(
            changed.key.clone(),
            PathState {
                actual_path: changed.actual_path.clone(),
                contents,
            },
        );
    }
    state
}

fn diff_affected_paths_subset(
    before: &PathStateMap,
    after: &PathStateMap,
    changed_paths: &[ChangedPath],
) -> AffectedPaths {
    let subset_before = changed_paths
        .iter()
        .filter_map(|changed| {
            before
                .get(&changed.key)
                .cloned()
                .map(|state| (changed.key.clone(), state))
        })
        .collect::<HashMap<_, _>>();
    diff_affected_paths(&subset_before, after, MissingAfterBehavior::TreatAsDeleted)
}

fn ranges_overlap(lhs: (usize, usize), rhs: (usize, usize)) -> bool {
    lhs.0 < rhs.1 && rhs.0 < lhs.1
}

fn selection_anchor(
    path: &Path,
    selection: &SelectionBlock,
    label: &str,
) -> Option<SpliceDiagnosticTargetAnchor> {
    selection.items.iter().find_map(|item| match item {
        SelectionItem::Line(SelectionLine {
            line_number,
            visible_content,
        }) => Some(SpliceDiagnosticTargetAnchor {
            path: path.to_path_buf(),
            line_number: *line_number,
            column_number: 1,
            label: Some(label.to_string()),
            excerpt: Some(visible_content.clone()),
        }),
        SelectionItem::Omission => None,
    })
}

fn affected_path_count(affected: &AffectedPaths) -> usize {
    affected.added.len() + affected.modified.len() + affected.deleted.len()
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn action_header_line(action: &SpliceAction) -> usize {
    match action {
        SpliceAction::Transfer(action) => action.authored_header_line,
        SpliceAction::Delete(action) => action.authored_header_line,
    }
}

fn transfer_target_path(action: &SpliceAction) -> Option<PathBuf> {
    match action {
        SpliceAction::Transfer(action) => match &action.target {
            TargetAction::Append { path }
            | TargetAction::InsertBefore { path, .. }
            | TargetAction::InsertAfter { path, .. }
            | TargetAction::Replace { path, .. } => Some(PathBuf::from(path)),
        },
        SpliceAction::Delete(_) => None,
    }
}

fn transfer_target_anchor(action: &SpliceAction) -> Option<SpliceDiagnosticTargetAnchor> {
    match action {
        SpliceAction::Transfer(action) => match &action.target {
            TargetAction::Append { .. } => None,
            TargetAction::InsertBefore { path, selection }
            | TargetAction::InsertAfter { path, selection }
            | TargetAction::Replace { path, selection } => {
                selection_anchor(Path::new(path), selection, "target anchor")
            }
        },
        SpliceAction::Delete(_) => None,
    }
}

trait HasAuthoredHeaderLine {
    fn authored_header_line(&self) -> usize;
}

impl HasAuthoredHeaderLine for DeleteAction {
    fn authored_header_line(&self) -> usize {
        self.authored_header_line
    }
}

impl HasAuthoredHeaderLine for TransferAction {
    fn authored_header_line(&self) -> usize {
        self.authored_header_line
    }
}
