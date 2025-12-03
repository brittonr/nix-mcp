/// Argument types for process management tools
use rmcp::schemars;

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
