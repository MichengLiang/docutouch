use docutouch_core::{
    SelectionItem, SelectionSide, SpliceAction, TargetAction, TransferSourceKind,
    parse_selection_block, parse_splice_program,
};

#[test]
fn parse_splice_program_accepts_begin_and_end_envelope() {
    let program = parse_splice_program(
        "*** Begin Splice\n*** Copy From File: source.py\n@@\n1 | alpha\n*** Append To File: target.py\n*** End Splice\n",
    )
    .expect("valid splice program");

    assert_eq!(program.actions.len(), 1);
    match &program.actions[0] {
        SpliceAction::Transfer(action) => {
            assert_eq!(action.source_kind, TransferSourceKind::Copy);
            assert_eq!(action.source_path, "source.py");
            assert_eq!(action.source_selection.items.len(), 1);
            assert_eq!(
                action.target,
                TargetAction::Append {
                    path: "target.py".to_string()
                }
            );
        }
        other => panic!("expected transfer action, got {other:?}"),
    }
}

#[test]
fn parse_splice_program_accepts_all_supported_action_headers() {
    let program = parse_splice_program(
        "*** Begin Splice\n*** Copy From File: source.py\n@@\n1 | alpha\n*** Insert Before In File: target.py\n@@\n4 | beta\n*** Move From File: other.py\n@@\n2 | gamma\n*** Replace In File: third.py\n@@\n8 | delta\n*** Delete Span From File: remove.py\n@@\n10 | obsolete\n*** End Splice\n",
    )
    .expect("valid multi-action program");

    assert_eq!(program.actions.len(), 3);
    assert!(matches!(
        program.actions[0],
        SpliceAction::Transfer(ref action)
            if action.source_kind == TransferSourceKind::Copy
                && matches!(action.target, TargetAction::InsertBefore { .. })
    ));
    assert!(matches!(
        program.actions[1],
        SpliceAction::Transfer(ref action)
            if action.source_kind == TransferSourceKind::Move
                && matches!(action.target, TargetAction::Replace { .. })
    ));
    assert!(matches!(program.actions[2], SpliceAction::Delete(_)));
}

#[test]
fn parse_selection_block_accepts_source_and_target_omission_tokens() {
    let source = parse_selection_block(
        SelectionSide::Source,
        &["1 | alpha", "... source lines omitted ...", "3 | gamma"],
    )
    .expect("valid source omission");
    assert_eq!(source.items.len(), 3);
    assert!(matches!(source.items[1], SelectionItem::Omission));

    let target = parse_selection_block(
        SelectionSide::Target,
        &["7 | start", "... target lines omitted ...", "11 | end"],
    )
    .expect("valid target omission");
    assert_eq!(target.items.len(), 3);
    assert!(matches!(target.items[1], SelectionItem::Omission));
}

#[test]
fn parse_selection_block_rejects_horizontal_truncation_marker() {
    let error = parse_selection_block(SelectionSide::Source, &["1 | alpha...[12 chars omitted]"])
        .expect_err("horizontal truncation must be rejected");
    assert!(
        error
            .message()
            .contains("forbidden horizontal truncation marker")
    );
}

#[test]
fn parse_selection_block_rejects_malformed_numbering() {
    let duplicate = parse_selection_block(SelectionSide::Source, &["1 | alpha", "1 | beta"])
        .expect_err("duplicate numbering must fail");
    assert!(
        duplicate
            .message()
            .contains("non-contiguous numbered lines require an omission token")
            || duplicate.message().contains("strictly increasing")
    );

    let descending = parse_selection_block(SelectionSide::Source, &["3 | gamma", "2 | beta"])
        .expect_err("descending numbering must fail");
    assert!(
        descending
            .message()
            .contains("non-contiguous numbered lines require an omission token")
            || descending.message().contains("strictly increasing")
    );

    let leading_zero = parse_selection_block(SelectionSide::Source, &["01 | alpha"])
        .expect_err("leading zeros must fail");
    assert!(leading_zero.message().contains("leading zero"));
}

#[test]
fn parse_selection_block_rejects_malformed_or_wrong_omission() {
    let bare = parse_selection_block(SelectionSide::Source, &["1 | alpha", "...", "3 | gamma"])
        .expect_err("bare omission must fail");
    assert!(bare.message().contains("malformed omission token"));

    let wrong_side = parse_selection_block(
        SelectionSide::Target,
        &["1 | alpha", "... source lines omitted ...", "3 | gamma"],
    )
    .expect_err("wrong-side omission must fail");
    assert!(wrong_side.message().contains("wrong omission token"));

    let empty_gap = parse_selection_block(
        SelectionSide::Source,
        &["1 | alpha", "... source lines omitted ...", "2 | beta"],
    )
    .expect_err("empty omission must fail");
    assert!(
        empty_gap
            .message()
            .contains("must denote at least one omitted line")
    );
}
