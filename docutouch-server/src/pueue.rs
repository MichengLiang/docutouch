use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::{Instant, sleep};

pub(crate) const DOCUTOUCH_PUEUE_BIN_ENV: &str = "DOCUTOUCH_PUEUE_BIN";
pub(crate) const DOCUTOUCH_PUEUE_RUNTIME_DIR_ENV: &str = "DOCUTOUCH_PUEUE_RUNTIME_DIR";
pub(crate) const DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV: &str = "DOCUTOUCH_PUEUE_TIMEOUT_SECONDS";
pub(crate) const PUEUE_LOG_HANDLE_PREFIX: &str = "pueue-log:";
const DEFAULT_PUEUE_BIN: &str = "pueue";
const TASK_LOGS_DIR_NAME: &str = "task_logs";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);
const STATUS_POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PueuePaths {
    pub state_dir: PathBuf,
    pub runtime_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct PueueRuntime {
    executable: OsString,
    paths: PueuePaths,
    default_timeout: Option<Duration>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WaitMode {
    Any,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WaitReason {
    TaskFinished,
    AllFinished,
    Timeout,
    NothingToWaitFor,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PueueTaskStatus {
    pub name: String,
    pub detail: Option<Value>,
    pub is_terminal: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PueueTaskSnapshot {
    pub id: u64,
    pub status: PueueTaskStatus,
    pub raw: Value,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PueueStatusSnapshot {
    pub tasks: BTreeMap<u64, PueueTaskSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PueueWaitSnapshot {
    pub reason: WaitReason,
    pub resolved_task_ids: Vec<u64>,
    pub triggered_task_ids: Vec<u64>,
    pub pending_task_ids: Vec<u64>,
    pub terminal_tasks: Vec<PueueTaskSnapshot>,
    pub waited: Duration,
}

#[derive(Debug)]
pub(crate) enum PueueError {
    InvalidHandle(String),
    InvalidTimeout(String),
    RuntimeResolution(String),
    CommandLaunch(String),
    CommandFailure(String),
    InvalidStatusSnapshot(String),
    TaskDoesNotExist(u64),
    TaskLogUnavailable(u64),
}

impl PueueRuntime {
    pub(crate) fn from_env() -> Result<Self, PueueError> {
        Ok(Self {
            executable: resolve_pueue_executable()?,
            paths: resolve_pueue_paths()?,
            default_timeout: resolve_default_timeout()?,
        })
    }

    pub(crate) fn executable(&self) -> &OsStr {
        &self.executable
    }

    pub(crate) fn paths(&self) -> &PueuePaths {
        &self.paths
    }

    pub(crate) fn default_timeout(&self) -> Option<Duration> {
        self.default_timeout
    }

    pub(crate) fn task_log_path(&self, task_id: u64) -> PathBuf {
        self.paths
            .state_dir
            .join(TASK_LOGS_DIR_NAME)
            .join(format!("{task_id}.log"))
    }

    pub(crate) async fn status_snapshot(&self) -> Result<PueueStatusSnapshot, PueueError> {
        let stdout = self.run_command(["status", "--json"]).await?;
        parse_status_snapshot(&stdout)
    }

    pub(crate) async fn snapshot_unfinished_task_ids(&self) -> Result<Vec<u64>, PueueError> {
        let snapshot = self.status_snapshot().await?;
        Ok(snapshot.unfinished_task_ids())
    }

    pub(crate) async fn resolve_explicit_task_ids(
        &self,
        task_ids: &[u64],
    ) -> Result<Vec<u64>, PueueError> {
        let resolved = dedupe_task_ids(task_ids);
        if resolved.is_empty() {
            return Ok(resolved);
        }

        let snapshot = self.status_snapshot().await?;
        for task_id in &resolved {
            if !snapshot.tasks.contains_key(task_id) {
                return Err(PueueError::TaskDoesNotExist(*task_id));
            }
        }
        Ok(resolved)
    }

    pub(crate) async fn resolve_task_log_path(&self, task_id: u64) -> Result<PathBuf, PueueError> {
        let snapshot = self.status_snapshot().await?;
        if !snapshot.tasks.contains_key(&task_id) {
            return Err(PueueError::TaskDoesNotExist(task_id));
        }

        let log_path = self.task_log_path(task_id);
        if !log_path.is_file() {
            return Err(PueueError::TaskLogUnavailable(task_id));
        }
        Ok(log_path)
    }

    pub(crate) async fn wait_for_tasks(
        &self,
        task_ids: &[u64],
        mode: WaitMode,
        timeout: Duration,
    ) -> Result<PueueWaitSnapshot, PueueError> {
        if timeout.is_zero() {
            return Err(PueueError::InvalidTimeout(
                "timeout must be greater than zero".to_string(),
            ));
        }

        let resolved_task_ids = dedupe_task_ids(task_ids);
        if resolved_task_ids.is_empty() {
            return Ok(PueueWaitSnapshot {
                reason: WaitReason::NothingToWaitFor,
                resolved_task_ids,
                triggered_task_ids: Vec::new(),
                pending_task_ids: Vec::new(),
                terminal_tasks: Vec::new(),
                waited: Duration::ZERO,
            });
        }

        let start = Instant::now();
        loop {
            let snapshot = self.status_snapshot().await?;
            let state = evaluate_wait_state(&snapshot, &resolved_task_ids, mode)?;
            if let Some(reason) = state.completion_reason() {
                return Ok(PueueWaitSnapshot {
                    reason,
                    resolved_task_ids: resolved_task_ids.clone(),
                    triggered_task_ids: state.triggered_task_ids,
                    pending_task_ids: state.pending_task_ids,
                    terminal_tasks: state.terminal_tasks,
                    waited: start.elapsed(),
                });
            }

            if start.elapsed() >= timeout {
                return Ok(PueueWaitSnapshot {
                    reason: WaitReason::Timeout,
                    resolved_task_ids: resolved_task_ids.clone(),
                    triggered_task_ids: state.triggered_task_ids,
                    pending_task_ids: state.pending_task_ids,
                    terminal_tasks: state.terminal_tasks,
                    waited: start.elapsed(),
                });
            }

            sleep(STATUS_POLL_INTERVAL).await;
        }
    }

    async fn run_command<const N: usize>(&self, args: [&str; N]) -> Result<String, PueueError> {
        let output = Command::new(&self.executable)
            .args(args)
            .output()
            .await
            .map_err(|err| {
                PueueError::CommandLaunch(format!(
                    "failed to launch {}: {err}",
                    self.executable.to_string_lossy()
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() {
            return Ok(stdout);
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let fallback = stdout.trim().to_string();
        let message = if stderr.is_empty() { fallback } else { stderr };
        Err(PueueError::CommandFailure(if message.is_empty() {
            format!(
                "{} exited with status {}",
                self.executable.to_string_lossy(),
                output
                    .status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            )
        } else {
            message
        }))
    }
}

impl PueueStatusSnapshot {
    pub(crate) fn unfinished_task_ids(&self) -> Vec<u64> {
        self.tasks
            .iter()
            .filter_map(|(task_id, task)| (!task.status.is_terminal).then_some(*task_id))
            .collect()
    }
}

impl std::fmt::Display for PueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PueueError::InvalidHandle(message)
            | PueueError::InvalidTimeout(message)
            | PueueError::RuntimeResolution(message)
            | PueueError::CommandLaunch(message)
            | PueueError::CommandFailure(message)
            | PueueError::InvalidStatusSnapshot(message) => write!(f, "{message}"),
            PueueError::TaskDoesNotExist(task_id) => write!(f, "Task does not exist: {task_id}"),
            PueueError::TaskLogUnavailable(task_id) => {
                write!(f, "Task log not available: {task_id}")
            }
        }
    }
}

impl std::error::Error for PueueError {}

pub(crate) fn parse_task_log_handle(input: &str) -> Result<Option<u64>, PueueError> {
    let Some(raw_id) = input.strip_prefix(PUEUE_LOG_HANDLE_PREFIX) else {
        return Ok(None);
    };
    if raw_id.is_empty() {
        return Err(PueueError::InvalidHandle(format!(
            "Invalid pueue log handle: {input}"
        )));
    }
    raw_id
        .parse::<u64>()
        .map(Some)
        .map_err(|_| PueueError::InvalidHandle(format!("Invalid pueue log handle: {input}")))
}

pub(crate) fn format_task_log_handle(task_id: u64) -> String {
    format!("{PUEUE_LOG_HANDLE_PREFIX}{task_id}")
}

fn resolve_pueue_executable() -> Result<OsString, PueueError> {
    match std::env::var_os(DOCUTOUCH_PUEUE_BIN_ENV) {
        Some(value) if value.to_string_lossy().trim().is_empty() => Err(
            PueueError::RuntimeResolution(format!("{DOCUTOUCH_PUEUE_BIN_ENV} cannot be empty")),
        ),
        Some(value) => Ok(value),
        None => Ok(OsString::from(DEFAULT_PUEUE_BIN)),
    }
}

fn resolve_pueue_paths() -> Result<PueuePaths, PueueError> {
    if let Some(override_dir) = env_path(DOCUTOUCH_PUEUE_RUNTIME_DIR_ENV)? {
        return Ok(PueuePaths {
            state_dir: override_dir.clone(),
            runtime_dir: override_dir,
        });
    }

    let config_path = resolve_native_config_path()?;
    let config = config_path
        .as_deref()
        .and_then(read_native_config)
        .map(|contents| parse_native_config(&contents, config_path.as_deref()))
        .unwrap_or_default();

    let state_dir = match config.state_dir {
        Some(path) => path,
        None => default_state_dir()?,
    };
    let runtime_dir = match config.runtime_dir {
        Some(path) => path,
        None => state_dir.clone(),
    };

    Ok(PueuePaths {
        state_dir,
        runtime_dir,
    })
}

fn resolve_default_timeout() -> Result<Option<Duration>, PueueError> {
    parse_timeout_override(std::env::var_os(DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV))
}

fn parse_timeout_override(value: Option<OsString>) -> Result<Option<Duration>, PueueError> {
    let Some(value) = value else {
        return Ok(Some(DEFAULT_TIMEOUT));
    };
    let raw = value.to_string_lossy().trim().to_string();
    if raw.is_empty() {
        return Err(PueueError::InvalidTimeout(format!(
            "{DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV} cannot be empty"
        )));
    }
    let seconds = raw.parse::<f64>().map_err(|_| {
        PueueError::InvalidTimeout(format!(
            "{DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV} must be a positive number"
        ))
    })?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(PueueError::InvalidTimeout(format!(
            "{DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV} must be a positive number"
        )));
    }
    Ok(Some(Duration::from_secs_f64(seconds)))
}

fn resolve_native_config_path() -> Result<Option<PathBuf>, PueueError> {
    if let Some(path) = env_path("PUEUE_CONFIG_PATH")? {
        return Ok(path.is_file().then_some(path));
    }

    let path = if cfg!(windows) {
        env_required_path("APPDATA")?
            .join("pueue")
            .join("pueue.yml")
    } else if let Some(path) = env_path("XDG_CONFIG_HOME")? {
        path.join("pueue").join("pueue.yml")
    } else {
        env_required_path("HOME")?
            .join(".config")
            .join("pueue")
            .join("pueue.yml")
    };

    Ok(path.is_file().then_some(path))
}

fn default_state_dir() -> Result<PathBuf, PueueError> {
    if cfg!(windows) {
        return Ok(env_required_path("LOCALAPPDATA")?.join("pueue"));
    }

    if let Some(path) = env_path("XDG_DATA_HOME")? {
        return Ok(path.join("pueue"));
    }

    Ok(env_required_path("HOME")?
        .join(".local")
        .join("share")
        .join("pueue"))
}

fn env_required_path(name: &str) -> Result<PathBuf, PueueError> {
    env_path(name)?.ok_or_else(|| {
        PueueError::RuntimeResolution(format!("unable to resolve Pueue path from {name}"))
    })
}

fn env_path(name: &str) -> Result<Option<PathBuf>, PueueError> {
    let Some(value) = std::env::var_os(name) else {
        return Ok(None);
    };
    if value.to_string_lossy().trim().is_empty() {
        return Err(PueueError::RuntimeResolution(format!(
            "{name} cannot be empty"
        )));
    }
    Ok(Some(PathBuf::from(value)))
}

fn read_native_config(config_path: &Path) -> Option<String> {
    std::fs::read_to_string(config_path).ok()
}

#[derive(Default)]
struct NativeConfig {
    state_dir: Option<PathBuf>,
    runtime_dir: Option<PathBuf>,
}

fn parse_native_config(contents: &str, config_path: Option<&Path>) -> NativeConfig {
    let mut in_shared = false;
    let mut shared_indent = 0usize;
    let mut state_dir = None;
    let mut runtime_dir = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = line.chars().take_while(|ch| ch.is_whitespace()).count();
        if !in_shared {
            if trimmed == "shared:" {
                in_shared = true;
                shared_indent = indent;
            }
            continue;
        }

        if indent <= shared_indent {
            in_shared = false;
            if trimmed == "shared:" {
                in_shared = true;
                shared_indent = indent;
            } else {
                continue;
            }
        }

        if let Some((key, raw_value)) = trimmed.split_once(':') {
            let value = parse_scalar_value(raw_value);
            match key.trim() {
                "pueue_directory" => {
                    state_dir = value.map(|raw| normalize_config_path(raw, config_path))
                }
                "runtime_directory" => {
                    runtime_dir = value.map(|raw| normalize_config_path(raw, config_path))
                }
                _ => {}
            }
        }
    }

    NativeConfig {
        state_dir,
        runtime_dir,
    }
}

fn parse_scalar_value(raw: &str) -> Option<String> {
    let trimmed = strip_inline_comment(raw).trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return None;
    }

    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return Some(unescape_quoted_scalar(
                &trimmed[1..trimmed.len() - 1],
                bytes[0] as char,
            ));
        }
    }

    Some(trimmed.to_string())
}

fn strip_inline_comment(raw: &str) -> &str {
    if let Some((before, _)) = raw.split_once(" #") {
        before
    } else {
        raw
    }
}

fn unescape_quoted_scalar(value: &str, quote: char) -> String {
    if quote == '\'' {
        return value.replace("''", "'");
    }

    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        match chars.next() {
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some(other) => {
                result.push('\\');
                result.push(other);
            }
            None => result.push('\\'),
        }
    }
    result
}

fn normalize_config_path(raw: String, config_path: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return path;
    }

    match config_path.and_then(Path::parent) {
        Some(parent) => parent.join(path),
        None => path,
    }
}

fn parse_status_snapshot(stdout: &str) -> Result<PueueStatusSnapshot, PueueError> {
    let root = serde_json::from_str::<Value>(stdout).map_err(|err| {
        PueueError::InvalidStatusSnapshot(format!("failed to decode `pueue status --json`: {err}"))
    })?;

    let tasks = root
        .get("tasks")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            PueueError::InvalidStatusSnapshot(
                "failed to decode `pueue status --json`: missing `tasks` object".to_string(),
            )
        })?;

    let mut parsed_tasks = BTreeMap::new();
    for raw_task in tasks.values() {
        let Some(task_id) = raw_task.get("id").and_then(Value::as_u64) else {
            return Err(PueueError::InvalidStatusSnapshot(
                "failed to decode `pueue status --json`: task missing numeric `id`".to_string(),
            ));
        };
        let status = raw_task
            .get("status")
            .ok_or_else(|| {
                PueueError::InvalidStatusSnapshot(format!(
                    "failed to decode `pueue status --json`: task {task_id} missing `status`"
                ))
            })?
            .clone();
        parsed_tasks.insert(
            task_id,
            PueueTaskSnapshot {
                id: task_id,
                status: parse_task_status(status),
                raw: raw_task.clone(),
            },
        );
    }

    Ok(PueueStatusSnapshot {
        tasks: parsed_tasks,
    })
}

fn parse_task_status(status: Value) -> PueueTaskStatus {
    match status {
        Value::Object(object) if object.len() == 1 => {
            let (name, detail) = object.into_iter().next().expect("one entry");
            let is_terminal = name == "Done";
            let detail = match detail {
                Value::Null => None,
                value => Some(value),
            };
            PueueTaskStatus {
                name,
                detail,
                is_terminal,
            }
        }
        Value::Object(object) if object.is_empty() => PueueTaskStatus {
            name: "Unknown".to_string(),
            detail: None,
            is_terminal: false,
        },
        other => PueueTaskStatus {
            name: "Unknown".to_string(),
            detail: Some(other),
            is_terminal: false,
        },
    }
}

fn dedupe_task_ids(task_ids: &[u64]) -> Vec<u64> {
    let mut seen = HashSet::new();
    let mut resolved = Vec::new();
    for task_id in task_ids {
        if seen.insert(*task_id) {
            resolved.push(*task_id);
        }
    }
    resolved
}

struct WaitState {
    triggered_task_ids: Vec<u64>,
    pending_task_ids: Vec<u64>,
    terminal_tasks: Vec<PueueTaskSnapshot>,
    mode: WaitMode,
}

impl WaitState {
    fn completion_reason(&self) -> Option<WaitReason> {
        match self.mode {
            WaitMode::Any if !self.triggered_task_ids.is_empty() => Some(WaitReason::TaskFinished),
            WaitMode::All if self.pending_task_ids.is_empty() => Some(WaitReason::AllFinished),
            _ => None,
        }
    }
}

fn evaluate_wait_state(
    snapshot: &PueueStatusSnapshot,
    resolved_task_ids: &[u64],
    mode: WaitMode,
) -> Result<WaitState, PueueError> {
    let mut triggered_task_ids = Vec::new();
    let mut pending_task_ids = Vec::new();
    let mut terminal_tasks = Vec::new();

    for task_id in resolved_task_ids {
        let task = snapshot
            .tasks
            .get(task_id)
            .cloned()
            .ok_or(PueueError::TaskDoesNotExist(*task_id))?;
        if task.status.is_terminal {
            triggered_task_ids.push(*task_id);
            terminal_tasks.push(task);
        } else {
            pending_task_ids.push(*task_id);
        }
    }

    Ok(WaitState {
        triggered_task_ids,
        pending_task_ids,
        terminal_tasks,
        mode,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV, NativeConfig, WaitMode, WaitReason, dedupe_task_ids,
        evaluate_wait_state, format_task_log_handle, parse_native_config, parse_status_snapshot,
        parse_task_log_handle, parse_task_status, parse_timeout_override,
    };
    use serde_json::json;
    use std::ffi::OsString;
    use std::path::Path;
    use std::time::Duration;

    #[test]
    fn parse_task_log_handle_accepts_zero_and_rejects_invalid_suffixes() {
        assert_eq!(
            parse_task_log_handle("pueue-log:0").expect("handle"),
            Some(0)
        );
        assert!(parse_task_log_handle("pueue-log:abc").is_err());
        assert_eq!(
            parse_task_log_handle("plain.txt").expect("non-handle"),
            None
        );
        assert_eq!(format_task_log_handle(42), "pueue-log:42");
    }

    #[test]
    fn parse_native_config_reads_shared_paths() {
        let config = r#"
client:
  dark_mode: false
shared:
  pueue_directory: "data\\pueue"
  runtime_directory: runtime/pueue
profiles: {}
"#;

        let NativeConfig {
            state_dir,
            runtime_dir,
        } = parse_native_config(config, Some(Path::new("C:/config/pueue.yml")));

        assert_eq!(
            state_dir.expect("state dir"),
            Path::new("C:/config/data\\pueue")
        );
        assert_eq!(
            runtime_dir.expect("runtime dir"),
            Path::new("C:/config/runtime/pueue")
        );
    }

    #[test]
    fn parse_status_snapshot_extracts_terminal_and_pending_tasks() {
        let snapshot = parse_status_snapshot(
            r#"{
  "tasks": {
    "1": { "id": 1, "status": { "Running": { "start": "now" } } },
    "2": { "id": 2, "status": { "Done": { "result": "Success" } } }
  }
}"#,
        )
        .expect("status snapshot");

        assert_eq!(snapshot.unfinished_task_ids(), vec![1]);
        assert!(!snapshot.tasks.get(&1).expect("task 1").status.is_terminal);
        assert!(snapshot.tasks.get(&2).expect("task 2").status.is_terminal);
    }

    #[test]
    fn parse_task_status_handles_unexpected_shapes() {
        let unknown = parse_task_status(json!({"Running": {"a": 1}, "Queued": {"b": 2}}));
        assert_eq!(unknown.name, "Unknown");
        assert!(!unknown.is_terminal);
    }

    #[test]
    fn evaluate_wait_state_obeys_any_and_all_modes() {
        let snapshot = parse_status_snapshot(
            r#"{
  "tasks": {
    "1": { "id": 1, "status": { "Done": { "result": "Success" } } },
    "2": { "id": 2, "status": { "Running": { "start": "now" } } }
  }
}"#,
        )
        .expect("status snapshot");

        let any = evaluate_wait_state(&snapshot, &[1, 2], WaitMode::Any).expect("any");
        assert_eq!(any.completion_reason(), Some(WaitReason::TaskFinished));
        assert_eq!(any.triggered_task_ids, vec![1]);
        assert_eq!(any.pending_task_ids, vec![2]);

        let all = evaluate_wait_state(&snapshot, &[1, 2], WaitMode::All).expect("all");
        assert_eq!(all.completion_reason(), None);
        assert_eq!(all.triggered_task_ids, vec![1]);
        assert_eq!(all.pending_task_ids, vec![2]);
    }

    #[test]
    fn dedupe_task_ids_preserves_first_appearance_order() {
        assert_eq!(dedupe_task_ids(&[2, 1, 2, 3, 1]), vec![2, 1, 3]);
    }

    #[test]
    fn invalid_status_snapshot_requires_tasks_object() {
        let err = parse_status_snapshot(r#"{"groups": {}}"#).expect_err("missing tasks");
        assert!(err.to_string().contains("missing `tasks` object"));
    }

    #[test]
    fn timeout_override_defaults_to_five_minutes_when_unset() {
        assert_eq!(
            parse_timeout_override(None).expect("default timeout"),
            Some(Duration::from_secs(300))
        );
    }

    #[test]
    fn timeout_override_accepts_explicit_positive_seconds() {
        assert_eq!(
            parse_timeout_override(Some(OsString::from("12.5"))).expect("override"),
            Some(Duration::from_secs_f64(12.5))
        );
    }

    #[test]
    fn timeout_override_rejects_empty_or_non_positive_values() {
        let empty = parse_timeout_override(Some(OsString::from("   "))).expect_err("empty");
        assert_eq!(
            empty.to_string(),
            format!("{DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV} cannot be empty")
        );

        let zero = parse_timeout_override(Some(OsString::from("0"))).expect_err("zero");
        assert_eq!(
            zero.to_string(),
            format!("{DOCUTOUCH_PUEUE_TIMEOUT_SECONDS_ENV} must be a positive number")
        );
    }
}
