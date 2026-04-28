use docutouch_core::{
    StructuralSearchMode, StructuralSearchOptions, StructuralSearchSession, StructuralSearchView,
};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

fn write_file(root: &std::path::Path, path: &str, contents: &str) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().expect("parent")).expect("create parent");
    fs::write(full, contents).expect("write file");
}

fn find_options(root: &std::path::Path, pattern: &str) -> StructuralSearchOptions {
    StructuralSearchOptions {
        mode: StructuralSearchMode::Find,
        pattern: Some(pattern.to_string()),
        rule: None,
        query: Some("test query".to_string()),
        reference: None,
        search_paths: vec![root.to_path_buf()],
        display_base_dir: Some(root.to_path_buf()),
        language: Some("rust".to_string()),
        include_tests: true,
        context: Vec::new(),
        limit: Some(8),
        view: StructuralSearchView::Preview,
    }
}

fn find_options_for_path(
    root: &std::path::Path,
    path: &str,
    pattern: &str,
) -> StructuralSearchOptions {
    StructuralSearchOptions {
        search_paths: vec![root.join(path)],
        ..find_options(root, pattern)
    }
}

async fn run_rule(
    root: &std::path::Path,
    rule: serde_json::Value,
) -> (StructuralSearchSession, String) {
    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(rule),
            ..find_options(root, "unused")
        })
        .await
        .expect("rule search");
    (session, output)
}

#[tokio::test]
async fn find_outputs_pretty_text_groups_evidence_and_next() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        r#"
pub fn run() {
    evaluate_exec_policy(ctx, command);
}
"#,
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("structural search");

    assert!(output.contains("structural_search[find] q1"));
    assert!(output.contains("pattern: evaluate_exec_policy($$$ARGS)"));
    assert!(output.contains("language: rust"));
    assert!(output.contains("matches:"));
    assert!(output.contains("[1]"));
    assert!(output.contains("src/lib.rs:3"));
    assert!(output.contains("evaluate_exec_policy(ctx, command)"));
    assert!(output.contains("captures:"));
    assert!(output.contains("omitted:"));
    assert!(output.contains("next:"));
    assert!(output.contains("expand 1"));
}

#[tokio::test]
async fn default_output_is_pretty_text_not_raw_ast_grep_json() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");

    assert!(output.starts_with("structural_search[find] q1\n"));
    assert!(!output.trim_start().starts_with('['));
    assert!(!output.trim_start().starts_with('{'));
    assert!(!output.contains("\"text\""));
    assert!(!output.contains("\"range\""));
    assert!(!output.contains("\"metaVariables\""));
}

#[tokio::test]
async fn successful_queries_allocate_q_numbers_sequentially() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { first(); second(); third(); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let first = session
        .search(find_options(dir.path(), "first()"))
        .await
        .expect("first");
    let second = session
        .search(find_options(dir.path(), "second()"))
        .await
        .expect("second");
    let third = session
        .search(find_options(dir.path(), "third()"))
        .await
        .expect("third");

    assert!(first.starts_with("structural_search[find] q1\n"));
    assert!(second.starts_with("structural_search[find] q2\n"));
    assert!(third.starts_with("structural_search[find] q3\n"));
}

#[tokio::test]
async fn structural_search_sessions_are_connection_local() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );
    write_file(dir.path(), "src/other.rs", "pub fn run() { finish(); }\n");

    let mut first_connection = StructuralSearchSession::default();
    let mut second_connection = StructuralSearchSession::default();

    let first = first_connection
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("first connection find");
    let second = second_connection
        .search(find_options(dir.path(), "finish()"))
        .await
        .expect("second connection find");

    assert!(first.starts_with("structural_search[find] q1\n"));
    assert!(second.starts_with("structural_search[find] q1\n"));

    let expanded = first_connection
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("first connection expand");
    assert!(expanded.contains("structural_search[expand] q2"));
    assert!(expanded.contains("evaluate_exec_policy(ctx, command)"));
    assert!(!expanded.contains("finish()"));
}

#[tokio::test]
async fn expand_uses_recent_query_and_invalid_ref_does_not_pollute_recent_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        r#"
pub fn run() {
    evaluate_exec_policy(ctx, command);
}
"#,
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");

    let invalid = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("q99.1".to_string()),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("invalid ref is rendered");
    assert!(invalid.contains("status: invalid-ref"));
    assert!(!invalid.contains(" q2"));

    let expanded = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("expand");
    assert!(expanded.contains("structural_search[expand] q2"));
    assert!(expanded.contains("from: q1.[1]"));
    assert!(expanded.contains("captures:"));
}

#[tokio::test]
async fn expand_without_recent_query_is_invalid_ref_and_allocates_no_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let invalid = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("invalid ref");
    assert!(invalid.contains("status: invalid-ref"));
    assert!(!invalid.contains(" q1"));

    let find = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");
    assert!(find.starts_with("structural_search[find] q1\n"));
}

#[tokio::test]
async fn expand_truncates_long_captures_and_omits_extra_capture_entries() {
    let dir = tempdir().expect("tempdir");
    let long_literal = "x".repeat(160);
    write_file(
        dir.path(),
        "src/lib.rs",
        &format!("pub fn run() {{ wrap(\"{long_literal}\", second, third, fourth, fifth); }}\n"),
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "wrap($A, $B, $C, $D, $E)"))
        .await
        .expect("find");
    let expanded = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("expand");

    assert!(expanded.contains("$A = \""));
    assert!(expanded.contains("chars omitted"));
    assert!(expanded.contains("captures not shown"));
    assert!(!expanded.contains(&long_literal));
}

#[tokio::test]
async fn expand_respects_preview_limit_and_reports_hidden_matches() {
    let dir = tempdir().expect("tempdir");
    for index in 0..5 {
        write_file(
            dir.path(),
            &format!("src/policy_{index}.rs"),
            &format!("pub fn run_{index}() {{ evaluate_exec_policy(ctx, command_{index}); }}\n"),
        );
    }

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");

    let expanded = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            limit: Some(2),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("expand");

    assert!(expanded.contains("matches: 2 displayed, 5 total"));
    assert!(expanded.contains("omitted:"));
    assert!(expanded.contains("3 matches not shown"));
    assert!(!expanded.contains("[3]"));
}

#[tokio::test]
async fn full_view_is_budgeted_and_limit_above_max_is_parameter_error() {
    let dir = tempdir().expect("tempdir");
    for index in 0..70 {
        write_file(
            dir.path(),
            &format!("src/policy_{index}.rs"),
            &format!("pub fn run_{index}() {{ evaluate_exec_policy(ctx, command_{index}); }}\n"),
        );
    }

    let mut session = StructuralSearchSession::default();
    let bad_limit = session
        .search(StructuralSearchOptions {
            limit: Some(1000),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("bad limit");
    assert!(bad_limit.contains("status: parameter-error"));
    assert!(bad_limit.contains("limit must be at most"));
    assert!(!bad_limit.contains(" q1"));

    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");
    let expanded = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            view: StructuralSearchView::Full,
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("full expand");

    assert!(expanded.contains("matches: 64 displayed, 70 total"));
    assert!(expanded.contains("6 matches not shown"));
    assert!(!expanded.contains("[65]"));
}

#[tokio::test]
async fn no_matches_allocates_query_but_has_no_expandable_group() {
    let dir = tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", "pub fn run() {}\n");

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("no matches");
    assert!(output.contains("structural_search[find] q1"));
    assert!(output.contains("status: no-matches"));

    let expanded = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("expand no match");
    assert!(expanded.contains("status: invalid-ref"));
}

#[tokio::test]
async fn include_tests_false_excludes_test_files() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );
    write_file(
        dir.path(),
        "tests/policy_tests.rs",
        "fn test_run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            include_tests: false,
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("find");

    assert!(output.contains("tests excluded"));
    assert!(output.contains("src/lib.rs:1"));
    assert!(!output.contains("tests/policy_tests.rs"));
}

#[tokio::test]
async fn include_tests_false_excludes_rust_tests_module_files() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/session/mod.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );
    write_file(
        dir.path(),
        "src/session/tests.rs",
        "fn test_run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            include_tests: false,
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("find");

    assert!(output.contains("tests excluded"));
    assert!(output.contains("src/session/mod.rs:1"));
    assert!(!output.contains("src/session/tests.rs"));
}

#[tokio::test]
async fn groups_include_production_tests_fixtures_and_generated_families() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn prod() { evaluate_exec_policy(ctx, prod); }\n",
    );
    write_file(
        dir.path(),
        "tests/policy_tests.rs",
        "fn test_policy() { evaluate_exec_policy(ctx, test); }\n",
    );
    write_file(
        dir.path(),
        "fixtures/policy_fixture.rs",
        "pub fn fixture() { evaluate_exec_policy(ctx, fixture); }\n",
    );
    write_file(
        dir.path(),
        "src/generated/policy.rs",
        "pub fn generated() { evaluate_exec_policy(ctx, generated); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");

    assert!(output.contains("[1] production matches"));
    assert!(output.contains("[2] test matches"));
    assert!(output.contains("[3] fixture matches"));
    assert!(output.contains("[4] generated matches"));
}

#[tokio::test]
async fn rule_with_edit_field_is_rejected_without_allocating_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(json!({
                "id": "bad",
                "language": "Rust",
                "rule": { "pattern": "evaluate_exec_policy($$$ARGS)" },
                "fix": "replacement"
            })),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("unsupported field is rendered");

    assert!(output.contains("status: unsupported-rule-field"));
    assert!(!output.contains(" q1"));
}

#[tokio::test]
async fn inline_rule_object_find_is_supported() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(json!({
                "id": "policy-call",
                "language": "Rust",
                "rule": { "pattern": "evaluate_exec_policy($$$ARGS)" }
            })),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("inline rule");

    assert!(output.contains("structural_search[find] q1"));
    assert!(output.contains("rule: pattern"));
    assert!(output.contains("src/lib.rs:1"));
}

#[tokio::test]
async fn stringified_inline_rule_object_is_accepted() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(json!(
                r#"{"id":"policy-call","language":"Rust","rule":{"pattern":"evaluate_exec_policy($$$ARGS)"}}"#
            )),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("stringified inline rule");

    assert!(output.contains("structural_search[find] q1"));
    assert!(output.contains("rule: pattern"));
    assert!(output.contains("src/lib.rs:1"));
}

#[tokio::test]
async fn malformed_string_rule_returns_parameter_error_without_allocating_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(json!("id: policy-call\nlanguage: Rust")),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("parameter error is rendered");

    assert!(output.contains("status: parameter-error"));
    assert!(output.contains("rule must be a JSON object"));
    assert!(!output.contains(" q1"));
    assert!(!output.contains("SerializableRuleConfig"));
}

#[tokio::test]
async fn rule_summary_reports_constraints_and_utils() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(json!({
                "id": "utility-call",
                "language": "Rust",
                "utils": {
                    "isPolicyCall": { "pattern": "evaluate_exec_policy($ARG, $$$REST)" }
                },
                "rule": { "matches": "isPolicyCall" },
                "constraints": { "ARG": { "regex": "ctx" } }
            })),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("inline rule with constraints and utils");

    assert!(output.contains("rule: matches"));
    assert!(output.contains("constraints: $ARG restricted by"));
    assert!(output.contains("utils: isPolicyCall"));
    assert!(output.contains("src/lib.rs:1"));
}

#[tokio::test]
async fn rule_summary_reports_relational_and_composite_parts() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            pattern: None,
            rule: Some(json!({
                "id": "call-inside-function",
                "language": "Rust",
                "rule": {
                    "all": [
                        { "pattern": "evaluate_exec_policy($$$ARGS)" },
                        { "inside": { "kind": "function_item" } }
                    ]
                }
            })),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("relational composite rule");

    assert!(output.contains("rule: all + inside + kind + pattern"));
    assert!(output.contains("status: no-matches"));
}

#[tokio::test]
async fn invalid_pattern_returns_status_without_allocating_query() {
    let dir = tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", "pub fn run() {}\n");

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(find_options(dir.path(), "match $POLICY { $$$ARMS"))
        .await
        .expect("invalid pattern is rendered");

    assert!(output.contains("status: invalid-pattern"));
    assert!(output.contains("pattern: match $POLICY { $$$ARMS"));
    assert!(!output.contains(" q1"));
}

#[tokio::test]
async fn invalid_pattern_does_not_pollute_recent_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");
    let invalid = session
        .search(find_options(dir.path(), "match $POLICY { $$$ARMS"))
        .await
        .expect("invalid pattern");
    assert!(invalid.contains("status: invalid-pattern"));
    assert!(!invalid.contains(" q2"));

    let expanded = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("expand");
    assert!(expanded.contains("structural_search[expand] q2"));
    assert!(expanded.contains("from: q1.[1]"));
}

#[tokio::test]
async fn parse_partial_reports_missing_language_coverage_and_allocates_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );
    write_file(
        dir.path(),
        "src/app.ts",
        "function run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("parse partial");

    assert!(output.contains("structural_search[find] q1"));
    assert!(output.contains("status: parse-partial"));
    assert!(output.contains("Missing coverage:"));
    assert!(output.contains("1 files could not be parsed as rust"));
}

#[tokio::test]
async fn around_reports_source_node_and_captures() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Around,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("around");

    assert!(output.contains("structural_search[around] q2"));
    assert!(output.contains("from: q1.[1]"));
    assert!(output.contains("Enclosing"));
    assert!(output.contains("Node"));
    assert!(output.contains("Captures"));
}

#[tokio::test]
async fn around_reports_real_local_structure_and_honors_context_filter() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() {\n    prepare();\n    evaluate_exec_policy(ctx, command);\n    finish();\n}\n",
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Around,
            reference: Some("1".to_string()),
            context: vec![
                "enclosing".to_string(),
                "siblings".to_string(),
                "children".to_string(),
                "captures".to_string(),
            ],
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("around");

    assert!(output.contains("item: function_item pub fn run"));
    assert!(output.contains("kind: call_expression"));
    assert!(output.contains("range: 3:5-3:"));
    assert!(output.contains("previous: prepare();"));
    assert!(output.contains("next: finish();"));
    assert!(output.contains("arguments: ctx, command"));
    assert!(!output.contains("line_context"));
    assert!(!output.contains("backend-dependent"));

    let captures_only = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Around,
            reference: Some("q1.1".to_string()),
            context: vec!["captures".to_string()],
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("around captures only");
    assert!(captures_only.contains("Captures"));
    assert!(!captures_only.contains("Enclosing"));
    assert!(!captures_only.contains("Siblings"));
    assert!(!captures_only.contains("Children"));
}

#[tokio::test]
async fn explain_ast_reports_path_line_local_tree_surface() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::ExplainAst,
            query: Some("src/lib.rs:1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("explain");

    assert!(output.contains("structural_search[explain_ast] q1"));
    assert!(output.contains("source: src/lib.rs:1"));
    assert!(output.contains("language: rust"));
    assert!(output.contains("local tree"));
    assert!(output.contains("pub fn run()"));
}

#[tokio::test]
async fn explain_ast_reports_real_node_kind_range_tree_and_candidate_pattern() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() {\n    evaluate_exec_policy(ctx, command);\n}\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::ExplainAst,
            query: Some("src/lib.rs:2".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("explain");

    assert!(output.contains("node kind: call_expression"));
    assert!(output.contains("range: 2:5-2:"));
    assert!(output.contains("function_item"));
    assert!(output.contains("call_expression evaluate_exec_policy(ctx, command)"));
    assert!(output.contains("evaluate_exec_policy($$$ARGS)"));
    assert!(!output.contains("line_context"));
}

#[tokio::test]
async fn explain_ast_accepts_path_plus_query_line_number() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() {\n    evaluate_exec_policy(ctx, command);\n}\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::ExplainAst,
            query: Some("2".to_string()),
            ..find_options_for_path(dir.path(), "src/lib.rs", "unused")
        })
        .await
        .expect("explain path plus line");

    assert!(output.contains("source: src/lib.rs:2"));
    assert!(output.contains("node kind: call_expression"));
}

#[tokio::test]
async fn rule_test_reports_matched_status() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::RuleTest,
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("rule test");

    assert!(output.contains("structural_search[rule_test] q1"));
    assert!(output.contains("status: matched"));
    assert!(output.contains("captures:"));
}

#[tokio::test]
async fn rule_test_uses_ref_as_small_test_source() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() {\n    prepare();\n    evaluate_exec_policy(ctx, command);\n    finish();\n}\n",
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");

    let matched = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::RuleTest,
            reference: Some("q1.1".to_string()),
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("rule test ref matched");
    assert!(matched.contains("structural_search[rule_test] q2"));
    assert!(matched.contains("test source: q1.[1] src/lib.rs:3"));
    assert!(matched.contains("status: matched"));

    let no_match = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::RuleTest,
            reference: Some("q1.1".to_string()),
            ..find_options(dir.path(), "finish()")
        })
        .await
        .expect("rule test ref no match");
    assert!(no_match.contains("structural_search[rule_test] q3"));
    assert!(no_match.contains("status: no-matches"));
}

#[tokio::test]
async fn rule_test_uses_path_line_as_small_test_source() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() {\n    prepare();\n    evaluate_exec_policy(ctx, command);\n}\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::RuleTest,
            query: Some("src/lib.rs:3".to_string()),
            ..find_options(dir.path(), "prepare()")
        })
        .await
        .expect("rule test path line");

    assert!(output.contains("structural_search[rule_test] q1"));
    assert!(output.contains("test source: src/lib.rs:3"));
    assert!(output.contains("status: no-matches"));
}

#[tokio::test]
async fn unsupported_context_returns_parameter_error_without_allocating_query() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            context: vec!["captures".to_string(), "unknown_context".to_string()],
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("bad context");

    assert!(output.contains("status: parameter-error"));
    assert!(output.contains("unsupported context"));
    assert!(!output.contains(" q1"));
}

#[tokio::test]
async fn multi_language_directory_without_language_returns_ambiguous_language() {
    let dir = tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", "pub fn run() {}\n");
    write_file(dir.path(), "src/app.ts", "function run() {}\n");

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            language: None,
            ..find_options(dir.path(), "run($$$ARGS)")
        })
        .await
        .expect("ambiguous");

    assert!(output.contains("status: ambiguous-language"));
    assert!(output.contains("rust: 1 files"));
    assert!(output.contains("typescript: 1 files"));
    assert!(!output.contains(" q1"));

    let narrowed = session
        .search(StructuralSearchOptions {
            language: None,
            ..find_options_for_path(dir.path(), "src/lib.rs", "run($$$ARGS)")
        })
        .await
        .expect("narrowed language inference");
    assert!(narrowed.contains("structural_search[find] q1"));
    assert!(narrowed.contains("language: rust"));

    let explicit = session
        .search(StructuralSearchOptions {
            language: Some("typescript".to_string()),
            pattern: Some("function $NAME($$$ARGS) { $$$BODY }".to_string()),
            ..find_options_for_path(dir.path(), "src/app.ts", "unused")
        })
        .await
        .expect("explicit language");
    assert!(explicit.contains("structural_search[find] q2"));
    assert!(explicit.contains("language: typescript"));
}

#[tokio::test]
async fn typescript_and_python_fixtures_support_patterns_and_test_filtering() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "web/app.ts",
        "function run(value) { evaluateExecPolicy(value); }\n",
    );
    write_file(
        dir.path(),
        "web/app.test.ts",
        "function testRun(value) { evaluateExecPolicy(value); }\n",
    );
    write_file(
        dir.path(),
        "py/app.py",
        "def run(value):\n    evaluate_exec_policy(value)\n",
    );

    let mut session = StructuralSearchSession::default();
    let ts = session
        .search(StructuralSearchOptions {
            language: Some("typescript".to_string()),
            include_tests: false,
            ..find_options_for_path(dir.path(), "web", "function $NAME($$$ARGS) { $$$BODY }")
        })
        .await
        .expect("typescript find");
    assert!(ts.contains("structural_search[find] q1"));
    assert!(ts.contains("web/app.ts:1"));
    assert!(!ts.contains("web/app.test.ts"));

    let py = session
        .search(StructuralSearchOptions {
            language: Some("python".to_string()),
            ..find_options_for_path(dir.path(), "py", "def $NAME($$$ARGS): $$$BODY")
        })
        .await
        .expect("python find");
    assert!(py.contains("structural_search[find] q2"));
    assert!(py.contains("py/app.py:1"));
}

#[tokio::test]
async fn unsupported_language_returns_status_without_allocating_query() {
    let dir = tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", "pub fn run() {}\n");

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            language: Some("not-a-language".to_string()),
            ..find_options(dir.path(), "run($$$ARGS)")
        })
        .await
        .expect("unsupported language");

    assert!(output.contains("status: unsupported-language"));
    assert!(!output.contains(" q1"));
}

#[tokio::test]
async fn workspace_required_and_scope_error_do_not_allocate_query_numbers() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let workspace_required = session
        .search(StructuralSearchOptions {
            search_paths: Vec::new(),
            display_base_dir: None,
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("workspace-required");
    assert!(workspace_required.contains("status: workspace-required"));
    assert!(!workspace_required.contains(" q1"));

    let scope_error = session
        .search(StructuralSearchOptions {
            search_paths: vec![dir.path().join("missing")],
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("scope error");
    assert!(scope_error.contains("status: scope-error"));
    assert!(!scope_error.contains(" q1"));

    let output = session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find after non-registering errors");
    assert!(output.contains("structural_search[find] q1"));
}

#[tokio::test]
async fn unrecognized_extension_without_language_returns_unsupported_language() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "notes/policy.txt",
        "evaluate_exec_policy(ctx, command)\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            language: None,
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("unsupported inferred language");

    assert!(output.contains("status: unsupported-language"));
    assert!(output.contains("language: <unrecognized>"));
    assert!(!output.contains(" q1"));
}

#[tokio::test]
async fn explicit_history_ref_and_implicit_recent_ref_resolve_to_different_queries() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { prepare(); evaluate_exec_policy(ctx, command); finish(); }\n",
    );

    let mut session = StructuralSearchSession::default();
    let first = session
        .search(find_options(dir.path(), "prepare()"))
        .await
        .expect("first find");
    assert!(first.contains("structural_search[find] q1"));
    let second = session
        .search(find_options(dir.path(), "finish()"))
        .await
        .expect("second find");
    assert!(second.contains("structural_search[find] q2"));

    let implicit = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("implicit expand");
    assert!(implicit.contains("structural_search[expand] q3"));
    assert!(implicit.contains("from: q2.[1]"));
    assert!(implicit.contains("finish()"));

    let explicit = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Expand,
            reference: Some("q1.1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("explicit expand");
    assert!(explicit.contains("structural_search[expand] q4"));
    assert!(explicit.contains("from: q1.[1]"));
    assert!(explicit.contains("prepare()"));
}

#[tokio::test]
async fn atomic_rule_fixtures_each_match_and_report_rule_summary() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { prepare(); evaluate_exec_policy(ctx, command); finish(); }\n",
    );

    let fixtures = [
        (
            "kind",
            json!({"id":"kind-call","language":"Rust","rule":{"kind":"call_expression"}}),
            "prepare()",
        ),
        (
            "pattern",
            json!({"id":"pattern-call","language":"Rust","rule":{"pattern":"evaluate_exec_policy($$$ARGS)"}}),
            "evaluate_exec_policy(ctx, command)",
        ),
        (
            "regex",
            json!({"id":"regex-call","language":"Rust","rule":{"kind":"identifier","regex":"evaluate_exec_policy"}}),
            "evaluate_exec_policy",
        ),
        (
            "nthChild",
            json!({"id":"nth-child-call","language":"Rust","rule":{"pattern":"evaluate_exec_policy($$$ARGS);","nthChild":2}}),
            "evaluate_exec_policy(ctx, command);",
        ),
        (
            "range",
            json!({"id":"range-call","language":"Rust","rule":{"kind":"call_expression","range":{"start":{"line":0,"column":15},"end":{"line":0,"column":24}}}}),
            "prepare()",
        ),
    ];

    for (summary_part, rule, expected_text) in fixtures {
        let (_session, output) = run_rule(dir.path(), rule).await;
        assert!(
            output.contains(summary_part),
            "missing summary part {summary_part}\n{output}"
        );
        assert!(
            output.contains(expected_text),
            "missing expected text {expected_text}\n{output}"
        );
        assert!(!output.contains("status: no-matches"), "{output}");
    }
}

#[tokio::test]
async fn relational_rule_fixtures_each_match_and_report_relation_summary() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { prepare(); evaluate_exec_policy(ctx, command); finish(); }\n",
    );

    let fixtures = [
        (
            "inside",
            json!({"id":"inside-call","language":"Rust","rule":{"pattern":"evaluate_exec_policy($$$ARGS)","inside":{"kind":"function_item","stopBy":"end"}}}),
            "evaluate_exec_policy(ctx, command)",
        ),
        (
            "has",
            json!({"id":"has-call","language":"Rust","rule":{"kind":"function_item","has":{"pattern":"evaluate_exec_policy($$$ARGS)","stopBy":"end"}}}),
            "pub fn run()",
        ),
        (
            "precedes",
            json!({"id":"precedes-call","language":"Rust","rule":{"pattern":"prepare();","precedes":{"pattern":"evaluate_exec_policy($$$ARGS);","stopBy":"end"}}}),
            "prepare();",
        ),
        (
            "follows",
            json!({"id":"follows-call","language":"Rust","rule":{"pattern":"finish();","follows":{"pattern":"evaluate_exec_policy($$$ARGS);","stopBy":"end"}}}),
            "finish();",
        ),
    ];

    for (relation, rule, expected_text) in fixtures {
        let (_session, output) = run_rule(dir.path(), rule).await;
        assert!(output.contains(relation), "{output}");
        assert!(
            output.contains(&format!("relation: {relation}")),
            "{output}"
        );
        assert!(output.contains(expected_text), "{output}");
        assert!(!output.contains("status: no-matches"), "{output}");
    }
}

#[tokio::test]
async fn relational_rule_output_includes_structure_fact_summary() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { prepare(); evaluate_exec_policy(ctx, command); finish(); }\n",
    );

    let (_session, output) = run_rule(
        dir.path(),
        json!({"id":"inside-call","language":"Rust","rule":{"pattern":"evaluate_exec_policy($$$ARGS)","inside":{"kind":"function_item","stopBy":"end"}}}),
    )
    .await;
    assert!(output.contains("context: call_expression inside function_item"));

    let (_session, output) = run_rule(
        dir.path(),
        json!({"id":"has-call","language":"Rust","rule":{"kind":"function_item","has":{"pattern":"evaluate_exec_policy($$$ARGS)","stopBy":"end"}}}),
    )
    .await;
    assert!(output.contains("relation: function_item has evaluate_exec_policy($$$ARGS)"));
}

#[tokio::test]
async fn composite_rule_fixtures_each_match_and_report_rule_summary() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { prepare(); evaluate_exec_policy(ctx, command); finish(); }\n",
    );

    let fixtures = [
        (
            "all",
            json!({"id":"all-call","language":"Rust","rule":{"all":[{"kind":"call_expression"},{"regex":"evaluate_exec_policy"}]}}),
            "evaluate_exec_policy(ctx, command)",
        ),
        (
            "any",
            json!({"id":"any-call","language":"Rust","rule":{"any":[{"pattern":"evaluate_exec_policy($$$ARGS)"},{"pattern":"missing($$$ARGS)"}]}}),
            "evaluate_exec_policy(ctx, command)",
        ),
        (
            "not",
            json!({"id":"not-call","language":"Rust","rule":{"all":[{"kind":"call_expression"},{"not":{"regex":"prepare"}}]}}),
            "finish()",
        ),
        (
            "matches",
            json!({"id":"matches-call","language":"Rust","utils":{"isPolicyCall":{"pattern":"evaluate_exec_policy($$$ARGS)"}},"rule":{"matches":"isPolicyCall"}}),
            "evaluate_exec_policy(ctx, command)",
        ),
    ];

    for (summary_part, rule, expected_text) in fixtures {
        let (_session, output) = run_rule(dir.path(), rule).await;
        assert!(output.contains(summary_part), "{output}");
        assert!(output.contains(expected_text), "{output}");
        assert!(!output.contains("status: no-matches"), "{output}");
    }
}

#[tokio::test]
async fn rule_test_no_matches_allocates_query_and_preserves_next() {
    let dir = tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", "pub fn run() { prepare(); }\n");

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::RuleTest,
            ..find_options(dir.path(), "evaluate_exec_policy($$$ARGS)")
        })
        .await
        .expect("rule test no matches");

    assert!(output.contains("structural_search[rule_test] q1"));
    assert!(output.contains("status: no-matches"));
    assert!(output.contains("next:"));
}

#[tokio::test]
async fn around_accepts_path_line_query_without_prior_reference() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() {\n    evaluate_exec_policy(ctx, command);\n}\n",
    );

    let mut session = StructuralSearchSession::default();
    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::Around,
            query: Some("src/lib.rs:2".to_string()),
            reference: None,
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("around path line");

    assert!(output.contains("structural_search[around] q1"));
    assert!(output.contains("source: src/lib.rs:2"));
    assert!(output.contains("Enclosing"));
    assert!(output.contains("Siblings"));
    assert!(output.contains("Children"));
}

#[tokio::test]
async fn explain_ast_accepts_result_reference_and_scope_errors_are_pretty_text() {
    let dir = tempdir().expect("tempdir");
    write_file(
        dir.path(),
        "src/lib.rs",
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    );

    let mut session = StructuralSearchSession::default();
    session
        .search(find_options(dir.path(), "evaluate_exec_policy($$$ARGS)"))
        .await
        .expect("find");

    let output = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::ExplainAst,
            reference: Some("q1.1".to_string()),
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("explain ref");
    assert!(output.contains("structural_search[explain_ast] q2"));
    assert!(output.contains("source: src/lib.rs:1"));
    assert!(output.contains("node kind:"));
    assert!(output.contains("candidate pattern hints"));

    let out_of_range = session
        .search(StructuralSearchOptions {
            mode: StructuralSearchMode::ExplainAst,
            query: Some("src/lib.rs:99".to_string()),
            reference: None,
            ..find_options(dir.path(), "unused")
        })
        .await
        .expect("explain out of range");
    assert!(out_of_range.contains("status: scope-error"));
    assert!(!out_of_range.contains(" q3"));
}
