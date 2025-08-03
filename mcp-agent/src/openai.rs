use anyhow::Result;
use mcp_client::clients::{Tool, io_client::start_io_client};
use openai_tools::{
    chat::{request::ChatCompletion, response::Response},
    common::{message::Message, parameters::ParameterProperty, tool::Tool as OpenAITool},
};

pub async fn call_with_io_tools(
    chat_completion: &mut ChatCompletion,
    tools: Vec<Tool>,
) -> Result<Response> {
    // Construct OpenAITool instances from MCP tools
    let tools_for_first_call = tools
        .iter()
        .map(|tool| {
            assert!(tool.io_path.is_some(), "Tool must have an io_path set");
            let params = tool
                .parameters
                .iter()
                .map(|param| (param.name.clone(), ParameterProperty::from(param.clone())))
                .collect::<Vec<(String, ParameterProperty)>>();
            OpenAITool::function(tool.name.clone(), tool.description.clone(), params, false)
        })
        .collect::<Vec<OpenAITool>>();
    chat_completion.tools(tools_for_first_call);

    tracing::info!("OpenAI Settings: {:?}", chat_completion);

    // Call OpenAI chat completion
    let mut response = match chat_completion.chat().await {
        Ok(response) => {
            // Process the response
            tracing::debug!("Response: {:#?}", response);
            response
        }
        Err(error) => {
            // Handle error
            tracing::error!("Error: {:#?}", error);
            return Err(anyhow::anyhow!("Failed to complete request: {:#?}", error));
        }
    };

    // Loop while there are tool calls
    loop {
        if response.choices.is_empty() {
            tracing::warn!("No choices in response, exiting loop.");
            break;
        }
        if response.choices[0].message.tool_calls.is_none() {
            tracing::info!("No tool calls in response, exiting loop.");
            break;
        }

        let message = response.choices[0].message.clone();
        chat_completion.add_message(message.clone());

        // Tool calls found
        let tool_calls = response.choices[0].message.tool_calls.clone().unwrap();
        for tool_call in tool_calls.iter() {
            let tool_name = tool_call.function.name.clone();
            let mut found_tool = tools
                .iter()
                .find(|t| t.name == tool_name)
                .expect("Tool not found in provided tools")
                .to_owned();

            found_tool.arguments = Some(match tool_call.function.arguments_as_map().ok() {
                Some(args) => serde_json::Map::from_iter(args),
                None => Err(anyhow::anyhow!("Failed to parse function arguments as map"))?,
            });

            if cfg!(test) {
                tracing::info!("Tool call: {tool_call:#?}");
                tracing::info!("Found tool: {found_tool:#?}");
            }

            let result = start_io_client(found_tool).await?;

            chat_completion.add_message(Message::from_tool_call_response(
                result,
                tool_call.id.clone(),
            ));
        }

        // Call OpenAI chat completion again with updated messages
        response = chat_completion.chat().await?;
        if cfg!(test) {
            tracing::info!("Response: {:#?}", response);
        }
    }

    if cfg!(test) {
        tracing::info!("Final response: {:#?}", response);
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_client::clients::{ParameterSetting, Tool};
    use openai_tools::common::{message::Message, role::Role};
    use std::path::PathBuf;

    #[tokio::test]
    #[test_log::test]
    async fn test_call_with_io_tools_calculator() {
        let messages = vec![
            Message::from_string(
                Role::System,
                r#"You are a helpful calculator assistant. Available tools: [add]"#,
            ),
            Message::from_string(Role::User, "Calculate 25 + 17 using the add tool."),
        ];

        let mut chat_completion = ChatCompletion::new();
        chat_completion
            .model_id(std::env::var("OPENAI_MODEL_ID").unwrap_or_else(|_| "gpt-4.1-mini".into()))
            .temperature(1.0)
            .messages(messages);

        assert!(
            PathBuf::from("../target/release/mcp-servers/sample-calculator-mcp-server").exists(),
            "Calculator MCP server binary does not exist"
        );

        let tools = vec![Tool::with_io_transport(
            "add".into(),
            "A simple calculator tool".into(),
            vec![
                ParameterSetting::new("a", "number", "First operand", None),
                ParameterSetting::new("b", "number", "Second operand", None),
            ],
            PathBuf::from("../target/release/mcp-servers/sample-calculator-mcp-server"),
        )];

        let response = call_with_io_tools(&mut chat_completion, tools).await;
        assert!(
            response.is_ok(),
            "Expected successful response, got: {:?}",
            response
        );
        let response = response.unwrap();
        assert!(
            !response.choices.is_empty(),
            "Expected non-empty choices in response"
        );

        tracing::info!("Response: {:#?}", response);
    }
}
