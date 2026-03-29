use crate::patch_adapter::{PatchInvocationAdapter, PatchNumberedEvidenceMode};
use crate::splice_adapter::SpliceInvocationAdapter;
use crate::tool_service::{
    ToolService, resolve_read_surface_path, resolve_search_surface_paths,
    rewrite_search_text_surface,
};
use crate::transport_shell::TransportSourceProvenance;
use anyhow::Result;
use docutouch_core::{
    DirectoryListOptions, ReadFileOptions, SearchTextView, TimestampField, list_directory,
    normalize_sampled_view_options, read_file_with_sampled_view, search_text,
};
use serde_json::json;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub enum Dispatch {
    RunServer,
    Exit(i32),
}

enum Command {
    List(ListCommand),
    Read(ReadCommand),
    Search(SearchCommand),
    WaitPueue(WaitPueueCommand),
    Patch(PatchCommand),
    Splice(SpliceCommand),
}

struct ListCommand {
    path: String,
    max_depth: usize,
    show_hidden: bool,
    include_gitignored: bool,
    timestamp_fields: Vec<TimestampField>,
}

struct ReadCommand {
    path: String,
    line_range: Option<(usize, usize)>,
    show_line_numbers: bool,
    sample_step: Option<usize>,
    sample_lines: Option<usize>,
    max_chars: Option<usize>,
}

struct SearchCommand {
    query: String,
    paths: Vec<String>,
    rg_args: String,
    view: SearchTextView,
}

struct WaitPueueCommand {
    task_ids: Option<Vec<u64>>,
    mode: Option<String>,
    timeout_seconds: Option<f64>,
}

#[derive(Debug)]
struct PatchCommand {
    patch_file: Option<String>,
    numbered_evidence_mode: Option<PatchNumberedEvidenceMode>,
}

struct SpliceCommand {
    splice_file: Option<String>,
}

pub async fn dispatch_from_env() -> Result<Dispatch> {
    let args = match collect_utf8_args() {
        Ok(args) => args,
        Err(message) => {
            write_stderr(&format!("{message}\n"))?;
            return Ok(Dispatch::Exit(2));
        }
    };

    let program = std::env::args()
        .next()
        .unwrap_or_else(|| "docutouch".to_string());
    if args.is_empty() {
        return Ok(Dispatch::RunServer);
    }

    match args[0].as_str() {
        "serve" => {
            if args.len() == 1 {
                Ok(Dispatch::RunServer)
            } else {
                write_stderr(&format!(
                    "serve does not accept additional arguments\n\n{}",
                    usage(&program)
                ))?;
                Ok(Dispatch::Exit(2))
            }
        }
        "mcp" => parse_mcp_dispatch(&program, &args[1..]),
        "cli" => {
            if args.len() == 1 {
                write_stdout(&usage(&program))?;
                return Ok(Dispatch::Exit(0));
            }
            let command = match parse_command(&args[1..]) {
                Ok(command) => command,
                Err(message) => {
                    write_stderr(&format!("{message}\n\n{}", usage(&program)))?;
                    return Ok(Dispatch::Exit(2));
                }
            };
            let exit_code = run_command(command).await?;
            Ok(Dispatch::Exit(exit_code))
        }
        "help" | "--help" | "-h" => {
            write_stdout(&usage(&program))?;
            Ok(Dispatch::Exit(0))
        }
        _ => {
            let command = match parse_command(&args) {
                Ok(command) => command,
                Err(message) => {
                    write_stderr(&format!("{message}\n\n{}", usage(&program)))?;
                    return Ok(Dispatch::Exit(2));
                }
            };
            let exit_code = run_command(command).await?;
            Ok(Dispatch::Exit(exit_code))
        }
    }
}

fn parse_mcp_dispatch(program: &str, args: &[String]) -> Result<Dispatch> {
    match args {
        [] => Ok(Dispatch::RunServer),
        [subcommand] if subcommand == "serve" => Ok(Dispatch::RunServer),
        [subcommand] if subcommand == "help" || subcommand == "--help" || subcommand == "-h" => {
            write_stdout(&usage(program))?;
            Ok(Dispatch::Exit(0))
        }
        _ => {
            write_stderr(&format!(
                "mcp only accepts an optional `serve` alias\n\n{}",
                usage(program)
            ))?;
            Ok(Dispatch::Exit(2))
        }
    }
}

fn collect_utf8_args() -> Result<Vec<String>, String> {
    std::env::args_os()
        .skip(1)
        .map(os_string_into_string)
        .collect()
}

fn os_string_into_string(value: OsString) -> Result<String, String> {
    value
        .into_string()
        .map_err(|_| "docutouch CLI arguments must be valid UTF-8".to_string())
}

fn parse_command(args: &[String]) -> Result<Command, String> {
    match args[0].as_str() {
        "list" => parse_list_command(&args[1..]).map(Command::List),
        "read" => parse_read_command(&args[1..]).map(Command::Read),
        "search" => parse_search_command(&args[1..]).map(Command::Search),
        "wait-pueue" => parse_wait_pueue_command(&args[1..]).map(Command::WaitPueue),
        "patch" => parse_patch_command(&args[1..]).map(Command::Patch),
        "splice" => parse_splice_command(&args[1..]).map(Command::Splice),
        other => Err(format!("unknown subcommand: {other}")),
    }
}

fn parse_list_command(args: &[String]) -> Result<ListCommand, String> {
    let mut path = None;
    let mut max_depth = 3usize;
    let mut show_hidden = false;
    let mut include_gitignored = false;
    let mut timestamp_fields = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--max-depth=") {
            max_depth = parse_usize_flag("--max-depth", value)?;
        } else if arg == "--max-depth" {
            index += 1;
            max_depth = parse_usize_flag("--max-depth", value_at(args, index, "--max-depth")?)?;
        } else if arg == "--show-hidden" {
            show_hidden = true;
        } else if arg == "--include-gitignored" {
            include_gitignored = true;
        } else if let Some(value) = arg.strip_prefix("--timestamp-field=") {
            timestamp_fields.push(parse_timestamp_field(value)?);
        } else if arg == "--timestamp-field" {
            index += 1;
            timestamp_fields.push(parse_timestamp_field(value_at(
                args,
                index,
                "--timestamp-field",
            )?)?);
        } else if arg.starts_with('-') {
            return Err(format!("unknown list flag: {arg}"));
        } else if path.is_none() {
            path = Some(arg.clone());
        } else {
            return Err("list accepts at most one path argument".to_string());
        }
        index += 1;
    }
    Ok(ListCommand {
        path: path.unwrap_or_else(|| ".".to_string()),
        max_depth,
        show_hidden,
        include_gitignored,
        timestamp_fields,
    })
}

fn parse_read_command(args: &[String]) -> Result<ReadCommand, String> {
    let mut path = None;
    let mut line_range = None;
    let mut show_line_numbers = false;
    let mut sample_step = None;
    let mut sample_lines = None;
    let mut max_chars = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--line-range=") {
            line_range = Some(parse_line_range_text(value)?);
        } else if arg == "--line-range" {
            index += 1;
            line_range = Some(parse_line_range_text(value_at(
                args,
                index,
                "--line-range",
            )?)?);
        } else if arg == "--show-line-numbers" {
            show_line_numbers = true;
        } else if let Some(value) = arg.strip_prefix("--sample-step=") {
            sample_step = Some(parse_usize_flag("--sample-step", value)?);
        } else if arg == "--sample-step" {
            index += 1;
            sample_step = Some(parse_usize_flag(
                "--sample-step",
                value_at(args, index, "--sample-step")?,
            )?);
        } else if let Some(value) = arg.strip_prefix("--sample-lines=") {
            sample_lines = Some(parse_usize_flag("--sample-lines", value)?);
        } else if arg == "--sample-lines" {
            index += 1;
            sample_lines = Some(parse_usize_flag(
                "--sample-lines",
                value_at(args, index, "--sample-lines")?,
            )?);
        } else if let Some(value) = arg.strip_prefix("--max-chars=") {
            max_chars = Some(parse_usize_flag("--max-chars", value)?);
        } else if arg == "--max-chars" {
            index += 1;
            max_chars = Some(parse_usize_flag(
                "--max-chars",
                value_at(args, index, "--max-chars")?,
            )?);
        } else if arg.starts_with('-') {
            return Err(format!("unknown read flag: {arg}"));
        } else if path.is_none() {
            path = Some(arg.clone());
        } else {
            return Err("read accepts exactly one path argument".to_string());
        }
        index += 1;
    }
    let path = path.ok_or_else(|| "read requires a file path".to_string())?;
    Ok(ReadCommand {
        path,
        line_range,
        show_line_numbers,
        sample_step,
        sample_lines,
        max_chars,
    })
}

fn parse_search_command(args: &[String]) -> Result<SearchCommand, String> {
    let mut positionals = Vec::new();
    let mut rg_args = String::new();
    let mut view = SearchTextView::Preview;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--rg-args=") {
            rg_args = value.to_string();
        } else if arg == "--rg-args" {
            index += 1;
            rg_args = value_at(args, index, "--rg-args")?.to_string();
        } else if let Some(value) = arg.strip_prefix("--view=") {
            view = parse_search_view(value)?;
        } else if arg == "--view" {
            index += 1;
            view = parse_search_view(value_at(args, index, "--view")?)?;
        } else if arg.starts_with('-') {
            return Err(format!("unknown search flag: {arg}"));
        } else {
            positionals.push(arg.clone());
        }
        index += 1;
    }
    if positionals.is_empty() {
        return Err("search requires a query and at least one path".to_string());
    }
    if positionals.len() == 1 {
        return Err("search requires at least one path after the query".to_string());
    }
    Ok(SearchCommand {
        query: positionals[0].clone(),
        paths: positionals[1..].to_vec(),
        rg_args,
        view,
    })
}

fn parse_wait_pueue_command(args: &[String]) -> Result<WaitPueueCommand, String> {
    let mut task_ids = Vec::new();
    let mut mode = None;
    let mut timeout_seconds = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--mode=") {
            mode = Some(parse_wait_mode(value)?);
        } else if arg == "--mode" {
            index += 1;
            mode = Some(parse_wait_mode(value_at(args, index, "--mode")?)?);
        } else if let Some(value) = arg.strip_prefix("--timeout-seconds=") {
            timeout_seconds = Some(parse_positive_f64_flag("--timeout-seconds", value)?);
        } else if arg == "--timeout-seconds" {
            index += 1;
            timeout_seconds = Some(parse_positive_f64_flag(
                "--timeout-seconds",
                value_at(args, index, "--timeout-seconds")?,
            )?);
        } else if arg.starts_with('-') {
            return Err(format!("unknown wait-pueue flag: {arg}"));
        } else {
            task_ids.push(parse_task_id(arg)?);
        }
        index += 1;
    }

    Ok(WaitPueueCommand {
        task_ids: (!task_ids.is_empty()).then_some(task_ids),
        mode,
        timeout_seconds,
    })
}

fn parse_patch_command(args: &[String]) -> Result<PatchCommand, String> {
    let mut patch_file = None;
    let mut numbered_evidence_mode = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--patch-file=") {
            patch_file = Some(value.to_string());
        } else if arg == "--patch-file" {
            index += 1;
            patch_file = Some(value_at(args, index, "--patch-file")?.to_string());
        } else if let Some(value) = arg.strip_prefix("--numbered-evidence-mode=") {
            numbered_evidence_mode = Some(PatchNumberedEvidenceMode::parse(value)?);
        } else if arg == "--numbered-evidence-mode" {
            index += 1;
            numbered_evidence_mode = Some(PatchNumberedEvidenceMode::parse(value_at(
                args,
                index,
                "--numbered-evidence-mode",
            )?)?);
        } else if arg.starts_with('-') {
            return Err(format!("unknown patch flag: {arg}"));
        } else if patch_file.is_none() {
            patch_file = Some(arg.clone());
        } else {
            return Err("patch accepts at most one patch file argument".to_string());
        }
        index += 1;
    }
    Ok(PatchCommand {
        patch_file,
        numbered_evidence_mode,
    })
}

fn parse_splice_command(args: &[String]) -> Result<SpliceCommand, String> {
    Ok(SpliceCommand {
        splice_file: parse_optional_transport_file_arg(
            args,
            "splice",
            "--splice-file",
            "splice file",
        )?,
    })
}

async fn run_command(command: Command) -> Result<i32> {
    match command {
        Command::List(command) => run_list(command).await,
        Command::Read(command) => run_read(command).await,
        Command::Search(command) => run_search(command).await,
        Command::WaitPueue(command) => run_wait_pueue(command).await,
        Command::Patch(command) => run_patch(command).await,
        Command::Splice(command) => run_splice(command).await,
    }
}

async fn run_list(command: ListCommand) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let path = resolve_cli_path(&cwd, &command.path);
    let result = list_directory(
        &path,
        DirectoryListOptions {
            max_depth: command.max_depth,
            show_hidden: command.show_hidden,
            include_gitignored: command.include_gitignored,
            timestamp_fields: command.timestamp_fields,
        },
    );
    emit_result(result.map(|value| value.display()))
}

async fn run_read(command: ReadCommand) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let path = match resolve_read_surface_path(&command.path, Some(cwd.as_path())).await {
        Ok(path) => path,
        Err(error) => return emit_text_result(Err(error.to_string())),
    };
    let sampled_view = match normalize_sampled_view_options(
        command.sample_step,
        command.sample_lines,
        command.max_chars,
    ) {
        Ok(sampled_view) => sampled_view,
        Err(message) => {
            write_stderr(&message)?;
            return Ok(1);
        }
    };
    let result = read_file_with_sampled_view(
        &path,
        ReadFileOptions {
            line_range: command.line_range,
            show_line_numbers: command.show_line_numbers,
        },
        sampled_view,
    )
    .map(|value| value.content)
    .map_err(|err| err.to_string());
    emit_text_result(result)
}

async fn run_search(command: SearchCommand) -> Result<i32> {
    if command.query.trim().is_empty() {
        return emit_result(Err(std::io::Error::other("query cannot be empty")));
    }
    let cwd = std::env::current_dir()?;
    let resolved = match resolve_search_surface_paths(
        &command.paths,
        Some(cwd.as_path()),
        Some(cwd.as_path()),
    )
    .await
    {
        Ok(resolved) => resolved,
        Err(error) => return emit_text_result(Err(error.to_string())),
    };
    let result = search_text(
        &command.query,
        &resolved.search_paths,
        &command.rg_args,
        command.view,
        Some(cwd.as_path()),
    )
    .await
    .map(|rendered| {
        rewrite_search_text_surface(rendered, &resolved.display_scope, &resolved.path_overrides)
    });
    emit_text_result(result)
}

async fn run_wait_pueue(command: WaitPueueCommand) -> Result<i32> {
    let service = ToolService::for_stdio()?;
    let arguments = json!({
        "task_ids": command.task_ids,
        "mode": command.mode,
        "timeout_seconds": command.timeout_seconds,
    });
    emit_text_result(
        service
            .call_json("wait_pueue", arguments)
            .await
            .map_err(|error| error.to_string()),
    )
}

async fn run_patch(command: PatchCommand) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let patch_source_path = resolve_transport_source_path(&cwd, command.patch_file.as_deref());
    let patch_source = transport_source_from_path(patch_source_path.as_deref());
    let patch = read_transport_text(patch_source_path.as_deref())?;
    let execution_anchor = infer_patch_execution_anchor(&cwd, patch_source_path.as_deref());
    let adapter = PatchInvocationAdapter::for_cli_with_anchors(
        execution_anchor.clone(),
        Some(execution_anchor),
        patch_source,
        command.numbered_evidence_mode,
    );
    emit_text_result(adapter.execute(&patch))
}

async fn run_splice(command: SpliceCommand) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let splice_source_path = resolve_transport_source_path(&cwd, command.splice_file.as_deref());
    let splice = read_transport_text(splice_source_path.as_deref())?;
    let splice_source = transport_source_from_path(splice_source_path.as_deref());
    let adapter = SpliceInvocationAdapter::for_cli(cwd, splice_source);
    emit_text_result(adapter.execute(&splice))
}

fn resolve_transport_source_path(cwd: &Path, source_file: Option<&str>) -> Option<PathBuf> {
    source_file.map(|raw_path| resolve_cli_path(cwd, raw_path))
}

fn read_transport_text(source_path: Option<&Path>) -> Result<String> {
    if let Some(path) = source_path {
        return Ok(std::fs::read_to_string(path)?);
    }

    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn transport_source_from_path(source_path: Option<&Path>) -> TransportSourceProvenance {
    source_path
        .map(|path| TransportSourceProvenance::File(path.to_path_buf()))
        .unwrap_or(TransportSourceProvenance::Inline)
}

fn infer_patch_execution_anchor(cwd: &Path, patch_source_path: Option<&Path>) -> PathBuf {
    patch_source_path
        .and_then(failed_patch_workspace_root)
        .unwrap_or_else(|| cwd.to_path_buf())
}

fn failed_patch_workspace_root(path: &Path) -> Option<PathBuf> {
    let failed_patches_dir = path.parent()?;
    if failed_patches_dir
        .file_name()
        .and_then(|name| name.to_str())
        != Some("failed-patches")
    {
        return None;
    }
    let docutouch_dir = failed_patches_dir.parent()?;
    if docutouch_dir.file_name().and_then(|name| name.to_str()) != Some(".docutouch") {
        return None;
    }
    docutouch_dir.parent().map(Path::to_path_buf)
}

fn parse_optional_transport_file_arg(
    args: &[String],
    command_name: &str,
    long_flag: &str,
    file_label: &str,
) -> Result<Option<String>, String> {
    let inline_flag = format!("{long_flag}=");
    let mut source_file = None;
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix(&inline_flag) {
            source_file = Some(value.to_string());
        } else if arg == long_flag {
            index += 1;
            source_file = Some(value_at(args, index, long_flag)?.to_string());
        } else if arg.starts_with('-') {
            return Err(format!("unknown {command_name} flag: {arg}"));
        } else if source_file.is_none() {
            source_file = Some(arg.clone());
        } else {
            return Err(format!(
                "{command_name} accepts at most one {file_label} argument"
            ));
        }
        index += 1;
    }
    Ok(source_file)
}

fn emit_result(result: std::io::Result<String>) -> Result<i32> {
    match result {
        Ok(message) => {
            write_stdout(&message)?;
            Ok(0)
        }
        Err(err) => {
            write_stderr(&err.to_string())?;
            Ok(1)
        }
    }
}

fn emit_text_result(result: Result<String, String>) -> Result<i32> {
    match result {
        Ok(message) => {
            write_stdout(&message)?;
            Ok(0)
        }
        Err(message) => {
            write_stderr(&message)?;
            Ok(1)
        }
    }
}

fn resolve_cli_path(cwd: &Path, raw_path: &str) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn parse_usize_flag(flag: &str, value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("{flag} requires an integer value"))
}

fn parse_timestamp_field(value: &str) -> Result<TimestampField, String> {
    match value {
        "created" => Ok(TimestampField::Created),
        "modified" => Ok(TimestampField::Modified),
        _ => Err(format!(
            "--timestamp-field must be `created` or `modified`, got `{value}`"
        )),
    }
}

fn parse_search_view(value: &str) -> Result<SearchTextView, String> {
    match value {
        "preview" => Ok(SearchTextView::Preview),
        "full" => Ok(SearchTextView::Full),
        _ => Err(format!("--view must be `preview` or `full`, got `{value}`")),
    }
}

fn parse_wait_mode(value: &str) -> Result<String, String> {
    match value {
        "any" | "all" => Ok(value.to_string()),
        _ => Err(format!("--mode must be `any` or `all`, got `{value}`")),
    }
}

fn parse_positive_f64_flag(flag: &str, value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("{flag} requires a positive number"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("{flag} requires a positive number"));
    }
    Ok(parsed)
}

fn parse_task_id(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("wait-pueue task IDs must be non-negative integers, got `{value}`"))
}

fn parse_line_range_text(text: &str) -> Result<(usize, usize), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("--line-range cannot be empty".to_string());
    }
    let normalized = trimmed.trim_matches(&['[', ']'][..]);
    let parts = normalized
        .split(|ch| [',', '-', ':'].contains(&ch))
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("--line-range must be `start,end`, `start-end`, or `start:end`".to_string());
    }
    let start = parts[0]
        .parse::<usize>()
        .map_err(|_| "--line-range must contain integers".to_string())?;
    let end = parts[1]
        .parse::<usize>()
        .map_err(|_| "--line-range must contain integers".to_string())?;
    Ok((start, end))
}

fn value_at<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn usage(program: &str) -> String {
    format!(
        "Usage:\n  {program}                Start the stdio MCP server\n  {program} mcp            Start the stdio MCP server\n  {program} serve          Start the stdio MCP server (alias)\n  {program} help           Show this help\n  {program} list [path] [--max-depth N] [--show-hidden] [--include-gitignored] [--timestamp-field created|modified]\n  {program} read <path> [--line-range START:END] [--show-line-numbers] [--sample-step N] [--sample-lines N] [--max-chars N]\n  {program} search <query> <path> [more_paths...] [--rg-args '...'] [--view preview|full]\n  {program} wait-pueue [TASK_ID ...] [--mode any|all] [--timeout-seconds N]\n  {program} patch [patch-file] [--numbered-evidence-mode header_only|full]\n  {program} patch --patch-file <path> [--numbered-evidence-mode header_only|full]\n  {program} splice [splice-file]\n  {program} splice --splice-file <path>\n  {program} cli <subcommand> ...    Run the same CLI commands through an explicit group alias\n\nNotes:\n  - Running `{program}` with no subcommand starts the stdio MCP server.\n  - `mcp` is an explicit alias for the same stdio MCP server entrypoint.\n  - Top-level `list`, `read`, `search`, `wait-pueue`, `patch`, and `splice` are the primary local CLI surface.\n  - `cli <subcommand>` remains available when you want an explicit grouping prefix.\n  - CLI relative paths resolve against the current working directory.\n  - `read` enters sampled local inspection mode when any sampled flag is present; omitted sampled flags are filled with stable defaults.\n  - `read` preserves full visible line width unless `--max-chars` is explicitly provided.\n  - `search` preserves the MCP `search_text` contract, including grouped preview/full views.\n  - `wait-pueue` preserves the MCP `wait_pueue` contract and returns the same wait summary surface.\n  - `patch` preserves MCP patch diagnostics and reads patch text from stdin when no file is provided.\n  - `patch` recovers the workspace anchor from `.docutouch/failed-patches/*.patch` when such a file is passed as a patch-file source.\n  - `patch` defaults to `header_only` numbered-evidence interpretation unless overridden by environment or `--numbered-evidence-mode`.\n  - `splice` reads splice text from stdin when no file is provided and applies the current splice runtime."
    )
}

fn write_stdout(message: &str) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn write_stderr(message: &str) -> Result<()> {
    let mut stderr = std::io::stderr().lock();
    stderr.write_all(message.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        failed_patch_workspace_root, infer_patch_execution_anchor,
        parse_optional_transport_file_arg, parse_patch_command, parse_splice_command,
        read_transport_text, resolve_cli_path, resolve_transport_source_path,
        transport_source_from_path,
    };
    use crate::patch_adapter::PatchNumberedEvidenceMode;
    use crate::transport_shell::TransportSourceProvenance;
    use std::path::Path;

    #[test]
    fn patch_and_splice_commands_share_optional_file_parsing() {
        let patch = parse_patch_command(&["--patch-file=input.patch".to_string()])
            .expect("patch parse should succeed");
        let splice = parse_splice_command(&["input.splice".to_string()])
            .expect("splice parse should succeed");

        assert_eq!(patch.patch_file.as_deref(), Some("input.patch"));
        assert_eq!(patch.numbered_evidence_mode, None);
        assert_eq!(splice.splice_file.as_deref(), Some("input.splice"));
    }

    #[test]
    fn patch_command_accepts_numbered_evidence_mode_flag() {
        let patch = parse_patch_command(&[
            "input.patch".to_string(),
            "--numbered-evidence-mode=full".to_string(),
        ])
        .expect("patch parse should succeed");

        assert_eq!(patch.patch_file.as_deref(), Some("input.patch"));
        assert_eq!(
            patch.numbered_evidence_mode,
            Some(PatchNumberedEvidenceMode::Full)
        );
    }

    #[test]
    fn patch_command_rejects_unknown_numbered_evidence_mode() {
        let error = parse_patch_command(&["--numbered-evidence-mode=weird".to_string()])
            .expect_err("patch parse should fail");
        assert_eq!(
            error,
            "--numbered-evidence-mode must be `header_only` or `full`, got `weird`"
        );
    }

    #[test]
    fn shared_transport_file_parser_preserves_command_specific_errors() {
        let patch_err = parse_optional_transport_file_arg(
            &["--unknown".to_string()],
            "patch",
            "--patch-file",
            "patch file",
        )
        .expect_err("patch parse should fail");
        let splice_err = parse_optional_transport_file_arg(
            &["one.splice".to_string(), "two.splice".to_string()],
            "splice",
            "--splice-file",
            "splice file",
        )
        .expect_err("splice parse should fail");

        assert_eq!(patch_err, "unknown patch flag: --unknown");
        assert_eq!(
            splice_err,
            "splice accepts at most one splice file argument"
        );
    }

    #[test]
    fn transport_source_and_reader_share_cli_path_resolution() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cwd = temp.path();
        let nested = cwd.join("input.patch");
        std::fs::write(&nested, "hello\n").expect("write input");

        let source_path = resolve_transport_source_path(cwd, Some("input.patch"));
        let source = transport_source_from_path(source_path.as_deref());
        let text = read_transport_text(source_path.as_deref()).expect("read transport text");

        match source {
            TransportSourceProvenance::Inline => panic!("expected file provenance"),
            TransportSourceProvenance::File(path) => {
                assert_eq!(path, resolve_cli_path(cwd, "input.patch"));
            }
        }
        assert_eq!(text, "hello\n");
    }

    #[test]
    fn shared_transport_reader_reads_absolute_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cwd = Path::new("unused");
        let source = temp.path().join("input.splice");
        std::fs::write(&source, "splice\n").expect("write source");

        let source_path =
            resolve_transport_source_path(cwd, Some(source.to_str().expect("utf-8 path")));
        let text = read_transport_text(source_path.as_deref()).expect("read");

        assert_eq!(text, "splice\n");
    }

    #[test]
    fn failed_patch_workspace_root_recovers_docutouch_workspace_parent() {
        let path = Path::new("workspace")
            .join(".docutouch")
            .join("failed-patches")
            .join("retry.patch");

        assert_eq!(
            failed_patch_workspace_root(&path),
            Some(Path::new("workspace").to_path_buf())
        );
    }

    #[test]
    fn failed_patch_workspace_root_ignores_regular_patch_files() {
        let path = Path::new("workspace").join("patches").join("retry.patch");

        assert_eq!(failed_patch_workspace_root(&path), None);
    }

    #[test]
    fn infer_patch_execution_anchor_prefers_failed_patch_workspace_root() {
        let cwd = Path::new("cwd");
        let path = Path::new("workspace")
            .join(".docutouch")
            .join("failed-patches")
            .join("retry.patch");

        assert_eq!(
            infer_patch_execution_anchor(cwd, Some(&path)),
            Path::new("workspace")
        );
        assert_eq!(infer_patch_execution_anchor(cwd, None), cwd);
    }
}
