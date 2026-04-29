#[macro_use]
mod support;

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::Value;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use support::{TEST_CHILD_TIMEOUT, json_object, timeout_result};
use tempfile::TempDir;

fn patch_success_text() -> &'static str {
    "*** Begin Patch\n*** Add File: docs/notes.md\n+hello\n*** End Patch\n"
}

fn patch_partial_failure_text() -> &'static str {
    "*** Begin Patch\n*** Add File: created.txt\n+hello\n*** Update File: missing.txt\n@@\n-old\n+new\n*** End Patch\n"
}

fn patch_full_failure_text() -> &'static str {
    "*** Begin Patch\n*** Update File: app.py\n@@\n-missing = 1\n+value = 2\n*** End Patch\n"
}

fn patch_no_op_text() -> &'static str {
    "*** Begin Patch\n*** Update File: app.py\n@@\n-value = 1\n+value = 1\n*** End Patch\n"
}

fn patch_space_target_text() -> &'static str {
    "*** Begin Patch\n*** Update File: space name.txt\n@@\n-old value\n+new value\n*** End Patch\n"
}

fn patch_move_write_failure_text() -> &'static str {
    "*** Begin Patch\n*** Update File: src/name.txt\n*** Move to: blocked/dir/name.txt\n@@\n-from\n+new\n*** End Patch\n"
}

fn patch_empty_add_file_text() -> &'static str {
    "*** Begin Patch\n*** Add File: empty.txt\n*** End Patch\n"
}

fn patch_preserve_crlf_text() -> &'static str {
    "*** Begin Patch\n*** Update File: crlf.txt\n@@\n-b\n+x\n*** End Patch\n"
}

fn patch_preserve_no_final_newline_text() -> &'static str {
    "*** Begin Patch\n*** Update File: no_newline.txt\n@@\n-no newline at end\n+first line\n+second line\n*** End Patch\n"
}

fn patch_dense_numbered_old_side_text() -> &'static str {
    "*** Begin Patch\n*** Update File: app.py\n@@\n-1 | value = 1\n+value = 2\n*** End Patch\n"
}

fn splice_success_text() -> &'static str {
    "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Append To File: dest.txt\n*** End Splice\n"
}

fn splice_failure_text() -> &'static str {
    "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Insert Before In File: missing.txt\n@@\n1 | alpha\n*** End Splice\n"
}

fn splice_partial_failure_text() -> &'static str {
    "*** Begin Splice\n*** Copy From File: source-a.txt\n@@\n1 | alpha\n*** Append To File: dest-a.txt\n*** Copy From File: source-b.txt\n@@\n1 | beta\n*** Insert Before In File: missing.txt\n@@\n1 | beta\n*** End Splice\n"
}

fn rewrite_success_text() -> &'static str {
    "*** Begin Rewrite\n*** Update File: app.txt\n@@ replace the selected line\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n"
}

fn rewrite_failure_text() -> &'static str {
    "*** Begin Rewrite\n*** Update File: app.txt\n@@\n1 | wrong\n*** Delete\n*** End Rewrite\n"
}

fn rewrite_move_overwrite_text() -> &'static str {
    "*** Begin Rewrite\n*** Update File: from.txt\n*** Move to: to.txt\n@@\n1 | old\n*** With\nnew\n*** End With\n*** End Rewrite\n"
}

fn rewrite_delete_missing_text() -> &'static str {
    "*** Begin Rewrite\n*** Delete File: missing.txt\n*** End Rewrite\n"
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

    fn configure_tokio_command(&self, command: &mut tokio::process::Command) {
        command.env("DOCUTOUCH_PUEUE_BIN", &self.bin_path);
        command.env("DOCUTOUCH_PUEUE_RUNTIME_DIR", &self.runtime_dir);
        command.env("DOCUTOUCH_TEST_PUEUE_PLAN", &self.plan_path);
        command.env("DOCUTOUCH_TEST_PUEUE_COUNTER", &self.counter_path);
    }

    fn configure_std_command(&self, command: &mut Command) {
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

fn normalize_simple_tool_error(text: &str) -> &str {
    text.strip_prefix("Mcp error: -32602: ").unwrap_or(text)
}

fn noisy_pueue_log() -> &'static str {
    "phase 1\rphase 2\n\u{1b}[32mDONE\u{1b}[0m\n"
}

async fn call_server_tool(
    cwd: &std::path::Path,
    name: &str,
    arguments: serde_json::Value,
) -> anyhow::Result<String> {
    with_server_client!(cwd, client, {
        timeout_result(
            format!("set_workspace before server tool `{name}` in cli_smoke"),
            client.call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": cwd }))),
                task: None,
            }),
        )
        .await?;
        let result = timeout_result(
            format!("server tool `{name}` in cli_smoke"),
            client.call_tool(CallToolRequestParams {
                meta: None,
                name: name.to_string().into(),
                arguments: Some(json_object(arguments)),
                task: None,
            }),
        )
        .await?;
        Ok(result.content[0].as_text().unwrap().text.clone())
    })
}

async fn call_server_tool_error(
    cwd: &std::path::Path,
    name: &str,
    arguments: serde_json::Value,
) -> anyhow::Result<String> {
    with_server_client!(cwd, client, {
        timeout_result(
            format!("set_workspace before failing server tool `{name}` in cli_smoke"),
            client.call_tool(CallToolRequestParams {
                meta: None,
                name: "set_workspace".into(),
                arguments: Some(json_object(json!({ "path": cwd }))),
                task: None,
            }),
        )
        .await?;
        let err = timeout_result(
            format!("failing server tool `{name}` in cli_smoke"),
            client.call_tool(CallToolRequestParams {
                meta: None,
                name: name.to_string().into(),
                arguments: Some(json_object(arguments)),
                task: None,
            }),
        )
        .await
        .expect_err("tool call should fail");
        Ok(err.to_string())
    })
}

async fn call_server_tool_with_config(
    cwd: &Path,
    name: &str,
    arguments: serde_json::Value,
    configure: impl FnOnce(&mut tokio::process::Command),
) -> anyhow::Result<String> {
    with_server_client!(
        cwd,
        |cmd| {
            configure(cmd);
        },
        client,
        {
            timeout_result(
                format!(
                    "set_workspace before server tool `{name}` in cli_smoke with command config"
                ),
                client.call_tool(CallToolRequestParams {
                    meta: None,
                    name: "set_workspace".into(),
                    arguments: Some(json_object(json!({ "path": cwd }))),
                    task: None,
                }),
            )
            .await?;
            let result = timeout_result(
                format!("server tool `{name}` in cli_smoke with command config"),
                client.call_tool(CallToolRequestParams {
                    meta: None,
                    name: name.to_string().into(),
                    arguments: Some(json_object(arguments)),
                    task: None,
                }),
            )
            .await?;
            Ok(result.content[0].as_text().unwrap().text.clone())
        }
    )
}

async fn call_server_tool_error_with_config(
    cwd: &Path,
    name: &str,
    arguments: serde_json::Value,
    configure: impl FnOnce(&mut tokio::process::Command),
) -> anyhow::Result<String> {
    with_server_client!(
        cwd,
        |cmd| {
            configure(cmd);
        },
        client,
        {
            timeout_result(
                format!(
                    "set_workspace before failing server tool `{name}` in cli_smoke with command config"
                ),
                client.call_tool(CallToolRequestParams {
                    meta: None,
                    name: "set_workspace".into(),
                    arguments: Some(json_object(json!({ "path": cwd }))),
                    task: None,
                }),
            )
            .await?;
            let err = timeout_result(
                format!("failing server tool `{name}` in cli_smoke with command config"),
                client.call_tool(CallToolRequestParams {
                    meta: None,
                    name: name.to_string().into(),
                    arguments: Some(json_object(arguments)),
                    task: None,
                }),
            )
            .await
            .expect_err("tool call should fail");
            Ok(err.to_string())
        }
    )
}

fn run_cli(cwd: &std::path::Path, args: &[&str], stdin: Option<&str>) -> anyhow::Result<Output> {
    support::run_cli(cwd, args, stdin)
}

fn run_cli_with_pueue_env(
    cwd: &Path,
    args: &[&str],
    stdin: Option<&str>,
    fixture: &PueueStubFixture,
    extra_envs: &[(&str, &str)],
) -> anyhow::Result<Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_docutouch"));
    command
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    fixture.configure_std_command(&mut command);
    for (key, value) in extra_envs {
        command.env(key, value);
    }
    let mut child = command.spawn()?;
    if let Some(input) = stdin {
        use std::io::Write as _;
        child
            .stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(input.as_bytes())?;
    }
    child.stdin.take();
    support::wait_with_output_timeout(child, "docutouch CLI child with pueue env in cli_smoke")
}

#[tokio::test]
async fn cli_without_args_starts_server() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("seed.txt"), "seed\n")?;

    with_server_client!(temp.path(), client, {
        let result = timeout_result(
            "read_file through bare docutouch server entry in cli_smoke",
            client.call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": temp.path().join("seed.txt")
                }))),
                task: None,
            }),
        )
        .await?;

        assert_eq!(result.content[0].as_text().unwrap().text, "seed\n");
        Ok(())
    })?;
    Ok(())
}

#[tokio::test]
async fn cli_serve_alias_starts_server() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("seed.txt"), "seed\n")?;

    let mut smoke_client = Some(
        ().serve(support::new_smoke_transport_with(temp.path(), |cmd| {
            cmd.arg("serve");
        })?)
        .await?,
    );
    let result = timeout_result("read_file through serve alias in cli_smoke", async {
        let client = smoke_client.as_ref().expect("smoke client");
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "read_file".into(),
                arguments: Some(json_object(json!({
                    "relative_path": temp.path().join("seed.txt")
                }))),
                task: None,
            })
            .await?;
        anyhow::Ok(result)
    })
    .await?;
    drop(smoke_client.take());

    assert_eq!(result.content[0].as_text().unwrap().text, "seed\n");
    Ok(())
}

fn run_cli_with_env(
    cwd: &std::path::Path,
    args: &[&str],
    stdin: Option<&str>,
    envs: &[(&str, &str)],
) -> anyhow::Result<Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_docutouch"));
    command
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    let mut child = command.spawn()?;
    if let Some(input) = stdin {
        use std::io::Write as _;
        child
            .stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(input.as_bytes())?;
    }
    child.stdin.take();
    support::wait_with_output_timeout(child, "docutouch CLI child with env in cli_smoke")
}

fn utf8(output: &[u8]) -> String {
    String::from_utf8(output.to_vec()).expect("utf-8 output")
}

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
        "expected persisted patch path instead of <patch>\n{message}"
    );
}

fn normalize_failed_patch_segments(mut line: String, marker: &str) -> String {
    while let Some(start) = line.find(marker) {
        let suffix = &line[start + marker.len()..];
        let Some(end) = suffix.find(".patch") else {
            break;
        };
        let replace_end = start + marker.len() + end + ".patch".len();
        line.replace_range(start..replace_end, "<failed.patch>");
    }
    line
}

fn normalize_patch_failure_text(text: &str, workspace_root: &std::path::Path) -> String {
    let workspace = workspace_root.display().to_string();
    let verbatim_workspace = format!(r"\\?\{}", workspace);
    text.lines()
        .map(|line| {
            line.strip_prefix("Mcp error: -32602: ")
                .unwrap_or(line)
                .to_string()
        })
        .map(|line| line.replace(&verbatim_workspace, "<workspace>"))
        .map(|line| line.replace(&workspace, "<workspace>"))
        .map(|line| normalize_failed_patch_segments(line, ".docutouch/failed-patches/"))
        .map(|line| normalize_failed_patch_segments(line, ".docutouch\\failed-patches\\"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn timeout_result_reports_hung_future() -> anyhow::Result<()> {
    let err = timeout_result("deliberately hung future in cli_smoke", async {
        tokio::time::sleep(TEST_CHILD_TIMEOUT + Duration::from_millis(50)).await;
        Ok::<(), anyhow::Error>(())
    })
    .await
    .expect_err("hung future should time out");
    assert!(
        err.to_string()
            .contains("timed out waiting for deliberately hung future in cli_smoke")
    );
    Ok(())
}

fn make_search_fixture(temp: &TempDir) -> anyhow::Result<()> {
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("one.txt"), "alpha\nalpha\n")?;
    std::fs::write(temp.path().join("src").join("two.txt"), "alpha\n")?;
    Ok(())
}

fn make_search_blackbox_fixture(temp: &TempDir) -> anyhow::Result<()> {
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(
        temp.path().join("src").join("one.txt"),
        "alpha\nbeta\nwarning[\n{ref:alpha}\nalpha alpha\nline.with.dots\n",
    )?;
    std::fs::write(
        temp.path().join("src").join("two.txt"),
        "before context\nalpha\nafter context\n",
    )?;
    std::fs::write(temp.path().join("src").join("three.txt"), "no hits here\n")?;
    std::fs::write(
        temp.path().join("docs").join("guide.md"),
        "{ref:doc}\nwarning[\n",
    )?;
    Ok(())
}

fn make_read_fixture(temp: &TempDir) -> anyhow::Result<()> {
    std::fs::write(
        temp.path().join("notes.txt"),
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n",
    )?;
    Ok(())
}

fn make_list_type_fixture(temp: &TempDir) -> anyhow::Result<()> {
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::create_dir_all(temp.path().join("docs"))?;
    std::fs::write(temp.path().join("src").join("main.rs"), "fn main() {}\n")?;
    std::fs::write(temp.path().join("src").join("main.cpp"), "int main() {}\n")?;
    std::fs::write(temp.path().join("docs").join("guide.md"), "# Guide\n")?;
    Ok(())
}

#[tokio::test]
async fn cli_list_can_filter_by_ripgrep_file_type() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    make_list_type_fixture(&temp)?;

    let cli_output = run_cli(temp.path(), &["list", ".", "-trust", "-Tmarkdown"], None)?;
    assert!(cli_output.status.success());
    let stdout = utf8(&cli_output.stdout);
    assert!(stdout.contains("main.rs"));
    assert!(!stdout.contains("main.cpp"));
    assert!(!stdout.contains("guide.md"));
    assert!(stdout.contains("2 type"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_preview_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_search_fixture(&server_temp)?;
    make_search_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "search_text",
        json!({
            "query": "alpha",
            "path": "src"
        }),
    )
    .await?;

    let cli_output = run_cli(cli_temp.path(), &["cli", "search", "alpha", "src"], None)?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_full_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_search_fixture(&server_temp)?;
    make_search_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "search_text",
        json!({
            "query": "alpha",
            "path": "src",
            "view": "full"
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["search", "alpha", "src", "--view", "full"],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_auto_literal_fallback_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_search_blackbox_fixture(&server_temp)?;
    make_search_blackbox_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "search_text",
        json!({
            "query": "{ref:",
            "path": ".",
            "view": "preview"
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["search", "{ref:", ".", "--view", "preview"],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stdout).contains("query_interpretation: literal_fallback"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_counts_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_search_blackbox_fixture(&server_temp)?;
    make_search_blackbox_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "search_text",
        json!({
            "query": "alpha",
            "path": ".",
            "rg_args": "--count-matches"
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["search", "alpha", ".", "--rg-args", "--count-matches"],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stdout).contains("search_text[counts]:"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_queryless_files_raw_text_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_search_blackbox_fixture(&server_temp)?;
    make_search_blackbox_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "search_text",
        json!({
            "query": "",
            "path": ".",
            "rg_args": "--files",
            "output_mode": "raw_text"
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &[
            "search",
            "",
            ".",
            "--rg-args",
            "--files",
            "--output-mode",
            "raw_text",
        ],
        None,
    )?;
    assert!(cli_output.status.success());
    let cli_text = utf8(&cli_output.stdout);
    let cli_lines = cli_text.lines().collect::<std::collections::BTreeSet<_>>();
    let server_lines = server_output
        .lines()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(cli_lines, server_lines);
    assert!(cli_text.contains("src/one.txt"));
    assert!(!cli_text.contains(cli_temp.path().to_string_lossy().as_ref()));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_wait_pueue_any_returns_terminal_task_summary() -> anyhow::Result<()> {
    let cli_temp = tempfile::tempdir()?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[
            status_snapshot(vec![running_task(501)]),
            status_snapshot(vec![done_task(501, "Success", 0)]),
        ],
        &[(501, "alpha\n")],
    )?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["wait-pueue", "501", "--timeout-seconds", "5"],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(cli_output.status.success());
    let text = utf8(&cli_output.stdout);
    assert!(text.contains("reason: task_finished"));
    assert!(text.contains("mode: any"));
    assert!(text.contains("resolved_task_ids: 501"));
    assert!(text.contains("triggered_task_ids: 501"));
    assert!(text.contains("status: Success"));
    assert!(text.contains("log_handle: pueue-log:501"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_wait_pueue_all_preserves_resolved_order() -> anyhow::Result<()> {
    let cli_temp = tempfile::tempdir()?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[
            status_snapshot(vec![running_task(601), running_task(602)]),
            status_snapshot(vec![done_task(601, "Success", 0), running_task(602)]),
            status_snapshot(vec![
                done_task(601, "Success", 0),
                done_task(602, "Success", 0),
            ]),
        ],
        &[(601, "one\n"), (602, "two\n")],
    )?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &[
            "wait-pueue",
            "602",
            "601",
            "602",
            "--mode",
            "all",
            "--timeout-seconds",
            "5",
        ],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(cli_output.status.success());
    let text = utf8(&cli_output.stdout);
    assert!(text.contains("reason: all_finished"));
    assert!(text.contains("mode: all"));
    assert!(text.contains("resolved_task_ids: 602, 601"));
    assert!(text.contains("triggered_task_ids: 602, 601"));
    assert!(text.contains("[1] task 602"));
    assert!(text.contains("[2] task 601"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_wait_pueue_uses_default_timeout_when_omitted() -> anyhow::Result<()> {
    let cli_temp = tempfile::tempdir()?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[
            status_snapshot(vec![done_task(702, "Success", 0), running_task(701)]),
            status_snapshot(vec![done_task(702, "Success", 0), running_task(701)]),
            status_snapshot(vec![done_task(702, "Success", 0), running_task(701)]),
        ],
        &[(701, "pending\n"), (702, "done\n")],
    )?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["wait-pueue", "701", "702", "--mode", "all"],
        None,
        &cli_fixture,
        &[("DOCUTOUCH_PUEUE_TIMEOUT_SECONDS", "0.1")],
    )?;
    assert!(cli_output.status.success());
    let text = utf8(&cli_output.stdout);
    assert!(text.contains("reason: timeout"));
    assert!(text.contains("mode: all"));
    assert!(text.contains("resolved_task_ids: 701, 702"));
    assert!(text.contains("triggered_task_ids: 702"));
    assert!(text.contains("pending_task_ids: 701"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_read_pueue_log_handle_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let server_fixture = PueueStubFixture::new(
        server_temp.path(),
        &[status_snapshot(vec![done_task(801, "Success", 0)])],
        &[(801, "alpha\nbeta\n")],
    )?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[status_snapshot(vec![done_task(801, "Success", 0)])],
        &[(801, "alpha\nbeta\n")],
    )?;

    let server_output = call_server_tool_with_config(
        server_temp.path(),
        "read_file",
        json!({
            "relative_path": "pueue-log:801"
        }),
        |cmd| {
            server_fixture.configure_tokio_command(cmd);
        },
    )
    .await?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["read", "pueue-log:801"],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert_eq!(utf8(&cli_output.stdout), "alpha\nbeta\n");
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_pueue_log_handle_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("notes.txt"), "alpha\n")?;
    std::fs::write(cli_temp.path().join("notes.txt"), "alpha\n")?;
    let server_fixture = PueueStubFixture::new(
        server_temp.path(),
        &[status_snapshot(vec![done_task(802, "Success", 0)])],
        &[(802, "alpha\n")],
    )?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[status_snapshot(vec![done_task(802, "Success", 0)])],
        &[(802, "alpha\n")],
    )?;

    let server_output = call_server_tool_with_config(
        server_temp.path(),
        "search_text",
        json!({
            "query": "alpha",
            "path": ["pueue-log:802", "notes.txt"],
            "view": "full"
        }),
        |cmd| {
            server_fixture.configure_tokio_command(cmd);
        },
    )
    .await?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &[
            "search",
            "alpha",
            "pueue-log:802",
            "notes.txt",
            "--view",
            "full",
        ],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stdout).contains("scope: [pueue-log:802, notes.txt]"));
    assert!(utf8(&cli_output.stdout).contains("pueue-log:802 (1 line, 1 match)"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_read_pueue_log_handle_surfaces_clean_text() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let server_fixture = PueueStubFixture::new(
        server_temp.path(),
        &[status_snapshot(vec![done_task(804, "Success", 0)])],
        &[(804, noisy_pueue_log())],
    )?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[status_snapshot(vec![done_task(804, "Success", 0)])],
        &[(804, noisy_pueue_log())],
    )?;

    let server_output = call_server_tool_with_config(
        server_temp.path(),
        "read_file",
        json!({
            "relative_path": "pueue-log:804"
        }),
        |cmd| {
            server_fixture.configure_tokio_command(cmd);
        },
    )
    .await?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["read", "pueue-log:804"],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert_eq!(utf8(&cli_output.stdout), "phase 2\nDONE\n");
    assert!(!utf8(&cli_output.stdout).contains("phase 1"));
    assert!(!utf8(&cli_output.stdout).contains("\u{1b}"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_search_pueue_log_handle_uses_clean_text_surface() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let server_fixture = PueueStubFixture::new(
        server_temp.path(),
        &[status_snapshot(vec![done_task(805, "Success", 0)])],
        &[(805, noisy_pueue_log())],
    )?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[status_snapshot(vec![done_task(805, "Success", 0)])],
        &[(805, noisy_pueue_log())],
    )?;

    let server_output = call_server_tool_with_config(
        server_temp.path(),
        "search_text",
        json!({
            "query": "DONE",
            "path": "pueue-log:805",
            "view": "full"
        }),
        |cmd| {
            server_fixture.configure_tokio_command(cmd);
        },
    )
    .await?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["search", "DONE", "pueue-log:805", "--view", "full"],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stdout).contains("scope: pueue-log:805"));
    assert!(utf8(&cli_output.stdout).contains("pueue-log:805 (1 line, 1 match)"));
    assert!(utf8(&cli_output.stdout).contains("2 | DONE"));
    assert!(!utf8(&cli_output.stdout).contains("phase 1"));
    assert!(!utf8(&cli_output.stdout).contains("\u{1b}"));
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_missing_pueue_task_error_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let server_fixture =
        PueueStubFixture::new(server_temp.path(), &[status_snapshot(vec![])], &[])?;
    let cli_fixture = PueueStubFixture::new(cli_temp.path(), &[status_snapshot(vec![])], &[])?;

    let server_error = call_server_tool_error_with_config(
        server_temp.path(),
        "read_file",
        json!({
            "relative_path": "pueue-log:999"
        }),
        |cmd| {
            server_fixture.configure_tokio_command(cmd);
        },
    )
    .await?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["read", "pueue-log:999"],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(!cli_output.status.success());
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        utf8(&cli_output.stderr),
        normalize_simple_tool_error(&server_error)
    );
    Ok(())
}

#[tokio::test]
async fn cli_missing_pueue_log_error_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let server_fixture = PueueStubFixture::new(
        server_temp.path(),
        &[status_snapshot(vec![running_task(803)])],
        &[],
    )?;
    let cli_fixture = PueueStubFixture::new(
        cli_temp.path(),
        &[status_snapshot(vec![running_task(803)])],
        &[],
    )?;

    let server_error = call_server_tool_error_with_config(
        server_temp.path(),
        "search_text",
        json!({
            "query": "alpha",
            "path": "pueue-log:803"
        }),
        |cmd| {
            server_fixture.configure_tokio_command(cmd);
        },
    )
    .await?;

    let cli_output = run_cli_with_pueue_env(
        cli_temp.path(),
        &["search", "alpha", "pueue-log:803"],
        None,
        &cli_fixture,
        &[],
    )?;
    assert!(!cli_output.status.success());
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        utf8(&cli_output.stderr),
        normalize_simple_tool_error(&server_error)
    );
    Ok(())
}

#[tokio::test]
async fn cli_patch_success_matches_mcp_output_and_reads_stdin() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::create_dir_all(server_temp.path().join("docs"))?;
    std::fs::create_dir_all(cli_temp.path().join("docs"))?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_success_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["cli", "patch"],
        Some(patch_success_text()),
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("docs").join("notes.md"))?,
        "hello\n"
    );
    Ok(())
}

#[test]
fn cli_patch_flag_full_enables_dense_numbered_old_side_evidence() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.py"), "value = 1\n")?;

    let cli_output = run_cli(
        temp.path(),
        &["patch", "--numbered-evidence-mode", "full"],
        Some(patch_dense_numbered_old_side_text()),
    )?;

    assert!(cli_output.status.success());
    assert_eq!(
        utf8(&cli_output.stdout),
        "Success. Updated the following files:\nM app.py"
    );
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("app.py"))?,
        "value = 2\n"
    );
    Ok(())
}

#[test]
fn cli_patch_flag_overrides_full_env_with_header_only_for_literal_numbered_text()
-> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("app.py"), "121 | value = 1\n")?;
    let patch = "*** Begin Patch\n*** Update File: app.py\n@@\n-121 | value = 1\n+value = 2\n*** End Patch\n";

    let cli_output = run_cli_with_env(
        temp.path(),
        &["patch", "--numbered-evidence-mode", "header_only"],
        Some(patch),
        &[("DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE", "full")],
    )?;

    assert!(cli_output.status.success());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("app.py"))?,
        "value = 2\n"
    );
    Ok(())
}

#[tokio::test]
async fn cli_splice_success_matches_mcp_output_and_reads_stdin() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("source.txt"), "alpha\n")?;
    std::fs::write(cli_temp.path().join("source.txt"), "alpha\n")?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_splice",
        json!({ "splice": splice_success_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["cli", "splice"],
        Some(splice_success_text()),
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("dest.txt"))?,
        "alpha\n"
    );
    Ok(())
}

#[tokio::test]
async fn cli_splice_failure_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("source.txt"), "alpha\n")?;
    std::fs::write(cli_temp.path().join("source.txt"), "alpha\n")?;

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_splice",
        json!({ "splice": splice_failure_text() }),
    )
    .await?;

    let cli_output = run_cli(cli_temp.path(), &["splice"], Some(splice_failure_text()))?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    let normalized_cli = normalize_patch_failure_text(&cli_stderr, cli_temp.path());
    let normalized_server = normalize_patch_failure_text(&server_error, server_temp.path());
    assert_eq!(normalized_cli, normalized_server);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert!(normalized_cli.contains("error[SPLICE_TARGET_STATE_INVALID]"));
    assert!(normalized_cli.contains("target state is invalid for this action"));
    assert!(normalized_cli.contains("target_anchor: missing.txt:1:1"));
    assert!(normalized_cli.contains("help: repair the target path or target selection"));
    Ok(())
}

#[tokio::test]
async fn cli_splice_partial_failure_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("source-a.txt"), "alpha\n")?;
    std::fs::write(server_temp.path().join("source-b.txt"), "beta\n")?;
    std::fs::write(cli_temp.path().join("source-a.txt"), "alpha\n")?;
    std::fs::write(cli_temp.path().join("source-b.txt"), "beta\n")?;

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_splice",
        json!({ "splice": splice_partial_failure_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["splice"],
        Some(splice_partial_failure_text()),
    )?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    let normalized_cli = normalize_patch_failure_text(&cli_stderr, cli_temp.path());
    let normalized_server = normalize_patch_failure_text(&server_error, server_temp.path());
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(normalized_cli, normalized_server);
    assert!(normalized_cli.contains("error[SPLICE_PARTIAL_UNIT_FAILURE]"));
    assert!(normalized_cli.contains("splice partially applied"));
    assert!(normalized_cli.contains("committed changes:"));
    assert!(normalized_cli.contains("A dest-a.txt"));
    assert!(normalized_cli.contains("failed_unit[1]:"));
    assert!(normalized_cli.contains("error[SPLICE_TARGET_STATE_INVALID]"));
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("dest-a.txt"))?,
        "alpha\n"
    );
    Ok(())
}

#[tokio::test]
async fn cli_rewrite_success_matches_mcp_output_and_reads_stdin() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("app.txt"), "old\n")?;
    std::fs::write(cli_temp.path().join("app.txt"), "old\n")?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_rewrite",
        json!({ "rewrite": rewrite_success_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["cli", "rewrite"],
        Some(rewrite_success_text()),
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("app.txt"))?,
        "new"
    );
    Ok(())
}

#[tokio::test]
async fn cli_rewrite_failure_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("app.txt"), "old\n")?;
    std::fs::write(cli_temp.path().join("app.txt"), "old\n")?;

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_rewrite",
        json!({ "rewrite": rewrite_failure_text() }),
    )
    .await?;

    let cli_output = run_cli(cli_temp.path(), &["rewrite"], Some(rewrite_failure_text()))?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    let normalized_cli = normalize_patch_failure_text(&cli_stderr, cli_temp.path());
    let normalized_server = normalize_patch_failure_text(&server_error, server_temp.path());
    assert_eq!(normalized_cli, normalized_server);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert!(normalized_cli.contains("error[REWRITE_SELECTION_INVALID]"));
    Ok(())
}

#[tokio::test]
async fn cli_rewrite_move_overwrite_warning_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("from.txt"), "old\n")?;
    std::fs::write(server_temp.path().join("to.txt"), "dest\n")?;
    std::fs::write(cli_temp.path().join("from.txt"), "old\n")?;
    std::fs::write(cli_temp.path().join("to.txt"), "dest\n")?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_rewrite",
        json!({ "rewrite": rewrite_move_overwrite_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["rewrite"],
        Some(rewrite_move_overwrite_text()),
    )?;
    assert!(cli_output.status.success());
    let cli_stdout = utf8(&cli_output.stdout);
    assert_eq!(cli_stdout, server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert!(cli_stdout.contains("A to.txt"));
    assert!(cli_stdout.contains("D from.txt"));
    assert!(!cli_stdout.contains("M to.txt"));
    assert!(cli_stdout.contains("Warnings:"));
    assert!(cli_stdout.contains("MOVE_REPLACED_EXISTING_DESTINATION"));
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("to.txt"))?,
        "new"
    );
    Ok(())
}

#[tokio::test]
async fn cli_rewrite_delete_missing_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_rewrite",
        json!({ "rewrite": rewrite_delete_missing_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["rewrite"],
        Some(rewrite_delete_missing_text()),
    )?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    let normalized_cli = normalize_patch_failure_text(&cli_stderr, cli_temp.path());
    let normalized_server = normalize_patch_failure_text(&server_error, server_temp.path());
    assert_eq!(normalized_cli, normalized_server);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert!(normalized_cli.contains("error[REWRITE_TARGET_STATE_INVALID]"));
    assert!(normalized_cli.contains("delete target does not exist"));
    Ok(())
}

#[tokio::test]
async fn cli_patch_partial_failure_matches_mcp_inline_diagnostics() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_partial_failure_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["patch"],
        Some(patch_partial_failure_text()),
    )?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        normalize_patch_failure_text(&cli_stderr, cli_temp.path()),
        normalize_patch_failure_text(&server_error, server_temp.path())
    );
    assert_failed_patch_source_persisted(server_temp.path(), &server_error);
    assert_failed_patch_source_persisted(cli_temp.path(), &cli_stderr);
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("created.txt"))?,
        "hello\n"
    );
    Ok(())
}

#[tokio::test]
async fn cli_patch_single_full_failure_matches_mcp_inline_diagnostics() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("app.py"), "value = 1\n")?;
    std::fs::write(cli_temp.path().join("app.py"), "value = 1\n")?;

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_full_failure_text() }),
    )
    .await?;

    let cli_output = run_cli(cli_temp.path(), &["patch"], Some(patch_full_failure_text()))?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        normalize_patch_failure_text(&cli_stderr, cli_temp.path()),
        normalize_patch_failure_text(&server_error, server_temp.path())
    );
    assert!(!cli_stderr.contains("failed file groups:"));
    assert!(!server_error.contains("failed file groups:"));
    assert_eq!(
        cli_stderr
            .matches("re-read the target file and regenerate the patch with fresh context")
            .count(),
        1
    );
    assert_failed_patch_source_persisted(server_temp.path(), &server_error);
    assert_failed_patch_source_persisted(cli_temp.path(), &cli_stderr);
    Ok(())
}

#[tokio::test]
async fn cli_patch_empty_input_matches_mcp_inline_diagnostics() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;

    let server_error =
        call_server_tool_error(server_temp.path(), "apply_patch", json!({ "patch": "" })).await?;

    let cli_output = run_cli(cli_temp.path(), &["patch"], Some(""))?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        normalize_patch_failure_text(&cli_stderr, cli_temp.path()),
        normalize_patch_failure_text(&server_error, server_temp.path())
    );
    assert!(cli_stderr.contains("OUTER_EMPTY_PATCH"));
    assert!(cli_stderr.contains("patch cannot be empty"));
    assert!(!cli_stderr.contains("| ^"));
    assert_failed_patch_source_persisted(server_temp.path(), &server_error);
    assert_failed_patch_source_persisted(cli_temp.path(), &cli_stderr);
    Ok(())
}

#[tokio::test]
async fn cli_patch_empty_add_file_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_empty_add_file_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["patch"],
        Some(patch_empty_add_file_text()),
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read(cli_temp.path().join("empty.txt"))?,
        Vec::<u8>::new()
    );
    Ok(())
}

#[tokio::test]
async fn cli_patch_preserves_crlf_bytes_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let original = b"a\r\nb\r\nc\r\n";
    let expected = b"a\r\nx\r\nc\r\n";
    std::fs::write(server_temp.path().join("crlf.txt"), original)?;
    std::fs::write(cli_temp.path().join("crlf.txt"), original)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_preserve_crlf_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["patch"],
        Some(patch_preserve_crlf_text()),
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(std::fs::read(cli_temp.path().join("crlf.txt"))?, expected);
    Ok(())
}

#[tokio::test]
async fn cli_patch_preserves_missing_final_newline_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    let original = b"no newline at end";
    let expected = b"first line\nsecond line";
    std::fs::write(server_temp.path().join("no_newline.txt"), original)?;
    std::fs::write(cli_temp.path().join("no_newline.txt"), original)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_preserve_no_final_newline_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["patch"],
        Some(patch_preserve_no_final_newline_text()),
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read(cli_temp.path().join("no_newline.txt"))?,
        expected
    );
    Ok(())
}

#[tokio::test]
async fn cli_patch_no_op_positional_patch_file_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("app.py"), "value = 1\n")?;
    std::fs::write(cli_temp.path().join("app.py"), "value = 1\n")?;
    let cli_patch_path = cli_temp.path().join("input.patch");
    std::fs::write(&cli_patch_path, patch_no_op_text())?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_no_op_text() }),
    )
    .await?;

    let cli_output = run_cli(cli_temp.path(), &["patch", "input.patch"], None)?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("app.py"))?,
        "value = 1\n"
    );
    Ok(())
}

#[tokio::test]
async fn cli_patch_long_form_patch_file_supports_space_target_paths() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::write(server_temp.path().join("space name.txt"), "old value\n")?;
    std::fs::write(cli_temp.path().join("space name.txt"), "old value\n")?;
    let cli_patch_path = cli_temp.path().join("input patch.txt");
    std::fs::write(&cli_patch_path, patch_space_target_text())?;

    let server_output = call_server_tool(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_space_target_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["patch", "--patch-file", "input patch.txt"],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("space name.txt"))?,
        "new value\n"
    );
    Ok(())
}

#[tokio::test]
async fn cli_patch_move_write_failure_matches_mcp_inline_diagnostics() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    std::fs::create_dir_all(server_temp.path().join("src"))?;
    std::fs::create_dir_all(cli_temp.path().join("src"))?;
    std::fs::write(server_temp.path().join("src").join("name.txt"), "from\n")?;
    std::fs::write(cli_temp.path().join("src").join("name.txt"), "from\n")?;
    std::fs::write(server_temp.path().join("blocked"), "not a directory\n")?;
    std::fs::write(cli_temp.path().join("blocked"), "not a directory\n")?;
    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch_move_write_failure_text() }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &["patch"],
        Some(patch_move_write_failure_text()),
    )?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        normalize_patch_failure_text(&cli_stderr, cli_temp.path()),
        normalize_patch_failure_text(&server_error, server_temp.path())
    );
    assert!(cli_stderr.contains("TARGET_READ_ERROR"));
    assert!(!cli_stderr.contains("failed file groups:"));
    assert_eq!(
        cli_stderr
            .matches("repair the target path permissions or filesystem state and retry")
            .count(),
        1
    );
    assert_failed_patch_source_persisted(server_temp.path(), &server_error);
    assert_failed_patch_source_persisted(cli_temp.path(), &cli_stderr);
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("src").join("name.txt"))?,
        "from\n"
    );
    assert!(
        !cli_temp
            .path()
            .join("blocked")
            .join("dir")
            .join("name.txt")
            .exists()
    );
    Ok(())
}

#[test]
fn cli_patch_file_failure_preserves_original_patch_file_path() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::create_dir_all(temp.path().join("src"))?;
    std::fs::write(temp.path().join("src").join("name.txt"), "from\n")?;
    std::fs::write(temp.path().join("blocked"), "not a directory\n")?;
    std::fs::write(
        temp.path().join("move-fail.patch"),
        patch_move_write_failure_text(),
    )?;

    let cli_output = run_cli(temp.path(), &["patch", "move-fail.patch"], None)?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert!(cli_stderr.contains("move-fail.patch:3:1"));
    assert!(cli_stderr.contains("= patch: move-fail.patch"));
    assert!(!cli_stderr.contains(".docutouch/failed-patches/"));
    assert!(
        !temp
            .path()
            .join(".docutouch")
            .join("failed-patches")
            .exists()
    );
    Ok(())
}

#[test]
fn cli_failed_patch_artifact_recovers_workspace_anchor_outside_invocation_cwd() -> anyhow::Result<()>
{
    let workspace = tempfile::tempdir()?;
    let invocation_cwd = tempfile::tempdir()?;
    std::fs::create_dir_all(workspace.path().join("src"))?;
    std::fs::create_dir_all(workspace.path().join(".docutouch").join("failed-patches"))?;
    std::fs::write(workspace.path().join("src").join("name.txt"), "from\n")?;
    std::fs::write(workspace.path().join("blocked"), "not a directory\n")?;
    let patch_path = workspace
        .path()
        .join(".docutouch")
        .join("failed-patches")
        .join("retry.patch");
    std::fs::write(&patch_path, patch_move_write_failure_text())?;

    let cli_output = run_cli(
        invocation_cwd.path(),
        &["patch", patch_path.to_str().expect("utf-8 patch path")],
        None,
    )?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert!(cli_stderr.contains("TARGET_READ_ERROR"), "{cli_stderr}");
    assert!(
        cli_stderr.contains(".docutouch/failed-patches/retry.patch"),
        "{cli_stderr}"
    );
    assert!(cli_stderr.contains("= patch: .docutouch/failed-patches/retry.patch"));
    assert!(
        !cli_stderr.contains("UPDATE_TARGET_MISSING"),
        "expected recovered workspace anchor instead of invocation-cwd miss\n{cli_stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src").join("name.txt"))?,
        "from\n"
    );
    assert!(!invocation_cwd.path().join("src").join("name.txt").exists());
    Ok(())
}

#[test]
fn cli_regular_patch_file_keeps_invocation_cwd_anchor_outside_patch_directory() -> anyhow::Result<()>
{
    let patch_owner = tempfile::tempdir()?;
    let invocation_cwd = tempfile::tempdir()?;
    let patch_path = patch_owner.path().join("input.patch");
    std::fs::write(&patch_path, patch_success_text())?;

    let cli_output = run_cli(
        invocation_cwd.path(),
        &["patch", patch_path.to_str().expect("utf-8 patch path")],
        None,
    )?;
    assert!(cli_output.status.success());
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(invocation_cwd.path().join("docs").join("notes.md"))?,
        "hello\n"
    );
    assert!(!patch_owner.path().join("docs").join("notes.md").exists());
    Ok(())
}

#[test]
fn cli_splice_file_failure_preserves_original_splice_file_path() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("source.txt"), "alpha\n")?;
    std::fs::write(temp.path().join("broken.splice"), splice_failure_text())?;

    let cli_output = run_cli(temp.path(), &["splice", "broken.splice"], None)?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert!(cli_stderr.contains("broken.splice:6:1"));
    assert!(cli_stderr.contains("error[SPLICE_TARGET_STATE_INVALID]"));
    Ok(())
}

#[tokio::test]
async fn cli_patch_large_partial_failure_enumerates_all_committed_paths() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;

    let mut patch = String::from("*** Begin Patch\n");
    for index in 0..10 {
        patch.push_str(&format!(
            "*** Add File: created-{index}.txt\n+hello {index}\n"
        ));
    }
    patch.push_str("*** Update File: missing.txt\n@@\n-old\n+new\n*** End Patch\n");

    let server_error = call_server_tool_error(
        server_temp.path(),
        "apply_patch",
        json!({ "patch": patch.as_str() }),
    )
    .await?;

    let cli_output = run_cli(cli_temp.path(), &["patch"], Some(&patch))?;
    assert!(!cli_output.status.success());
    let cli_stderr = utf8(&cli_output.stderr);
    assert!(utf8(&cli_output.stdout).is_empty());
    assert_eq!(
        normalize_patch_failure_text(&cli_stderr, cli_temp.path()),
        normalize_patch_failure_text(&server_error, server_temp.path())
    );
    assert!(cli_stderr.contains("committed changes:"));
    assert!(!cli_stderr.contains("showing 8 of 10"));
    assert!(!cli_stderr.contains("... and 2 more committed changes"));
    assert!(cli_stderr.contains("A created-8.txt"));
    assert!(cli_stderr.contains("A created-9.txt"));
    assert_failed_patch_source_persisted(server_temp.path(), &server_error);
    assert_failed_patch_source_persisted(cli_temp.path(), &cli_stderr);
    Ok(())
}

#[test]
fn cli_read_uses_cwd_as_workspace_anchor() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    std::fs::write(temp.path().join("notes.txt"), "line one\nline two\n")?;

    let output = run_cli(temp.path(), &["cli", "read", "notes.txt"], None)?;
    assert!(output.status.success());
    assert_eq!(utf8(&output.stdout), "line one\nline two\n");
    assert!(utf8(&output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_read_sampled_view_matches_mcp_output() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_read_fixture(&server_temp)?;
    make_read_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "read_file",
        json!({
            "relative_path": "notes.txt",
            "line_range": "1:7",
            "sample_step": 5,
            "sample_lines": 2,
            "max_chars": 80
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &[
            "read",
            "notes.txt",
            "--line-range",
            "1:7",
            "--sample-step",
            "5",
            "--sample-lines",
            "2",
            "--max-chars",
            "80",
        ],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_read_partial_sampled_flags_match_mcp_defaults() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_read_fixture(&server_temp)?;
    make_read_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "read_file",
        json!({
            "relative_path": "notes.txt",
            "line_range": "1:7",
            "sample_step": 5
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &[
            "read",
            "notes.txt",
            "--line-range",
            "1:7",
            "--sample-step",
            "5",
        ],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}

#[tokio::test]
async fn cli_read_supports_slice_like_tail_ranges() -> anyhow::Result<()> {
    let server_temp = tempfile::tempdir()?;
    let cli_temp = tempfile::tempdir()?;
    make_read_fixture(&server_temp)?;
    make_read_fixture(&cli_temp)?;

    let server_output = call_server_tool(
        server_temp.path(),
        "read_file",
        json!({
            "relative_path": "notes.txt",
            "line_range": "-3:",
            "show_line_numbers": true
        }),
    )
    .await?;

    let cli_output = run_cli(
        cli_temp.path(),
        &[
            "read",
            "notes.txt",
            "--line-range",
            "-3:",
            "--show-line-numbers",
        ],
        None,
    )?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    Ok(())
}
