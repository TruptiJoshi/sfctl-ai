use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;

// Import the pwsh module from the parent crate
use sfctl_ai::pwsh::PwshSession;

// Define a wrapper for tracing that writes to a file instead
fn log_to_file(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/mcp-server.log")
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}

#[derive(Clone)]
pub struct ServiceFabricServer {
    tool_router: ToolRouter<ServiceFabricServer>,
    pwsh_session: Arc<Mutex<PwshSession>>,
}

impl ServiceFabricServer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pwsh_session = PwshSession::new()?;
        Ok(Self {
            tool_router: Self::tool_router(),
            pwsh_session: Arc::new(Mutex::new(pwsh_session)),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ServiceFabricCommandParams {
    /// PowerShell command to execute, e.g. "Get-ServiceFabricClusterHealth"
    pub command: String,
}

#[tool_router]
impl ServiceFabricServer {
    #[tool(description = "Execute a Service Fabric PowerShell command")]
    async fn sf_command(
        &self,
        Parameters(ServiceFabricCommandParams { command }): Parameters<ServiceFabricCommandParams>,
    ) -> Result<CallToolResult, McpError> {
        log_to_file(&format!("sf_command called with: {}", command));

        let mut session = self.pwsh_session.lock().await;

        match session.run_command(&command).await {
            Ok(output) => {
                log_to_file(&format!("SF command executed successfully: {}", command));
                let result = if output.is_empty() {
                    format!("Command '{}' executed successfully (no output)", command)
                } else {
                    output
                };
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                log_to_file(&format!("SF command failed: {}", e));
                Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("PowerShell command failed: {}", e)),
                    data: None,
                })
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for ServiceFabricServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Service Fabric MCP Server for managing Service Fabric clusters.

AVAILABLE TOOL:
- sf_command: Execute any Service Fabric PowerShell command

EXAMPLE COMMANDS:
- Import-Module ServiceFabric
- Connect-ServiceFabricCluster -ConnectionEndpoint localhost:19000  
- Connect-ServiceFabricCluster -ConnectionEndpoint remote:19000 -AzureActiveDirectory
- Get-ServiceFabricClusterHealth
- Get-ServiceFabricNode  
- Get-ServiceFabricApplication
- Get-ServiceFabricService
- Get-ServiceFabricPartition
- Get-ServiceFabricReplica
- Update-ServiceFabricService
- New-ServiceFabricApplication
- Remove-ServiceFabricApplication

Use sf_command with any Service Fabric PowerShell cmdlet. For connections, use the full Connect-ServiceFabricCluster command with appropriate parameters.".to_string()),
        }
    }
}
