use anyhow::Context;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AffectedPaths {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

impl AffectedPaths {
    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathIdentityKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPath {
    pub actual_path: PathBuf,
    pub key: PathIdentityKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedPathGroup {
    pub item_indices: Vec<usize>,
    pub touched_paths: Vec<ResolvedPath>,
}

#[derive(Debug, Clone)]
pub struct RuntimePathState<T> {
    pub actual_path: PathBuf,
    pub contents: Option<T>,
}

pub type RuntimePathMap<T> = HashMap<PathIdentityKey, RuntimePathState<T>>;
pub(crate) type StagedPathState = RuntimePathState<String>;
pub(crate) type StagedPathMap = RuntimePathMap<String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingAfterBehavior {
    TreatAsDeleted,
    TreatAsUnchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteFileChange {
    pub key: PathIdentityKey,
    pub actual_path: PathBuf,
    pub after: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteFileCommitOperation {
    ReadMetadata,
    BackupExisting,
    CreateParent,
    WriteTemp,
    InstallTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteFileCommitError {
    pub key: PathIdentityKey,
    pub actual_path: PathBuf,
    pub operation: ByteFileCommitOperation,
    pub error: String,
}

static FILE_TRANSACTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn commit_path_for_key(
    key: &PathIdentityKey,
    before: &StagedPathMap,
    after: &StagedPathMap,
) -> Option<PathBuf> {
    after
        .get(key)
        .map(|state| state.actual_path.clone())
        .or_else(|| before.get(key).map(|state| state.actual_path.clone()))
}

pub fn commit_byte_changes_atomically(
    changes: &[ByteFileChange],
) -> Result<(), ByteFileCommitError> {
    if changes.is_empty() {
        return Ok(());
    }

    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut written_targets: Vec<PathBuf> = Vec::new();

    let result = (|| -> Result<(), ByteFileCommitError> {
        for change in changes {
            if change.actual_path.exists() {
                let metadata = std::fs::metadata(&change.actual_path)
                    .with_context(|| {
                        format!(
                            "Failed to read metadata for {}",
                            change.actual_path.display()
                        )
                    })
                    .map_err(|err| ByteFileCommitError {
                        key: change.key.clone(),
                        actual_path: change.actual_path.clone(),
                        operation: ByteFileCommitOperation::ReadMetadata,
                        error: err.to_string(),
                    })?;
                if metadata.is_dir() {
                    return Err(ByteFileCommitError {
                        key: change.key.clone(),
                        actual_path: change.actual_path.clone(),
                        operation: ByteFileCommitOperation::InstallTarget,
                        error: "Is a directory".to_string(),
                    });
                }
                let backup = unique_sibling_path(&change.actual_path, "bak");
                std::fs::rename(&change.actual_path, &backup)
                    .with_context(|| {
                        format!("Failed to write file {}", change.actual_path.display())
                    })
                    .map_err(|err| ByteFileCommitError {
                        key: change.key.clone(),
                        actual_path: change.actual_path.clone(),
                        operation: ByteFileCommitOperation::BackupExisting,
                        error: err.to_string(),
                    })?;
                backups.push((change.actual_path.clone(), backup));
            }
        }

        for change in changes {
            if let Some(bytes) = &change.after {
                if let Some(parent) = change.actual_path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    std::fs::create_dir_all(parent)
                        .with_context(|| {
                            format!(
                                "Failed to create parent directories for {}",
                                change.actual_path.display()
                            )
                        })
                        .map_err(|err| ByteFileCommitError {
                            key: change.key.clone(),
                            actual_path: change.actual_path.clone(),
                            operation: ByteFileCommitOperation::CreateParent,
                            error: err.to_string(),
                        })?;
                }
                let temp_path = unique_sibling_path(&change.actual_path, "tmp");
                std::fs::write(&temp_path, bytes)
                    .with_context(|| {
                        format!("Failed to write file {}", change.actual_path.display())
                    })
                    .map_err(|err| ByteFileCommitError {
                        key: change.key.clone(),
                        actual_path: change.actual_path.clone(),
                        operation: ByteFileCommitOperation::WriteTemp,
                        error: err.to_string(),
                    })?;
                std::fs::rename(&temp_path, &change.actual_path)
                    .with_context(|| {
                        format!("Failed to write file {}", change.actual_path.display())
                    })
                    .map_err(|err| ByteFileCommitError {
                        key: change.key.clone(),
                        actual_path: change.actual_path.clone(),
                        operation: ByteFileCommitOperation::InstallTarget,
                        error: err.to_string(),
                    })?;
                written_targets.push(change.actual_path.clone());
            }
        }
        Ok(())
    })();

    if let Err(err) = result {
        for path in written_targets.iter().rev() {
            let _ = std::fs::remove_file(path);
        }
        for (original, backup) in backups.iter().rev() {
            if original.exists() {
                let _ = std::fs::remove_file(original);
            }
            let _ = std::fs::rename(backup, original);
        }
        return Err(err);
    }

    for (_original, backup) in backups {
        let _ = std::fs::remove_file(backup);
    }

    Ok(())
}

pub fn path_identity_key(path: &Path) -> PathIdentityKey {
    let normalized = normalize_patch_path(path);
    let display = normalized.to_string_lossy().replace('\\', "/");
    let identity = if cfg!(windows) {
        display.to_lowercase()
    } else {
        display
    };
    PathIdentityKey(identity)
}

pub fn normalize_patch_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(std::path::MAIN_SEPARATOR.to_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _ = normalized.pop();
            }
            std::path::Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

pub fn resolve_runtime_path(anchor_dir: &Path, path: &Path) -> (PathBuf, PathIdentityKey) {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        anchor_dir.join(path)
    };
    let actual_path = normalize_patch_path(&absolute);
    let key = path_identity_key(&actual_path);
    (actual_path, key)
}

pub fn build_connected_path_groups(path_sets: &[Vec<ResolvedPath>]) -> Vec<ConnectedPathGroup> {
    if path_sets.is_empty() {
        return Vec::new();
    }

    let mut parent: Vec<usize> = (0..path_sets.len()).collect();
    let mut first_by_path: HashMap<PathIdentityKey, usize> = HashMap::new();

    for (index, paths) in path_sets.iter().enumerate() {
        for path in paths {
            if let Some(&previous) = first_by_path.get(&path.key) {
                union_find_union(&mut parent, index, previous);
            } else {
                first_by_path.insert(path.key.clone(), index);
            }
        }
    }

    let mut grouped_indices: HashMap<usize, Vec<usize>> = HashMap::new();
    for index in 0..path_sets.len() {
        let root = union_find_find(&mut parent, index);
        grouped_indices.entry(root).or_default().push(index);
    }

    let mut groups = grouped_indices
        .into_values()
        .map(|mut indices| {
            indices.sort_unstable();
            let mut touched_paths = Vec::new();
            let mut seen = HashSet::new();
            for index in &indices {
                for path in &path_sets[*index] {
                    if seen.insert(path.key.clone()) {
                        touched_paths.push(path.clone());
                    }
                }
            }
            ConnectedPathGroup {
                item_indices: indices,
                touched_paths,
            }
        })
        .collect::<Vec<_>>();
    groups.sort_by_key(|group| group.item_indices[0]);
    groups
}

fn union_find_find(parent: &mut [usize], index: usize) -> usize {
    if parent[index] != index {
        let root = union_find_find(parent, parent[index]);
        parent[index] = root;
    }
    parent[index]
}

fn union_find_union(parent: &mut [usize], lhs: usize, rhs: usize) {
    let lhs_root = union_find_find(parent, lhs);
    let rhs_root = union_find_find(parent, rhs);
    if lhs_root != rhs_root {
        parent[rhs_root] = lhs_root;
    }
}

pub fn diff_affected_paths<T: Clone + PartialEq>(
    before: &RuntimePathMap<T>,
    after: &RuntimePathMap<T>,
    missing_after: MissingAfterBehavior,
) -> AffectedPaths {
    let mut keys = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();

    let mut affected = AffectedPaths::default();
    for key in keys {
        let before_state = before.get(&key);
        let after_state = after.get(&key);
        let before_value = before_state.and_then(|state| state.contents.clone());
        let after_value = match (after_state, missing_after) {
            (Some(state), _) => state.contents.clone(),
            (None, MissingAfterBehavior::TreatAsDeleted) => None,
            (None, MissingAfterBehavior::TreatAsUnchanged) => before_value.clone(),
        };
        if before_value == after_value {
            continue;
        }
        let path = after_state
            .map(|state| state.actual_path.clone())
            .or_else(|| before_state.map(|state| state.actual_path.clone()))
            .expect("path state must exist");
        match (before_value, after_value) {
            (None, Some(_)) => affected.added.push(path),
            (Some(_), None) => affected.deleted.push(path),
            (Some(_), Some(_)) => affected.modified.push(path),
            (None, None) => {}
        }
    }
    affected
}

pub fn extend_affected_paths(target: &mut AffectedPaths, mut incoming: AffectedPaths) {
    target.added.append(&mut incoming.added);
    target.modified.append(&mut incoming.modified);
    target.deleted.append(&mut incoming.deleted);
}

fn unique_sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let counter = FILE_TRANSACTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("target");
    let candidate = format!(".{file_name}.docutouch.{counter}.{suffix}");
    match path.parent() {
        Some(parent) => parent.join(candidate),
        None => PathBuf::from(candidate),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ByteFileChange, ConnectedPathGroup, MissingAfterBehavior, PathIdentityKey, ResolvedPath,
        RuntimePathMap, RuntimePathState, build_connected_path_groups,
        commit_byte_changes_atomically, diff_affected_paths, normalize_patch_path,
        path_identity_key, resolve_runtime_path,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn normalize_patch_path_collapses_current_and_parent_segments() {
        let path = Path::new("sub/./nested/../item.txt");
        assert_eq!(
            normalize_patch_path(path),
            PathBuf::from("sub").join("item.txt")
        );
    }

    #[test]
    fn path_identity_key_groups_normalized_aliases() {
        let lhs = path_identity_key(Path::new("sub/../item.txt"));
        let rhs = path_identity_key(Path::new("item.txt"));
        assert_eq!(lhs, rhs);
    }

    #[cfg(windows)]
    #[test]
    fn path_identity_key_groups_case_aliases_on_windows() {
        let lhs = path_identity_key(Path::new("Name.txt"));
        let rhs = path_identity_key(Path::new("name.txt"));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn resolve_runtime_path_anchors_relative_paths_before_normalizing() {
        let (actual_path, key) =
            resolve_runtime_path(Path::new("workspace"), Path::new("sub/../item.txt"));
        assert_eq!(actual_path, PathBuf::from("workspace").join("item.txt"));
        assert_eq!(key, path_identity_key(&actual_path));
    }

    #[test]
    fn diff_affected_paths_classifies_add_modify_and_delete() {
        let add_key = PathIdentityKey("add".to_string());
        let modify_key = PathIdentityKey("modify".to_string());
        let delete_key = PathIdentityKey("delete".to_string());

        let mut before = RuntimePathMap::new();
        before.insert(
            modify_key.clone(),
            RuntimePathState {
                actual_path: PathBuf::from("modify.txt"),
                contents: Some("before".to_string()),
            },
        );
        before.insert(
            delete_key.clone(),
            RuntimePathState {
                actual_path: PathBuf::from("delete.txt"),
                contents: Some("gone".to_string()),
            },
        );

        let mut after = RuntimePathMap::new();
        after.insert(
            add_key,
            RuntimePathState {
                actual_path: PathBuf::from("add.txt"),
                contents: Some("new".to_string()),
            },
        );
        after.insert(
            modify_key,
            RuntimePathState {
                actual_path: PathBuf::from("modify.txt"),
                contents: Some("after".to_string()),
            },
        );
        after.insert(
            delete_key,
            RuntimePathState {
                actual_path: PathBuf::from("delete.txt"),
                contents: None,
            },
        );

        let affected = diff_affected_paths(&before, &after, MissingAfterBehavior::TreatAsDeleted);
        assert_eq!(affected.added, vec![PathBuf::from("add.txt")]);
        assert_eq!(affected.modified, vec![PathBuf::from("modify.txt")]);
        assert_eq!(affected.deleted, vec![PathBuf::from("delete.txt")]);
    }

    #[test]
    fn diff_affected_paths_can_treat_missing_after_state_as_unchanged() {
        let key = PathIdentityKey("keep".to_string());
        let mut before = RuntimePathMap::new();
        before.insert(
            key,
            RuntimePathState {
                actual_path: PathBuf::from("keep.txt"),
                contents: Some("same".to_string()),
            },
        );

        let affected = diff_affected_paths(
            &before,
            &RuntimePathMap::new(),
            MissingAfterBehavior::TreatAsUnchanged,
        );
        assert_eq!(affected, super::AffectedPaths::default());
    }

    #[test]
    fn diff_affected_paths_can_treat_missing_after_state_as_deleted() {
        let key = PathIdentityKey("drop".to_string());
        let mut before = RuntimePathMap::new();
        before.insert(
            key,
            RuntimePathState {
                actual_path: PathBuf::from("drop.txt"),
                contents: Some("gone".to_string()),
            },
        );

        let affected = diff_affected_paths(
            &before,
            &RuntimePathMap::new(),
            MissingAfterBehavior::TreatAsDeleted,
        );
        assert_eq!(affected.deleted, vec![PathBuf::from("drop.txt")]);
    }

    #[test]
    fn build_connected_path_groups_merges_overlapping_sets_and_preserves_first_path_order() {
        let path_a = ResolvedPath {
            actual_path: PathBuf::from("a.txt"),
            key: PathIdentityKey("a".to_string()),
        };
        let path_b = ResolvedPath {
            actual_path: PathBuf::from("b.txt"),
            key: PathIdentityKey("b".to_string()),
        };
        let path_c = ResolvedPath {
            actual_path: PathBuf::from("c.txt"),
            key: PathIdentityKey("c".to_string()),
        };

        let groups = build_connected_path_groups(&[
            vec![path_a.clone()],
            vec![path_b.clone(), path_c.clone()],
            vec![path_c.clone()],
        ]);

        assert_eq!(
            groups,
            vec![
                ConnectedPathGroup {
                    item_indices: vec![0],
                    touched_paths: vec![path_a],
                },
                ConnectedPathGroup {
                    item_indices: vec![1, 2],
                    touched_paths: vec![path_b, path_c],
                },
            ]
        );
    }

    #[test]
    fn commit_byte_changes_atomically_rolls_back_earlier_writes_when_later_change_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let existing = temp.path().join("existing.txt");
        let blocked_parent = temp.path().join("blocked");
        std::fs::write(&existing, b"before\n").expect("write existing");
        std::fs::write(&blocked_parent, b"not a directory\n").expect("write blocker");

        let changes = vec![
            ByteFileChange {
                key: path_identity_key(&existing),
                actual_path: existing.clone(),
                after: Some(b"after\n".to_vec()),
            },
            ByteFileChange {
                key: path_identity_key(&blocked_parent.join("child.txt")),
                actual_path: blocked_parent.join("child.txt"),
                after: Some(b"new\n".to_vec()),
            },
        ];

        let error = commit_byte_changes_atomically(&changes).expect_err("second write should fail");
        assert_eq!(
            error.operation,
            super::ByteFileCommitOperation::CreateParent
        );
        assert_eq!(
            std::fs::read(&existing).expect("existing restored"),
            b"before\n"
        );
        assert!(!blocked_parent.join("child.txt").exists());
    }
}
