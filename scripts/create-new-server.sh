#!/bin/bash

set -e

# Function to display usage
usage() {
    echo "Usage: $0 <server-name>"
    echo "  server-name: Name of the new MCP server to create"
    echo ""
    echo "Example: $0 weather"
    echo "  This will create a new MCP server named 'weather' at mcp-servers/weather/"
    exit 1
}

# Check if server name is provided
if [ $# -ne 1 ]; then
    echo "Error: Server name is required"
    usage
fi

SERVER_NAME="$1"
SERVER_DIR="mcp-servers/${SERVER_NAME}"
BINARY_NAME="${SERVER_NAME}-mcp-server"

# Check if we're in the project root
if [ ! -f "Cargo.toml" ] || [ ! -d "mcp-servers" ]; then
    echo "Error: Please run this script from the project root directory"
    exit 1
fi

# Check if server directory already exists
if [ -d "$SERVER_DIR" ]; then
    echo "Error: Server '${SERVER_NAME}' already exists at ${SERVER_DIR}"
    exit 1
fi

echo "Creating new MCP server: ${SERVER_NAME}"
echo "Location: ${SERVER_DIR}"
echo "Binary name: ${BINARY_NAME}"
echo ""

# Create new Rust project
echo "Creating Rust project..."
cd mcp-servers
cargo new "${SERVER_NAME}" --name "${BINARY_NAME}"
cd "${SERVER_NAME}"

# Update Cargo.toml with [[bin]] section
echo "Updating Cargo.toml..."
cat << EOF >> Cargo.toml

[[bin]]
name = "${BINARY_NAME}"
path = "src/main.rs"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
EOF

# Create servers directory structure
echo "Creating servers directory structure..."
mkdir -p src/servers

# Create mcp_io.rs module
cat << 'EOF' > src/servers/mcp_io.rs
//! MCP IO (stdio) server implementation

use anyhow::Result;
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

/// MCP IO Server handler
pub struct McpIoServer {
    // Add server state here
}

impl McpIoServer {
    /// Create a new MCP IO server
    pub fn new() -> Self {
        Self {
            // Initialize server state
        }
    }

    /// Start the MCP IO server
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting MCP IO server...");

        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Err(e) = self.handle_message(&line).await {
                        tracing::error!("Error handling message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming MCP message
    async fn handle_message(&mut self, message: &str) -> Result<()> {
        let trimmed = message.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        // Parse JSON message
        let request: Value = serde_json::from_str(trimmed)?;
        tracing::debug!("Received message: {}", request);

        // TODO: Implement MCP protocol handling
        // For now, just echo back a simple response
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "message": "Hello from MCP server!"
            }
        });

        self.send_response(&response).await?;
        Ok(())
    }

    /// Send response back to client
    async fn send_response(&self, response: &Value) -> Result<()> {
        let response_str = serde_json::to_string(response)?;
        let mut stdout = tokio::io::stdout();
        stdout.write_all(response_str.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }
}
EOF

# Create mcp_sse.rs module
cat << 'EOF' > src/servers/mcp_sse.rs
//! MCP SSE (Server-Sent Events) server implementation

use anyhow::Result;
use serde_json::Value;
use std::convert::Infallible;
use tokio::sync::mpsc;

/// MCP SSE Server handler
pub struct McpSseServer {
    // Add server state here
}

impl McpSseServer {
    /// Create a new MCP SSE server
    pub fn new() -> Self {
        Self {
            // Initialize server state
        }
    }

    /// Start the MCP SSE server
    pub async fn start(&mut self, port: u16) -> Result<()> {
        tracing::info!("Starting MCP SSE server on port {}...", port);

        // TODO: Implement HTTP server with SSE support
        // This is a placeholder implementation
        
        let addr = ([127, 0, 0, 1], port);
        tracing::info!("MCP SSE server would be listening on http://{}", 
                      format!("{}:{}", addr.0.map(|x| x.to_string()).join("."), addr.1));

        // For now, just run indefinitely
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    /// Handle incoming MCP message via HTTP
    async fn handle_http_message(&mut self, message: &str) -> Result<String> {
        let request: Value = serde_json::from_str(message)?;
        tracing::debug!("Received HTTP message: {}", request);

        // TODO: Implement MCP protocol handling for HTTP/SSE
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "message": "Hello from MCP SSE server!"
            }
        });

        Ok(serde_json::to_string(&response)?)
    }

    /// Send SSE event to client
    async fn send_sse_event(&self, event: &str, data: &str) -> Result<()> {
        // TODO: Implement SSE event sending
        tracing::debug!("SSE Event: {} - Data: {}", event, data);
        Ok(())
    }
}
EOF

# Create mod.rs for servers module
cat << 'EOF' > src/servers/mod.rs
//! MCP Server implementations

pub mod mcp_io;
pub mod mcp_sse;

pub use mcp_io::McpIoServer;
pub use mcp_sse::McpSseServer;
EOF

# Update main.rs
cat << 'EOF' > src/main.rs
//! MCP Server Application

mod servers;

use anyhow::Result;
use servers::{McpIoServer, McpSseServer};
use std::env;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::init();

    let args: Vec<String> = env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("stdio") | None => {
            // Default to stdio mode
            tracing::info!("Starting MCP server in stdio mode");
            let mut server = McpIoServer::new();
            server.start().await?;
        }
        Some("sse") => {
            // SSE mode
            let port = args.get(2)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(3000);
            
            tracing::info!("Starting MCP server in SSE mode");
            let mut server = McpSseServer::new();
            server.start(port).await?;
        }
        Some(mode) => {
            eprintln!("Unknown mode: {}. Use 'stdio' or 'sse'", mode);
            std::process::exit(1);
        }
    }

    Ok(())
}
EOF

# Go back to project root
cd ../..

# Update Makefile.toml to include the new server in build-servers task
echo "Updating Makefile.toml..."
if [ -f "Makefile.toml" ]; then
    # Create a temporary file for the updated content
    temp_file=$(mktemp)
    
    # Process the Makefile.toml line by line
    while IFS= read -r line; do
        echo "$line" >> "$temp_file"
        
        # Check if this is the line with sample-calculator build command
        if [[ "$line" == *"cargo build --release -p sample-calculator"* ]]; then
            # Add the new server build command after the sample-calculator line
            echo "    docker compose run --rm instance cargo build --release -p ${SERVER_NAME} && \\" >> "$temp_file"
        fi
    done < "Makefile.toml"
    
    # Replace the original file with the updated content
    mv "$temp_file" "Makefile.toml"
    echo "✅ Added ${SERVER_NAME} to Makefile.toml build-servers task"
else
    echo "⚠️  Warning: Makefile.toml not found. Please manually add the build command:"
    echo "   docker compose run --rm instance cargo build --release -p ${SERVER_NAME} && \\"
fi

echo ""
echo "✅ Successfully created new MCP server: ${SERVER_NAME}"
echo ""
echo "📁 Project structure:"
echo "   ${SERVER_DIR}/"
echo "   ├── Cargo.toml (with [[bin]] section)"
echo "   └── src/"
echo "       ├── main.rs"
echo "       └── servers/"
echo "           ├── mod.rs"
echo "           ├── mcp_io.rs"
echo "           └── mcp_sse.rs"
echo ""
echo "🚀 Next steps:"
echo "   1. cd ${SERVER_DIR}"
echo "   2. cargo check  # Verify the project compiles"
echo "   3. Implement your MCP server logic in the servers/ modules"
echo "   4. Add the server to the workspace build with:"
echo "      cargo build --release -p ${SERVER_NAME}"
echo ""
echo "💡 The server supports two modes:"
echo "   - stdio: ${BINARY_NAME} stdio (default)"
echo "   - sse:   ${BINARY_NAME} sse [port]"
