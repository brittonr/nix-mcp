//! Process management tools module.
//!
//! This module provides MCP tools for managing background processes and
//! interactive sessions using pueue (task queue) and pexpect (interactive automation).
//!
//! # Tools
//!
//! - [`PueueTools`] - Async task queue for long-running commands
//! - [`PexpectTools`] - Interactive session automation with expect-like functionality
//!
//! # Pueue Task Queue
//!
//! Pueue allows queueing and managing background tasks:
//! - Add commands to queue
//! - Monitor task status and logs
//! - Pause/resume/kill tasks
//! - Wait for task completion
//!
//! # Pexpect Interactive Sessions
//!
//! Pexpect enables automation of interactive programs:
//! - Start interactive sessions (ssh, python REPL, etc.)
//! - Send commands and code to running sessions
//! - Close sessions gracefully
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::process::{PueueTools, PueueAddArgs};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let audit = Arc::new(/* audit logger */);
//! let tools = PueueTools::new(audit);
//!
//! // Add a long-running task to the queue
//! // let result = tools.pueue_add(Parameters(PueueAddArgs {
//! //     command: "nix build .#mypackage".to_string(),
//! //     args: None,
//! //     label: Some("build-mypackage".to_string()),
//! //     working_directory: None,
//! // })).await?;
//! # Ok(())
//! # }
//! ```

pub mod pexpect;
pub mod pueue;
pub mod types;

pub use pexpect::PexpectTools;
pub use pueue::PueueTools;
pub use types::{
    PexpectCloseArgs, PexpectSendArgs, PexpectStartArgs, PueueAddArgs, PueueCleanArgs,
    PueueLogArgs, PueuePauseArgs, PueueRemoveArgs, PueueStartArgs, PueueStatusArgs, PueueWaitArgs,
};
