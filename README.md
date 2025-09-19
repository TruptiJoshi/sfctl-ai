# Service Fabric MCP Server for VS Code

## Overview

This repository contains a Service Fabric MCP (Model Context Protocol) Server that integrates with VS Code to provide natural language interactions with Service Fabric clusters.

## Prerequisites

- **Rust** (latest stable version)
- **VS Code** with MCP extension
- **Service Fabric SDK** installed on your machine
- **PowerShell** (Windows PowerShell or PowerShell Core)
- **Service Fabric Local Cluster** (for local development)

## Setup

### 1. Setup Service Fabric Local Cluster

First, ensure you have a local Service Fabric cluster running:

1. **Install Service Fabric SDK** from Microsoft
2. **Start Local Cluster Manager**:
   - Search for "Service Fabric Local Cluster Manager" in Start Menu
   - Click "Setup Local Cluster" → "1 Node"
   - Wait for cluster to start (status shows "Ready")

3. **Verify cluster is running**:
   - Open PowerShell and run: `Connect-ServiceFabricCluster localhost:19000`
   - Check: `Get-ServiceFabricClusterHealth`

### 2. Build and Install MCP Server

```bash
cd sfctl-ai/sfctl-ai
cargo build --release
cargo install --path . --bin sfctl-ai-mcp
```

### 3. Configure VS Code

Create `.vscode/mcp.json` in your workspace:

```json
{
  "servers": {
    "sfctl-ai-server": {
      "type": "stdio",
      "command": "C:\\Users\\[USERNAME]\\.cargo\\bin\\sfctl-ai-mcp.exe",
      "args": [],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  },
  "inputs": []
}
```

**Note**: Replace `[USERNAME]` with your actual Windows username. The full path ensures VS Code can find the executable.

**Note**: The `RUST_LOG: debug` environment variable helps with troubleshooting if needed.

### 4. Enable in GitHub Copilot Chat

1. Open VS Code in this workspace
2. Ensure MCP extension is installed
3. Open GitHub Copilot Chat (Ctrl+Shift+I or click chat icon)
4. **Enable the Service Fabric MCP tool**:
   - In Copilot Chat, type `@` to see available tools
   - Look for `@sfctl-ai` in the list
   - Click to enable it, or use it directly in your chat

### 5. Start Using

Use natural language commands in GitHub Copilot Chat with the `@sfctl-ai` tool:

- `@sfctl-ai Connect to my local Service Fabric cluster`
- `@sfctl-ai Show me cluster health`
- `@sfctl-ai List all applications`

## Available Tools

### Connection Management

- **`sf_connect`** - Connect to Service Fabric cluster
- **`sf_connection_status`** - Check current connection status

### Cluster Information

- **`sf_cluster_health`** - Get cluster health status
- **`sf_applications`** - List applications in cluster
- **`sf_services`** - List services in cluster  
- **`sf_nodes`** - List cluster nodes

### Advanced Operations

- **`sf_command`** - Execute custom Service Fabric PowerShell commands

## Usage Examples

### Basic Workflow with Local Cluster

1. **Connect to Local Cluster**:

   ```text
   "Connect to local Service Fabric cluster"
   ```

2. **Check Cluster Health**:

   ```text
   "Show me the cluster health"
   ```

3. **List Applications**:

   ```text
   "What applications are running in the cluster?"
   ```

4. **Custom Commands**:

   ```text
   "Execute Get-ServiceFabricApplication command"
   ```

## Natural Language Examples

The MCP server supports natural language queries:

- "Connect to my Service Fabric cluster"
- "Show me all applications"
- "What's the health status of the cluster?"
- "List all services in MyApp"
- "Show me cluster nodes"
- "Check connection status"

## Development

### Running in Development Mode

```bash
cd sfctl-ai/sfctl-ai
cargo run --bin sfctl-ai-mcp
```

### Building for Release

```bash
cargo build --release
cargo install --path . --bin sfctl-ai-mcp --force
```

### Logs

Check application logs in the `logs/` directory:

- `mcp-server.log` - MCP server operations
- `sfctl-ai.log.*` - General application logs

**Note**: This MCP server provides a bridge between natural language interactions in VS Code and Service Fabric cluster management operations.
