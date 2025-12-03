use crate::common::security::audit::AuditLogger;
use crate::dev::types::{CheckPreCommitStatusArgs, PreCommitRunArgs, SetupPreCommitArgs};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

/// Tools for managing pre-commit hooks in git repositories.
///
/// This struct provides operations for checking, setting up, and running pre-commit
/// hooks that enforce code quality standards before commits. Pre-commit hooks can
/// run formatters, linters, tests, and other checks automatically.
///
/// # Available Operations
///
/// - **Hook Management**: [`setup_pre_commit`](Self::setup_pre_commit)
/// - **Status Check**: [`check_pre_commit_status`](Self::check_pre_commit_status)
/// - **Execution**: [`pre_commit_run`](Self::pre_commit_run)
///
/// # Caching Strategy
///
/// No caching for pre-commit operations (hook status and results change frequently).
///
/// # Timeouts
///
/// - `pre_commit_run`: 300 seconds (5 minutes - hooks may be slow)
/// - `check_pre_commit_status`: No timeout (quick filesystem checks)
/// - `setup_pre_commit`: No timeout (quick configuration)
///
/// # Security
///
/// All operations include audit logging:
/// - Hook IDs are not validated (trusted configuration)
/// - All operations logged with parameters
///
/// # Pre-commit Integration
///
/// These tools work with the standard pre-commit framework:
/// - Expects pre-commit to be available in the environment
/// - Reads from `.pre-commit-config.yaml`
/// - Installs hooks to `.git/hooks/pre-commit`
///
/// # Common Hooks
///
/// Typical pre-commit hooks include:
/// - **Formatters**: rustfmt, nixpkgs-fmt, black, prettier
/// - **Linters**: clippy, shellcheck, eslint
/// - **Security**: cargo-audit, bandit
/// - **General**: trailing whitespace, merge conflicts, large files
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::dev::PreCommitTools;
/// use onix_mcp::dev::types::PreCommitRunArgs;
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # async fn example(tools: PreCommitTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Run all pre-commit hooks on all files
/// let result = tools.pre_commit_run(Parameters(PreCommitRunArgs {
///     all_files: Some(true),
///     hook_ids: None,
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct PreCommitTools {
    pub audit: Arc<AuditLogger>,
}

impl PreCommitTools {
    /// Creates a new `PreCommitTools` instance with audit logging.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    ///
    /// # Note
    ///
    /// PreCommitTools does not use caching as hook status and execution
    /// results change frequently during development.
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl PreCommitTools {
    #[tool(
        description = "Run pre-commit hooks to check code quality (formatting, linting, etc.)",
        annotations(read_only_hint = false)
    )]
    pub async fn pre_commit_run(
        &self,
        Parameters(PreCommitRunArgs {
            all_files,
            hook_ids,
        }): Parameters<PreCommitRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pre_commit_run",
            Some(serde_json::json!({"all_files": &all_files, "hook_ids": &hook_ids})),
            || async {
                with_timeout(&self.audit, "pre_commit_run", 300, || async {
                    let mut cmd = tokio::process::Command::new("pre-commit");
                    cmd.arg("run");

                    if all_files.unwrap_or(false) {
                        cmd.arg("--all-files");
                    }

                    if let Some(hooks) = hook_ids {
                        for hook_id in hooks.split(',') {
                            cmd.arg("--hook-stage").arg("manual");
                            cmd.arg(hook_id.trim());
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pre-commit: {}. Make sure you're in a git repository with pre-commit hooks installed (run 'nix develop' first).", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str("STDERR:\n");
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = "All pre-commit hooks passed successfully!".to_string();
                    }

                    // Include exit status information
                    if !output.status.success() {
                        result.push_str(&format!(
                            "\n\nExit code: {}\nSome hooks failed. Fix the issues above and try again.",
                            output.status.code().unwrap_or(-1)
                        ));
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Check if pre-commit hooks are installed and configured in the current repository",
        annotations(read_only_hint = true)
    )]
    pub async fn check_pre_commit_status(
        &self,
        Parameters(_args): Parameters<CheckPreCommitStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        audit_tool_execution(
            &self.audit,
            "check_pre_commit_status",
            None,
            || async {
                let mut result = String::new();
                let mut warnings = Vec::new();

                // Check if .git directory exists
                let git_exists = tokio::fs::metadata(".git").await.is_ok();
                if !git_exists {
                    result.push_str("❌ Not a git repository (no .git directory found)\n");
                    return Ok(CallToolResult::success(vec![Content::text(result)]));
                }

                // Check if pre-commit is installed (in PATH or via nix develop)
                let pre_commit_check = tokio::process::Command::new("pre-commit")
                    .arg("--version")
                    .output()
                    .await;

                let pre_commit_available = match pre_commit_check {
                    Ok(output) if output.status.success() => {
                        let version = String::from_utf8_lossy(&output.stdout);
                        result.push_str(&format!("✅ pre-commit is available: {}", version.trim()));
                        true
                    }
                    _ => {
                        result.push_str("⚠️  pre-commit command not found in PATH\n");
                        result.push_str("   Run 'nix develop' to enter development shell with pre-commit\n");
                        warnings.push("pre-commit not in PATH");
                        false
                    }
                };

                // Check if .pre-commit-config.yaml exists
                let config_exists = tokio::fs::metadata(".pre-commit-config.yaml").await.is_ok();
                if config_exists {
                    result.push_str("\n✅ .pre-commit-config.yaml found\n");
                } else {
                    result.push_str("\n❌ .pre-commit-config.yaml not found\n");
                    warnings.push("config missing");
                }

                // Check if hooks are installed in .git/hooks/pre-commit
                let hook_exists = tokio::fs::metadata(".git/hooks/pre-commit").await.is_ok();
                if hook_exists {
                    result.push_str("✅ Git pre-commit hook is installed\n");
                } else {
                    result.push_str("❌ Git pre-commit hook not installed\n");
                    if config_exists && pre_commit_available {
                        result.push_str("   Run 'pre-commit install' to install hooks\n");
                        warnings.push("hooks not installed");
                    }
                }

                // Summary and recommendations
                result.push_str("\n--- SUMMARY ---\n");
                if warnings.is_empty() {
                    result.push_str("✅ Pre-commit hooks are fully configured and ready to use!\n");
                } else {
                    result.push_str("⚠️  Pre-commit hooks are not fully set up.\n\n");
                    result.push_str("RECOMMENDED ACTIONS:\n");

                    if !pre_commit_available {
                        result.push_str("1. Enter the Nix development shell: nix develop\n");
                    }

                    if !config_exists {
                        result.push_str("2. Pre-commit hooks are typically configured in flake.nix for Nix projects\n");
                        result.push_str("   Check if your flake.nix has pre-commit-hooks.nix configuration\n");
                        result.push_str("   Consider using the setup_pre_commit tool to set this up automatically\n");
                    } else if !hook_exists {
                        result.push_str("2. Install the hooks: pre-commit install\n");
                        result.push_str("   Or use: nix develop -c pre-commit install\n");
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(result)]))
            },
        )
        .await
    }

    #[tool(
        description = "Set up pre-commit hooks for a project (creates config and installs hooks)",
        annotations(read_only_hint = false)
    )]
    pub async fn setup_pre_commit(
        &self,
        Parameters(SetupPreCommitArgs { install }): Parameters<SetupPreCommitArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        audit_tool_execution(
            &self.audit,
            "setup_pre_commit",
            Some(serde_json::json!({"install": &install})),
            || async {
                let mut result = String::new();

                // Check if .git directory exists
                let git_exists = tokio::fs::metadata(".git").await.is_ok();
                if !git_exists {
                    return Err(McpError::internal_error(
                        "Not a git repository. Initialize git first with 'git init'".to_string(),
                        None,
                    ));
                }

                // Check if flake.nix exists
                let flake_exists = tokio::fs::metadata("flake.nix").await.is_ok();

                if flake_exists {
                    result.push_str("✅ flake.nix found\n\n");
                    result.push_str("For Nix projects, pre-commit hooks should be configured in flake.nix using pre-commit-hooks.nix.\n\n");
                    result.push_str("RECOMMENDED SETUP:\n");
                    result.push_str("1. Add pre-commit-hooks.nix to flake inputs\n");
                    result.push_str("2. Configure hooks in the flake\n");
                    result.push_str("3. Integrate with devShell\n");
                    result.push_str("4. Enter dev shell: nix develop\n\n");
                    result.push_str("The hooks will then auto-install when entering the dev shell.\n\n");
                    result.push_str("See https://github.com/cachix/pre-commit-hooks.nix for examples.\n");
                } else {
                    result.push_str("⚠️  No flake.nix found. Setting up basic pre-commit configuration.\n\n");
                    result.push_str("For better integration with Nix projects, consider using flake.nix with pre-commit-hooks.nix.\n\n");
                }

                // If install flag is set, run pre-commit install
                if install.unwrap_or(false) {
                    result.push_str("Installing pre-commit hooks...\n");
                    let install_output = tokio::process::Command::new("pre-commit")
                        .arg("install")
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to run pre-commit install: {}. Make sure pre-commit is available (run 'nix develop' first).", e),
                                None,
                            )
                        })?;

                    if install_output.status.success() {
                        result.push_str("✅ Pre-commit hooks installed successfully!\n");
                        let stdout = String::from_utf8_lossy(&install_output.stdout);
                        if !stdout.is_empty() {
                            result.push_str(&format!("\n{}", stdout));
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&install_output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to install pre-commit hooks: {}", stderr),
                            None,
                        ));
                    }
                }

                Ok(CallToolResult::success(vec![Content::text(result)]))
            },
        )
        .await
    }
}
