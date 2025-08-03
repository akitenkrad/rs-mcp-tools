use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters, wrapper::Json},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::{SseServer, sse_server::SseServerConfig},
};

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    pub b: i32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    #[schemars(description = "the right hand side number")]
    pub b: i32,
}

#[derive(Debug, Clone)]
pub struct Calculator {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Calculator {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Calculate the addition of two numbers")]
    fn add(&self, Parameters(AddRequest { a, b }): Parameters<AddRequest>) -> String {
        (a + b).to_string()
    }

    #[tool(description = "Calculate the subtraction of two numbers")]
    fn sub(&self, Parameters(SubRequest { a, b }): Parameters<SubRequest>) -> Json<i32> {
        Json(a - b)
    }
}

#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

pub async fn start_calculator_sse() -> anyhow::Result<()> {
    tracing::info!("Starting calculator server...");

    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse().expect("Invalid bind address"),
        sse_path: "/sse".into(),
        post_path: "/message".into(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };
    let (sse_server, router) = SseServer::new(config);
    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;
    let ct = sse_server.config.ct.child_token();
    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        ct.cancelled().await;
        tracing::info!("sse server cancelled");
    });

    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!(error = %e, "Failed to start SSE server");
        }
    });

    let ct = sse_server.with_service(Calculator::new);

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    ct.cancel();
    Ok(())
}
