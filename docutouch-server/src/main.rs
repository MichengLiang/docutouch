mod cli;
mod patch_adapter;
#[allow(dead_code)]
mod pueue;
mod server;
mod splice_adapter;
mod tool_service;
mod transport_shell;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    match cli::dispatch_from_env().await? {
        cli::Dispatch::RunServer => server::run_stdio_server().await,
        cli::Dispatch::Exit(code) => std::process::exit(code),
    }
}
