use crate::patch_adapter::{PatchInvocationAdapter, PatchSourceProvenance};
use crate::pueue::{
    PueueError, PueueRuntime, PueueTaskSnapshot, PueueWaitSnapshot, WaitMode, WaitReason,
    format_task_log_handle, parse_task_log_handle,
};
use crate::splice_adapter::SpliceInvocationAdapter;
use anyhow::Result;
use docutouch_core::{
    DirectoryListOptions, PatchWorkspaceRequirement, ReadFileOptions, SearchTextView,
    SpliceWorkspaceRequirement, TimestampField, list_directory, normalize_sampled_view_options,
    patch_workspace_requirement, read_file_with_sampled_view, search_text,
    splice_workspace_requirement,
};
use rmcp::model::{JsonObject, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::process::Command as TokioCommand;
use tokio::sync::RwLock;

const APPLY_PATCH_TOOL_DESCRIPTION: &str = include_str!("../tool_docs/apply_patch.md");
const APPLY_SPLICE_TOOL_DESCRIPTION: &str = include_str!("../tool_docs/apply_splice.md");
pub(crate) const DEFAULT_WORKSPACE_ENV: &str = "DOCUTOUCH_DEFAULT_WORKSPACE";
const SEARCH_TEXT_TOOL_DESCRIPTION: &str = "基于 ripgrep 的文本搜索包装。保留原始终端 `rg` 作为无限制逃生口；当前工具服务于常见的 LLM 搜索路径，按文件分组返回结果，并区分 `preview` 概览视图与 `full` 全量分组视图。`rg_args` 仅用于 search-behavior flags，如 `-F`、`-i`、`-g`、`-P`；render-shaping flags（如 `--json`、`-n`、`-c`、`-l`、`-A/-B/-C`）由 `search_text` 自身保留控制。";

#[derive(Clone)]
pub struct ToolService {
    workspace: Arc<RwLock<Option<PathBuf>>>,
    execution_lock: Arc<RwLock<()>>,
    mcp_tools: Arc<Vec<Tool>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWorkspaceArgs {
    #[schemars(description = "作为 relative path 默认解析基准的目录路径。")]
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDirectoryArgs {
    #[schemars(
        description = "相对于 workspace 的目录路径；未设置 workspace 时也可直接传 absolute path。"
    )]
    #[serde(default = "default_relative_path")]
    pub relative_path: String,
    #[schemars(description = "目录树展开深度；默认 3。")]
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[schemars(description = "是否显示以 . 开头的隐藏文件和目录。默认 false。")]
    #[serde(default)]
    pub show_hidden: bool,
    #[schemars(description = "是否显示命中 .gitignore 规则的条目。默认 false。")]
    #[serde(default)]
    pub include_gitignored: bool,
    #[schemars(description = "可选时间戳字段。支持 created、modified；默认不显示时间戳。")]
    #[serde(default)]
    pub timestamp_fields: Vec<TimestampFieldInput>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    #[schemars(
        description = "相对于 workspace 的文件路径；未设置 workspace 时也可直接传 absolute path。"
    )]
    pub relative_path: String,
    #[schemars(
        description = "可选的 1-indexed 闭区间行范围。支持 [start, end] 或字符串形式 start,end / start-end / start:end。"
    )]
    #[serde(default)]
    pub line_range: Option<LineRangeInput>,
    #[schemars(description = "是否在返回内容中显示 1-indexed 行号。默认 false。")]
    #[serde(default)]
    pub show_line_numbers: bool,
    #[schemars(
        description = "可选的 sampled inspection 步长。任一 sampled 参数出现时都会启用该视图；未提供的 sampled 参数会补默认值。局部检查常见推荐值为 3-5。"
    )]
    #[serde(default)]
    pub sample_step: Option<usize>,
    #[schemars(
        description = "可选的 sampled inspection 连续行数。默认会补为稳定局部检查值；常见推荐为 `2`。"
    )]
    #[serde(default)]
    pub sample_lines: Option<usize>,
    #[schemars(
        description = "可选的每行最大字符数。超过该值的行会以内联 `...[N chars omitted]` 形式显式裁切；未提供时不做横向裁切。"
    )]
    #[serde(default)]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum LineRangeInput {
    Pair(Vec<usize>),
    Text(String),
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApplyPatchArgs {
    #[schemars(
        description = "freeform patch 文本。补丁接受 Add / Delete / Update / Move 语义，并按文件组级原子边界执行。"
    )]
    pub patch: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApplySpliceArgs {
    #[schemars(
        description = "freeform splice 文本。程序接受 Begin/End Splice envelope、Copy/Move/Delete Span action 以及 Append/Insert/Replace target clauses，并按当前 splice runtime 执行。"
    )]
    pub splice: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchTextArgs {
    #[schemars(description = "要搜索的文本或 ripgrep 模式。")]
    pub query: String,
    #[schemars(
        description = "必填搜索范围。可传单个 relative/absolute path，或传由文件/目录组成的 path 数组；数组中的范围会合并为同一次搜索。"
    )]
    pub path: SearchTextPathInput,
    #[schemars(
        description = "可选的原始 ripgrep 参数字符串，用作高级 escape hatch。仅推荐 search-behavior flags，例如 `-F`、`-i`、`-g '*.rs'`、`-P`。render-shaping flags（如 `--json`、`-n`、`-c`、`-l`、`-A/-B/-C`）由 `search_text` 自身保留控制。"
    )]
    #[serde(default)]
    pub rg_args: String,
    #[schemars(
        description = "搜索结果视图。`preview` 返回显式概览并附 omission accounting；`full` 返回全量分组结果。默认 `preview`。"
    )]
    #[serde(default)]
    pub view: SearchTextViewInput,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WaitPueueArgs {
    #[schemars(
        description = "可选 task id 列表。缺省时，等待调用开始瞬间当前所有未完成 task 的快照。"
    )]
    #[serde(default)]
    pub task_ids: Option<Vec<u64>>,
    #[schemars(description = "等待模式。只允许 `any` 或 `all`；默认 `any`。")]
    #[serde(default)]
    pub mode: Option<WaitModeInput>,
    #[schemars(description = "可选超时秒数。必须为正数；缺省时读取 runtime default。")]
    #[serde(default)]
    pub timeout_seconds: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum SearchTextPathInput {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchTextViewInput {
    #[default]
    Preview,
    Full,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WaitModeInput {
    Any,
    All,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimestampFieldInput {
    Created,
    Modified,
}

#[derive(Debug)]
pub struct ServiceError {
    kind: ServiceErrorKind,
    message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceErrorKind {
    InvalidArgument,
    ToolFailure,
}

#[derive(Debug)]
pub(crate) struct ResolvedSearchTextPaths {
    pub search_paths: Vec<PathBuf>,
    pub display_scope: String,
    pub path_overrides: Vec<(String, String)>,
}

impl ToolService {
    pub fn for_stdio() -> Result<Self> {
        Ok(Self {
            workspace: Arc::new(RwLock::new(default_workspace_from_env())),
            execution_lock: Arc::new(RwLock::new(())),
            mcp_tools: Arc::new(build_mcp_tools(true)?),
        })
    }

    pub fn mcp_tools(&self) -> Arc<Vec<Tool>> {
        self.mcp_tools.clone()
    }

    pub async fn call_json(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String, ServiceError> {
        match tool_name {
            "set_workspace" => {
                let args = parse_json_args::<SetWorkspaceArgs>(arguments)?;
                self.set_workspace_impl(&args.path).await
            }
            "list_directory" => {
                let args = parse_json_args::<ListDirectoryArgs>(arguments)?;
                self.list_directory_impl(args).await
            }
            "read_file" => {
                let args = parse_json_args::<ReadFileArgs>(arguments)?;
                self.read_file_impl(args).await
            }
            "search_text" => {
                let args = parse_json_args::<SearchTextArgs>(arguments)?;
                self.search_text_impl(args).await
            }
            "wait_pueue" => {
                let args = parse_json_args::<WaitPueueArgs>(arguments)?;
                self.wait_pueue_impl(args).await
            }
            "apply_patch" => {
                let args = parse_json_args::<ApplyPatchArgs>(arguments)?;
                self.apply_patch_impl(args.patch).await
            }
            "apply_splice" => {
                let args = parse_json_args::<ApplySpliceArgs>(arguments)?;
                self.apply_splice_impl(args.splice).await
            }
            other => Err(ServiceError::tool_not_found(format!(
                "unknown tool: {other}"
            ))),
        }
    }

    pub async fn call_mcp_tool(
        &self,
        tool_name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<String, ServiceError> {
        let arguments = arguments
            .map(|value| serde_json::Value::Object(value.into_iter().collect()))
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
        self.call_json(tool_name, arguments).await
    }

    async fn set_workspace_impl(&self, path: &str) -> Result<String, ServiceError> {
        let canonical = validate_workspace_path(&PathBuf::from(path))
            .map_err(ServiceError::invalid_argument)?;
        *self.workspace.write().await = Some(canonical.clone());
        Ok(format!("✓ Workspace set to: {}", canonical.display()))
    }

    async fn list_directory_impl(&self, args: ListDirectoryArgs) -> Result<String, ServiceError> {
        let _guard = self.execution_lock.read().await;
        let dir_path = self.resolve_path(&args.relative_path).await?;
        let result = list_directory(
            &dir_path,
            DirectoryListOptions {
                max_depth: args.max_depth,
                show_hidden: args.show_hidden,
                include_gitignored: args.include_gitignored,
                timestamp_fields: args
                    .timestamp_fields
                    .into_iter()
                    .map(map_timestamp_field)
                    .collect(),
            },
        )
        .map_err(map_directory_error)?;
        Ok(result.display())
    }

    async fn read_file_impl(&self, args: ReadFileArgs) -> Result<String, ServiceError> {
        let _guard = self.execution_lock.read().await;
        let workspace = self.workspace.read().await.clone();
        let file_path =
            resolve_read_surface_path(&args.relative_path, workspace.as_deref()).await?;
        let line_range =
            normalize_line_range(args.line_range).map_err(ServiceError::invalid_argument)?;
        let sampled_view =
            normalize_sampled_view_options(args.sample_step, args.sample_lines, args.max_chars)
                .map_err(ServiceError::invalid_argument)?;
        let result = read_file_with_sampled_view(
            &file_path,
            ReadFileOptions {
                line_range,
                show_line_numbers: args.show_line_numbers,
            },
            sampled_view,
        )
        .map_err(map_read_error)?;
        Ok(result.content)
    }

    async fn search_text_impl(&self, args: SearchTextArgs) -> Result<String, ServiceError> {
        if args.query.trim().is_empty() {
            return Err(ServiceError::invalid_argument("query cannot be empty"));
        }
        if args.path.is_empty() {
            return Err(ServiceError::invalid_argument("path cannot be empty"));
        }

        let _guard = self.execution_lock.read().await;
        let display_base_dir = self.workspace.read().await.clone();
        let raw_paths = args
            .path
            .paths()
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let resolved = resolve_search_surface_paths(
            &raw_paths,
            display_base_dir.as_deref(),
            display_base_dir.as_deref(),
        )
        .await?;
        let rendered = search_text(
            &args.query,
            &resolved.search_paths,
            &args.rg_args,
            args.view.into(),
            display_base_dir.as_deref(),
        )
        .await
        .map_err(ServiceError::invalid_argument)?;
        Ok(rewrite_search_text_surface(
            rendered,
            &resolved.display_scope,
            &resolved.path_overrides,
        ))
    }

    async fn wait_pueue_impl(&self, args: WaitPueueArgs) -> Result<String, ServiceError> {
        let runtime = PueueRuntime::from_env().map_err(map_pueue_runtime_error)?;
        let mode = args.mode.unwrap_or(WaitModeInput::Any).into();
        let resolved_task_ids = match args.task_ids {
            Some(task_ids) => runtime
                .resolve_explicit_task_ids(&task_ids)
                .await
                .map_err(map_wait_argument_error)?,
            None => runtime
                .snapshot_unfinished_task_ids()
                .await
                .map_err(map_pueue_runtime_error)?,
        };
        let timeout = resolve_wait_timeout(args.timeout_seconds, runtime.default_timeout())?;
        let snapshot = runtime
            .wait_for_tasks(&resolved_task_ids, mode, timeout)
            .await
            .map_err(map_wait_execution_error)?;
        let current_time = current_time_surface().await;
        Ok(format_wait_pueue_output(mode, &snapshot, &current_time))
    }

    async fn apply_patch_impl(&self, patch: String) -> Result<String, ServiceError> {
        let _guard = self.execution_lock.write().await;
        let adapter = self.resolve_patch_invocation(&patch).await?;
        adapter
            .execute(&patch)
            .map_err(ServiceError::patch_apply_failed)
    }

    async fn apply_splice_impl(&self, splice: String) -> Result<String, ServiceError> {
        let _guard = self.execution_lock.write().await;
        let adapter = self.resolve_splice_invocation(&splice).await?;
        adapter
            .execute(&splice)
            .map_err(ServiceError::splice_apply_failed)
    }

    async fn resolve_path(&self, raw_path: &str) -> Result<PathBuf, ServiceError> {
        let workspace = self.workspace.read().await;
        resolve_user_path(raw_path, workspace.as_deref())
    }

    async fn resolve_patch_invocation(
        &self,
        patch: &str,
    ) -> Result<PatchInvocationAdapter, ServiceError> {
        if let Some(workspace) = self.workspace.read().await.clone() {
            return Ok(PatchInvocationAdapter::for_workspace(
                workspace,
                PatchSourceProvenance::Inline,
            ));
        }

        match patch_workspace_requirement(patch) {
            PatchWorkspaceRequirement::NeedsWorkspace => Err(relative_workspace_error()),
            PatchWorkspaceRequirement::AbsoluteOnly { anchor_dir } => {
                Ok(PatchInvocationAdapter::for_absolute_only(
                    anchor_dir,
                    PatchSourceProvenance::Inline,
                ))
            }
            PatchWorkspaceRequirement::Unanchored => Ok(PatchInvocationAdapter::unanchored(
                PatchSourceProvenance::Inline,
            )),
        }
    }

    async fn resolve_splice_invocation(
        &self,
        splice: &str,
    ) -> Result<SpliceInvocationAdapter, ServiceError> {
        if let Some(workspace) = self.workspace.read().await.clone() {
            return Ok(SpliceInvocationAdapter::for_workspace(workspace));
        }

        match splice_workspace_requirement(splice) {
            SpliceWorkspaceRequirement::NeedsWorkspace => Err(relative_workspace_error()),
            SpliceWorkspaceRequirement::AbsoluteOnly { anchor_dir } => {
                Ok(SpliceInvocationAdapter::for_execution_only(
                    anchor_dir,
                    crate::splice_adapter::SpliceSourceProvenance::Inline,
                ))
            }
            SpliceWorkspaceRequirement::Unanchored => Ok(SpliceInvocationAdapter::unanchored(
                crate::splice_adapter::SpliceSourceProvenance::Inline,
            )),
        }
    }
}

impl SearchTextPathInput {
    pub fn is_empty(&self) -> bool {
        match self {
            SearchTextPathInput::One(path) => path.trim().is_empty(),
            SearchTextPathInput::Many(paths) => paths.is_empty(),
        }
    }

    pub fn paths(&self) -> Vec<&str> {
        match self {
            SearchTextPathInput::One(path) => vec![path.as_str()],
            SearchTextPathInput::Many(paths) => paths.iter().map(String::as_str).collect(),
        }
    }
}

impl From<SearchTextViewInput> for SearchTextView {
    fn from(value: SearchTextViewInput) -> Self {
        match value {
            SearchTextViewInput::Preview => SearchTextView::Preview,
            SearchTextViewInput::Full => SearchTextView::Full,
        }
    }
}

impl From<WaitModeInput> for WaitMode {
    fn from(value: WaitModeInput) -> Self {
        match value {
            WaitModeInput::Any => WaitMode::Any,
            WaitModeInput::All => WaitMode::All,
        }
    }
}

impl ServiceError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            kind: ServiceErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn tool_not_found(message: impl Into<String>) -> Self {
        Self {
            kind: ServiceErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn path_not_found(message: impl Into<String>) -> Self {
        Self {
            kind: ServiceErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn patch_apply_failed(message: impl Into<String>) -> Self {
        Self {
            kind: ServiceErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn splice_apply_failed(message: impl Into<String>) -> Self {
        Self {
            kind: ServiceErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn tool_failure(message: impl Into<String>) -> Self {
        Self {
            kind: ServiceErrorKind::ToolFailure,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ServiceErrorKind {
        self.kind
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ServiceError {}

fn build_mcp_tools(include_set_workspace: bool) -> Result<Vec<Tool>, serde_json::Error> {
    let mut tools = Vec::new();
    if include_set_workspace {
        tools.push(build_mcp_tool::<SetWorkspaceArgs>(
            "set_workspace",
            "设置 relative path 的默认解析基准。",
        )?);
    }
    tools.push(build_mcp_tool::<ListDirectoryArgs>(
        "list_directory",
        "以 ASCII 树列出目录内容。默认显示文件大小与总行数；适合获取文件清单、判断阅读优先级和选择下一步要读的文件；可选显示隐藏项、Git ignore 项以及时间戳字段。",
    )?);
    tools.push(build_mcp_tool::<ReadFileArgs>(
        "read_file",
        "默认返回全文；可选用 line_range 读取局部片段。也可结合 `sample_step`、`sample_lines`、`max_chars` 请求一个低成本局部检查视图。返回结果始终保持 content-first，不附加额外模式头；若发生纵向省略，使用单独一行 `...`，若发生横向裁切，使用 `...[N chars omitted]`。",
    )?);
    tools.push(build_mcp_tool::<SearchTextArgs>(
        "search_text",
        SEARCH_TEXT_TOOL_DESCRIPTION,
    )?);
    tools.push(build_mcp_tool::<WaitPueueArgs>(
        "wait_pueue",
        "等待一个或多个 Pueue 后台 task 进入满足条件的终态，并返回稳定的 wait summary surface。缺省时对调用开始瞬间的未完成 task 快照进行等待；完成后返回可继续交给 `read_file` / `search_text` 的 `log_handle`。",
    )?);
    tools.push(build_mcp_tool::<ApplyPatchArgs>(
        "apply_patch",
        APPLY_PATCH_TOOL_DESCRIPTION,
    )?);
    tools.push(build_mcp_tool::<ApplySpliceArgs>(
        "apply_splice",
        APPLY_SPLICE_TOOL_DESCRIPTION,
    )?);
    Ok(tools)
}

fn build_mcp_tool<T: JsonSchema>(
    name: &'static str,
    description: &'static str,
) -> Result<Tool, serde_json::Error> {
    let schema = schemars::schema_for!(T);
    let input_schema: JsonObject = serde_json::from_value(serde_json::to_value(schema.schema)?)?;
    Ok(Tool::new(
        Cow::Borrowed(name),
        Cow::Borrowed(description),
        Arc::new(input_schema),
    ))
}

fn parse_json_args<T: DeserializeOwned>(arguments: serde_json::Value) -> Result<T, ServiceError> {
    serde_json::from_value(arguments).map_err(|err| ServiceError::invalid_argument(err.to_string()))
}

fn default_relative_path() -> String {
    ".".to_string()
}

fn default_max_depth() -> usize {
    3
}

fn normalize_line_range(
    line_range: Option<LineRangeInput>,
) -> Result<Option<(usize, usize)>, String> {
    match line_range {
        None => Ok(None),
        Some(LineRangeInput::Pair(values)) => {
            if values.len() != 2 {
                return Err("line_range must contain exactly two integers".to_string());
            }
            Ok(Some((values[0], values[1])))
        }
        Some(LineRangeInput::Text(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Err("line_range cannot be empty".to_string());
            }
            let normalized = trimmed.trim_matches(&['[', ']'][..]);
            let parts = normalized
                .split(|ch| [',', '-', ':'].contains(&ch))
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err("line_range must be [start, end] or a string like 'start,end', 'start-end', or 'start:end'".to_string());
            }
            let start = parts[0].parse::<usize>().map_err(|_| {
                "line_range must be [start, end] or a string like 'start,end', 'start-end', or 'start:end'".to_string()
            })?;
            let end = parts[1].parse::<usize>().map_err(|_| {
                "line_range must be [start, end] or a string like 'start,end', 'start-end', or 'start:end'".to_string()
            })?;
            Ok(Some((start, end)))
        }
    }
}

fn map_directory_error(err: std::io::Error) -> ServiceError {
    let message = err.to_string();
    if message.contains("Path does not exist") {
        return ServiceError::path_not_found(message.replace("Path", "Directory"));
    }
    ServiceError::invalid_argument(message)
}

fn map_read_error(err: std::io::Error) -> ServiceError {
    let message = err.to_string();
    if message.contains("Path does not exist") {
        return ServiceError::path_not_found(message);
    }
    ServiceError::invalid_argument(message)
}

fn map_pueue_log_handle_error(err: PueueError) -> ServiceError {
    match err {
        PueueError::InvalidHandle(message) => ServiceError::invalid_argument(message),
        PueueError::TaskDoesNotExist(task_id) => {
            ServiceError::invalid_argument(format!("Task does not exist: {task_id}"))
        }
        PueueError::TaskLogUnavailable(task_id) => {
            ServiceError::invalid_argument(format!("Task log not available: {task_id}"))
        }
        other => ServiceError::tool_failure(other.to_string()),
    }
}

fn map_timestamp_field(field: TimestampFieldInput) -> TimestampField {
    match field {
        TimestampFieldInput::Created => TimestampField::Created,
        TimestampFieldInput::Modified => TimestampField::Modified,
    }
}

fn relative_workspace_error() -> ServiceError {
    ServiceError::invalid_argument(relative_path_requires_workspace_message())
}

fn resolve_wait_timeout(
    timeout_seconds: Option<f64>,
    default_timeout: Option<Duration>,
) -> Result<Duration, ServiceError> {
    match timeout_seconds {
        Some(seconds) => duration_from_seconds(seconds),
        None => Ok(default_timeout.unwrap_or(Duration::MAX)),
    }
}

fn duration_from_seconds(seconds: f64) -> Result<Duration, ServiceError> {
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(ServiceError::invalid_argument(
            "timeout_seconds must be a positive number",
        ));
    }
    Ok(Duration::from_secs_f64(seconds))
}

fn map_wait_argument_error(err: PueueError) -> ServiceError {
    match err {
        PueueError::InvalidTimeout(message) | PueueError::InvalidHandle(message) => {
            ServiceError::invalid_argument(message)
        }
        PueueError::TaskDoesNotExist(task_id) => {
            ServiceError::invalid_argument(format!("Task does not exist: {task_id}"))
        }
        PueueError::TaskLogUnavailable(task_id) => {
            ServiceError::invalid_argument(format!("Task log not available: {task_id}"))
        }
        other => ServiceError::tool_failure(other.to_string()),
    }
}

fn map_wait_execution_error(err: PueueError) -> ServiceError {
    match err {
        PueueError::TaskDoesNotExist(task_id) => {
            ServiceError::invalid_argument(format!("Task does not exist: {task_id}"))
        }
        PueueError::InvalidTimeout(message) => ServiceError::invalid_argument(message),
        other => ServiceError::tool_failure(other.to_string()),
    }
}

fn map_pueue_runtime_error(err: PueueError) -> ServiceError {
    match err {
        PueueError::InvalidTimeout(message) => ServiceError::invalid_argument(message),
        other => ServiceError::tool_failure(other.to_string()),
    }
}

fn format_wait_pueue_output(
    mode: WaitMode,
    snapshot: &PueueWaitSnapshot,
    current_time: &str,
) -> String {
    let mut lines = Vec::new();
    lines.push("wait_pueue:".to_string());
    lines.push(format!("reason: {}", wait_reason_label(snapshot.reason)));
    lines.push(format!("mode: {}", wait_mode_label(mode)));
    push_task_id_line(&mut lines, "resolved_task_ids", &snapshot.resolved_task_ids);
    if !snapshot.triggered_task_ids.is_empty() {
        push_task_id_line(
            &mut lines,
            "triggered_task_ids",
            &snapshot.triggered_task_ids,
        );
    }
    if !snapshot.pending_task_ids.is_empty() {
        push_task_id_line(&mut lines, "pending_task_ids", &snapshot.pending_task_ids);
    }
    lines.push(format!(
        "waited_seconds: {:.1}",
        snapshot.waited.as_secs_f64()
    ));
    lines.push(format!("current_time: {current_time}"));

    if !snapshot.terminal_tasks.is_empty() {
        lines.push(String::new());
        for (index, task) in snapshot.terminal_tasks.iter().enumerate() {
            if index > 0 {
                lines.push(String::new());
            }
            let (status, exit_code) = summarize_terminal_task(task);
            lines.push(format!("[{}] task {}", index + 1, task.id));
            lines.push(format!("  status: {status}"));
            if let Some(exit_code) = exit_code {
                lines.push(format!("  exit_code: {exit_code}"));
            }
            lines.push(format!("  log_handle: {}", format_task_log_handle(task.id)));
        }
    }

    lines.join("\n")
}

fn push_task_id_line(lines: &mut Vec<String>, label: &str, task_ids: &[u64]) {
    if task_ids.is_empty() {
        lines.push(format!("{label}:"));
    } else {
        let rendered = task_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("{label}: {rendered}"));
    }
}

fn wait_mode_label(mode: WaitMode) -> &'static str {
    match mode {
        WaitMode::Any => "any",
        WaitMode::All => "all",
    }
}

fn wait_reason_label(reason: WaitReason) -> &'static str {
    match reason {
        WaitReason::TaskFinished => "task_finished",
        WaitReason::AllFinished => "all_finished",
        WaitReason::Timeout => "timeout",
        WaitReason::NothingToWaitFor => "nothing_to_wait_for",
    }
}

fn summarize_terminal_task(task: &PueueTaskSnapshot) -> (String, Option<i64>) {
    let Some(detail) = task.status.detail.as_ref() else {
        return (task.status.name.clone(), None);
    };
    let (status, exit_code) = extract_terminal_summary(detail);
    (
        status.unwrap_or_else(|| task.status.name.clone()),
        exit_code,
    )
}

fn extract_terminal_summary(detail: &Value) -> (Option<String>, Option<i64>) {
    let direct_exit_code = extract_exit_code(detail);
    if let Some(result) = detail.get("result") {
        let (status, nested_exit_code) = extract_status_and_exit_code(result);
        if status.is_some() || nested_exit_code.is_some() {
            let exit_code = direct_exit_code
                .or(nested_exit_code)
                .or_else(|| status.as_deref().and_then(infer_exit_code));
            return (status, exit_code);
        }
    }

    let (status, nested_exit_code) = extract_status_and_exit_code(detail);
    let exit_code = direct_exit_code
        .or(nested_exit_code)
        .or_else(|| status.as_deref().and_then(infer_exit_code));
    (status, exit_code)
}

fn extract_status_and_exit_code(value: &Value) -> (Option<String>, Option<i64>) {
    match value {
        Value::String(status) => (Some(status.clone()), None),
        Value::Object(object) if object.len() == 1 => {
            let (status, detail) = object.iter().next().expect("single-entry object");
            (Some(status.clone()), extract_exit_code(detail))
        }
        _ => (None, extract_exit_code(value)),
    }
}

fn extract_exit_code(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::Object(object) => object
            .get("exit_code")
            .and_then(Value::as_i64)
            .or_else(|| object.get("code").and_then(Value::as_i64)),
        _ => None,
    }
}

fn infer_exit_code(status: &str) -> Option<i64> {
    (status == "Success").then_some(0)
}

async fn current_time_surface() -> String {
    shell_current_time_surface()
        .await
        .unwrap_or_else(format_utc_now_surface)
}

#[cfg(windows)]
async fn shell_current_time_surface() -> Option<String> {
    let output = TokioCommand::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'",
        ])
        .output()
        .await
        .ok()?;
    normalize_current_time_output(output)
}

#[cfg(not(windows))]
async fn shell_current_time_surface() -> Option<String> {
    let output = TokioCommand::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .await
        .ok()?;
    normalize_current_time_output(output)
}

fn normalize_current_time_output(output: std::process::Output) -> Option<String> {
    if !output.status.success() {
        return None;
    }
    let rendered = String::from_utf8_lossy(&output.stdout).trim().to_string();
    is_current_time_surface(&rendered).then_some(rendered)
}

fn is_current_time_surface(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 19
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b' '
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16) || byte.is_ascii_digit())
}

fn format_utc_now_surface() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

fn relative_path_requires_workspace_message() -> String {
    format!(
        "Relative path requires workspace. Call set_workspace first, set {DEFAULT_WORKSPACE_ENV}, or use an absolute path."
    )
}

pub(crate) async fn resolve_read_surface_path(
    raw_path: &str,
    workspace: Option<&Path>,
) -> Result<PathBuf, ServiceError> {
    if let Some(path) = try_resolve_pueue_log_handle(raw_path).await? {
        return Ok(path);
    }
    resolve_user_path(raw_path, workspace)
}

pub(crate) async fn resolve_search_surface_paths(
    raw_paths: &[String],
    workspace: Option<&Path>,
    display_base_dir: Option<&Path>,
) -> Result<ResolvedSearchTextPaths, ServiceError> {
    let mut search_paths = Vec::new();
    let mut display_labels = Vec::new();
    let mut path_overrides = Vec::new();
    let mut seen = HashSet::new();

    for raw_path in raw_paths {
        if raw_path.trim().is_empty() {
            return Err(ServiceError::invalid_argument(
                "path cannot contain an empty entry",
            ));
        }

        let handle_task_id = parse_task_log_handle(raw_path).map_err(map_pueue_log_handle_error)?;
        let resolved_path = if let Some(task_id) = handle_task_id {
            resolve_task_log_path(task_id).await?
        } else {
            let path = resolve_user_path(raw_path, workspace)?;
            if !path.exists() {
                return Err(ServiceError::path_not_found(format!(
                    "Path does not exist: {}",
                    path.display()
                )));
            }
            path
        };

        if seen.insert(resolved_path.clone()) {
            let display_path = display_path_for_output(display_base_dir, &resolved_path);
            let display_label = handle_task_id
                .map(format_task_log_handle)
                .unwrap_or_else(|| display_path.clone());
            if display_label != display_path {
                path_overrides.push((display_path, display_label.clone()));
            }
            search_paths.push(resolved_path);
            display_labels.push(display_label);
        }
    }

    Ok(ResolvedSearchTextPaths {
        search_paths,
        display_scope: format_display_scope(&display_labels),
        path_overrides,
    })
}

pub(crate) fn rewrite_search_text_surface(
    rendered: String,
    display_scope: &str,
    path_overrides: &[(String, String)],
) -> String {
    rendered
        .lines()
        .map(|line| {
            if line.starts_with("scope: ") {
                return format!("scope: {display_scope}");
            }
            if !line.starts_with('[') {
                return line.to_string();
            }

            let mut updated = line.to_string();
            for (resolved_path, display_path) in path_overrides {
                let needle = format!("{resolved_path} (");
                if updated.contains(&needle) {
                    updated = updated.replacen(&needle, &format!("{display_path} ("), 1);
                    break;
                }
            }
            updated
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn display_path_for_output(display_base_dir: Option<&Path>, path: &Path) -> String {
    let path = normalize_display_path(path);
    if let Some(base_dir) = display_base_dir {
        let base_dir = normalize_display_path(base_dir);
        if let Ok(relative) = path.strip_prefix(&base_dir)
            && !relative.as_os_str().is_empty()
        {
            return normalize_display_text(&relative.display().to_string());
        }
    }
    normalize_display_text(&path.display().to_string())
}

pub(crate) fn format_display_scope(display_paths: &[String]) -> String {
    if display_paths.len() == 1 {
        return display_paths[0].clone();
    }
    format!("[{}]", display_paths.join(", "))
}

fn normalize_display_path(path: &Path) -> PathBuf {
    PathBuf::from(strip_windows_verbatim_prefix(&path.display().to_string()))
}

fn strip_windows_verbatim_prefix(raw: &str) -> String {
    raw.strip_prefix(r"\\?\").unwrap_or(raw).to_string()
}

fn normalize_display_text(raw: &str) -> String {
    raw.replace('\\', "/")
}

fn resolve_user_path(raw_path: &str, workspace: Option<&Path>) -> Result<PathBuf, ServiceError> {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() {
        return Ok(path);
    }

    let workspace = workspace.ok_or_else(relative_workspace_error)?;
    Ok(workspace.join(path))
}

async fn try_resolve_pueue_log_handle(raw_path: &str) -> Result<Option<PathBuf>, ServiceError> {
    let Some(task_id) = parse_task_log_handle(raw_path).map_err(map_pueue_log_handle_error)? else {
        return Ok(None);
    };
    resolve_task_log_path(task_id).await.map(Some)
}

async fn resolve_task_log_path(task_id: u64) -> Result<PathBuf, ServiceError> {
    PueueRuntime::from_env()
        .map_err(map_pueue_runtime_error)?
        .resolve_task_log_path(task_id)
        .await
        .map_err(map_pueue_log_handle_error)
}

pub fn validate_workspace_path(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }
    path.canonicalize()
        .map_err(|err| format!("Invalid path: {err}"))
}

fn default_workspace_from_env() -> Option<PathBuf> {
    let raw = std::env::var_os(DEFAULT_WORKSPACE_ENV)?;
    let rendered = raw.to_string_lossy().trim().to_string();
    if rendered.is_empty() {
        eprintln!("warning: ignoring {DEFAULT_WORKSPACE_ENV}: path is empty");
        return None;
    }

    let path = PathBuf::from(raw);
    match validate_workspace_path(&path) {
        Ok(canonical) => Some(canonical),
        Err(err) => {
            eprintln!(
                "warning: ignoring {DEFAULT_WORKSPACE_ENV}={}: {err}",
                path.display()
            );
            None
        }
    }
}
