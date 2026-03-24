use crate::tool_service::ToolService;
use anyhow::Result;
use rmcp::ErrorData as McpError;
use rmcp::ServiceExt;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ListToolsResult, ServerCapabilities, ServerInfo,
};

#[derive(Clone)]
struct DocuTouchServer {
    service: ToolService,
}

pub fn stdio() -> (tokio::io::Stdin, tokio::io::Stdout) {
    (tokio::io::stdin(), tokio::io::stdout())
}

impl DocuTouchServer {
    fn new() -> Result<Self> {
        Ok(Self {
            service: ToolService::for_stdio()?,
        })
    }
}

impl ServerHandler for DocuTouchServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..ServerInfo::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        Ok(ListToolsResult {
            tools: self.service.mcp_tools().as_ref().clone(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let output = self
            .service
            .call_mcp_tool(request.name.as_ref(), request.arguments)
            .await
            .map_err(|message| McpError::invalid_params(message.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

pub async fn run_stdio_server() -> Result<()> {
    let service = DocuTouchServer::new()?;
    let running = service.serve(stdio()).await?;
    running.waiting().await?;
    Ok(())
}
