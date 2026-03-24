use crate::patch_adapter::{PatchInvocationAdapter, PatchSourceProvenance};
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
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimestampFieldInput {
    Created,
    Modified,
}

#[derive(Debug)]
pub struct ServiceError {
    message: String,
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
        let file_path = self.resolve_path(&args.relative_path).await?;
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
        let search_paths = self.resolve_search_text_paths(&args.path).await?;
        search_text(
            &args.query,
            &search_paths,
            &args.rg_args,
            args.view.into(),
            display_base_dir.as_deref(),
        )
        .await
        .map_err(ServiceError::invalid_argument)
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

    async fn resolve_search_text_paths(
        &self,
        input: &SearchTextPathInput,
    ) -> Result<Vec<PathBuf>, ServiceError> {
        let mut resolved = Vec::new();
        let mut seen = HashSet::new();
        for raw_path in input.paths() {
            if raw_path.trim().is_empty() {
                return Err(ServiceError::invalid_argument(
                    "path cannot contain an empty entry",
                ));
            }
            let path = self.resolve_path(raw_path).await?;
            if !path.exists() {
                return Err(ServiceError::path_not_found(format!(
                    "Path does not exist: {}",
                    path.display()
                )));
            }
            if seen.insert(path.clone()) {
                resolved.push(path);
            }
        }
        Ok(resolved)
    }

    async fn resolve_path(&self, raw_path: &str) -> Result<PathBuf, ServiceError> {
        let path = PathBuf::from(raw_path);
        if path.is_absolute() {
            return Ok(path);
        }

        let workspace = self.workspace.read().await;
        let workspace = workspace.as_ref().ok_or_else(relative_workspace_error)?;
        Ok(workspace.join(path))
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

impl ServiceError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn tool_not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn path_not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn patch_apply_failed(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn splice_apply_failed(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
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

fn map_timestamp_field(field: TimestampFieldInput) -> TimestampField {
    match field {
        TimestampFieldInput::Created => TimestampField::Created,
        TimestampFieldInput::Modified => TimestampField::Modified,
    }
}

fn relative_workspace_error() -> ServiceError {
    ServiceError::invalid_argument(relative_path_requires_workspace_message())
}

fn relative_path_requires_workspace_message() -> String {
    format!(
        "Relative path requires workspace. Call set_workspace first, set {DEFAULT_WORKSPACE_ENV}, or use an absolute path."
    )
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
