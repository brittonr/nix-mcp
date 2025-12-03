//! Parameter types for process management MCP tools.
//!
//! This module defines parameter types for managing background tasks (pueue)
//! and interactive sessions (pexpect). Each type corresponds to a specific
//! operation and includes field-level documentation with examples.

use rmcp::schemars;

// ===== Pexpect Types =====

/// Parameters for starting a new pexpect interactive session.
///
/// Used by [`PexpectTools::pexpect_start`](crate::process::PexpectTools::pexpect_start).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PexpectStartArgs;
///
/// // Start an SSH session
/// let args = PexpectStartArgs {
///     command: "ssh".to_string(),
///     args: Some(vec!["user@host".to_string()]),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectStartArgs {
    /// Command to run interactively (e.g., "bash", "python", "ssh user@host")
    pub command: String,
    /// Arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Parameters for sending code to an active pexpect session.
///
/// Used by [`PexpectTools::pexpect_send`](crate::process::PexpectTools::pexpect_send).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PexpectSendArgs;
///
/// let args = PexpectSendArgs {
///     session_id: "abc123".to_string(),
///     code: "child.sendline('ls'); child.expect('$')".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectSendArgs {
    /// Session ID from pexpect_start
    pub session_id: String,
    /// Python pexpect code to execute (e.g., "child.sendline('ls'); child.expect('$'); print(child.before.decode())")
    pub code: String,
}

/// Parameters for closing a pexpect interactive session.
///
/// Used by [`PexpectTools::pexpect_close`](crate::process::PexpectTools::pexpect_close).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PexpectCloseArgs;
///
/// let args = PexpectCloseArgs {
///     session_id: "abc123".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectCloseArgs {
    /// Session ID to close
    pub session_id: String,
}

// ===== Pueue Types =====

/// Parameters for adding a command to the pueue task queue.
///
/// Used by [`PueueTools::pueue_add`](crate::process::PueueTools::pueue_add).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueAddArgs;
///
/// let args = PueueAddArgs {
///     command: "nix build .#mypackage".to_string(),
///     args: None,
///     working_directory: Some("/home/user/project".to_string()),
///     label: Some("build-mypackage".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueAddArgs {
    /// Command to run
    pub command: String,
    /// Arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Working directory for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Label for the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Parameters for getting pueue task status.
///
/// Used by [`PueueTools::pueue_status`](crate::process::PueueTools::pueue_status).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueStatusArgs;
///
/// // Get all task status
/// let args = PueueStatusArgs {
///     task_ids: None,
/// };
///
/// // Get specific task status
/// let args = PueueStatusArgs {
///     task_ids: Some("1,2,3".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueStatusArgs {
    /// Show only specific task IDs (comma-separated, e.g., "1,2,3")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

/// Parameters for retrieving pueue task logs.
///
/// Used by [`PueueTools::pueue_log`](crate::process::PueueTools::pueue_log).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueLogArgs;
///
/// // Get last 50 lines of task 1
/// let args = PueueLogArgs {
///     task_id: 1,
///     lines: Some(50),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueLogArgs {
    /// Task ID to get logs for
    pub task_id: u32,
    /// Number of lines to show from the end (like tail -n)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<usize>,
}

/// Parameters for waiting for pueue tasks to complete.
///
/// Used by [`PueueTools::pueue_wait`](crate::process::PueueTools::pueue_wait).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueWaitArgs;
///
/// let args = PueueWaitArgs {
///     task_ids: "1,2,3".to_string(),
///     timeout: Some(600),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueWaitArgs {
    /// Task IDs to wait for (comma-separated, e.g., "1,2,3")
    pub task_ids: String,
    /// Timeout in seconds (default: 300)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// Parameters for removing pueue tasks.
///
/// Used by [`PueueTools::pueue_remove`](crate::process::PueueTools::pueue_remove).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueRemoveArgs;
///
/// let args = PueueRemoveArgs {
///     task_ids: "1,2,3".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueRemoveArgs {
    /// Task IDs to remove (comma-separated, e.g., "1,2,3")
    pub task_ids: String,
}

/// Parameters for cleaning finished pueue tasks.
///
/// Used by [`PueueTools::pueue_clean`](crate::process::PueueTools::pueue_clean).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueCleanArgs;
///
/// let args = PueueCleanArgs {};
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueCleanArgs {
    // No parameters needed
}

/// Parameters for pausing pueue tasks.
///
/// Used by [`PueueTools::pueue_pause`](crate::process::PueueTools::pueue_pause).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueuePauseArgs;
///
/// // Pause all tasks
/// let args = PueuePauseArgs {
///     task_ids: None,
/// };
///
/// // Pause specific tasks
/// let args = PueuePauseArgs {
///     task_ids: Some("1,2".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueuePauseArgs {
    /// Task IDs to pause (comma-separated, e.g., "1,2,3"). Leave empty to pause all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

/// Parameters for starting/resuming pueue tasks.
///
/// Used by [`PueueTools::pueue_start`](crate::process::PueueTools::pueue_start).
///
/// # Examples
///
/// ```
/// use onix_mcp::process::types::PueueStartArgs;
///
/// // Start all tasks
/// let args = PueueStartArgs {
///     task_ids: None,
/// };
///
/// // Start specific tasks
/// let args = PueueStartArgs {
///     task_ids: Some("1,2".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueStartArgs {
    /// Task IDs to start/resume (comma-separated, e.g., "1,2,3"). Leave empty to start all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}
