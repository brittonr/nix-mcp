use anyhow::Result;
use common::nix_server::NixServer;
use rmcp::transport::stdio;
use rmcp::ServiceExt;
use tracing_subscriber::{self, EnvFilter};
mod common;
mod dev;
mod process;
mod prompts;

/// Nix MCP Server - provides tools for Nix package management and development
/// Run with: nix develop -c cargo run -p mcp-basic-server --features transport-io
/// Test with: npx @modelcontextprotocol/inspector nix develop -c cargo run -p mcp-basic-server --features transport-io
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with stderr logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Nix MCP Server");

    // Create an instance of our Nix server
    #[cfg(feature = "transport-io")]
    let service = NixServer::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    #[cfg(not(feature = "transport-io"))]
    compile_error!("`transport-io` feature is required for this server to run.");

    tracing::info!("Nix MCP Server is ready and waiting for connections");

    #[cfg(feature = "transport-io")]
    service.waiting().await?;
    Ok(())
}
