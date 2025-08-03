pub mod servers;

use crate::servers::mcp_io::start_calculator_io;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    start_calculator_io().await?;
    Ok(())
}
