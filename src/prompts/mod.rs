//! MCP prompts module for interactive guidance.
//!
//! This module provides MCP prompts that guide users through common Nix workflows
//! with context-aware, step-by-step instructions.
//!
//! # Prompts
//!
//! - [`NixPrompts`] - Interactive prompts for Nix operations
//!
//! # Available Prompts
//!
//! - **generate_flake** - Generate a nix flake template for a project
//! - **setup_dev_environment** - Set up development environment for specific project types
//! - **troubleshoot_build** - Help diagnose and fix Nix build failures
//! - **migrate_to_flakes** - Guide migration from legacy Nix to flakes
//! - **optimize_closure** - Help reduce package closure size
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::prompts::{NixPrompts, SetupDevEnvironmentArgs};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let prompts = NixPrompts::new();
//!
//! // Get setup instructions for a Rust project
//! // let result = prompts.setup_dev_environment(
//! //     Parameters(SetupDevEnvironmentArgs {
//! //         project_type: "rust".to_string(),
//! //         additional_requirements: None,
//! //     }),
//! //     ctx,
//! // ).await?;
//! # Ok(())
//! # }
//! ```

pub mod nix_prompts;
pub mod types;

pub use nix_prompts::NixPrompts;
pub use types::{
    MigrateToFlakesArgs, OptimizeClosureArgs, SetupDevEnvironmentArgs, TroubleshootBuildArgs,
};
