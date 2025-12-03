/// Argument types for process management tools
use rmcp::schemars;

// Pexpect argument types
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectStartArgs {
    /// Command to run interactively (e.g., "bash", "python", "ssh user@host")
    pub command: String,
    /// Arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectSendArgs {
    /// Session ID from pexpect_start
    pub session_id: String,
    /// Python pexpect code to execute (e.g., "child.sendline('ls'); child.expect('$'); print(child.before.decode())")
    pub code: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectCloseArgs {
    /// Session ID to close
    pub session_id: String,
}

// Pueue task queue argument types
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueStatusArgs {
    /// Show only specific task IDs (comma-separated, e.g., "1,2,3")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueLogArgs {
    /// Task ID to get logs for
    pub task_id: u32,
    /// Number of lines to show from the end (like tail -n)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueWaitArgs {
    /// Task IDs to wait for (comma-separated, e.g., "1,2,3")
    pub task_ids: String,
    /// Timeout in seconds (default: 300)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueRemoveArgs {
    /// Task IDs to remove (comma-separated, e.g., "1,2,3")
    pub task_ids: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueCleanArgs {
    // No parameters needed
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueuePauseArgs {
    /// Task IDs to pause (comma-separated, e.g., "1,2,3"). Leave empty to pause all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueStartArgs {
    /// Task IDs to start/resume (comma-separated, e.g., "1,2,3"). Leave empty to start all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}
