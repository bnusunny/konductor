mod config;
mod hook;
mod mcp;
mod state;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "konductor", about = "Konductor CLI — MCP server and hook processor", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the MCP server (stdio transport)
    Mcp,
    /// Process hook events from stdin
    Hook,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Mcp => mcp::run().await,
        Commands::Hook => hook::run(),
    }
}
