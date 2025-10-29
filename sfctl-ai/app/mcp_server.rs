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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ServiceFabricConnectParams {
    /// Connection endpoint, e.g. "localhost:19000" for local cluster
    pub endpoint: Option<String>,
    /// Use Azure Active Directory authentication for remote clusters
    pub use_aad: Option<bool>,
}



#[tool_router]
impl ServiceFabricServer {
    #[tool(description = "Connect to a Service Fabric cluster")]
    async fn sf_connect(
        &self,
        Parameters(ServiceFabricConnectParams { endpoint, use_aad }): Parameters<ServiceFabricConnectParams>,
    ) -> Result<CallToolResult, McpError> {
        let endpoint = endpoint.unwrap_or_else(|| "localhost:19000".to_string());
        
        // Auto-detect if AAD should be used based on endpoint
        let should_use_aad = use_aad.unwrap_or_else(|| {
            // Use AAD for remote clusters (not localhost or 127.0.0.1)
            !endpoint.starts_with("localhost") && !endpoint.starts_with("127.0.0.1")
        });
        
        log_to_file(&format!(
            "sf_connect called with endpoint: {}, use_aad: {}", 
            endpoint, should_use_aad
        ));

        let mut session = self.pwsh_session.lock().await;

        // First import the Service Fabric module
        match session.run_command("Import-Module ServiceFabric").await {
            Ok(_) => log_to_file("ServiceFabric module imported successfully"),
            Err(e) => {
                log_to_file(&format!("Failed to import ServiceFabric module: {}", e));
                return Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("Failed to import ServiceFabric module: {}", e)),
                    data: None,
                });
            }
        }

        // Build connection command based on authentication method
        let connect_command = if should_use_aad {
            format!(
                "Connect-ServiceFabricCluster -ConnectionEndpoint {} -AzureActiveDirectory",
                endpoint
            )
        } else {
            format!(
                "Connect-ServiceFabricCluster -ConnectionEndpoint {}",
                endpoint
            )
        };
        
        log_to_file(&format!("Executing connection command: {}", connect_command));

        match session.run_command(&connect_command).await {
            Ok(output) => {
                log_to_file(&format!("Connected to SF cluster: {}", output));
                let auth_method = if should_use_aad { "AAD" } else { "None" };
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Connected to Service Fabric cluster at {} using {} authentication\n{}",
                    endpoint, auth_method, output
                ))]))
            }
            Err(e) => {
                log_to_file(&format!("Failed to connect to SF cluster: {}", e));
                Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!(
                        "Failed to connect to Service Fabric cluster: {}",
                        e
                    )),
                    data: None,
                })
            }
        }
    }

    #[tool(description = "Get Service Fabric cluster health status")]
    async fn sf_cluster_health(&self) -> Result<CallToolResult, McpError> {
        log_to_file("sf_cluster_health called");
        let mut session = self.pwsh_session.lock().await;
        
        match session.run_command("Get-ServiceFabricClusterHealth").await {
            Ok(output) => {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Cluster Health Status:\n{}",
                    output
                ))]))
            }
            Err(e) => {
                log_to_file(&format!("sf_cluster_health failed: {}", e));
                Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("Failed to get cluster health: {}", e)),
                    data: None,
                })
            }
        }
    }

    #[tool(description = "List all applications in the Service Fabric cluster")]
    async fn sf_applications(&self) -> Result<CallToolResult, McpError> {
        log_to_file("sf_applications called");
        let mut session = self.pwsh_session.lock().await;
        
        match session.run_command("Get-ServiceFabricApplication").await {
            Ok(output) => {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Applications in cluster:\n{}",
                    output
                ))]))
            }
            Err(e) => {
                log_to_file(&format!("sf_applications failed: {}", e));
                Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("Failed to get applications: {}", e)),
                    data: None,
                })
            }
        }
    }

    #[tool(description = "List all services in the Service Fabric cluster")]  
    async fn sf_services(&self) -> Result<CallToolResult, McpError> {
        log_to_file("sf_services called");
        let mut session = self.pwsh_session.lock().await;
        
        match session.run_command("Get-ServiceFabricService").await {
            Ok(output) => {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Services in cluster:\n{}",
                    output
                ))]))
            }
            Err(e) => {
                log_to_file(&format!("sf_services failed: {}", e));
                Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("Failed to get services: {}", e)),
                    data: None,
                })
            }
        }
    }

    #[tool(description = "List all nodes in the Service Fabric cluster")]
    async fn sf_nodes(&self) -> Result<CallToolResult, McpError> {
        log_to_file("sf_nodes called");
        let mut session = self.pwsh_session.lock().await;
        
        match session.run_command("Get-ServiceFabricNode").await {
            Ok(output) => {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Nodes in cluster:\n{}",
                    output
                ))]))
            }
            Err(e) => {
                log_to_file(&format!("sf_nodes failed: {}", e));
                Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("Failed to get nodes: {}", e)),
                    data: None,
                })
            }
        }
    }



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
            instructions: Some("Service Fabric AI Assistant for managing Service Fabric clusters. 

AVAILABLE TOOLS:
- sf_connect: Connect to Service Fabric clusters (auto-detects AAD for remote endpoints)
- sf_command: Execute any Service Fabric PowerShell command

COMMON WORKFLOWS:
1. Cluster Management: Get-ServiceFabricClusterHealth, Get-ServiceFabricNode
2. Application Management: Get-ServiceFabricApplication, Get-ServiceFabricService, Get-ServiceFabricServiceHealth
3. Deployment: New-ServiceFabricApplication, Remove-ServiceFabricApplication
4. Monitoring: Get-ServiceFabricPartition, Get-ServiceFabricReplica, Get-ServiceFabricReplicaHealth
5. Troubleshooting: Get-ServiceFabricApplicationHealth, Get-ServiceFabricServiceHealth, Get-ServiceFabricNodeHealth

SFCTL COMMAND CATEGORIES:
- application: Create, delete, and manage applications and application types
- cluster: Select, manage, and operate Service Fabric clusters  
- node: Manage cluster nodes
- service: Create, delete and manage services, service types and service packages
- partition: Query and manage partitions for any service
- replica: Manage replicas that belong to service partitions
- events: Retrieve events from the events store
- chaos: Start, stop, and report on the chaos test service

Use natural language - I'll translate to appropriate PowerShell commands automatically.".to_string()),
        }
    }
}
