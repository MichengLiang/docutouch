use crate::line_boundary::normalize_replacement_payload_for_result_suffix;
use crate::rewrite_program::{
    AddFileOperation, DeleteFileOperation, RewriteAction, RewriteFileOperation, RewriteProgram,
    RewriteProgramParseError, UpdateFileOperation, extract_rewrite_paths, parse_rewrite_program,
};
use crate::splice_selection::{
    ResolvedSelectionOffsets, SelectionBlock, SelectionDiagnosticCode, SelectionItem,
    SelectionLine, SelectionResolveError, resolve_selection_offsets,
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
pub struct RewriteDiagnosticTargetAnchor {
    pub path: PathBuf,
    pub line_number: usize,
    pub column_number: usize,
    pub label: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteWarning {
    pub code: String,
    pub summary: String,
    pub target_path: PathBuf,
    pub help: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteFailureDetails {
    pub error_code: String,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub operation_index: Option<usize>,
    pub target_path: Option<PathBuf>,
    pub target_anchor: Option<RewriteDiagnosticTargetAnchor>,
    pub fix_hint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteFailedUnit {
    pub touched_paths: Vec<PathBuf>,
    pub attempted: AffectedPaths,
    pub committed: AffectedPaths,
    pub code: String,
    pub summary: String,
    pub target_path: Option<PathBuf>,
    pub operation_index: Option<usize>,
    pub source_line: Option<usize>,
    pub source_column: Option<usize>,
    pub target_anchor: Option<RewriteDiagnosticTargetAnchor>,
    pub help: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRuntimeError {
    summary: String,
    details: RewriteFailureDetails,
    affected: AffectedPaths,
    warnings: Vec<RewriteWarning>,
    failed_units: Vec<RewriteFailedUnit>,
}

impl RewriteRuntimeError {
    fn from_parse_error(error: RewriteProgramParseError) -> Self {
        Self {
            summary: error.message().to_string(),
            details: RewriteFailureDetails {
                error_code: error.code().to_string(),
                source_line: error.source_line(),
                source_column: error.source_column(),
                operation_index: None,
                target_path: None,
                target_anchor: None,
                fix_hint: Some("repair the rewrite program syntax and retry".to_string()),
            },
            affected: AffectedPaths::default(),
            warnings: Vec::new(),
            failed_units: Vec::new(),
        }
    }

    fn from_failed_units(
        affected: AffectedPaths,
        warnings: Vec<RewriteWarning>,
        failed_units: Vec<RewriteFailedUnit>,
    ) -> Self {
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
                    "rewrite partially applied: {} {} committed, {} {} failed",
                    affected_path_count(&affected),
                    pluralize(affected_path_count(&affected), "path", "paths"),
                    failed_units.len(),
                    pluralize(failed_units.len(), "unit", "units"),
                ),
                RewriteFailureDetails {
                    error_code: "REWRITE_PARTIAL_UNIT_FAILURE".to_string(),
                    source_line: primary.source_line,
                    source_column: primary.source_column,
                    operation_index: primary.operation_index,
                    target_path: primary.target_path.clone(),
                    target_anchor: primary.target_anchor.clone(),
                    fix_hint: Some(
                        "repair the failing rewrite unit and rerun the same rewrite program"
                            .to_string(),
                    ),
                },
            )
        } else {
            (
                primary.message.clone(),
                RewriteFailureDetails {
                    error_code: primary.code.clone(),
                    source_line: primary.source_line,
                    source_column: primary.source_column,
                    operation_index: primary.operation_index,
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
            warnings,
            failed_units,
        }
    }

    pub fn message(&self) -> &str {
        &self.summary
    }

    pub fn details(&self) -> &RewriteFailureDetails {
        &self.details
    }

    pub fn affected(&self) -> &AffectedPaths {
        &self.affected
    }

    pub fn warnings(&self) -> &[RewriteWarning] {
        &self.warnings
    }

    pub fn failed_units(&self) -> &[RewriteFailedUnit] {
        &self.failed_units
    }
}

impl fmt::Display for RewriteRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary)
    }
}

impl std::error::Error for RewriteRuntimeError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRuntimeOutcome {
    pub affected: AffectedPaths,
    pub warnings: Vec<RewriteWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteWorkspaceRequirement {
    NeedsWorkspace,
    AbsoluteOnly { anchor_dir: PathBuf },
    Unanchored,
}

#[derive(Clone, Debug)]
struct FailureDraft {
    code: String,
    summary: String,
    operation_index: usize,
    source_line: Option<usize>,
    source_column: Option<usize>,
    target_path: Option<PathBuf>,
    target_anchor: Option<RewriteDiagnosticTargetAnchor>,
    help: Option<String>,
    message: String,
}

#[derive(Clone, Debug)]
struct PathMutationContext {
    operation_index: usize,
    source_line: Option<usize>,
    source_column: Option<usize>,
    target_path: Option<PathBuf>,
    target_anchor: Option<RewriteDiagnosticTargetAnchor>,
}

#[derive(Clone, Debug)]
struct ConnectedUnit {
    operation_indices: Vec<usize>,
    touched_paths: Vec<ResolvedPath>,
}

#[derive(Clone, Debug)]
struct UnitPlan {
    original_state: PathStateMap,
    working_state: PathStateMap,
    affected: AffectedPaths,
    warnings: Vec<RewriteWarning>,
    path_contexts: HashMap<PathIdentityKey, PathMutationContext>,
}

#[derive(Clone, Debug)]
struct ChangedPath {
    key: PathIdentityKey,
    actual_path: PathBuf,
    after: Option<Vec<u8>>,
    context: PathMutationContext,
}

#[derive(Clone, Debug)]
struct ResolvedRewriteAction {
    authored_selection_line: usize,
    selection: SelectionBlock,
    offsets: ResolvedSelectionOffsets,
    replacement: Vec<u8>,
}

type RuntimePath = ResolvedPath;
type PathState = RuntimePathState<Vec<u8>>;
type PathStateMap = RuntimePathMap<Vec<u8>>;

pub fn apply_rewrite_program(
    program: &str,
    base_dir: &Path,
) -> Result<RewriteRuntimeOutcome, RewriteRuntimeError> {
    let parsed = parse_rewrite_program(program).map_err(RewriteRuntimeError::from_parse_error)?;
    execute_rewrite_program(&parsed, base_dir)
}

pub fn rewrite_workspace_requirement(program: &str) -> RewriteWorkspaceRequirement {
    let mut anchor_dir = None;
    for path in extract_rewrite_paths(program).unwrap_or_default() {
        if path.is_absolute() {
            if anchor_dir.is_none() {
                anchor_dir = Some(path_anchor_dir(&path));
            }
            continue;
        }
        return RewriteWorkspaceRequirement::NeedsWorkspace;
    }
    anchor_dir
        .map(|anchor_dir| RewriteWorkspaceRequirement::AbsoluteOnly { anchor_dir })
        .unwrap_or(RewriteWorkspaceRequirement::Unanchored)
}

pub fn execute_rewrite_program(
    program: &RewriteProgram,
    base_dir: &Path,
) -> Result<RewriteRuntimeOutcome, RewriteRuntimeError> {
    let mut affected = AffectedPaths::default();
    let mut warnings = Vec::new();
    let mut failed_units = Vec::new();

    for unit in build_connected_units(program, base_dir) {
        match plan_unit(program, base_dir, &unit) {
            Ok(plan) => match commit_unit(&unit, &plan) {
                Ok(unit_affected) => {
                    extend_affected_paths(&mut affected, unit_affected);
                    warnings.extend(plan.warnings.clone());
                }
                Err(failure) => {
                    extend_affected_paths(&mut affected, failure.committed.clone());
                    warnings.extend(plan.warnings.clone());
                    failed_units.push(failure);
                }
            },
            Err(failure) => failed_units.push(failure),
        }
    }

    if failed_units.is_empty() {
        return Ok(RewriteRuntimeOutcome { affected, warnings });
    }

    Err(RewriteRuntimeError::from_failed_units(
        affected,
        warnings,
        failed_units,
    ))
}

fn build_connected_units(program: &RewriteProgram, base_dir: &Path) -> Vec<ConnectedUnit> {
    build_connected_path_groups(
        &program
            .operations
            .iter()
            .map(|operation| touched_paths_for_operation(operation, base_dir))
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|group| ConnectedUnit {
        operation_indices: group.item_indices,
        touched_paths: group.touched_paths,
    })
    .collect()
}

fn touched_paths_for_operation(
    operation: &RewriteFileOperation,
    base_dir: &Path,
) -> Vec<RuntimePath> {
    match operation {
        RewriteFileOperation::Add(operation) => {
            vec![resolve_program_path(base_dir, &operation.path)]
        }
        RewriteFileOperation::Delete(operation) => {
            vec![resolve_program_path(base_dir, &operation.path)]
        }
        RewriteFileOperation::Update(operation) => {
            let mut paths = vec![resolve_program_path(base_dir, &operation.path)];
            if let Some(move_path) = &operation.move_path {
                let move_target = resolve_program_path(base_dir, move_path);
                if move_target.key != paths[0].key {
                    paths.push(move_target);
                }
            }
            paths
        }
    }
}

fn plan_unit(
    program: &RewriteProgram,
    base_dir: &Path,
    unit: &ConnectedUnit,
) -> Result<UnitPlan, RewriteFailedUnit> {
    let original_state = load_original_state(&unit.touched_paths).map_err(|draft| {
        failed_unit_from_draft(
            unit,
            draft,
            AffectedPaths::default(),
            AffectedPaths::default(),
        )
    })?;
    let mut working_state = original_state.clone();
    let mut warnings = Vec::new();
    let mut path_contexts = HashMap::new();

    for operation_index in &unit.operation_indices {
        let operation = &program.operations[*operation_index];
        let planning = match operation {
            RewriteFileOperation::Add(operation) => apply_add_operation(
                *operation_index,
                operation,
                base_dir,
                &mut working_state,
                &mut warnings,
                &mut path_contexts,
            ),
            RewriteFileOperation::Delete(operation) => apply_delete_operation(
                *operation_index,
                operation,
                base_dir,
                &mut working_state,
                &mut path_contexts,
            ),
            RewriteFileOperation::Update(operation) => apply_update_operation(
                *operation_index,
                operation,
                base_dir,
                &mut working_state,
                &mut warnings,
                &mut path_contexts,
            ),
        };
        if let Err(draft) = planning {
            let attempted = diff_affected_paths(
                &original_state,
                &working_state,
                MissingAfterBehavior::TreatAsDeleted,
            );
            return Err(failed_unit_from_draft(
                unit,
                draft,
                attempted,
                AffectedPaths::default(),
            ));
        }
    }

    let affected = diff_affected_paths(
        &original_state,
        &working_state,
        MissingAfterBehavior::TreatAsDeleted,
    );
    let affected = normalize_move_rewrite_accounting(program, unit, base_dir, affected);
    Ok(UnitPlan {
        original_state,
        working_state,
        affected,
        warnings,
        path_contexts,
    })
}

fn apply_add_operation(
    operation_index: usize,
    operation: &AddFileOperation,
    base_dir: &Path,
    working_state: &mut PathStateMap,
    warnings: &mut Vec<RewriteWarning>,
    path_contexts: &mut HashMap<PathIdentityKey, PathMutationContext>,
) -> Result<(), FailureDraft> {
    let path = resolve_program_path(base_dir, &operation.path);
    if current_contents(working_state, &path).is_some() {
        warnings.push(RewriteWarning {
            code: "ADD_REPLACED_EXISTING_FILE".to_string(),
            summary: format!(
                "add file replaced existing file: {}",
                path.actual_path.display()
            ),
            target_path: path.actual_path.clone(),
            help: Some(
                "prefer `Add File` only for true creation; existing targets are compatibility-only"
                    .to_string(),
            ),
        });
    }
    working_state.insert(
        path.key.clone(),
        PathState {
            actual_path: path.actual_path.clone(),
            contents: Some(operation.content.clone().into_bytes()),
        },
    );
    touch_path_context(
        path_contexts,
        &path,
        PathMutationContext {
            operation_index,
            source_line: Some(operation.authored_header_line),
            source_column: Some(1),
            target_path: Some(path.actual_path.clone()),
            target_anchor: None,
        },
    );
    Ok(())
}

fn apply_delete_operation(
    operation_index: usize,
    operation: &DeleteFileOperation,
    base_dir: &Path,
    working_state: &mut PathStateMap,
    path_contexts: &mut HashMap<PathIdentityKey, PathMutationContext>,
) -> Result<(), FailureDraft> {
    let path = resolve_program_path(base_dir, &operation.path);
    if current_contents(working_state, &path).is_none() {
        return Err(target_state_failure(
            "REWRITE_TARGET_STATE_INVALID",
            "delete target does not exist",
            operation_index,
            Some(operation.authored_header_line),
            Some(path.actual_path.clone()),
            None,
            "repair the target path and retry the same rewrite program",
        ));
    }
    working_state.insert(
        path.key.clone(),
        PathState {
            actual_path: path.actual_path.clone(),
            contents: None,
        },
    );
    touch_path_context(
        path_contexts,
        &path,
        PathMutationContext {
            operation_index,
            source_line: Some(operation.authored_header_line),
            source_column: Some(1),
            target_path: Some(path.actual_path.clone()),
            target_anchor: None,
        },
    );
    Ok(())
}

fn apply_update_operation(
    operation_index: usize,
    operation: &UpdateFileOperation,
    base_dir: &Path,
    working_state: &mut PathStateMap,
    warnings: &mut Vec<RewriteWarning>,
    path_contexts: &mut HashMap<PathIdentityKey, PathMutationContext>,
) -> Result<(), FailureDraft> {
    let source_path = resolve_program_path(base_dir, &operation.path);
    let original_bytes = current_contents(working_state, &source_path).ok_or_else(|| {
        target_state_failure(
            "REWRITE_TARGET_STATE_INVALID",
            "update target does not exist",
            operation_index,
            Some(operation.authored_header_line),
            Some(source_path.actual_path.clone()),
            None,
            "repair the target path and retry the same rewrite program",
        )
    })?;
    let resolved_actions = resolve_rewrite_actions(
        operation_index,
        operation,
        &source_path.actual_path,
        &original_bytes,
    )?;
    let result_bytes = compose_rewrite_bytes(&original_bytes, &resolved_actions);

    let destination = operation
        .move_path
        .as_ref()
        .map(|move_path| resolve_program_path(base_dir, move_path));
    match destination {
        Some(destination) if destination.key != source_path.key => {
            if current_contents(working_state, &destination).is_some() {
                warnings.push(RewriteWarning {
                    code: "MOVE_REPLACED_EXISTING_DESTINATION".to_string(),
                    summary: format!(
                        "move replaced existing destination: {}",
                        destination.actual_path.display()
                    ),
                    target_path: destination.actual_path.clone(),
                    help: Some(
                        "prefer `Move to` only for true rename targets; overwriting an existing destination is compatibility-only"
                            .to_string(),
                    ),
                });
            }
            let target_anchor = resolved_actions
                .first()
                .and_then(|action| selection_anchor(&destination.actual_path, &action.selection));
            working_state.insert(
                destination.key.clone(),
                PathState {
                    actual_path: destination.actual_path.clone(),
                    contents: Some(result_bytes),
                },
            );
            working_state.insert(
                source_path.key.clone(),
                PathState {
                    actual_path: source_path.actual_path.clone(),
                    contents: None,
                },
            );
            let dest_context = PathMutationContext {
                operation_index,
                source_line: Some(operation.authored_header_line),
                source_column: Some(1),
                target_path: Some(destination.actual_path.clone()),
                target_anchor,
            };
            touch_path_context(path_contexts, &destination, dest_context.clone());
            touch_path_context(
                path_contexts,
                &source_path,
                PathMutationContext {
                    target_path: Some(destination.actual_path.clone()),
                    ..dest_context
                },
            );
        }
        Some(destination) => {
            let target_anchor = resolved_actions
                .first()
                .and_then(|action| selection_anchor(&destination.actual_path, &action.selection));
            working_state.insert(
                source_path.key.clone(),
                PathState {
                    actual_path: destination.actual_path.clone(),
                    contents: Some(result_bytes),
                },
            );
            touch_path_context(
                path_contexts,
                &source_path,
                PathMutationContext {
                    operation_index,
                    source_line: Some(operation.authored_header_line),
                    source_column: Some(1),
                    target_path: Some(destination.actual_path.clone()),
                    target_anchor,
                },
            );
        }
        None => {
            let target_anchor = resolved_actions
                .first()
                .and_then(|action| selection_anchor(&source_path.actual_path, &action.selection));
            working_state.insert(
                source_path.key.clone(),
                PathState {
                    actual_path: source_path.actual_path.clone(),
                    contents: Some(result_bytes),
                },
            );
            touch_path_context(
                path_contexts,
                &source_path,
                PathMutationContext {
                    operation_index,
                    source_line: Some(operation.authored_header_line),
                    source_column: Some(1),
                    target_path: Some(source_path.actual_path.clone()),
                    target_anchor,
                },
            );
        }
    }

    Ok(())
}

fn resolve_rewrite_actions(
    operation_index: usize,
    operation: &UpdateFileOperation,
    source_path: &Path,
    original_bytes: &[u8],
) -> Result<Vec<ResolvedRewriteAction>, FailureDraft> {
    let mut resolved = Vec::with_capacity(operation.actions.len());
    for action in &operation.actions {
        let offsets = resolve_selection_offsets(&action.selection, original_bytes, "file text")
            .map_err(|error| selection_failure(operation_index, action, source_path, error))?;
        resolved.push(ResolvedRewriteAction {
            authored_selection_line: action.authored_selection_line,
            selection: action.selection.clone(),
            offsets,
            replacement: action
                .replacement_text
                .clone()
                .unwrap_or_default()
                .into_bytes(),
        });
    }
    resolved.sort_by_key(|action| {
        (
            action.offsets.start_offset,
            action.offsets.end_offset,
            action.authored_selection_line,
        )
    });
    for pair in resolved.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        if right.offsets.start_offset < left.offsets.end_offset {
            return Err(FailureDraft {
                code: "REWRITE_SELECTION_OVERLAP".to_string(),
                summary: "rewrite selections overlap against one original snapshot".to_string(),
                operation_index,
                source_line: Some(right.authored_selection_line),
                source_column: Some(1),
                target_path: Some(source_path.to_path_buf()),
                target_anchor: selection_anchor(source_path, &right.selection),
                help: Some(
                    "rewrite the overlapping selections so each action targets a disjoint original range"
                        .to_string(),
                ),
                message: "rewrite selections overlap against one original snapshot".to_string(),
            });
        }
    }
    Ok(resolved)
}

fn compose_rewrite_bytes(original: &[u8], actions: &[ResolvedRewriteAction]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut cursor = 0usize;
    for action in actions {
        result.extend_from_slice(&original[cursor..action.offsets.start_offset]);
        let replacement = normalize_replacement_payload_for_result_suffix(
            action.replacement.clone(),
            original,
            action.offsets.end_offset,
        );
        result.extend_from_slice(&replacement);
        cursor = action.offsets.end_offset;
    }
    result.extend_from_slice(&original[cursor..]);
    result
}

fn normalize_move_rewrite_accounting(
    program: &RewriteProgram,
    unit: &ConnectedUnit,
    base_dir: &Path,
    mut affected: AffectedPaths,
) -> AffectedPaths {
    for operation_index in &unit.operation_indices {
        let RewriteFileOperation::Update(operation) = &program.operations[*operation_index] else {
            continue;
        };
        let Some(move_path) = &operation.move_path else {
            continue;
        };
        let source_path = resolve_program_path(base_dir, &operation.path);
        let destination_path = resolve_program_path(base_dir, move_path);
        if source_path.key == destination_path.key {
            continue;
        }

        if remove_affected_path(&mut affected.modified, &destination_path.actual_path)
            || affected.added.contains(&destination_path.actual_path)
        {
            push_affected_path(&mut affected.added, destination_path.actual_path.clone());
        }
        if remove_affected_path(&mut affected.modified, &source_path.actual_path)
            || affected.deleted.contains(&source_path.actual_path)
        {
            push_affected_path(&mut affected.deleted, source_path.actual_path.clone());
        }
    }
    affected
}

fn remove_affected_path(paths: &mut Vec<PathBuf>, target: &Path) -> bool {
    let original_len = paths.len();
    paths.retain(|path| path != target);
    paths.len() != original_len
}

fn push_affected_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn selection_failure(
    operation_index: usize,
    action: &RewriteAction,
    source_path: &Path,
    error: SelectionResolveError,
) -> FailureDraft {
    let source_line = error
        .item_index()
        .and_then(|item_index| action.selection.authored_line_for_item_index(item_index))
        .or(action.selection.authored_header_line)
        .or(Some(action.authored_selection_line));
    FailureDraft {
        code: error.code().rewrite_code().to_string(),
        summary: match error.code() {
            SelectionDiagnosticCode::Invalid => {
                "rewrite selection did not resolve against the current file text".to_string()
            }
            SelectionDiagnosticCode::Truncated => {
                "rewrite selection contains forbidden truncation or omission syntax".to_string()
            }
        },
        operation_index,
        source_line,
        source_column: Some(1),
        target_path: Some(source_path.to_path_buf()),
        target_anchor: selection_anchor(source_path, &action.selection),
        help: Some(
            "re-read the numbered excerpt against the current file text and regenerate the rewrite"
                .to_string(),
        ),
        message: format!("invalid rewrite selection: {}", error.message()),
    }
}

fn target_state_failure(
    code: &str,
    summary: &str,
    operation_index: usize,
    source_line: Option<usize>,
    target_path: Option<PathBuf>,
    target_anchor: Option<RewriteDiagnosticTargetAnchor>,
    help: &str,
) -> FailureDraft {
    FailureDraft {
        code: code.to_string(),
        summary: summary.to_string(),
        operation_index,
        source_line,
        source_column: Some(1),
        target_path,
        target_anchor,
        help: Some(help.to_string()),
        message: summary.to_string(),
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
                    code: "REWRITE_TARGET_STATE_INVALID".to_string(),
                    summary: format!(
                        "target path could not be read: {}",
                        path.actual_path.display()
                    ),
                    operation_index: 0,
                    source_line: None,
                    source_column: Some(1),
                    target_path: Some(path.actual_path.clone()),
                    target_anchor: None,
                    help: Some(
                        "repair the referenced path and retry the same rewrite program".to_string(),
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

fn commit_unit(unit: &ConnectedUnit, plan: &UnitPlan) -> Result<AffectedPaths, RewriteFailedUnit> {
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
                code: "REWRITE_WRITE_ERROR".to_string(),
                summary: format!("failed to write {}", error.actual_path.display()),
                operation_index: changed.context.operation_index,
                source_line: changed.context.source_line,
                source_column: changed.context.source_column,
                target_path: changed
                    .context
                    .target_path
                    .clone()
                    .or_else(|| Some(error.actual_path.clone())),
                target_anchor: changed.context.target_anchor.clone(),
                help: Some(
                    "repair the failing filesystem path and rerun the same rewrite program"
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
                operation_index: 0,
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

fn failed_unit_from_draft(
    unit: &ConnectedUnit,
    draft: FailureDraft,
    attempted: AffectedPaths,
    committed: AffectedPaths,
) -> RewriteFailedUnit {
    RewriteFailedUnit {
        touched_paths: unit
            .touched_paths
            .iter()
            .map(|path| path.actual_path.clone())
            .collect(),
        attempted,
        committed,
        code: draft.code,
        summary: draft.summary,
        target_path: draft.target_path,
        operation_index: Some(draft.operation_index + 1),
        source_line: draft.source_line,
        source_column: draft.source_column,
        target_anchor: draft.target_anchor,
        help: draft.help,
        message: draft.message,
    }
}

fn current_contents(state: &PathStateMap, path: &RuntimePath) -> Option<Vec<u8>> {
    state
        .get(&path.key)
        .and_then(|state| state.contents.clone())
}

fn touch_path_context(
    contexts: &mut HashMap<PathIdentityKey, PathMutationContext>,
    path: &RuntimePath,
    context: PathMutationContext,
) {
    contexts
        .entry(path.key.clone())
        .and_modify(|existing| {
            if context.operation_index < existing.operation_index {
                *existing = context.clone();
            }
        })
        .or_insert(context);
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

fn selection_anchor(
    path: &Path,
    selection: &SelectionBlock,
) -> Option<RewriteDiagnosticTargetAnchor> {
    selection.items.iter().find_map(|item| match item {
        SelectionItem::Line(SelectionLine {
            line_number,
            visible_content,
        }) => Some(RewriteDiagnosticTargetAnchor {
            path: path.to_path_buf(),
            line_number: *line_number,
            column_number: 1,
            label: Some("target anchor".to_string()),
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
