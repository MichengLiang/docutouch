use docutouch_core::{
    RewriteWorkspaceRequirement, apply_rewrite_program, rewrite_workspace_requirement,
};

fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).expect("read file")
}

#[test]
fn rewrite_workspace_requirement_requires_workspace_for_relative_paths() {
    let requirement = rewrite_workspace_requirement(
        "*** Begin Rewrite\n*** Add File: created.txt\n*** With\nhello\n*** End With\n*** End Rewrite\n",
    );

    assert_eq!(requirement, RewriteWorkspaceRequirement::NeedsWorkspace);
}

#[test]
fn rewrite_workspace_requirement_uses_absolute_anchor_for_absolute_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("created.txt");
    let requirement = rewrite_workspace_requirement(&format!(
        "*** Begin Rewrite\n*** Add File: {}\n*** With\nhello\n*** End With\n*** End Rewrite\n",
        target.display()
    ));

    assert_eq!(
        requirement,
        RewriteWorkspaceRequirement::AbsoluteOnly {
            anchor_dir: dir.path().to_path_buf()
        }
    );
}

#[test]
fn rewrite_runtime_replaces_selected_range() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "alpha\nbeta\ngamma\n").expect("write app");

    let program = "*** Begin Rewrite
*** Update File: app.txt
@@
2 | beta
*** With
BETA
*** End With
*** End Rewrite
";

    let outcome = apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(read(&temp.path().join("app.txt")), "alpha\nBETA\ngamma\n");
    assert_eq!(outcome.affected.modified, vec![temp.path().join("app.txt")]);
}

#[test]
fn rewrite_runtime_treats_selection_intent_comment_as_semantically_neutral() {
    let with_comment = "*** Begin Rewrite
*** Update File: app.txt
@@ replace the selected middle line
2 | beta
*** With
BETA
*** End With
*** End Rewrite
";
    let without_comment = "*** Begin Rewrite
*** Update File: app.txt
@@
2 | beta
*** With
BETA
*** End With
*** End Rewrite
";

    let commented_dir = tempfile::tempdir().expect("commented tempdir");
    let plain_dir = tempfile::tempdir().expect("plain tempdir");
    std::fs::write(commented_dir.path().join("app.txt"), "alpha\nbeta\ngamma\n").expect("write app");
    std::fs::write(plain_dir.path().join("app.txt"), "alpha\nbeta\ngamma\n").expect("write app");

    let commented =
        apply_rewrite_program(with_comment, commented_dir.path()).expect("commented rewrite");
    let plain = apply_rewrite_program(without_comment, plain_dir.path()).expect("plain rewrite");

    assert_eq!(read(&commented_dir.path().join("app.txt")), "alpha\nBETA\ngamma\n");
    assert_eq!(read(&plain_dir.path().join("app.txt")), "alpha\nBETA\ngamma\n");
    assert!(commented.affected.added.is_empty());
    assert!(plain.affected.added.is_empty());
    assert_eq!(commented.affected.modified.len(), 1);
    assert_eq!(plain.affected.modified.len(), 1);
    assert_eq!(
        commented.affected.modified[0].file_name(),
        plain.affected.modified[0].file_name()
    );
    assert!(commented.affected.deleted.is_empty());
    assert!(plain.affected.deleted.is_empty());
    assert_eq!(commented.warnings, plain.warnings);
}

#[test]
fn rewrite_runtime_delete_action_removes_selected_range() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "alpha\nbeta\ngamma\n").expect("write app");

    let program = "*** Begin Rewrite
*** Update File: app.txt
@@
2 | beta
*** Delete
*** End Rewrite
";

    apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(read(&temp.path().join("app.txt")), "alpha\ngamma\n");
}

#[test]
fn rewrite_runtime_add_and_delete_file_operations_work() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("obsolete.txt"), "old\n").expect("write obsolete");

    let program = "*** Begin Rewrite
*** Add File: created.txt
*** With
hello
*** End With
*** Delete File: obsolete.txt
*** End Rewrite
";

    let outcome = apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(read(&temp.path().join("created.txt")), "hello");
    assert!(!temp.path().join("obsolete.txt").exists());
    assert_eq!(
        outcome.affected.added,
        vec![temp.path().join("created.txt")]
    );
    assert_eq!(
        outcome.affected.deleted,
        vec![temp.path().join("obsolete.txt")]
    );
}

#[test]
fn rewrite_runtime_move_and_rewrite_updates_destination() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("from.txt"), "old\n").expect("write source");

    let program = "*** Begin Rewrite
*** Update File: from.txt
*** Move to: to.txt
@@
1 | old
*** With
new
*** End With
*** End Rewrite
";

    let outcome = apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert!(!temp.path().join("from.txt").exists());
    assert_eq!(read(&temp.path().join("to.txt")), "new");
    assert_eq!(outcome.affected.added, vec![temp.path().join("to.txt")]);
    assert_eq!(outcome.affected.deleted, vec![temp.path().join("from.txt")]);
}

#[test]
fn rewrite_runtime_move_and_rewrite_over_existing_destination_uses_final_path_accounting() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("from.txt"), "old\n").expect("write source");
    std::fs::write(temp.path().join("to.txt"), "dest\n").expect("write dest");

    let program = "*** Begin Rewrite
*** Update File: from.txt
*** Move to: to.txt
@@
1 | old
*** With
new
*** End With
*** End Rewrite
";

    let outcome = apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert!(!temp.path().join("from.txt").exists());
    assert_eq!(read(&temp.path().join("to.txt")), "new");
    assert_eq!(outcome.affected.added, vec![temp.path().join("to.txt")]);
    assert!(outcome.affected.modified.is_empty());
    assert_eq!(outcome.affected.deleted, vec![temp.path().join("from.txt")]);
    assert_eq!(outcome.warnings.len(), 1);
    assert_eq!(
        outcome.warnings[0].code,
        "MOVE_REPLACED_EXISTING_DESTINATION".to_string()
    );
}

#[test]
fn rewrite_runtime_add_over_existing_file_reports_warning() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "old\n").expect("write app");

    let program = "*** Begin Rewrite
*** Add File: app.txt
*** With
new
*** End With
*** End Rewrite
";

    let outcome = apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(read(&temp.path().join("app.txt")), "new");
    assert_eq!(outcome.warnings.len(), 1);
    assert_eq!(
        outcome.warnings[0].code,
        "ADD_REPLACED_EXISTING_FILE".to_string()
    );
}

#[test]
fn rewrite_runtime_with_block_preserves_result_line_boundaries_before_suffix() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "alpha\nbeta\ngamma\n").expect("write app");

    let program = "*** Begin Rewrite
*** Update File: app.txt
@@
2 | beta
*** With
BETA
*** End With
*** End Rewrite
";

    apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(read(&temp.path().join("app.txt")), "alpha\nBETA\ngamma\n");
}

#[test]
fn rewrite_runtime_with_block_preserves_authored_eof_at_file_end() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "alpha\nbeta").expect("write app");

    let program = "*** Begin Rewrite
*** Update File: app.txt
@@
2 | beta
*** With
BETA
*** End With
*** End Rewrite
";

    apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(read(&temp.path().join("app.txt")), "alpha\nBETA");
}

#[test]
fn rewrite_runtime_delete_missing_file_is_hard_failure() {
    let temp = tempfile::tempdir().expect("tempdir");

    let error = apply_rewrite_program(
        "*** Begin Rewrite\n*** Delete File: missing.txt\n*** End Rewrite\n",
        temp.path(),
    )
    .expect_err("missing delete target must fail");

    assert_eq!(
        error.details().error_code,
        "REWRITE_TARGET_STATE_INVALID".to_string()
    );
    assert_eq!(error.details().source_line, Some(2));
    assert_eq!(error.message(), "delete target does not exist");
    assert!(error.affected().added.is_empty());
    assert!(error.affected().modified.is_empty());
    assert!(error.affected().deleted.is_empty());
}

#[test]
fn rewrite_runtime_multiple_actions_use_original_snapshot_numbering() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "one\ntwo\nthree\nfour\n").expect("write app");

    let program = "*** Begin Rewrite
*** Update File: app.txt
@@
2 | two
*** With
SECOND
*** End With
@@
4 | four
*** With
FOURTH
*** End With
*** End Rewrite
";

    apply_rewrite_program(program, temp.path()).expect("rewrite should succeed");
    assert_eq!(
        read(&temp.path().join("app.txt")),
        "one\nSECOND\nthree\nFOURTH"
    );
}

#[test]
fn rewrite_runtime_rejects_overlapping_actions() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "one\ntwo\nthree\n").expect("write app");

    let error = apply_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.txt\n@@\n1 | one\n2 | two\n*** With\nA\n*** End With\n@@\n2 | two\n*** With\nB\n*** End With\n*** End Rewrite\n",
        temp.path(),
    )
    .expect_err("overlap must fail");

    assert_eq!(
        error.details().error_code,
        "REWRITE_SELECTION_OVERLAP".to_string()
    );
    assert_eq!(error.details().source_line, Some(9));
    assert_eq!(read(&temp.path().join("app.txt")), "one\ntwo\nthree\n");
}

#[test]
fn rewrite_runtime_reports_selection_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "alpha\nbeta\n").expect("write app");

    let error = apply_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.txt\n@@\n2 | wrong\n*** Delete\n*** End Rewrite\n",
        temp.path(),
    )
    .expect_err("mismatch must fail");

    assert_eq!(
        error.details().error_code,
        "REWRITE_SELECTION_INVALID".to_string()
    );
    assert_eq!(error.details().source_line, Some(4));
}

#[test]
fn rewrite_runtime_can_partially_apply_independent_units() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("app.txt"), "alpha\n").expect("write app");

    let error = apply_rewrite_program(
        "*** Begin Rewrite\n*** Add File: created.txt\n*** With\nhello\n*** End With\n*** Delete File: missing.txt\n*** End Rewrite\n",
        temp.path(),
    )
    .expect_err("independent unit failure should report partial apply");

    assert_eq!(
        error.details().error_code,
        "REWRITE_PARTIAL_UNIT_FAILURE".to_string()
    );
    assert_eq!(read(&temp.path().join("created.txt")), "hello");
}
