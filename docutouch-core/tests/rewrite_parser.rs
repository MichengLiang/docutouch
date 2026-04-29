use docutouch_core::{
    RewriteFileOperation, SelectionItem, SelectionSide, extract_rewrite_paths,
    parse_rewrite_program,
};
use std::path::PathBuf;

#[test]
fn parse_rewrite_program_accepts_add_delete_and_update_operations() {
    let program = parse_rewrite_program(
        "*** Begin Rewrite\n*** Add File: created.txt\n*** With\nhello\n*** End With\n*** Delete File: old.txt\n*** Update File: app.py\n*** Move to: app2.py\n@@\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n",
    )
    .expect("valid rewrite program");

    assert_eq!(program.operations.len(), 3);
    match &program.operations[0] {
        RewriteFileOperation::Add(operation) => {
            assert_eq!(operation.path, "created.txt");
            assert_eq!(operation.content, "hello");
        }
        other => panic!("expected add operation, got {other:?}"),
    }
    assert!(matches!(
        program.operations[1],
        RewriteFileOperation::Delete(_)
    ));
    match &program.operations[2] {
        RewriteFileOperation::Update(operation) => {
            assert_eq!(operation.path, "app.py");
            assert_eq!(operation.move_path.as_deref(), Some("app2.py"));
            assert_eq!(operation.actions.len(), 1);
            assert_eq!(
                operation.actions[0].replacement_text.as_deref(),
                Some("new")
            );
        }
        other => panic!("expected update operation, got {other:?}"),
    }
}

#[test]
fn parse_rewrite_program_accepts_delete_action_and_rewrite_omission_token() {
    let program = parse_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.py\n@@\n1 | alpha\n... lines omitted ...\n3 | gamma\n*** Delete\n*** End Rewrite\n",
    )
    .expect("valid rewrite program");

    match &program.operations[0] {
        RewriteFileOperation::Update(operation) => {
            assert_eq!(operation.actions.len(), 1);
            assert_eq!(operation.actions[0].replacement_text, None);
            assert_eq!(operation.actions[0].selection.side, SelectionSide::Rewrite);
            assert!(matches!(
                operation.actions[0].selection.items[1],
                SelectionItem::Omission
            ));
        }
        other => panic!("expected update operation, got {other:?}"),
    }
}

#[test]
fn parse_rewrite_program_accepts_same_line_selection_intent_comment() {
    let program = parse_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.py\n@@ replace the selected line\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n",
    )
    .expect("valid rewrite program");

    match &program.operations[0] {
        RewriteFileOperation::Update(operation) => {
            assert_eq!(operation.actions.len(), 1);
            assert_eq!(
                operation.actions[0].selection_intent_comment.as_deref(),
                Some("replace the selected line")
            );
            assert_eq!(
                operation.actions[0].replacement_text.as_deref(),
                Some("new")
            );
        }
        other => panic!("expected update operation, got {other:?}"),
    }
}

#[test]
fn parse_rewrite_program_accepts_bare_selection_header() {
    let program = parse_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.py\n@@\n1 | old\n*** Delete\n*** End Rewrite\n",
    )
    .expect("valid rewrite program");

    match &program.operations[0] {
        RewriteFileOperation::Update(operation) => {
            assert_eq!(operation.actions.len(), 1);
            assert_eq!(operation.actions[0].selection_intent_comment, None);
        }
        other => panic!("expected update operation, got {other:?}"),
    }
}

#[test]
fn parse_rewrite_program_rejects_comment_only_selection_header_without_body() {
    let error = parse_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.py\n@@ remove the old block\n*** Delete\n*** End Rewrite\n",
    )
    .expect_err("selection comment must not replace numbered body");

    assert_eq!(error.code(), "REWRITE_SELECTION_INVALID");
    assert_eq!(error.source_line(), Some(3));
    assert!(
        error
            .message()
            .contains("selection body must contain at least one numbered line")
    );
}

#[test]
fn parse_rewrite_program_rejects_multiline_selection_comment_shape() {
    let error = parse_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.py\n@@ remove the old block\nbecause startup changed\n1 | old\n*** Delete\n*** End Rewrite\n",
    )
    .expect_err("selection comment must stay on the header line");

    assert_eq!(error.code(), "REWRITE_SELECTION_INVALID");
    assert_eq!(error.source_line(), Some(4));
    assert!(error.message().contains("exact `N | content` delimiter"));
}

#[test]
fn parse_rewrite_program_rejects_unclosed_with_block() {
    let error = parse_rewrite_program(
        "*** Begin Rewrite\n*** Add File: created.txt\n*** With\nhello\n*** End Rewrite\n",
    )
    .expect_err("unclosed with block must fail");

    assert_eq!(error.code(), "REWRITE_PROGRAM_INVALID");
    assert_eq!(error.source_line(), Some(3));
    assert!(error.message().contains("with block must end"));
}

#[test]
fn parse_rewrite_program_rejects_wrong_omission_token() {
    let error = parse_rewrite_program(
        "*** Begin Rewrite\n*** Update File: app.py\n@@\n1 | alpha\n... source lines omitted ...\n3 | gamma\n*** Delete\n*** End Rewrite\n",
    )
    .expect_err("wrong omission token must fail");

    assert_eq!(error.code(), "REWRITE_SELECTION_TRUNCATED");
    assert_eq!(error.source_line(), Some(5));
    assert!(error.message().contains("wrong omission token"));
}

#[test]
fn extract_rewrite_paths_collects_move_destinations() {
    let paths = extract_rewrite_paths(
        "*** Begin Rewrite\n*** Add File: created.txt\n*** With\nhello\n*** End With\n*** Update File: app.py\n*** Move to: app2.py\n@@\n1 | old\n*** Delete\n*** End Rewrite\n",
    )
    .expect("paths should parse");

    assert_eq!(
        paths,
        vec![
            PathBuf::from("created.txt"),
            PathBuf::from("app.py"),
            PathBuf::from("app2.py"),
        ]
    );
}
