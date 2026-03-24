#[macro_use]
mod support;

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::json;
use std::process::Output;
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

fn splice_success_text() -> &'static str {
    "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Append To File: dest.txt\n*** End Splice\n"
}

fn splice_failure_text() -> &'static str {
    "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Insert Before In File: missing.txt\n@@\n1 | alpha\n*** End Splice\n"
}

fn splice_partial_failure_text() -> &'static str {
    "*** Begin Splice\n*** Copy From File: source-a.txt\n@@\n1 | alpha\n*** Append To File: dest-a.txt\n*** Copy From File: source-b.txt\n@@\n1 | beta\n*** Insert Before In File: missing.txt\n@@\n1 | beta\n*** End Splice\n"
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

fn run_cli(cwd: &std::path::Path, args: &[&str], stdin: Option<&str>) -> anyhow::Result<Output> {
    support::run_cli(cwd, args, stdin)
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

fn make_read_fixture(temp: &TempDir) -> anyhow::Result<()> {
    std::fs::write(
        temp.path().join("notes.txt"),
        "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n",
    )?;
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

    let cli_output = run_cli(cli_temp.path(), &["search", "alpha", "src"], None)?;
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

    let cli_output = run_cli(cli_temp.path(), &["patch"], Some(patch_success_text()))?;
    assert!(cli_output.status.success());
    assert_eq!(utf8(&cli_output.stdout), server_output);
    assert!(utf8(&cli_output.stderr).is_empty());
    assert_eq!(
        std::fs::read_to_string(cli_temp.path().join("docs").join("notes.md"))?,
        "hello\n"
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

    let cli_output = run_cli(cli_temp.path(), &["splice"], Some(splice_success_text()))?;
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
    assert!(cli_stderr.contains("TARGET_WRITE_ERROR"));
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

    let output = run_cli(temp.path(), &["read", "notes.txt"], None)?;
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
