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

    let patch =
        wrap_patch("*** Update File: app.txt\n@@ fn handler():\n-    missing = 1\n+    value = 2");
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

#[test]
fn test_apply_update_file_supports_numbered_context_anchor() {
    let path = Path::new("app.py");
    let chunks = vec![UpdateFileChunk {
        change_context: Some("4 | def handler():".to_string()),
        old_lines: vec!["    value = 1".to_string()],
        new_lines: vec!["    value = 2".to_string()],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content(
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
        path,
        &chunks,
    )
    .unwrap();

    assert_eq!(
        updated,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
    );
}

#[test]
fn test_apply_update_file_supports_duplicate_first_old_side_after_numbered_context_anchor() {
    let path = Path::new("app.py");
    let chunks = vec![UpdateFileChunk {
        change_context: Some("4 | def handler():".to_string()),
        old_lines: vec!["def handler():".to_string(), "    value = 1".to_string()],
        new_lines: vec!["def handler():".to_string(), "    value = 2".to_string()],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content(
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
        path,
        &chunks,
    )
    .unwrap();

    assert_eq!(
        updated,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
    );
}

#[test]
fn test_apply_patch_duplicate_first_old_side_does_not_fallback_to_later_match() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("app.txt");
    fs::write(&path, "anchor\nwrong\nanchor\nexpected\n").unwrap();

    let patch = wrap_patch(
        "*** Update File: app.txt\n@@ 1 | anchor\n-anchor\n-expected\n+anchor\n+replacement",
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
    assert_eq!(anchor.excerpt.as_deref(), Some("anchor"));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "anchor\nwrong\nanchor\nexpected\n"
    );
}

#[test]
fn test_apply_update_file_duplicate_first_old_side_only_triggers_on_first_old_side_line() {
    let path = Path::new("app.py");
    let chunks = vec![UpdateFileChunk {
        change_context: Some("1 | anchor".to_string()),
        old_lines: vec!["beta".to_string(), "anchor".to_string()],
        new_lines: vec!["beta".to_string(), "omega".to_string()],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content("anchor\nbeta\nanchor\n", path, &chunks).unwrap();

    assert_eq!(updated, "anchor\nbeta\nomega\n");
}

#[test]
fn test_apply_update_file_supports_dense_numbered_old_side_evidence() {
    let path = Path::new("app.py");
    let chunks = vec![UpdateFileChunk {
        change_context: None,
        old_lines: vec![
            "4 | def handler():".to_string(),
            "5 |     value = 1".to_string(),
        ],
        new_lines: vec![
            "4 | def handler():".to_string(),
            "    value = 2".to_string(),
        ],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content_with_mode(
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
        path,
        &chunks,
        NumberedEvidenceMode::Full,
    )
    .unwrap();

    assert_eq!(
        updated,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
    );
}

#[test]
fn test_apply_update_file_supports_duplicate_first_old_side_with_numbered_body_in_full_mode() {
    let path = Path::new("app.py");
    let chunks = vec![UpdateFileChunk {
        change_context: Some("4 | def handler():".to_string()),
        old_lines: vec![
            "4 | def handler():".to_string(),
            "5 |     value = 1".to_string(),
        ],
        new_lines: vec![
            "4 | def handler():".to_string(),
            "    value = 2".to_string(),
        ],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content_with_mode(
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
        path,
        &chunks,
        NumberedEvidenceMode::Full,
    )
    .unwrap();

    assert_eq!(
        updated,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
    );
}

#[test]
fn test_apply_update_file_keeps_number_like_added_line_literal_in_full_mode() {
    let path = Path::new("app.py");
    let chunks = vec![UpdateFileChunk {
        change_context: None,
        old_lines: vec!["1 | value = 1".to_string()],
        new_lines: vec!["1 | value = 1".to_string(), "2 | inserted = 2".to_string()],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content_with_mode(
        "value = 1\n",
        path,
        &chunks,
        NumberedEvidenceMode::Full,
    )
    .expect("number-like added lines should remain literal text");

    assert_eq!(updated, "value = 1\n2 | inserted = 2\n");
}

#[test]
fn test_apply_update_file_treats_dense_numbered_old_side_as_literal_by_default() {
    let path = Path::new("rendered.txt");
    let chunks = vec![UpdateFileChunk {
        change_context: None,
        old_lines: vec!["121 | value = 1".to_string()],
        new_lines: vec!["changed".to_string()],
        is_end_of_file: false,
    }];

    let updated = apply_update_file_to_content("121 | value = 1\n", path, &chunks).unwrap();

    assert_eq!(updated, "changed\n");
}

#[test]
fn test_apply_patch_in_dir_numbered_context_does_not_fallback_to_other_text_match() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("app.txt");
    fs::write(&path, "alpha\nbeta\nalpha\n").unwrap();

    let patch = wrap_patch("*** Update File: app.txt\n@@ 2 | alpha\n-beta\n+gamma");
    let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

    assert_eq!(report.status, ApplyPatchStatus::Failure);
    assert_eq!(report.failed_units[0].code, "MATCH_INVALID_CONTEXT");
    let anchor = report.failed_units[0]
        .target_anchor
        .as_ref()
        .expect("target anchor");
    assert_eq!(anchor.path, path);
    assert_eq!(anchor.line_number, 2);
    assert_eq!(anchor.excerpt.as_deref(), Some("beta"));
    assert_eq!(fs::read_to_string(&path).unwrap(), "alpha\nbeta\nalpha\n");
}

#[test]
fn test_apply_patch_in_dir_interprets_numbered_old_side_against_original_snapshot() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("lines.txt");
    fs::write(&path, "a\nb\nc\n").unwrap();

    let patch = wrap_patch("*** Update File: lines.txt\n@@\n-1 | a\n@@\n-2 | b\n+beta");
    let report =
        apply_patch_in_dir_with_mode(&patch, dir.path(), NumberedEvidenceMode::Full).unwrap();

    assert_eq!(report.status, ApplyPatchStatus::FullSuccess);
    assert_eq!(fs::read_to_string(&path).unwrap(), "beta\nc\n");
}

#[cfg(windows)]
#[test]
fn test_apply_patch_in_dir_groups_case_alias_paths_atomically_on_windows() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("Name.txt");
    fs::write(&path, "a\nb\n").unwrap();

    let patch =
        wrap_patch("*** Update File: Name.txt\n@@\n-a\n+x\n*** Update File: name.txt\n@@\n-c\n+z");
    let report = apply_patch_in_dir(&patch, dir.path()).unwrap();

    assert_eq!(report.status, ApplyPatchStatus::Failure);
    assert_eq!(report.failed_units[0].code, "MATCH_INVALID_CONTEXT");
    assert_eq!(fs::read_to_string(&path).unwrap(), "a\nb\n");
}
