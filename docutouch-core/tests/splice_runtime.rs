use docutouch_core::{
    SpliceWorkspaceRequirement, apply_splice_program, splice_workspace_requirement,
};

fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).expect("read file")
}

#[test]
fn splice_workspace_requirement_requires_workspace_for_relative_paths() {
    let requirement = splice_workspace_requirement(
        "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Append To File: dest.txt\n*** End Splice\n",
    );

    assert_eq!(requirement, SpliceWorkspaceRequirement::NeedsWorkspace);
}

#[test]
fn splice_workspace_requirement_uses_absolute_anchor_for_absolute_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = dir.path().join("source.txt");
    let dest = dir.path().join("dest.txt");
    let requirement = splice_workspace_requirement(&format!(
        "*** Begin Splice\n*** Copy From File: {}\n@@\n1 | alpha\n*** Append To File: {}\n*** End Splice\n",
        source.display(),
        dest.display()
    ));

    assert_eq!(
        requirement,
        SpliceWorkspaceRequirement::AbsoluteOnly {
            anchor_dir: dir.path().to_path_buf()
        }
    );
}

#[test]
fn splice_workspace_requirement_is_unanchored_when_splice_does_not_parse() {
    let requirement = splice_workspace_requirement("*** Begin Splice\n");

    assert_eq!(requirement, SpliceWorkspaceRequirement::Unanchored);
}

#[test]
fn splice_runtime_copy_append_creates_destination_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\nbeta\n").expect("write source");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
2 | beta
*** Append To File: dest.txt
*** End Splice
";

    let outcome = apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("source.txt")), "alpha\nbeta\n");
    assert_eq!(read(&temp.path().join("dest.txt")), "alpha\nbeta\n");
    assert_eq!(outcome.affected.added, vec![temp.path().join("dest.txt")]);
}

#[test]
fn splice_runtime_move_append_removes_source_range() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\nbeta\ngamma\n").expect("write source");
    std::fs::write(temp.path().join("dest.txt"), "tail\n").expect("write dest");

    let program = "*** Begin Splice
*** Move From File: source.txt
@@
2 | beta
*** Append To File: dest.txt
*** End Splice
";

    let outcome = apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("source.txt")), "alpha\ngamma\n");
    assert_eq!(read(&temp.path().join("dest.txt")), "tail\nbeta\n");
    assert_eq!(outcome.affected.modified.len(), 2);
}

#[test]
fn splice_runtime_copy_insert_before_uses_target_range_start() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("write source");
    std::fs::write(temp.path().join("dest.txt"), "one\ntwo\nthree\n").expect("write dest");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
*** Insert Before In File: dest.txt
@@
2 | two
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(
        read(&temp.path().join("dest.txt")),
        "one\nalpha\ntwo\nthree\n"
    );
}

#[test]
fn splice_runtime_copy_insert_after_uses_target_range_end() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("write source");
    std::fs::write(temp.path().join("dest.txt"), "one\ntwo\nthree\n").expect("write dest");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
*** Insert After In File: dest.txt
@@
2 | two
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(
        read(&temp.path().join("dest.txt")),
        "one\ntwo\nalpha\nthree\n"
    );
}

#[test]
fn splice_runtime_move_replace_replaces_target_and_removes_source() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\nbeta\n").expect("write source");
    std::fs::write(temp.path().join("dest.txt"), "one\ntwo\nthree\n").expect("write dest");

    let program = "*** Begin Splice
*** Move From File: source.txt
@@
1 | alpha
2 | beta
*** Replace In File: dest.txt
@@
2 | two
3 | three
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("source.txt")), "");
    assert_eq!(read(&temp.path().join("dest.txt")), "one\nalpha\nbeta\n");
}

#[test]
fn splice_runtime_delete_span_removes_selected_bytes() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\nbeta\ngamma\n").expect("write source");

    let program = "*** Begin Splice
*** Delete Span From File: source.txt
@@
2 | beta
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("source.txt")), "alpha\ngamma\n");
}

#[test]
fn splice_runtime_rejects_same_file_overlap_for_anchored_action() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("note.txt"), "alpha\nbeta\ngamma\n").expect("write note");

    let program = "*** Begin Splice
*** Copy From File: note.txt
@@
2 | beta
*** Insert Before In File: note.txt
@@
2 | beta
*** End Splice
";

    let error = apply_splice_program(program, temp.path()).expect_err("overlap must fail");
    assert!(error.message().contains("overlap"));
}

#[test]
fn splice_runtime_rejects_missing_anchored_target() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("write source");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
*** Insert Before In File: missing.txt
@@
1 | alpha
*** End Splice
";

    let error = apply_splice_program(program, temp.path()).expect_err("missing target must fail");
    assert!(error.message().contains("target file does not exist"));
}

#[test]
fn splice_runtime_reports_missing_source_file_with_source_state_code() {
    let temp = tempfile::tempdir().expect("tempdir");

    let program = "*** Begin Splice
*** Copy From File: missing.txt
@@
1 | alpha
*** Append To File: dest.txt
*** End Splice
";

    let error = apply_splice_program(program, temp.path()).expect_err("missing source must fail");
    assert_eq!(error.details().error_code, "SPLICE_SOURCE_STATE_INVALID");
    assert!(error.message().contains("source file does not exist"));
    assert!(!temp.path().join("dest.txt").exists());
}

#[test]
fn splice_runtime_preserves_source_bytes_and_newlines_when_copying() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), b"alpha\r\nbeta\r\n")
        .expect("write source bytes");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
2 | beta
*** Append To File: dest.txt
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(
        std::fs::read(temp.path().join("dest.txt")).expect("read dest bytes"),
        b"alpha\r\nbeta\r\n"
    );
}

#[test]
fn splice_runtime_copy_omission_backed_source_selection_copies_full_byte_interval() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        temp.path().join("source.txt"),
        b"alpha\r\nbeta\r\ngamma\r\ndelta\r\n",
    )
    .expect("write source bytes");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
... source lines omitted ...
3 | gamma
*** Append To File: dest.txt
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(
        std::fs::read(temp.path().join("dest.txt")).expect("read dest bytes"),
        b"alpha\r\nbeta\r\ngamma\r\n"
    );
}

#[test]
fn splice_runtime_replace_omission_backed_target_selection_spans_full_interval() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("write source");
    std::fs::write(temp.path().join("dest.txt"), "one\ntwo\nthree\nfour\n").expect("write dest");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha
*** Replace In File: dest.txt
@@
2 | two
... target lines omitted ...
4 | four
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("dest.txt")), "one\nalpha\n");
}

#[test]
fn splice_runtime_allows_multiple_actions_that_touch_the_same_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("note.txt"), "alpha\nbeta\ngamma\n").expect("write note");

    let program = "*** Begin Splice
*** Copy From File: note.txt
@@
1 | alpha
*** Insert After In File: note.txt
@@
3 | gamma
*** Delete Span From File: note.txt
@@
2 | beta
*** End Splice
";

    apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("note.txt")), "alpha\ngamma\nalpha\n");
}

#[test]
fn splice_runtime_supports_non_overlapping_same_file_move() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("note.txt"), "alpha\nbeta\ngamma\n").expect("write note");

    let program = "*** Begin Splice
*** Move From File: note.txt
@@
1 | alpha
*** Insert After In File: note.txt
@@
3 | gamma
*** End Splice
";

    let outcome = apply_splice_program(program, temp.path()).expect("runtime should succeed");
    assert_eq!(read(&temp.path().join("note.txt")), "beta\ngamma\nalpha\n");
    assert_eq!(
        outcome.affected.modified,
        vec![temp.path().join("note.txt")]
    );
}

#[test]
fn splice_runtime_rejects_intermediate_state_dependent_target_interpretation() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("note.txt"), "alpha\nbeta\n").expect("write note");

    let program = "*** Begin Splice
*** Copy From File: note.txt
@@
1 | alpha
*** Insert Before In File: note.txt
@@
2 | beta
*** Copy From File: note.txt
@@
1 | alpha
*** Replace In File: note.txt
@@
2 | alpha
*** End Splice
";

    let error = apply_splice_program(program, temp.path())
        .expect_err("intermediate-state-dependent target interpretation must fail");
    assert!(
        error.message().contains("invalid target selection")
            || error.message().contains("does not match the file text")
    );
}

#[test]
fn splice_runtime_reports_partial_success_for_disjoint_units() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source-a.txt"), "alpha\n").expect("write source a");
    std::fs::write(temp.path().join("source-b.txt"), "beta\n").expect("write source b");

    let program = "*** Begin Splice
*** Copy From File: source-a.txt
@@
1 | alpha
*** Append To File: dest-a.txt
*** Copy From File: source-b.txt
@@
1 | beta
*** Insert Before In File: missing.txt
@@
1 | beta
*** End Splice
";

    let error = apply_splice_program(program, temp.path()).expect_err("second unit must fail");
    assert_eq!(read(&temp.path().join("dest-a.txt")), "alpha\n");
    assert_eq!(error.details().error_code, "SPLICE_PARTIAL_UNIT_FAILURE");
    assert_eq!(error.affected().added, vec![temp.path().join("dest-a.txt")]);
    assert_eq!(error.failed_units().len(), 1);
    assert_eq!(error.failed_units()[0].code, "SPLICE_TARGET_STATE_INVALID");
}

#[test]
fn splice_runtime_rejects_alias_group_partial_commit() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("note.txt"), "alpha\n").expect("write note");

    let program = "*** Begin Splice
*** Copy From File: note.txt
@@
1 | alpha
*** Append To File: mirror.txt
*** Copy From File: sub/../note.txt
@@
1 | alpha
*** Insert Before In File: missing.txt
@@
1 | alpha
*** End Splice
";

    let error =
        apply_splice_program(program, temp.path()).expect_err("connected unit must fail together");
    assert!(!temp.path().join("mirror.txt").exists());
    assert!(error.affected().added.is_empty());
    assert_eq!(error.failed_units().len(), 1);
}

#[test]
fn splice_runtime_reports_stable_truncation_code() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("write source");

    let program = "*** Begin Splice
*** Copy From File: source.txt
@@
1 | alpha...[12 chars omitted]
*** Append To File: dest.txt
*** End Splice
";

    let error = apply_splice_program(program, temp.path()).expect_err("truncation must fail");
    assert_eq!(error.details().error_code, "SPLICE_SELECTION_TRUNCATED");
    assert_eq!(error.details().source_line, Some(4));
}

#[test]
fn splice_runtime_groups_alias_paths_into_one_connected_unit() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("note.txt"), "alpha\nbeta\n").expect("write note");

    let program = "*** Begin Splice
*** Copy From File: sub/../note.txt
@@
1 | alpha
*** Append To File: note.txt
*** Copy From File: note.txt
@@
1 | alpha
*** Replace In File: sub/../note.txt
@@
2 | alpha
*** End Splice
";

    let error = apply_splice_program(program, temp.path())
        .expect_err("alias-grouped unit must fail atomically");
    assert_eq!(
        error.details().error_code,
        "SPLICE_TARGET_SELECTION_INVALID"
    );
    assert!(error.affected().added.is_empty());
    assert_eq!(read(&temp.path().join("note.txt")), "alpha\nbeta\n");
}

#[test]
fn splice_runtime_reports_partial_success_across_disjoint_units() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source-a.txt"), "alpha\n").expect("write source a");
    std::fs::write(temp.path().join("source-b.txt"), "beta\n").expect("write source b");

    let program = "*** Begin Splice
*** Copy From File: source-a.txt
@@
1 | alpha
*** Append To File: dest-a.txt
*** Copy From File: source-b.txt
@@
1 | beta
*** Insert Before In File: missing.txt
@@
1 | beta
*** End Splice
";

    let error = apply_splice_program(program, temp.path())
        .expect_err("disjoint failure should preserve earlier success");
    assert_eq!(error.details().error_code, "SPLICE_PARTIAL_UNIT_FAILURE");
    assert_eq!(error.affected().added, vec![temp.path().join("dest-a.txt")]);
    assert_eq!(read(&temp.path().join("dest-a.txt")), "alpha\n");
    assert_eq!(error.failed_units().len(), 1);
    assert_eq!(error.failed_units()[0].code, "SPLICE_TARGET_STATE_INVALID");
}

#[test]
fn splice_runtime_reports_write_failure_without_committing_failed_unit() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("source-a.txt"), "alpha\n").expect("write source a");
    std::fs::write(temp.path().join("source-b.txt"), "beta\n").expect("write source b");
    std::fs::write(temp.path().join("blocked"), "not a directory\n").expect("write blocker");

    let program = "*** Begin Splice
*** Copy From File: source-a.txt
@@
1 | alpha
*** Append To File: dest-a.txt
*** Copy From File: source-b.txt
@@
1 | beta
*** Append To File: blocked/out.txt
*** End Splice
";

    let error = apply_splice_program(program, temp.path())
        .expect_err("write failure should surface as partial unit failure");
    assert_eq!(error.details().error_code, "SPLICE_PARTIAL_UNIT_FAILURE");
    assert_eq!(error.affected().added, vec![temp.path().join("dest-a.txt")]);
    assert_eq!(read(&temp.path().join("dest-a.txt")), "alpha\n");
    assert_eq!(error.failed_units().len(), 1);
    assert_eq!(error.failed_units()[0].code, "SPLICE_WRITE_ERROR");
    assert!(error.failed_units()[0].committed.added.is_empty());
    assert!(!temp.path().join("blocked").join("out.txt").exists());
}
