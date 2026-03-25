#![allow(dead_code)]

use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::transport::TokioChildProcess;
use serde_json::json;
use std::future::Future;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

pub const TEST_CHILD_TIMEOUT: Duration = Duration::from_secs(15);

macro_rules! with_server_client {
    ($cwd:expr, $client:ident, $body:block) => {{
        let mut smoke_client = Some(().serve(support::new_smoke_transport($cwd)?).await?);
        let result = support::timeout_result(
            concat!("smoke server test at ", file!(), ":", line!()),
            async {
                let $client = smoke_client.as_ref().expect("smoke client");
                let result: anyhow::Result<_> = { $body };
                result
            },
        )
        .await;
        drop(smoke_client.take());
        result
    }};
    ($cwd:expr, |$cmd:ident| $configure:block, $client:ident, $body:block) => {{
        let mut smoke_client = Some(
            ().serve(support::new_smoke_transport_with($cwd, |$cmd| $configure)?)
                .await?,
        );
        let result = support::timeout_result(
            concat!("smoke server test at ", file!(), ":", line!()),
            async {
                let $client = smoke_client.as_ref().expect("smoke client");
                let result: anyhow::Result<_> = { $body };
                result
            },
        )
        .await;
        drop(smoke_client.take());
        result
    }};
}

pub fn json_object(value: serde_json::Value) -> JsonObject {
    serde_json::from_value(value).expect("json object")
}

pub fn tool_call(name: &str, arguments: serde_json::Value) -> CallToolRequestParams {
    CallToolRequestParams {
        meta: None,
        name: name.to_string().into(),
        arguments: Some(json_object(arguments)),
        task: None,
    }
}

pub fn workspace_tool_call(path: &Path) -> CallToolRequestParams {
    tool_call("set_workspace", json!({ "path": path }))
}

pub fn new_smoke_transport(cwd: &Path) -> anyhow::Result<TokioChildProcess> {
    new_smoke_transport_with(cwd, |_| {})
}

pub fn new_smoke_transport_with(
    cwd: &Path,
    configure: impl FnOnce(&mut tokio::process::Command),
) -> anyhow::Result<TokioChildProcess> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_docutouch"));
    command.current_dir(cwd);
    command.kill_on_drop(true);
    configure(&mut command);
    Ok(TokioChildProcess::new(command)?)
}

pub async fn run_with_timeout_and_cleanup<
    T,
    Body,
    CleanupFactory,
    Cleanup,
    CleanupSuccess,
    CleanupError,
>(
    label: &str,
    body: Body,
    cleanup: CleanupFactory,
) -> anyhow::Result<T>
where
    Body: Future<Output = anyhow::Result<T>>,
    CleanupFactory: FnOnce() -> Cleanup,
    Cleanup: Future<Output = Result<CleanupSuccess, CleanupError>>,
    CleanupError: Into<anyhow::Error>,
{
    run_with_timeout_and_cleanup_using(TEST_CHILD_TIMEOUT, label, body, cleanup).await
}

pub async fn timeout_result<T, E, Fut>(label: impl Into<String>, future: Fut) -> anyhow::Result<T>
where
    Fut: Future<Output = Result<T, E>>,
    E: Into<anyhow::Error>,
{
    let label = label.into();
    match tokio::time::timeout(TEST_CHILD_TIMEOUT, future).await {
        Ok(result) => result.map_err(Into::into),
        Err(_) => anyhow::bail!("timed out waiting for {label}"),
    }
}

pub async fn run_with_timeout_and_cleanup_using<
    T,
    Body,
    CleanupFactory,
    Cleanup,
    CleanupSuccess,
    CleanupError,
>(
    timeout: Duration,
    label: &str,
    body: Body,
    cleanup: CleanupFactory,
) -> anyhow::Result<T>
where
    Body: Future<Output = anyhow::Result<T>>,
    CleanupFactory: FnOnce() -> Cleanup,
    Cleanup: Future<Output = Result<CleanupSuccess, CleanupError>>,
    CleanupError: Into<anyhow::Error>,
{
    let body_result = tokio::time::timeout(timeout, body).await;
    let cleanup_result = tokio::time::timeout(timeout, cleanup()).await;

    match body_result {
        Ok(Ok(value)) => match cleanup_result {
            Ok(Ok(_)) => Ok(value),
            Ok(Err(err)) => Err(err.into()),
            Err(_) => anyhow::bail!("timed out waiting to clean up {label}"),
        },
        Ok(Err(err)) => {
            let _ = cleanup_result;
            Err(err)
        }
        Err(_) => {
            let _ = cleanup_result;
            anyhow::bail!("timed out waiting for {label}")
        }
    }
}

pub fn run_cli(cwd: &Path, args: &[&str], stdin: Option<&str>) -> anyhow::Result<Output> {
    let child = spawn_cli_child(cwd, args, stdin)?;
    wait_with_output_timeout(child, "docutouch CLI child in cli_smoke")
}

pub fn spawn_cli_child(cwd: &Path, args: &[&str], stdin: Option<&str>) -> anyhow::Result<Child> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_docutouch"));
    command
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn()?;
    if let Some(input) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(input.as_bytes())?;
    }
    child.stdin.take();
    Ok(child)
}

pub fn wait_with_output_timeout(child: Child, label: &str) -> anyhow::Result<Output> {
    wait_with_output_timeout_using(child, label, TEST_CHILD_TIMEOUT)
}

pub fn wait_with_output_timeout_using(
    mut child: Child,
    label: &str,
    timeout: Duration,
) -> anyhow::Result<Output> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut reader) = child.stdout.take() {
                reader.read_to_end(&mut stdout)?;
            }
            if let Some(mut reader) = child.stderr.take() {
                reader.read_to_end(&mut stderr)?;
            }
            return Ok(Output {
                status,
                stdout,
                stderr,
            });
        }

        if Instant::now() >= deadline {
            cleanup_timed_out_child(&mut child);
            anyhow::bail!("timed out waiting for {label}");
        }

        std::thread::sleep(Duration::from_millis(25));
    }
}

fn cleanup_timed_out_child(child: &mut Child) {
    let _ = terminate_process_tree(child);
    let _ = child.wait();
}

#[cfg(windows)]
fn terminate_process_tree(child: &mut Child) -> std::io::Result<()> {
    let status = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &child.id().to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        child.kill()
    }
}

#[cfg(not(windows))]
fn terminate_process_tree(child: &mut Child) -> std::io::Result<()> {
    child.kill()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[tokio::test]
    async fn timeout_cleanup_still_runs_when_body_times_out() {
        let cleaned_up = Arc::new(AtomicBool::new(false));
        let cleanup_flag = Arc::clone(&cleaned_up);

        let err = run_with_timeout_and_cleanup_using(
            Duration::from_millis(10),
            "smoke helper timeout regression",
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok::<_, anyhow::Error>(())
            },
            || async move {
                cleanup_flag.store(true, Ordering::SeqCst);
                Ok::<_, anyhow::Error>(())
            },
        )
        .await
        .expect_err("body should time out");

        assert!(
            err.to_string()
                .contains("timed out waiting for smoke helper timeout regression")
        );
        assert!(cleaned_up.load(Ordering::SeqCst));
    }
}
