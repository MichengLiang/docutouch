use docutouch_core::{
    ResolvedSelection, SelectionSide, parse_selection_block, resolve_selection_block,
};

#[test]
fn resolve_selection_block_resolves_exact_contiguous_interval() {
    let selection = parse_selection_block(SelectionSide::Source, &["2 | beta", "3 | gamma"])
        .expect("selection should parse");
    let resolved = resolve_selection_block(&selection, "alpha\nbeta\ngamma\ndelta\n")
        .expect("selection should resolve");

    assert_eq!(
        resolved,
        ResolvedSelection {
            side: SelectionSide::Source,
            start_line: 2,
            end_line: 3,
            lines: vec!["beta".to_string(), "gamma".to_string()],
        }
    );
}

#[test]
fn resolve_selection_block_resolves_omission_backed_gap_to_contiguous_interval() {
    let selection = parse_selection_block(
        SelectionSide::Target,
        &["2 | beta", "... target lines omitted ...", "5 | epsilon"],
    )
    .expect("selection should parse");
    let resolved = resolve_selection_block(&selection, "alpha\nbeta\ngamma\ndelta\nepsilon\n")
        .expect("selection should resolve");

    assert_eq!(
        resolved,
        ResolvedSelection {
            side: SelectionSide::Target,
            start_line: 2,
            end_line: 5,
            lines: vec![
                "beta".to_string(),
                "gamma".to_string(),
                "delta".to_string(),
                "epsilon".to_string(),
            ],
        }
    );
}

#[test]
fn resolve_selection_block_rejects_visible_content_mismatch() {
    let selection = parse_selection_block(SelectionSide::Source, &["2 | wrong"])
        .expect("selection should parse");
    let error = resolve_selection_block(&selection, "alpha\nbeta\ngamma\n")
        .expect_err("visible-content mismatch must fail");

    assert!(error.message().contains("does not match the target text"));
}

#[test]
fn resolve_selection_block_rejects_missing_line_anchor() {
    let selection = parse_selection_block(SelectionSide::Source, &["5 | epsilon"])
        .expect("selection should parse");
    let error = resolve_selection_block(&selection, "alpha\nbeta\ngamma\n")
        .expect_err("missing anchor line must fail");

    assert!(
        error
            .message()
            .contains("does not exist in the target text")
    );
}

#[test]
fn resolve_selection_block_rejects_interval_that_runs_past_file_end() {
    let selection = parse_selection_block(
        SelectionSide::Source,
        &["2 | beta", "... source lines omitted ...", "5 | epsilon"],
    )
    .expect("selection should parse");
    let error = resolve_selection_block(&selection, "alpha\nbeta\ngamma\n")
        .expect_err("interval past file end must fail");

    assert!(
        error
            .message()
            .contains("does not exist in the target text")
            || error.message().contains("extends past the target text")
    );
}
