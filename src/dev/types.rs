/// Argument types for pre-commit tools
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PreCommitRunArgs {
    /// Run hooks on all files instead of just staged files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_files: Option<bool>,
    /// Specific hook IDs to run (comma-separated, e.g., "rustfmt,clippy")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_ids: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckPreCommitStatusArgs {
    // No parameters needed
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetupPreCommitArgs {
    /// Install hooks immediately after setup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<bool>,
}
