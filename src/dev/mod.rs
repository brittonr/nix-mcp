//! Development tools module.
//!
//! This module provides MCP tools for development workflow automation,
//! focusing on code quality and pre-commit hook management.
//!
//! # Tools
//!
//! - [`PreCommitTools`] - Manage and run pre-commit hooks (formatting, linting, etc.)
//!
//! # Pre-commit Integration
//!
//! The pre-commit tools help enforce code quality before commits:
//! - Check if pre-commit hooks are installed
//! - Set up pre-commit hooks from configuration
//! - Run hooks manually or via git commit
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::dev::{PreCommitTools, PreCommitRunArgs};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let audit = Arc::new(/* audit logger */);
//! let tools = PreCommitTools::new(audit);
//!
//! // Run pre-commit hooks on all files
//! // let result = tools.pre_commit_run(Parameters(PreCommitRunArgs {
//! //     all_files: Some(true),
//! //     hook_ids: None,
//! // })).await?;
//! # Ok(())
//! # }
//! ```

pub mod precommit;
pub mod types;

pub use precommit::PreCommitTools;
pub use types::{CheckPreCommitStatusArgs, PreCommitRunArgs, SetupPreCommitArgs};
