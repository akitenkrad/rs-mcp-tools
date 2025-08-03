use crate::clients::Tool;
use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::SseClientTransport,
};
use std::borrow::Cow;

pub async fn start_sse_client(tool: Tool) -> Result<()> {
    let transport = SseClientTransport::start(tool.sse_url.unwrap().as_str()).await?;
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test sse client".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;

    // Initialize
    let server_info = client.peer_info();
    tracing::info!("Connected to server: {server_info:#?}");

    // List tools
    let tools = client.list_tools(Default::default()).await?;
    tracing::info!("Available tools: {tools:#?}");

    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: Cow::Owned(tool.name.to_string()),
            arguments: tool.arguments,
        })
        .await?;
    tracing::info!("Tool result: {tool_result:#?}");

    client.cancel().await?;

    Ok(())
}
