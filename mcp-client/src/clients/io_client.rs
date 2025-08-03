use crate::clients::Tool;
use rmcp::{model::CallToolRequestParam, service::ServiceExt, transport::TokioChildProcess};
use std::borrow::Cow;
use tokio::process::Command;

pub async fn start_io_client(tool: Tool) -> anyhow::Result<String> {
    let transport = TokioChildProcess::new(Command::new(tool.io_path.clone().unwrap()))?;
    let service = ().serve(transport).await?;

    // Initialize
    let server_info = service.peer_info();
    if cfg!(test) {
        tracing::info!("Connected to server: {server_info:#?}");
    }

    // List tools
    let tools = service.list_all_tools().await?;
    if cfg!(test) {
        tracing::info!("Available tools: {tools:#?}");
        tracing::info!("Calling tool: {:#?}", tool);
    }

    // Call a tool
    let tool_result = service
        .call_tool(CallToolRequestParam {
            name: Cow::Owned(tool.name.to_string()),
            arguments: tool.arguments,
        })
        .await?;
    tracing::info!("Tool result: {tool_result:#?}");

    let result = tool_result.content.unwrap()[0]
        .as_text()
        .unwrap()
        .text
        .clone();
    service.cancel().await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::{ParameterSetting, Tool};
    use serde_json::{Map, Value};
    use std::path::PathBuf;

    #[tokio::test]
    #[test_log::test]
    async fn test_start_io_client() {
        let mut tool = Tool::with_io_transport(
            "add".to_string(),
            "A simple calculator tool that provides 'add' functionality.".to_string(),
            vec![
                ParameterSetting::new("a", "number", "First operand", None),
                ParameterSetting::new("b", "number", "Second operand", None),
            ],
            std::fs::canonicalize(PathBuf::from(
                "../target/release/mcp-servers/sample-calculator-mcp-server",
            ))
            .unwrap(),
        );

        tool.arguments = Some(Map::from_iter(vec![
            ("a".to_string(), Value::Number(5.into())),
            ("b".to_string(), Value::Number(3.into())),
        ]));

        tracing::debug!("Tool: {:#?}", tool);

        let result = start_io_client(tool).await;

        tracing::debug!("Result: {:#?}", result);

        assert!(result.is_ok(), "Failed to execute IO client: {:?}", result);
        assert_eq!(result.unwrap(), "8");
    }
}
