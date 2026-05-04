#[macro_use]
mod support;

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::Value;
use serde_json::json;
use std::path::{Path, PathBuf};
use support::json_object;

const DEFAULT_WORKSPACE_ENV: &str = "DOCUTOUCH_DEFAULT_WORKSPACE";

fn patch_files_in(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut patch_files = Vec::new();
    if !dir.exists() {
        return patch_files;
    }
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            patch_files.extend(patch_files_in(&path));
        } else if path.extension().is_some_and(|ext| ext == "patch") {
            patch_files.push(path);
        }
    }
    patch_files
}

fn assert_no_audit_patch_artifacts(workspace_root: &std::path::Path, message: &str) {
    assert!(!message.contains("patch artifacts written to"));
    assert!(!message.contains("failed-groups.json"));
    assert!(!message.contains("failed-groups.txt"));
    assert!(!message.contains("failure-summary.txt"));
    assert!(!message.contains("committed-files.txt"));
    assert!(
        !workspace_root
            .join(".docutouch")
            .join("failed-groups.json")
            .exists(),
        "unexpected failed-groups.json artifact\n{message}"
    );
    assert!(
        !workspace_root
            .join(".docutouch")
            .join("failed-groups.txt")
            .exists(),
        "unexpected failed-groups.txt artifact\n{message}"
    );
    assert!(
        !workspace_root
            .join(".docutouch")
            .join("failure-summary.txt")
            .exists(),
        "unexpected failure-summary.txt artifact\n{message}"
    );
    assert!(
        !workspace_root
            .join(".docutouch")
            .join("committed-files.txt")
            .exists(),
        "unexpected committed-files.txt artifact\n{message}"
    );
}

fn assert_failed_patch_source_persisted(workspace_root: &std::path::Path, message: &str) {
    let patch_files = patch_files_in(&workspace_root.join(".docutouch"));
    assert!(
        !patch_files.is_empty(),
        "expected persisted patch source under .docutouch, got none\n{message}"
    );
    assert!(
        message.contains(".docutouch"),
        "expected diagnostics to mention persisted patch path\n{message}"
    );
    assert!(
        !message.contains("--> <patch>:"),
        "expected a real persisted patch path instead of <patch>\n{message}"
    );
}

fn patch_text() -> String {
    "*** Begin Patch\n*** Add File: docs/notes.md\n+hello\n*** End Patch\n".to_string()
}

fn splice_text() -> String {
    "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Append To File: dest.txt\n*** End Splice\n".to_string()
}

fn rewrite_text() -> String {
    "*** Begin Rewrite\n*** Update File: app.txt\n@@ replace the selected line\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n".to_string()
}

fn rewrite_move_overwrite_text() -> String {
    "*** Begin Rewrite\n*** Update File: from.txt\n*** Move to: to.txt\n@@\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n".to_string()
}

fn rewrite_delete_missing_text() -> String {
    "*** Begin Rewrite\n*** Delete File: missing.txt\n*** End Rewrite\n".to_string()
}

struct PueueStubFixture {
    bin_path: PathBuf,
    runtime_dir: PathBuf,
    plan_path: PathBuf,
    counter_path: PathBuf,
}

impl PueueStubFixture {
    fn new(base_dir: &Path, snapshots: &[Value], logs: &[(u64, &str)]) -> anyhow::Result<Self> {
        let runtime_dir = base_dir.join("pueue-runtime");
        let tool_dir = base_dir.join("pueue-stub");
        let task_logs_dir = runtime_dir.join("task_logs");
        std::fs::create_dir_all(&task_logs_dir)?;
        std::fs::create_dir_all(&tool_dir)?;

        for (task_id, contents) in logs {
            std::fs::write(task_logs_dir.join(format!("{task_id}.log")), contents)?;
        }

        let plan_path = tool_dir.join("plan.json");
        let counter_path = tool_dir.join("counter.txt");
        std::fs::write(&plan_path, json!({ "snapshots": snapshots }).to_string())?;
        std::fs::write(&counter_path, "0")?;

        let script_path = tool_dir.join("pueue_stub.py");
        std::fs::write(
            &script_path,
            r#"import json
import os
import pathlib
import sys


def main() -> int:
    if sys.argv[1:] != ["status", "--json"]:
        print(f"unsupported pueue stub args: {sys.argv[1:]}", file=sys.stderr)
        return 1

    plan_path = pathlib.Path(os.environ["DOCUTOUCH_TEST_PUEUE_PLAN"])
    counter_path = pathlib.Path(os.environ["DOCUTOUCH_TEST_PUEUE_COUNTER"])
    snapshots = json.loads(plan_path.read_text(encoding="utf-8"))["snapshots"]

    counter = 0
    if counter_path.exists():
        raw_counter = counter_path.read_text(encoding="utf-8").strip()
        if raw_counter:
            counter = int(raw_counter)

    snapshot_index = min(counter, len(snapshots) - 1)
    counter_path.write_text(str(counter + 1), encoding="utf-8")
    sys.stdout.write(json.dumps(snapshots[snapshot_index]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#,
        )?;

        let bin_path = if cfg!(windows) {
            let powershell_path = tool_dir.join("pueue_stub.ps1");
            std::fs::write(
                &powershell_path,
                r#"$ErrorActionPreference = 'Stop'
if ($args.Count -ne 2 -or $args[0] -ne 'status' -or $args[1] -ne '--json') {
    [Console]::Error.WriteLine("unsupported pueue stub args: $args")
    exit 1
}

$planPath = $env:DOCUTOUCH_TEST_PUEUE_PLAN
$counterPath = $env:DOCUTOUCH_TEST_PUEUE_COUNTER
$snapshots = (Get-Content -LiteralPath $planPath -Raw | ConvertFrom-Json).snapshots

$counter = 0
if (Test-Path -LiteralPath $counterPath) {
    $rawCounter = (Get-Content -LiteralPath $counterPath -Raw).Trim()
    if ($rawCounter) {
        $counter = [int]$rawCounter
    }
}

$snapshotIndex = [Math]::Min($counter, $snapshots.Count - 1)
Set-Content -LiteralPath $counterPath -Value ($counter + 1) -NoNewline
[Console]::Out.Write(($snapshots[$snapshotIndex] | ConvertTo-Json -Compress -Depth 32))
"#,
            )?;
            let bin_path = tool_dir.join("pueue.cmd");
            std::fs::write(
                &bin_path,
                "@echo off\r\npowershell -NoProfile -ExecutionPolicy Bypass -File \"%~dp0pueue_stub.ps1\" %*\r\n",
            )?;
            bin_path
        } else {
            let bin_path = tool_dir.join("pueue");
            std::fs::write(
                &bin_path,
                "#!/usr/bin/env sh\nuv run python \"$(dirname \"$0\")/pueue_stub.py\" \"$@\"\n",
            )?;
            set_executable(&bin_path)?;
            bin_path
        };

        Ok(Self {
            bin_path,
            runtime_dir,
            plan_path,
            counter_path,
        })
    }

    fn configure_command(&self, command: &mut tokio::process::Command) {
        command.env("DOCUTOUCH_PUEUE_BIN", &self.bin_path);
        command.env("DOCUTOUCH_PUEUE_RUNTIME_DIR", &self.runtime_dir);
        command.env("DOCUTOUCH_TEST_PUEUE_PLAN", &self.plan_path);
        command.env("DOCUTOUCH_TEST_PUEUE_COUNTER", &self.counter_path);
    }
}

#[cfg(not(windows))]
fn set_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(windows)]
fn set_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

fn running_task(task_id: u64) -> Value {
    json!({
        "id": task_id,
        "status": {
            "Running": {
                "start": "now"
            }
        }
    })
}

fn done_task(task_id: u64, status: &str, exit_code: i64) -> Value {
    json!({
        "id": task_id,
        "status": {
            "Done": {
                "result": status,
                "exit_code": exit_code
            }
        }
    })
}

fn status_snapshot(tasks: Vec<Value>) -> Value {
    let mut map = serde_json::Map::new();
    for task in tasks {
        let task_id = task
            .get("id")
            .and_then(Value::as_u64)
            .expect("task id for stub snapshot");
        map.insert(task_id.to_string(), task);
    }
    Value::Object(
        [("tasks".to_string(), Value::Object(map))]
            .into_iter()
            .collect(),
    )
}

fn noisy_pueue_log() -> &'static str {
    "phase 1\rphase 2\n\u{1b}[32mDONE\u{1b}[0m\n"
}

fn assert_current_time_surface(output: &str) {
    let line = output
        .lines()
        .find(|line| line.starts_with("current_time: "))
        .expect("current_time header");
    let value = &line["current_time: ".len()..];
    let bytes = value.as_bytes();
    assert_eq!(bytes.len(), 19, "unexpected current_time surface: {value}");
    assert_eq!(bytes[4], b'-');
    assert_eq!(bytes[7], b'-');
    assert_eq!(bytes[10], b' ');
    assert_eq!(bytes[13], b':');
    assert_eq!(bytes[16], b':');
}

fn input_schema_property_description(tool: &rmcp::model::Tool, property: &str) -> String {
    input_schema_property_value(tool, property)
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn input_schema_property_value(tool: &rmcp::model::Tool, property: &str) -> Value {
    serde_json::to_value(tool.input_schema.as_ref())
        .expect("input schema json")
        .get("properties")
        .and_then(|properties| properties.get(property))
        .cloned()
        .unwrap_or(Value::Null)
}

fn tool_description(tool: &rmcp::model::Tool) -> String {
    tool.description
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default()
}

#[tokio::test]
async fn server_lists_expected_tools() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let tools = client.list_all_tools().await?;
        let tool_names = tools
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();

        assert!(tool_names.contains(&"set_workspace".to_string()));
        assert!(tool_names.contains(&"list_directory".to_string()));
        assert!(tool_names.contains(&"read_file".to_string()));
        assert!(tool_names.contains(&"search_text".to_string()));
        assert!(tool_names.contains(&"structural_search".to_string()));
        assert!(tool_names.contains(&"wait_pueue".to_string()));
        assert!(!tool_names.contains(&"read_files".to_string()));
        assert!(tool_names.contains(&"apply_patch".to_string()));
        assert!(tool_names.contains(&"apply_rewrite".to_string()));
        assert!(tool_names.contains(&"apply_splice".to_string()));

        Ok(())
    })
}

#[tokio::test]
async fn server_structural_search_find_and_expand_use_workspace_paths() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let src_dir = temp.path().join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    )?;

    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "find",
                    "pattern": "evaluate_exec_policy($$$ARGS)",
                    "path": "src",
                    "language": "rust",
                    "include_tests": true
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("structural_search[find] q1"));
        assert!(text.contains("src/lib.rs:1"));
        assert!(text.contains("expand 1"));

        let expanded = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "expand",
                    "ref": "1"
                }))),
                task: None,
            })
            .await?;
        let expanded_text = &expanded.content[0].as_text().unwrap().text;
        assert!(expanded_text.contains("structural_search[expand] q2"));
        assert!(expanded_text.contains("from: q1.[1]"));

        Ok(())
    })
}

#[tokio::test]
async fn server_structural_search_missing_path_returns_pretty_scope_error() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;

    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "find",
                    "pattern": "evaluate_exec_policy($$$ARGS)",
                    "path": "missing",
                    "language": "rust"
                }))),
                task: None,
            })
            .await;

        assert!(
            result.is_ok(),
            "missing path should be rendered by structural_search"
        );
        let result = result?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("structural_search[find]"));
        assert!(text.contains("status: scope-error"));
        assert!(!text.contains(" q1"));

        Ok(())
    })
}

#[tokio::test]
async fn server_structural_search_accepts_rule_objects_and_rejects_edit_fields()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let src_dir = temp.path().join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn run() { evaluate_exec_policy(ctx, command); }\n",
    )?;

    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "find",
                    "path": "src",
                    "rule": {
                        "id": "policy-call",
                        "language": "Rust",
                        "rule": { "pattern": "evaluate_exec_policy($$$ARGS)" }
                    }
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("structural_search[find] q1"));
        assert!(text.contains("rule: pattern"));
        assert!(text.contains("evaluate_exec_policy(ctx, command)"));

        let rejected = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "find",
                    "path": "src",
                    "rule": {
                        "id": "bad-rule",
                        "language": "Rust",
                        "rule": { "pattern": "evaluate_exec_policy($$$ARGS)" },
                        "fix": "replacement"
                    }
                }))),
                task: None,
            })
            .await?;
        let rejected_text = &rejected.content[0].as_text().unwrap().text;
        assert!(rejected_text.contains("status: unsupported-rule-field"));
        assert!(!rejected_text.contains(" q2"));

        Ok(())
    })
}

#[tokio::test]
async fn server_structural_search_query_numbers_are_connection_local() -> anyhow::Result<()> {
    let first = tempfile::tempdir()?;
    std::fs::create_dir_all(first.path().join("src"))?;
    std::fs::write(
        first.path().join("src/lib.rs"),
        "pub fn run() { first_policy(ctx); }\n",
    )?;
    let second = tempfile::tempdir()?;
    std::fs::create_dir_all(second.path().join("src"))?;
    std::fs::write(
        second.path().join("src/lib.rs"),
        "pub fn run() { second_policy(ctx); }\n",
    )?;

    with_server_client!(first.path(), first_client, {
        first_client
            .call_tool(support::workspace_tool_call(first.path()))
            .await?;
        let first_result = first_client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "find",
                    "pattern": "first_policy($$$ARGS)",
                    "path": "src",
                    "language": "rust",
                    "include_tests": true
                }))),
                task: None,
            })
            .await?;
        let first_text = &first_result.content[0].as_text().unwrap().text;
        assert!(first_text.contains("structural_search[find] q1"));
        assert!(first_text.contains("first_policy(ctx)"));

        with_server_client!(second.path(), second_client, {
            second_client
                .call_tool(support::workspace_tool_call(second.path()))
                .await?;
            let second_result = second_client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "structural_search".into(),
                    arguments: Some(json_object(json!({
                        "mode": "find",
                        "pattern": "second_policy($$$ARGS)",
                        "path": "src",
                        "language": "rust",
                        "include_tests": true
                    }))),
                    task: None,
                })
                .await?;
            let second_text = &second_result.content[0].as_text().unwrap().text;
            assert!(second_text.contains("structural_search[find] q1"));
            assert!(second_text.contains("second_policy(ctx)"));

            Ok(())
        })?;

        let expanded = first_client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "structural_search".into(),
                arguments: Some(json_object(json!({
                    "mode": "expand",
                    "ref": "1"
                }))),
                task: None,
            })
            .await?;
        let expanded_text = &expanded.content[0].as_text().unwrap().text;
        assert!(expanded_text.contains("structural_search[expand] q2"));
        assert!(expanded_text.contains("from: q1.[1]"));
        assert!(expanded_text.contains("first_policy(ctx)"));

        Ok(())
    })
}

#[tokio::test]
async fn server_tool_descriptions_surface_pueue_log_contract() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let tools = client.list_all_tools().await?;
        let read_file = tools
            .iter()
            .find(|tool| tool.name.as_ref() == "read_file")
            .expect("read_file tool");
        let search_text = tools
            .iter()
            .find(|tool| tool.name.as_ref() == "search_text")
            .expect("search_text tool");
        let structural_search = tools
            .iter()
            .find(|tool| tool.name.as_ref() == "structural_search")
            .expect("structural_search tool");
        let wait_pueue = tools
            .iter()
            .find(|tool| tool.name.as_ref() == "wait_pueue")
            .expect("wait_pueue tool");

        assert!(tool_description(read_file).contains("pueue-log:<id>"));
        assert!(tool_description(search_text).contains("pueue-log:<id>"));
        assert!(tool_description(structural_search).contains("AST 结构查询工具"));
        assert!(tool_description(structural_search).contains("unsupported-rule-field"));
        assert!(tool_description(wait_pueue).contains("pueue-log:<id>"));

        assert!(
            input_schema_property_description(read_file, "relative_path")
                .contains("pueue-log:<id>")
        );
        let read_line_range_description =
            input_schema_property_description(read_file, "line_range");
        assert!(read_line_range_description.contains("start:stop"));
        assert!(read_line_range_description.contains("1-indexed"));
        assert!(!read_line_range_description.contains("start,end"));
        assert!(!read_line_range_description.contains("step"));
        assert!(input_schema_property_description(search_text, "path").contains("pueue-log:<id>"));
        assert!(input_schema_property_description(structural_search, "mode").contains("find"));
        assert!(input_schema_property_description(structural_search, "ref").contains("qN.N"));
        assert!(
            input_schema_property_description(structural_search, "rule")
                .contains("直接传 JSON object")
        );
        assert_eq!(
            input_schema_property_value(structural_search, "rule")
                .get("type")
                .and_then(Value::as_str),
            Some("object")
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_wait_pueue_empty_explicit_task_set_returns_nothing_to_wait_for()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "wait_pueue".into(),
                arguments: Some(json_object(json!({
                    "task_ids": []
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("reason: nothing_to_wait_for"));
        assert!(text.contains("mode: any"));
        assert!(text.contains("resolved_task_ids:"));
        assert!(!text.contains("triggered_task_ids:"));
        assert!(!text.contains("pending_task_ids:"));
        assert!(!text.contains("log_handle:"));
        assert_current_time_surface(text);
        assert!(text.contains("waited_seconds: 0.0"));

        Ok(())
    })
}

#[tokio::test]
async fn server_wait_pueue_invalid_timeout_is_reported_as_invalid_argument() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;
        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "wait_pueue".into(),
                arguments: Some(json_object(json!({
                    "task_ids": [],
                    "timeout_seconds": 0
                }))),
                task: None,
            })
            .await
            .expect_err("non-positive timeout should be rejected");
        assert!(
            err.to_string()
                .contains("timeout_seconds must be a positive number")
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_accepts_pueue_log_handle_without_metadata_header() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture = PueueStubFixture::new(
        temp.path(),
        &[status_snapshot(vec![done_task(42, "Success", 0)])],
        &[(42, "alpha\nbeta\n")],
    )?;
    with_server_client!(
        temp.path(),
        |cmd| {
            fixture.configure_command(cmd);
        },
        client,
        {
            let result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({
                        "relative_path": "pueue-log:42"
                    }))),
                    task: None,
                })
                .await?;
            let text = &result.content[0].as_text().unwrap().text;
            assert_eq!(text, "alpha\nbeta\n");
            assert!(!text.contains("pueue-log:42"));
            assert!(!text.contains("task_logs"));
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_search_text_accepts_pueue_log_handle_in_path_arrays() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.txt"), "alpha\n")?;
    let fixture = PueueStubFixture::new(
        temp.path(),
        &[status_snapshot(vec![done_task(42, "Success", 0)])],
        &[(42, "alpha\n")],
    )?;
    with_server_client!(
        temp.path(),
        |cmd| {
            fixture.configure_command(cmd);
        },
        client,
        {
            client
                .call_tool(support::workspace_tool_call(temp.path()))
                .await?;
            let result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "search_text".into(),
                    arguments: Some(json_object(json!({
                        "query": "alpha",
                        "path": ["pueue-log:42", "notes.txt"],
                        "view": "full"
                    }))),
                    task: None,
                })
                .await?;
            let text = &result.content[0].as_text().unwrap().text;
            assert!(text.contains("search_text[full]:"));
            assert!(text.contains("scope: [pueue-log:42, notes.txt]"));
            assert!(text.contains("pueue-log:42 (1 line, 1 match)"));
            assert!(text.contains("notes.txt (1 line, 1 match)"));
            assert!(!text.contains("task_logs/42.log"));
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_read_file_cleans_pueue_log_surface() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture = PueueStubFixture::new(
        temp.path(),
        &[status_snapshot(vec![done_task(77, "Success", 0)])],
        &[(77, noisy_pueue_log())],
    )?;
    with_server_client!(
        temp.path(),
        |cmd| {
            fixture.configure_command(cmd);
        },
        client,
        {
            let result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({
                        "relative_path": "pueue-log:77"
                    }))),
                    task: None,
                })
                .await?;
            let text = &result.content[0].as_text().unwrap().text;
            assert_eq!(text, "phase 2\nDONE\n");
            assert!(!text.contains("phase 1"));
            assert!(!text.contains("\u{1b}"));
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_search_text_uses_the_same_clean_pueue_log_surface() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture = PueueStubFixture::new(
        temp.path(),
        &[status_snapshot(vec![done_task(78, "Success", 0)])],
        &[(78, noisy_pueue_log())],
    )?;
    with_server_client!(
        temp.path(),
        |cmd| {
            fixture.configure_command(cmd);
        },
        client,
        {
            let result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "search_text".into(),
                    arguments: Some(json_object(json!({
                        "query": "DONE",
                        "path": "pueue-log:78",
                        "view": "full"
                    }))),
                    task: None,
                })
                .await?;
            let text = &result.content[0].as_text().unwrap().text;
            assert!(text.contains("scope: pueue-log:78"));
            assert!(text.contains("pueue-log:78 (1 line, 1 match)"));
            assert!(text.contains("2 | DONE"));
            assert!(!text.contains("phase 1"));
            assert!(!text.contains("\u{1b}"));
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_read_file_reports_missing_pueue_task_truthfully() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture = PueueStubFixture::new(temp.path(), &[status_snapshot(vec![])], &[])?;
    with_server_client!(
        temp.path(),
        |cmd| {
            fixture.configure_command(cmd);
        },
        client,
        {
            let err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({
                        "relative_path": "pueue-log:99"
                    }))),
                    task: None,
                })
                .await
                .expect_err("missing task should fail");
            assert!(err.to_string().contains("Task does not exist: 99"));
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_search_text_reports_missing_pueue_log_truthfully() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let fixture =
        PueueStubFixture::new(temp.path(), &[status_snapshot(vec![running_task(41)])], &[])?;
    with_server_client!(
        temp.path(),
        |cmd| {
            fixture.configure_command(cmd);
        },
        client,
        {
            let err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "search_text".into(),
                    arguments: Some(json_object(json!({
                        "query": "alpha",
                        "path": "pueue-log:41"
                    }))),
                    task: None,
                })
                .await
                .expect_err("missing log should fail");
            assert!(err.to_string().contains("Task log not available: 41"));
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_round_trips_workspace_splice_and_read() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("source.txt"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let splice_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_splice".into(),
                arguments: Some(json_object(json!({ "splice": splice_text() }))),
                task: None,
            })
            .await?;
        let message = &splice_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert!(message.contains("A dest.txt"));

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": "dest.txt" }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "alpha\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_round_trips_workspace_rewrite_and_read() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.txt"), "old\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;

        let rewrite_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_rewrite".into(),
                arguments: Some(json_object(json!({ "rewrite": rewrite_text() }))),
                task: None,
            })
            .await?;
        let message = &rewrite_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert!(message.contains("M app.txt"));

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": "app.txt" }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "new");

        Ok(())
    })
}

#[tokio::test]
async fn server_rewrite_warns_and_uses_final_path_accounting_for_move_overwrite()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("from.txt"), "old\n")?;
    std::fs::write(temp.path().join("to.txt"), "dest\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;

        let rewrite_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_rewrite".into(),
                arguments: Some(json_object(
                    json!({ "rewrite": rewrite_move_overwrite_text() }),
                )),
                task: None,
            })
            .await?;
        let message = &rewrite_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert!(message.contains("A to.txt"));
        assert!(message.contains("D from.txt"));
        assert!(!message.contains("M to.txt"));
        assert!(message.contains("Warnings:"));
        assert!(message.contains("MOVE_REPLACED_EXISTING_DESTINATION"));

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": "to.txt" }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "new");
        assert!(!temp.path().join("from.txt").exists());

        Ok(())
    })
}

#[tokio::test]
async fn server_rewrite_delete_missing_is_reported_as_hard_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_rewrite".into(),
                arguments: Some(json_object(
                    json!({ "rewrite": rewrite_delete_missing_text() }),
                )),
                task: None,
            })
            .await
            .expect_err("missing delete target should fail");

        let message = err.to_string();
        assert!(message.contains("error[REWRITE_TARGET_STATE_INVALID]"));
        assert!(message.contains("delete target does not exist"));
        assert!(message.contains("2 | *** Delete File: missing.txt"));
        assert!(message.contains("caused by:\n  delete target does not exist"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_groups_matches_by_file() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(
        temp.path().join("src").join("one.txt"),
        "alpha\nbeta\nalpha\n",
    )?;
    std::fs::write(temp.path().join("src").join("two.txt"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": "src"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("search_text[preview]:"));
        assert!(text.contains("scope: src"));
        assert!(text.contains("files: 2"));
        assert!(text.contains("matched_lines: 3"));
        assert!(text.contains("matches: 3"));
        assert!(text.contains("rendered_files: 2"));
        assert!(text.contains("rendered_lines: 3"));
        assert!(text.contains("[1] src/one.txt (2 lines, 2 matches)"));
        assert!(text.contains("  1 | alpha"));
        assert!(text.contains("  3 | alpha"));
        assert!(text.contains("[2] src/two.txt (1 line, 1 match)"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_preview_accounts_for_omission() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(
        temp.path().join("src").join("noisy.txt"),
        "alpha\nalpha\nalpha\nalpha\nalpha\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": "src"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("search_text[preview]:"));
        assert!(text.contains("files: 1"));
        assert!(text.contains("matched_lines: 5"));
        assert!(text.contains("matches: 5"));
        assert!(text.contains("rendered_files: 1"));
        assert!(text.contains("rendered_lines: 3"));
        assert!(text.contains("[1] src/noisy.txt (5 lines, 5 matches)"));
        assert!(text.contains("  note: 2 more rendered lines in this file"));
        assert!(text.contains("omitted:"));
        assert!(text.contains("- 2 more rendered lines not shown"));
        assert!(!text.contains("  4 | alpha"));
        assert!(!text.contains("  5 | alpha"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_full_returns_all_grouped_matches() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(
        temp.path().join("src").join("full.txt"),
        "alpha\nalpha\nalpha\nalpha\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": "src",
                    "view": "full"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("search_text[full]:"));
        assert!(text.contains("files: 1"));
        assert!(text.contains("matched_lines: 4"));
        assert!(text.contains("matches: 4"));
        assert!(!text.contains("rendered_files:"));
        assert!(!text.contains("omitted:"));
        assert!(text.contains("  1 | alpha"));
        assert!(text.contains("  2 | alpha"));
        assert!(text.contains("  3 | alpha"));
        assert!(text.contains("  4 | alpha"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_accepts_path_arrays() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha\n")?;
    std::fs::write(temp.path().join("docs").join("two.txt"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": ["src", "docs/two.txt"],
                    "view": "full"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("search_text[full]:"));
        assert!(text.contains("scope: [src, docs/two.txt]"));
        assert!(text.contains("files: 2"));
        assert!(text.contains("[1] docs/two.txt (1 line, 1 match)"));
        assert!(text.contains("[2] src/one.txt (1 line, 1 match)"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_absorbs_redundant_line_number_flag() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": "src",
                    "rg_args": "-n"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("search_text[preview]:"));
        assert!(text.contains("rg_args: -n"));
        assert!(text.contains("[1] src/one.txt (1 line, 1 match)"));
        assert!(text.contains("  1 | alpha"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_infers_grouped_context_for_context_flag() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(
        temp.path().join("src").join("one.txt"),
        "before\nalpha\nafter\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": "src",
                    "rg_args": "-C 2"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("search_text[grouped_context]:"));
        assert!(text.contains("context: before=2 after=2"));
        assert!(text.contains("M 2 | alpha") || text.contains("M  2 | alpha"));
        assert!(text.contains("C 1 | before") || text.contains("C  1 | before"));
        assert!(text.contains("C 3 | after") || text.contains("C  3 | after"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_auto_falls_back_to_literal_for_invalid_regex() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "{ref:alpha}\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "{ref:",
                    "path": "src"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("query_interpretation: literal_fallback"));
        assert!(text.contains("  1 | {ref:alpha}"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_returns_raw_json_when_requested_by_rg_args() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": "src",
                    "rg_args": "--json"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.starts_with('{'));
        assert!(!text.contains("search_text["));
        assert!(text.contains("\"type\":\"match\""));
        assert!(text.contains("one.txt"));
        assert!(!text.contains(temp.path().to_string_lossy().as_ref()));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_invalid_context_value_is_reported_instead_of_being_swallowed()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.txt"), "search_text\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "search_text",
                    "path": ".",
                    "rg_args": "-C nope"
                }))),
                task: None,
            })
            .await
            .expect_err("invalid context value should fail");
        assert!(err.to_string().contains("error parsing flag -C"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_invalid_color_value_is_reported_instead_of_being_swallowed()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.txt"), "search_text\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "search_text",
                    "path": ".",
                    "rg_args": "--color banana"
                }))),
                task: None,
            })
            .await
            .expect_err("invalid color value should fail");
        assert!(err.to_string().contains("choice 'banana' is unrecognized"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_json_files_combination_falls_back_to_raw_text() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha\n")?;
    std::fs::write(temp.path().join("src").join("two.txt"), "beta\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "",
                    "path": ".",
                    "rg_args": "--json --files"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(!text.contains("search_text["));
        assert!(!text.starts_with('{'));
        assert!(text.contains("./src/one.txt"));
        assert!(text.contains("./src/two.txt"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_json_count_combination_falls_back_to_raw_text() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha",
                    "path": ".",
                    "rg_args": "--json --count-matches"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(!text.contains("search_text["));
        assert!(!text.starts_with('{'));
        assert!(text.contains("./src/one.txt:2"));

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_ranks_by_matched_lines_then_hits_then_path() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha\nbeta\n")?;
    std::fs::write(
        temp.path().join("src").join("two.txt"),
        "alpha beta\nbeta\n",
    )?;
    std::fs::write(temp.path().join("src").join("zzz.txt"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_text".into(),
                arguments: Some(json_object(json!({
                    "query": "alpha|beta",
                    "path": "src",
                    "view": "full"
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        let two = text
            .find("[1] src/two.txt")
            .expect("two.txt should rank first");
        let one = text
            .find("[2] src/one.txt")
            .expect("one.txt should rank second");
        let zzz = text
            .find("[3] src/zzz.txt")
            .expect("zzz.txt should rank third");
        assert!(two < one && one < zzz);

        Ok(())
    })
}

#[tokio::test]
async fn server_search_text_requires_workspace_for_relative_paths_without_default()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.txt"), "alpha\n")?;
    with_server_client!(
        temp.path(),
        |cmd| {
            cmd.env_remove(DEFAULT_WORKSPACE_ENV);
        },
        client,
        {
            let err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "search_text".into(),
                    arguments: Some(json_object(json!({
                        "query": "alpha",
                        "path": "."
                    }))),
                    task: None,
                })
                .await
                .expect_err("relative search should require workspace");

            assert!(err.to_string().contains(
                "Call set_workspace first, set DOCUTOUCH_DEFAULT_WORKSPACE, or use an absolute path"
            ));

            Ok(())
        }
    )
}

#[tokio::test]
async fn server_round_trips_workspace_patch_and_read() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({ "patch": patch_text() }))),
                task: None,
            })
            .await?;
        assert!(
            patch_result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains("Success. Updated the following files:")
        );
        assert!(
            patch_result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains("A docs")
        );
        assert!(
            patch_result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains("notes.md")
        );

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": "docs/notes.md" }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "hello\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_warns_when_add_replaces_existing_file() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.md"), "old\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({
                    "patch": "*** Begin Patch\n*** Add File: notes.md\n+new\n*** End Patch\n"
                }))),
                task: None,
            })
            .await?;
        let message = &patch_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert!(message.contains("warning[ADD_REPLACED_EXISTING_FILE]"));
        assert!(message.contains("  --> notes.md"));
        assert!(message.contains("prefer Update File when editing an existing file"));

        Ok(())
    })
}

#[tokio::test]
async fn server_warns_when_move_replaces_existing_destination() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("from.txt"), "from\n")?;
    std::fs::write(temp.path().join("to.txt"), "dest\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let patch_result = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Update File: from.txt\n*** Move to: to.txt\n@@\n-from\n+new\n*** End Patch\n"
            }))),
            task: None,
        })
        .await?;
        let message = &patch_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert!(message.contains("warning[MOVE_REPLACED_EXISTING_DESTINATION]"));
        assert!(message.contains("  --> to.txt"));
        assert!(message.contains("prefer a fresh destination path when renaming"));

        Ok(())
    })
}

#[tokio::test]
async fn server_allows_empty_add_file_and_creates_empty_file() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let patch = "*** Begin Patch\n*** Add File: empty.txt\n*** End Patch\n";
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({ "patch": patch }))),
                task: None,
            })
            .await?;
        let message = &patch_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert!(message.contains("A empty.txt"));
        assert_eq!(
            std::fs::read(temp.path().join("empty.txt"))?,
            Vec::<u8>::new()
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_preserves_crlf_bytes_during_update() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let patch = "*** Begin Patch\n*** Update File: crlf.txt\n@@\n-b\n+x\n*** End Patch\n";
    std::fs::write(temp.path().join("crlf.txt"), b"a\r\nb\r\nc\r\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({ "patch": patch }))),
                task: None,
            })
            .await?;
        let message = &patch_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert_eq!(
            std::fs::read(temp.path().join("crlf.txt"))?,
            b"a\r\nx\r\nc\r\n"
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_preserves_missing_final_newline_during_update() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let patch = "*** Begin Patch\n*** Update File: no_newline.txt\n@@\n-no newline at end\n+first line\n+second line\n*** End Patch\n";
    std::fs::write(temp.path().join("no_newline.txt"), b"no newline at end")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({ "patch": patch }))),
                task: None,
            })
            .await?;
        let message = &patch_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert_eq!(
            std::fs::read(temp.path().join("no_newline.txt"))?,
            b"first line\nsecond line"
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_can_show_one_indexed_line_numbers() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.md"), "alpha\nbeta\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.md",
                    "line_range": [2, 2],
                    "show_line_numbers": true
                }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "2 | beta\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_aligns_line_numbers_to_widest_visible_line() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let content = (1..=12)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    std::fs::write(temp.path().join("notes.md"), content)?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.md",
                    "line_range": [9, 12],
                    "show_line_numbers": true
                }))),
                task: None,
            })
            .await?;
        assert_eq!(
            read_result.content[0].as_text().unwrap().text,
            " 9 | line 9\n10 | line 10\n11 | line 11\n12 | line 12\n"
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_supports_sampled_view() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(
        temp.path().join("notes.txt"),
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.txt",
                    "line_range": "1:7",
                    "sample_step": 5,
                    "sample_lines": 2,
                    "max_chars": 80
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert_eq!(text, "line 1\nline 2\n...\nline 6\nline 7\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_supports_slice_like_tail_ranges() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(
        temp.path().join("notes.txt"),
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let tail_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.txt",
                    "line_range": "-3:",
                    "show_line_numbers": true
                }))),
                task: None,
            })
            .await?;
        assert_eq!(
            tail_result.content[0].as_text().unwrap().text,
            "5 | line 5\n6 | line 6\n7 | line 7\n"
        );

        let head_without_last = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.txt",
                    "line_range": ":-1"
                }))),
                task: None,
            })
            .await?;
        assert_eq!(
            head_without_last.content[0].as_text().unwrap().text,
            "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\n"
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_applies_defaults_for_partial_sampled_view_args() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(
        temp.path().join("notes.txt"),
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.txt",
                    "sample_step": 5
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert_eq!(text, "line 1\nline 2\n...\nline 6\nline 7\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_read_file_max_chars_alone_keeps_exact_contiguous_range() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(
        temp.path().join("notes.txt"),
        "line 1 has more text here\nline 2 has more text here\nline 3 has more text here\nline 4 has more text here\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": "notes.txt",
                    "line_range": "2:4",
                    "show_line_numbers": true,
                    "max_chars": 12
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert_eq!(
            text,
            "2 | line 2 has m...[13 chars omitted]\n3 | line 3 has m...[13 chars omitted]\n4 | line 4 has m...[13 chars omitted]\n"
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_requires_workspace_for_relative_paths_without_default() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let invalid_workspace = temp.path().join("missing-workspace");
    std::fs::write(temp.path().join("notes.md"), "alpha\nbeta\n")?;
    with_server_client!(
        temp.path(),
        |cmd| {
            cmd.env(DEFAULT_WORKSPACE_ENV, &invalid_workspace);
        },
        client,
        {
            let read_err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({ "relative_path": "notes.md" }))),
                    task: None,
                })
                .await
                .expect_err("relative read_file should require workspace");
            let read_err_text = read_err.to_string();
            assert!(
        read_err_text.contains(
            "Relative path requires workspace. Call set_workspace first, set DOCUTOUCH_DEFAULT_WORKSPACE, or use an absolute path."
        ),
        "actual read_file error: {read_err_text}"
    );

            let list_err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "list_directory".into(),
                    arguments: Some(json_object(json!({ "relative_path": "." }))),
                    task: None,
                })
                .await
                .expect_err("relative list_directory should require workspace");
            let list_err_text = list_err.to_string();
            assert!(
        list_err_text.contains(
            "Relative path requires workspace. Call set_workspace first, set DOCUTOUCH_DEFAULT_WORKSPACE, or use an absolute path."
        ),
        "actual list_directory error: {list_err_text}"
    );

            let patch_err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "apply_patch".into(),
                    arguments: Some(json_object(json!({ "patch": patch_text() }))),
                    task: None,
                })
                .await
                .expect_err("relative apply_patch should require workspace");
            let patch_err_text = patch_err.to_string();
            assert!(
        patch_err_text.contains(
            "Relative path requires workspace. Call set_workspace first, set DOCUTOUCH_DEFAULT_WORKSPACE, or use an absolute path."
        ),
        "actual apply_patch error: {patch_err_text}"
    );

            Ok(())
        }
    )
}

#[tokio::test]
async fn server_uses_default_workspace_from_env_without_set_workspace() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let workspace = temp.path().join("workspace");
    let launch_dir = temp.path().join("launch");
    std::fs::create_dir_all(&workspace)?;
    std::fs::create_dir_all(&launch_dir)?;
    std::fs::write(workspace.join("seed.txt"), "seed\n")?;
    with_server_client!(
        launch_dir.as_path(),
        |cmd| {
            cmd.env(DEFAULT_WORKSPACE_ENV, &workspace);
        },
        client,
        {
            let read_result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({ "relative_path": "seed.txt" }))),
                    task: None,
                })
                .await?;
            assert_eq!(read_result.content[0].as_text().unwrap().text, "seed\n");

            let list_result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "list_directory".into(),
                    arguments: Some(json_object(json!({ "relative_path": "." }))),
                    task: None,
                })
                .await?;
            assert!(
                list_result.content[0]
                    .as_text()
                    .unwrap()
                    .text
                    .contains("seed.txt")
            );

            let patch_result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "apply_patch".into(),
                    arguments: Some(json_object(json!({ "patch": patch_text() }))),
                    task: None,
                })
                .await?;
            assert!(
                patch_result.content[0]
                    .as_text()
                    .unwrap()
                    .text
                    .contains("A docs")
            );
            assert_eq!(
                std::fs::read_to_string(workspace.join("docs").join("notes.md"))?,
                "hello\n"
            );

            Ok(())
        }
    )
}

#[tokio::test]
async fn server_set_workspace_overrides_default_workspace_env() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let default_workspace = temp.path().join("default-workspace");
    let override_workspace = temp.path().join("override-workspace");
    let launch_dir = temp.path().join("launch");
    std::fs::create_dir_all(&default_workspace)?;
    std::fs::create_dir_all(&override_workspace)?;
    std::fs::create_dir_all(&launch_dir)?;
    std::fs::write(default_workspace.join("notes.md"), "default\n")?;
    std::fs::write(override_workspace.join("notes.md"), "override\n")?;
    with_server_client!(
        launch_dir.as_path(),
        |cmd| {
            cmd.env(DEFAULT_WORKSPACE_ENV, &default_workspace);
        },
        client,
        {
            client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "set_workspace".into(),
                    arguments: Some(json_object(json!({ "path": &override_workspace }))),
                    task: None,
                })
                .await?;

            let read_result = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({ "relative_path": "notes.md" }))),
                    task: None,
                })
                .await?;
            assert_eq!(read_result.content[0].as_text().unwrap().text, "override\n");

            Ok(())
        }
    )
}

#[tokio::test]
async fn server_ignores_invalid_default_workspace_env() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let invalid_workspace = temp.path().join("missing-workspace");
    std::fs::write(temp.path().join("notes.md"), "cwd\n")?;
    with_server_client!(
        temp.path(),
        |cmd| {
            cmd.env(DEFAULT_WORKSPACE_ENV, &invalid_workspace);
        },
        client,
        {
            let err = client
                .call_tool(CallToolRequestParams {
                    meta: None,
                    name: "read_file".into(),
                    arguments: Some(json_object(json!({ "relative_path": "notes.md" }))),
                    task: None,
                })
                .await
                .expect_err("invalid default workspace should behave like no workspace");
            assert!(
        err.to_string()
            .contains("Relative path requires workspace. Call set_workspace first, set DOCUTOUCH_DEFAULT_WORKSPACE, or use an absolute path.")
    );

            Ok(())
        }
    )
}

#[tokio::test]
async fn server_apply_patch_accepts_absolute_paths_without_workspace() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let launch_dir = temp.path().join("launch");
    std::fs::create_dir_all(&launch_dir)?;
    let target = temp.path().join("absolute-notes.md");
    with_server_client!(launch_dir.as_path(), client, {
        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({
                    "patch": format!(
                        "*** Begin Patch\n*** Add File: {}\n+hello\n*** End Patch\n",
                        target.display()
                    )
                }))),
                task: None,
            })
            .await?;
        assert!(
            patch_result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains(&target.display().to_string().replace('\\', "/"))
        );

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": target }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "hello\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_apply_splice_accepts_absolute_paths_without_workspace() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let launch_dir = temp.path().join("launch");
    std::fs::create_dir_all(&launch_dir)?;
    let source = temp.path().join("absolute-source.txt");
    let dest = temp.path().join("absolute-dest.txt");
    std::fs::write(&source, "alpha\n")?;
    with_server_client!(launch_dir.as_path(), client, {
        let splice_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_splice".into(),
                arguments: Some(json_object(json!({
                    "splice": format!(
                        "*** Begin Splice\n*** Copy From File: {}\n@@\n1 | alpha\n*** Append To File: {}\n*** End Splice\n",
                        source.display(),
                        dest.display()
                    )
                }))),
                task: None,
            })
            .await?;
        assert!(
            splice_result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains(&dest.display().to_string().replace('\\', "/"))
        );

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": dest }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "alpha\n");

        Ok(())
    })
}

#[tokio::test]
async fn server_apply_rewrite_accepts_absolute_paths_without_workspace() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let launch_dir = temp.path().join("launch");
    std::fs::create_dir_all(&launch_dir)?;
    let target = temp.path().join("absolute-app.txt");
    std::fs::write(&target, "old\n")?;
    with_server_client!(launch_dir.as_path(), client, {
        let rewrite_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_rewrite".into(),
                arguments: Some(json_object(json!({
                    "rewrite": format!(
                        "*** Begin Rewrite\n*** Update File: {}\n@@\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n",
                        target.display()
                    )
                }))),
                task: None,
            })
            .await?;
        assert!(
            rewrite_result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains(&target.display().to_string().replace('\\', "/"))
        );

        let read_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({ "relative_path": target }))),
                task: None,
            })
            .await?;
        assert_eq!(read_result.content[0].as_text().unwrap().text, "new");

        Ok(())
    })
}

#[tokio::test]
async fn server_list_directory_can_show_requested_timestamps() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.md"), "alpha\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "list_directory".into(),
                arguments: Some(json_object(json!({
                    "relative_path": ".",
                    "timestamp_fields": ["modified"]
                }))),
                task: None,
            })
            .await?;
        assert!(
            result.content[0]
                .as_text()
                .unwrap()
                .text
                .contains("modified=")
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_list_directory_can_filter_by_ripgrep_file_type() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("main.rs"), "fn main() {}\n")?;
    std::fs::write(temp.path().join("src").join("main.cpp"), "int main() {}\n")?;
    std::fs::write(temp.path().join("README.md"), "# notes\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "list_directory".into(),
                arguments: Some(json_object(json!({
                    "relative_path": ".",
                    "file_types": ["rust"]
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("main.rs"));
        assert!(!text.contains("main.cpp"));
        assert!(!text.contains("README.md"));
        assert!(text.contains("2 type"));

        Ok(())
    })
}

#[tokio::test]
async fn server_list_directory_can_exclude_ripgrep_file_type() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("main.rs"), "fn main() {}\n")?;
    std::fs::write(temp.path().join("README.md"), "# notes\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "list_directory".into(),
                arguments: Some(json_object(json!({
                    "relative_path": ".",
                    "file_types_not": ["markdown"]
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("main.rs"));
        assert!(!text.contains("README.md"));
        assert!(text.contains("1 type"));

        Ok(())
    })
}

#[tokio::test]
async fn server_list_directory_warns_and_ignores_unknown_ripgrep_file_type() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("README.md"), "# notes\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "list_directory".into(),
                arguments: Some(json_object(json!({
                    "relative_path": ".",
                    "file_types": ["notatype"]
                }))),
                task: None,
            })
            .await?;
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("README.md"));
        assert!(text.contains("warnings:"));
        assert!(text.contains("notatype"));
        assert!(text.contains("type filtering was disabled"));

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_outer_format_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(
                    json!({ "patch": "*** Patch File: changes.patch\n" }),
                )),
                task: None,
            })
            .await
            .expect_err("invalid patch should fail");

        let message = err.to_string();
        assert!(message.contains("OUTER_INVALID_PATCH"));
        assert!(message.contains("repair the patch shell"));

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_outer_hunk_failure_with_source_excerpt() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Add File: broken.txt\nbroken line\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("invalid hunk should fail");

        let message = err.to_string();
        assert!(message.contains("OUTER_INVALID_ADD_LINE"));
        assert!(message.contains("Add File block is malformed"));
        assert!(message.contains("prefix each Add File content line with '+'"));
        assert!(message.contains(":3:1"));
        assert!(message.contains("3 | broken line"));
        assert!(message.contains("| ^"));

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_empty_patch_as_structured_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({ "patch": "" }))),
                task: None,
            })
            .await
            .expect_err("empty patch should fail");

        let message = err.to_string();
        assert!(message.contains("OUTER_EMPTY_PATCH"));
        assert!(message.contains("patch cannot be empty"));
        assert!(message.contains("provide a complete patch envelope before retrying"));
        assert!(!message.contains("  --> <patch>"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);
        assert!(message.contains("\ncaused by:\n  patch cannot be empty"));
        assert!(!message.contains("| ^"));
        assert!(!message.contains("failed file groups:"));

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_patch_failure_with_persisted_patch_source() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.py"), "value = 1\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Update File: app.py\n@@\n-missing = 1\n+value = 2\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("patch should fail");

        let message = err.to_string();
        assert!(message.contains("MATCH_INVALID_CONTEXT"));
        assert!(message.contains("patch context did not match target file"));
        assert!(!message.contains("<patch>:4:1"));
        assert!(message.contains("4 | -missing = 1"));
        assert_eq!(
            message
                .matches("re-read the target file and regenerate the patch with fresh context")
                .count(),
            1
        );
        assert!(!message.contains("Patch context could not be matched against the target file"));
        assert!(message.contains("= action: 1"));
        assert!(message.contains("= hunk: 1"));
        assert!(!message.contains("failed file groups:"));
        assert!(message.contains("\ncaused by:\n"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);

        Ok(())
    })
}

#[tokio::test]
async fn server_default_header_only_treats_dense_numbered_old_side_as_literal_text()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.py"), "value = 1\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({
                    "patch": "*** Begin Patch\n*** Update File: app.py\n@@\n-1 | value = 1\n+value = 2\n*** End Patch\n"
                }))),
                task: None,
            })
            .await
            .expect_err("dense numbered evidence should fail under default header_only mode");

        let message = err.to_string();
        assert!(message.contains("MATCH_INVALID_CONTEXT"));
        assert!(message.contains("1 | value = 1"));
        Ok(())
    })
}

#[tokio::test]
async fn server_env_full_enables_dense_numbered_old_side_evidence() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.py"), "value = 1\n")?;
    with_server_client!(
        temp.path(),
        |cmd| {
            cmd.env("DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE", "full");
        },
        client,
        {
            client
                .call_tool(support::workspace_tool_call(temp.path()))
                .await?;

            let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({
                    "patch": "*** Begin Patch\n*** Update File: app.py\n@@\n-1 | value = 1\n+value = 2\n*** End Patch\n"
                }))),
                task: None,
            })
            .await?;

            let message = &patch_result.content[0].as_text().unwrap().text;
            assert!(message.contains("Success. Updated the following files:"));
            assert_eq!(
                std::fs::read_to_string(temp.path().join("app.py"))?,
                "value = 2\n"
            );
            Ok(())
        }
    )
}

#[tokio::test]
async fn server_accepts_duplicate_first_old_side_after_numbered_header() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(
        temp.path().join("app.py"),
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(support::workspace_tool_call(temp.path()))
            .await?;

        let patch_result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({
                    "patch": "*** Begin Patch\n*** Update File: app.py\n@@ 4 | def handler():\n-def handler():\n-    value = 1\n+def handler():\n+    value = 2\n*** End Patch\n"
                }))),
                task: None,
            })
            .await?;

        let message = &patch_result.content[0].as_text().unwrap().text;
        assert!(message.contains("Success. Updated the following files:"));
        assert_eq!(
            std::fs::read_to_string(temp.path().join("app.py"))?,
            "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
        );
        Ok(())
    })
}

#[tokio::test]
async fn server_reports_first_removed_line_when_context_precedes_mismatch() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.py"), "context\nother\nvalue = 1\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Update File: app.py\n@@\n context\n other\n-missing = 1\n+value = 2\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("patch should fail");

        let message = err.to_string();
        assert!(!message.contains("<patch>:6:1"));
        assert!(message.contains("6 | -missing = 1"));
        assert!(!message.contains("3 | @@"));

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_target_anchor_for_context_guided_mismatch() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(
        temp.path().join("app.py"),
        "def handler():\n    value = 1\n",
    )?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Update File: app.py\n@@ 1 | def handler():\n-    missing = 1\n+    value = 2\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("patch should fail");

        let message = err.to_string();
        assert!(message.contains("MATCH_INVALID_CONTEXT"));
        assert!(message.contains("= target anchor: app.py:1:1"));
        assert!(!message.contains("<patch>:4:1"));
        assert!(message.contains("4 | -    missing = 1"));
        assert!(message.contains("1 | def handler():"));
        assert!(message.contains("^"));
        assert_eq!(
            message
                .matches("re-read the target file and regenerate the patch with fresh context")
                .count(),
            1
        );
        assert!(message.contains("Failed to find expected lines in"));
        assert!(!message.contains("Patch context could not be matched against the target file"));
        assert!(!message.contains("failed file groups:"));
        assert!(message.contains("\ncaused by:\n"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_commit_stage_source_span_for_write_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("name.txt"), "from\n")?;
    std::fs::write(temp.path().join("blocked"), "not a directory\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Update File: src/name.txt\n*** Move to: blocked/dir/name.txt\n@@\n-from\n+new\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("patch should fail");

        let message = err.to_string();
        assert!(message.contains("TARGET_READ_ERROR"));
        assert!(!message.contains("<patch>:3:1"));
        assert!(message.contains("3 | *** Move to: blocked/dir/name.txt"));
        assert!(message.contains("= action: 1"));
        assert_eq!(
            message
                .matches("repair the target path permissions or filesystem state and retry")
                .count(),
            1
        );
        assert!(!message.contains("failed file groups:"));
        assert!(message.contains("\ncaused by:\n"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_update_target_missing_as_compact_full_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Update File: missing.txt\n@@\n-old\n+new\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("missing update target should fail");

        let message = err.to_string();
        assert!(message.contains("UPDATE_TARGET_MISSING"));
        assert!(!message.contains("<patch>:2:1"));
        assert!(message.contains("2 | *** Update File: missing.txt"));
        assert_eq!(
            message
                .matches("create the file first or use Add File if you intend to create it")
                .count(),
            1
        );
        assert!(message.contains("Failed to read file to update"));
        assert!(!message.contains("failed file groups:"));
        assert!(message.contains("\ncaused by:\n"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_delete_target_missing_as_compact_full_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({
                    "patch": "*** Begin Patch\n*** Delete File: missing.txt\n*** End Patch\n"
                }))),
                task: None,
            })
            .await
            .expect_err("missing delete target should fail");

        let message = err.to_string();
        assert!(message.contains("DELETE_TARGET_MISSING"));
        assert!(!message.contains("<patch>:2:1"));
        assert!(message.contains("2 | *** Delete File: missing.txt"));
        assert_eq!(
            message
                .matches("re-read the workspace and remove the delete if the file is already gone")
                .count(),
            1
        );
        assert!(message.contains("Failed to delete file"));
        assert!(!message.contains("failed file groups:"));
        assert!(message.contains("\ncaused by:\n"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);

        Ok(())
    })
}

#[tokio::test]
async fn server_reports_partial_success_with_applied_files() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": "*** Begin Patch\n*** Add File: created.txt\n+hello\n*** Update File: missing.txt\n@@\n-old\n+new\n*** End Patch\n"
            }))),
            task: None,
        })
        .await
        .expect_err("partial apply should still surface as failure");

        let message = err.to_string();
        assert!(message.contains("PARTIAL_UNIT_FAILURE"));
        assert!(message.contains("patch partially applied"));
        assert!(!message.contains("<patch>:4:1"));
        assert!(message.contains("4 | *** Update File: missing.txt"));
        assert!(message.contains("committed changes:"));
        assert!(message.contains("A created.txt"));
        assert!(message.contains("failed file groups:"));
        assert!(message.contains("error[UPDATE_TARGET_MISSING]"));
        assert!(message.contains("M missing.txt"));
        assert!(message.contains("retry only the failing group"));
        assert!(message.contains("do not reapply committed groups unchanged"));
        assert_eq!(
            message
                .matches("create the file first or use Add File if you intend to create it")
                .count(),
            1
        );
        assert!(!message.contains("full committed change list written to"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);
        assert_eq!(
            std::fs::read_to_string(temp.path().join("created.txt"))?,
            "hello\n"
        );

        Ok(())
    })
}

#[tokio::test]
async fn server_enumerates_large_committed_file_lists_without_omission_prose() -> anyhow::Result<()>
{
    let temp = tempfile::tempdir()?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let mut patch = String::from("*** Begin Patch\n");
        for index in 0..10 {
            patch.push_str(&format!(
                "*** Add File: created-{index}.txt\n+hello {index}\n"
            ));
        }
        patch.push_str("*** Update File: missing.txt\n@@\n-old\n+new\n*** End Patch\n");

        let err = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "apply_patch".into(),
                arguments: Some(json_object(json!({ "patch": patch }))),
                task: None,
            })
            .await
            .expect_err("partial apply should fail with summary");

        let message = err.to_string();
        assert!(message.contains("committed changes:"));
        assert!(!message.contains("showing 8 of 10"));
        assert!(!message.contains("... and 2 more committed changes"));
        assert!(!message.contains("full committed change list written to"));
        assert!(message.contains("A created-0.txt"));
        assert!(message.contains("A created-7.txt"));
        assert!(message.contains("A created-8.txt"));
        assert!(message.contains("A created-9.txt"));
        assert_no_audit_patch_artifacts(temp.path(), &message);
        assert_failed_patch_source_persisted(temp.path(), &message);

        Ok(())
    })
}

#[tokio::test]
async fn server_rolls_back_failed_move_group() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("name.txt"), "from\n")?;
    std::fs::write(temp.path().join("blocked"), "not a directory\n")?;
    with_server_client!(temp.path(), client, {
        client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": temp.path() }))),
                task: None,
            })
            .await?;

        let err = client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "apply_patch".into(),
            arguments: Some(json_object(json!({
                "patch": format!(
                    "*** Begin Patch\n*** Update File: {}\n*** Move to: {}\n@@\n-from\n+new\n*** End Patch\n",
                    temp.path().join("src").join("name.txt").display(),
                    temp.path().join("blocked").join("dir").join("name.txt").display()
                )
            }))),
            task: None,
        })
        .await
        .expect_err("move group should fail");

        let message = err.to_string();
        assert!(message.contains("TARGET_READ_ERROR"));
        assert!(!message.contains("failed file groups:"));
        assert_eq!(
            std::fs::read_to_string(temp.path().join("src").join("name.txt"))?,
            "from\n"
        );

        Ok(())
    })
}
