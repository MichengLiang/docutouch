use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_apply_patch_scenarios() -> anyhow::Result<()> {
    let scenarios_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("scenarios");
    for scenario in fs::read_dir(scenarios_dir)? {
        let scenario = scenario?;
        let path = scenario.path();
        if path.is_dir() {
            run_apply_patch_scenario(&path)?;
        }
    }
    Ok(())
}

/// Reads a scenario directory, copies the input files to a temporary directory, runs apply-patch,
/// and asserts that the final state matches the expected state exactly.
fn run_apply_patch_scenario(dir: &Path) -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let line_ending_policies = read_line_ending_manifest(dir)?;

    // Copy the input files to the temporary directory
    let input_dir = dir.join("input");
    if input_dir.is_dir() {
        copy_dir_recursive(&input_dir, tmp.path())?;
        apply_line_ending_policies_to_dir(tmp.path(), &line_ending_policies, "input")?;
    }

    // Read the patch.txt file
    let patch = fs::read_to_string(dir.join("patch.txt"))?;

    // Run apply_patch in the temporary directory. We intentionally do not assert
    // on the exit status here; the scenarios are specified purely in terms of
    // final filesystem state, which we compare below.
    Command::new(assert_cmd::cargo::cargo_bin("apply_patch"))
        .arg(patch)
        .current_dir(tmp.path())
        .output()?;

    // Assert that the final state matches the expected state exactly
    let expected_dir = dir.join("expected");
    let mut expected_snapshot = snapshot_dir(&expected_dir)?;
    apply_line_ending_policies(&mut expected_snapshot, &line_ending_policies, "expected")?;
    let actual_snapshot = snapshot_dir(tmp.path())?;

    assert_eq!(
        actual_snapshot,
        expected_snapshot,
        "Scenario {} did not match expected final state",
        dir.display()
    );

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    File(Vec<u8>),
    Dir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineEndingPolicy {
    path: PathBuf,
    mode: LineEndingMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEndingMode {
    Crlf,
    MixedLeadingCrlf,
}

fn read_line_ending_manifest(root: &Path) -> anyhow::Result<Vec<LineEndingPolicy>> {
    let manifest_path = root.join("line-endings.txt");
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }

    let manifest = fs::read_to_string(&manifest_path)?;
    let mut policies = Vec::new();
    for (line_index, raw_line) in manifest.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(path) = parts.next() else {
            continue;
        };
        let Some(mode) = parts.next() else {
            anyhow::bail!(
                "{}:{}: missing line ending mode",
                manifest_path.display(),
                line_index + 1
            );
        };
        if parts.next().is_some() {
            anyhow::bail!(
                "{}:{}: expected '<path> <mode>'",
                manifest_path.display(),
                line_index + 1
            );
        }

        let path = PathBuf::from(path);
        ensure_relative_fixture_path(&manifest_path, line_index + 1, &path)?;
        let mode = match mode {
            "crlf" => LineEndingMode::Crlf,
            "mixed-leading-crlf" => LineEndingMode::MixedLeadingCrlf,
            _ => {
                anyhow::bail!(
                    "{}:{}: unsupported line ending mode '{}'",
                    manifest_path.display(),
                    line_index + 1,
                    mode
                );
            }
        };

        policies.push(LineEndingPolicy { path, mode });
    }

    Ok(policies)
}

fn ensure_relative_fixture_path(
    manifest_path: &Path,
    line_number: usize,
    path: &Path,
) -> anyhow::Result<()> {
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        anyhow::bail!(
            "{}:{}: fixture path must stay inside the scenario",
            manifest_path.display(),
            line_number
        );
    }

    Ok(())
}

fn apply_line_ending_policies_to_dir(
    root: &Path,
    policies: &[LineEndingPolicy],
    prefix: &str,
) -> anyhow::Result<()> {
    for (relative_path, mode) in line_ending_policies_for_prefix(policies, prefix) {
        let path = root.join(relative_path);
        let bytes = fs::read(&path)?;
        fs::write(path, materialize_line_endings(&bytes, mode))?;
    }

    Ok(())
}

fn apply_line_ending_policies(
    snapshot: &mut BTreeMap<PathBuf, Entry>,
    policies: &[LineEndingPolicy],
    prefix: &str,
) -> anyhow::Result<()> {
    for (relative_path, mode) in line_ending_policies_for_prefix(policies, prefix) {
        let Some(entry) = snapshot.get_mut(&relative_path) else {
            anyhow::bail!(
                "line ending manifest references missing fixture '{}'",
                relative_path.display()
            );
        };
        let Entry::File(bytes) = entry else {
            anyhow::bail!(
                "line ending manifest references non-file fixture '{}'",
                relative_path.display()
            );
        };
        *bytes = materialize_line_endings(bytes, mode);
    }

    Ok(())
}

fn line_ending_policies_for_prefix(
    policies: &[LineEndingPolicy],
    prefix: &str,
) -> Vec<(PathBuf, LineEndingMode)> {
    let prefix = Path::new(prefix);
    policies
        .iter()
        .filter_map(|policy| {
            policy
                .path
                .strip_prefix(prefix)
                .ok()
                .map(|path| (path.to_path_buf(), policy.mode))
        })
        .collect()
}

fn materialize_line_endings(bytes: &[u8], mode: LineEndingMode) -> Vec<u8> {
    let lf_bytes = normalize_to_lf(bytes);
    match mode {
        LineEndingMode::Crlf => lf_bytes
            .into_iter()
            .flat_map(|byte| {
                if byte == b'\n' {
                    Vec::from([b'\r', b'\n'])
                } else {
                    Vec::from([byte])
                }
            })
            .collect(),
        LineEndingMode::MixedLeadingCrlf => {
            let mut converted = Vec::with_capacity(lf_bytes.len() + 1);
            let mut replaced_first_lf = false;
            for byte in lf_bytes {
                if byte == b'\n' && !replaced_first_lf {
                    converted.extend_from_slice(b"\r\n");
                    replaced_first_lf = true;
                } else {
                    converted.push(byte);
                }
            }
            converted
        }
    }
}

fn normalize_to_lf(bytes: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' {
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 1;
            }
            normalized.push(b'\n');
        } else {
            normalized.push(bytes[index]);
        }
        index += 1;
    }
    normalized
}

fn snapshot_dir(root: &Path) -> anyhow::Result<BTreeMap<PathBuf, Entry>> {
    let mut entries = BTreeMap::new();
    if root.is_dir() {
        snapshot_dir_recursive(root, root, &mut entries)?;
    }
    Ok(entries)
}

fn snapshot_dir_recursive(
    base: &Path,
    dir: &Path,
    entries: &mut BTreeMap<PathBuf, Entry>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(stripped) = path.strip_prefix(base).ok() else {
            continue;
        };
        let rel = stripped.to_path_buf();

        // Under Buck2, files in `__srcs` are often materialized as symlinks.
        // Use `metadata()` (follows symlinks) so our fixture snapshots work
        // under both Cargo and Buck2.
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            entries.insert(rel.clone(), Entry::Dir);
            snapshot_dir_recursive(base, &path, entries)?;
        } else if metadata.is_file() {
            let contents = fs::read(&path)?;
            entries.insert(rel, Entry::File(contents));
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        // See note in `snapshot_dir_recursive` about Buck2 symlink trees.
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else if metadata.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

#[test]
fn line_ending_manifest_materializes_non_lf_fixture_bytes() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let root = tmp.path();
    let input = root.join("input");
    fs::create_dir_all(&input)?;
    fs::write(input.join("crlf.txt"), "a\nb\n")?;
    fs::write(input.join("mixed.txt"), "a\nb\nc\n")?;
    fs::write(
        root.join("line-endings.txt"),
        "input/crlf.txt crlf\ninput/mixed.txt mixed-leading-crlf\n",
    )?;

    let policies = read_line_ending_manifest(root)?;
    let mut snapshot = snapshot_dir(&input)?;
    apply_line_ending_policies(&mut snapshot, &policies, "input")?;

    assert_eq!(
        snapshot.get(Path::new("crlf.txt")),
        Some(&Entry::File(b"a\r\nb\r\n".to_vec()))
    );
    assert_eq!(
        snapshot.get(Path::new("mixed.txt")),
        Some(&Entry::File(b"a\r\nb\nc\n".to_vec()))
    );

    Ok(())
}
