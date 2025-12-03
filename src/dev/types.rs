//! Parameter types for development tools.
//!
//! This module defines parameter types for pre-commit hook management.
//! Each type corresponds to a specific operation and includes field-level
//! documentation with examples.

use rmcp::schemars;

/// Parameters for running pre-commit hooks.
///
/// Used by [`PreCommitTools::pre_commit_run`](crate::dev::PreCommitTools::pre_commit_run).
///
/// # Examples
///
/// ```
/// use onix_mcp::dev::types::PreCommitRunArgs;
///
/// // Run all hooks on all files
/// let args = PreCommitRunArgs {
///     all_files: Some(true),
///     hook_ids: None,
/// };
///
/// // Run specific hooks on staged files
/// let args = PreCommitRunArgs {
///     all_files: Some(false),
///     hook_ids: Some("rustfmt,clippy".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PreCommitRunArgs {
    /// Run hooks on all files instead of just staged files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_files: Option<bool>,
    /// Specific hook IDs to run (comma-separated, e.g., "rustfmt,clippy")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_ids: Option<String>,
}

/// Parameters for checking pre-commit hook status.
///
/// Used by [`PreCommitTools::check_pre_commit_status`](crate::dev::PreCommitTools::check_pre_commit_status).
///
/// # Examples
///
/// ```
/// use onix_mcp::dev::types::CheckPreCommitStatusArgs;
///
/// let args = CheckPreCommitStatusArgs {};
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CheckPreCommitStatusArgs {
    // No parameters needed
}

/// Parameters for setting up pre-commit hooks.
///
/// Used by [`PreCommitTools::setup_pre_commit`](crate::dev::PreCommitTools::setup_pre_commit).
///
/// # Examples
///
/// ```
/// use onix_mcp::dev::types::SetupPreCommitArgs;
///
/// // Set up and install hooks immediately
/// let args = SetupPreCommitArgs {
///     install: Some(true),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetupPreCommitArgs {
    /// Install hooks immediately after setup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<bool>,
}
