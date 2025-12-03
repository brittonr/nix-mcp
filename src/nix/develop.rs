use crate::common::cache_registry::CacheRegistry;
use crate::common::caching::CachedExecutor;
use crate::common::security::audit::AuditLogger;
use crate::common::security::helpers::{
    audit_tool_execution, validation_error_to_mcp, with_timeout,
};
use crate::common::security::{
    validate_command, validate_flake_ref, validate_nix_expression, validate_package_name,
    validate_path,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

use super::types::{
    NixDevelopArgs, NixEvalArgs, NixLogArgs, NixRunArgs, RunInShellArgs, SearchOptionsArgs,
};

/// Tools for Nix development environments and expression evaluation.
///
/// This struct provides operations for working with Nix development shells,
/// evaluating expressions, running packages, and debugging builds. These tools
/// support rapid development workflows and exploratory Nix operations.
///
/// # Available Operations
///
/// - **Shell Environments**: [`run_in_shell`](Self::run_in_shell), [`nix_develop`](Self::nix_develop)
/// - **Package Execution**: [`nix_run`](Self::nix_run)
/// - **Expression Evaluation**: [`nix_eval`](Self::nix_eval)
/// - **Debugging**: [`nix_log`](Self::nix_log)
/// - **Configuration**: [`search_options`](Self::search_options)
///
/// # Caching Strategy
///
/// - Nix evaluations: 5-minute TTL (expressions may change frequently)
/// - No caching for shell/run operations (execution must be fresh)
/// - Option searches: 10-minute TTL (options are relatively stable)
///
/// # Timeouts
///
/// - `nix_eval`: 30 seconds (expression evaluation should be quick)
/// - `run_in_shell`: 120 seconds (2 minutes for shell commands)
/// - `nix_run`: 300 seconds (5 minutes for package execution)
/// - `nix_develop`: 300 seconds (5 minutes for dev shell commands)
/// - `nix_log`: 30 seconds (log retrieval is I/O bound)
///
/// # Security
///
/// All commands are validated before execution:
/// - Nix expressions scanned for dangerous patterns (shell substitution, etc.)
/// - Commands checked for null bytes and length limits
/// - Package names validated for injection attacks
/// - Shell operations marked as dangerous and logged
/// - All operations include audit logging with parameters
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::nix::DevelopTools;
/// use onix_mcp::nix::types::RunInShellArgs;
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # async fn example(tools: DevelopTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Run a command with temporary packages
/// let result = tools.run_in_shell(Parameters(RunInShellArgs {
///     packages: vec!["python3".to_string(), "numpy".to_string()],
///     command: "python -c 'import numpy; print(numpy.__version__)'".to_string(),
///     use_flake: Some(false),
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct DevelopTools {
    audit: Arc<AuditLogger>,
    caches: Arc<CacheRegistry>,
}

impl DevelopTools {
    /// Creates a new `DevelopTools` instance with audit logging and caching.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    /// * `caches` - Shared cache registry containing eval cache
    pub fn new(audit: Arc<AuditLogger>, caches: Arc<CacheRegistry>) -> Self {
        Self { audit, caches }
    }
}

#[tool_router]
impl DevelopTools {
    #[tool(
        description = "Search NixOS configuration options",
        annotations(read_only_hint = true)
    )]
    pub async fn search_options(
        &self,
        Parameters(SearchOptionsArgs { query }): Parameters<SearchOptionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate query input
        validate_package_name(&query).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + timeout)
        audit_tool_execution(
            &self.audit,
            "search_options",
            Some(serde_json::json!({"query": &query})),
            || async {
                with_timeout(&self.audit, "search_options", 30, || async {
                    // Check if we're on NixOS and can query options directly
                    let nixos_check = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg("test -f /etc/NIXOS")
                        .output()
                        .await;

                    let on_nixos = nixos_check.map(|o| o.status.success()).unwrap_or(false);

                    if on_nixos {
                        // Try to search using nixos-option if available
                        let output = tokio::process::Command::new("nixos-option")
                            .arg(&query)
                            .output()
                            .await;

                        if let Ok(output) = output {
                            if output.status.success() {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                return Ok(CallToolResult::success(vec![Content::text(
                                    stdout.to_string(),
                                )]));
                            }
                        }
                    }

                    // Provide helpful information with web search links
                    use crate::common::nix_tools_helpers::format_option_search_response;
                    Ok(CallToolResult::success(vec![Content::text(
                        format_option_search_response(&query),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Evaluate a Nix expression")]
    pub async fn nix_eval(
        &self,
        Parameters(NixEvalArgs { expression }): Parameters<NixEvalArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate Nix expression for dangerous patterns
        validate_nix_expression(&expression).map_err(validation_error_to_mcp)?;

        // Use cached executor for cache-check-execute-cache pattern
        let cached_executor = CachedExecutor::new(self.caches.eval.clone());
        let audit = self.audit.clone();
        let expression_clone = expression.clone();

        cached_executor
            .execute_with_string_cache(expression.clone(), || async move {
                let audit_inner = audit.clone();
                // Execute with security features (audit logging + 30s timeout for eval)
                audit_tool_execution(
                    &audit,
                    "nix_eval",
                    Some(serde_json::json!({"expression_length": expression_clone.len()})),
                    || async move {
                        with_timeout(&audit_inner, "nix_eval", 30, || async {
                            let output = tokio::process::Command::new("nix")
                                .args(["eval", "--expr", &expression_clone])
                                .output()
                                .await
                                .map_err(|e| {
                                    McpError::internal_error(
                                        format!("Failed to execute nix eval: {}", e),
                                        None,
                                    )
                                })?;

                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                return Err(McpError::internal_error(
                                    format!("Evaluation failed: {}", stderr),
                                    None,
                                ));
                            }

                            Ok(String::from_utf8_lossy(&output.stdout).to_string())
                        })
                        .await
                    },
                )
                .await
            })
            .await
    }

    #[tool(description = "Run a command in a Nix shell with specified packages available")]
    pub async fn run_in_shell(
        &self,
        Parameters(RunInShellArgs {
            packages,
            command,
            use_flake,
        }): Parameters<RunInShellArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate command for dangerous patterns
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Validate package names if provided
        for package in &packages {
            validate_package_name(package).map_err(validation_error_to_mcp)?;
        }

        // Log potentially dangerous operation
        self.audit.log_dangerous_operation(
            "run_in_shell",
            true,
            &format!("Running command: {}", command),
        );

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(
            &self.audit,
            "run_in_shell",
            Some(serde_json::json!({"command": &command, "packages": &packages})),
            || async {
                with_timeout(&self.audit, "run_in_shell", 120, || async {
                    let use_flake = use_flake.unwrap_or(false);

                    let output = if use_flake {
                        // Use nix develop -c
                        tokio::process::Command::new("nix")
                            .args(["develop", "-c", "sh", "-c", &command])
                            .output()
                            .await
                            .map_err(|e| {
                                McpError::internal_error(
                                    format!("Failed to run in dev shell: {}", e),
                                    None,
                                )
                            })?
                    } else {
                        // Use nix-shell -p
                        let package_args: Vec<String> = packages
                            .iter()
                            .flat_map(|pkg| vec!["-p".to_string(), pkg.clone()])
                            .collect();

                        let mut args = package_args;
                        args.push("--run".to_string());
                        args.push(command.clone());

                        tokio::process::Command::new("nix-shell")
                            .args(&args)
                            .output()
                            .await
                            .map_err(|e| {
                                McpError::internal_error(
                                    format!("Failed to run in shell: {}", e),
                                    None,
                                )
                            })?
                    };

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let result_text = if output.status.success() {
                        format!(
                            "Command executed successfully!\n\nOutput:\n{}{}",
                            stdout, stderr
                        )
                    } else {
                        format!(
                            "Command failed with exit code: {:?}\n\nOutput:\n{}\n\nError:\n{}",
                            output.status.code(),
                            stdout,
                            stderr
                        )
                    };

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get Nix build logs directly from store path, optionally filtered with grep pattern",
        annotations(read_only_hint = true)
    )]
    pub async fn nix_log(
        &self,
        Parameters(NixLogArgs {
            store_path,
            grep_pattern,
        }): Parameters<NixLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate store path
        validate_path(&store_path).map_err(validation_error_to_mcp)?;

        // Validate grep pattern if provided
        if let Some(ref pattern) = grep_pattern {
            if pattern.contains('\0') || pattern.is_empty() {
                return Err(McpError::invalid_params(
                    "Invalid grep pattern".to_string(),
                    Some(serde_json::json!({"pattern": pattern})),
                ));
            }
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_log",
            Some(serde_json::json!({"store_path": &store_path, "grep_pattern": &grep_pattern})),
            || async {
                with_timeout(&self.audit, "nix_log", 30, || async {
                    // Use nix log with store path
                    let output = tokio::process::Command::new("nix")
                        .args(["log", &store_path])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix log: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to get log: {}", stderr),
                            None,
                        ));
                    }

                    let log = String::from_utf8_lossy(&output.stdout);

                    // Apply grep filter if provided
                    let result = if let Some(ref pattern) = grep_pattern {
                        let filtered_lines: Vec<&str> = log
                            .lines()
                            .filter(|line| line.contains(pattern.as_str()))
                            .collect();

                        if filtered_lines.is_empty() {
                            format!(
                                "No lines matching '{}' found in log for {}",
                                pattern, store_path
                            )
                        } else {
                            format!(
                                "Lines matching '{}' in {}:\n\n{}",
                                pattern,
                                store_path,
                                filtered_lines.join("\n")
                            )
                        }
                    } else {
                        // Return full log, truncate if too long
                        if log.len() > 50000 {
                            let truncated = &log[..50000];
                            format!(
                                "{}\n\n... [Log truncated - showing first 50KB of {} KB total]",
                                truncated,
                                log.len() / 1024
                            )
                        } else {
                            log.to_string()
                        }
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Run an application from nixpkgs without installing it",
        annotations(read_only_hint = false)
    )]
    pub async fn nix_run(
        &self,
        Parameters(NixRunArgs { package, args }): Parameters<NixRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate package/flake reference (accepts nixpkgs#hello format)
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_run",
            Some(serde_json::json!({"package": &package, "args": &args})),
            || async {
                with_timeout(&self.audit, "nix_run", 300, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg(&package);

                    if let Some(program_args) = args {
                        cmd.arg("--");
                        for arg in program_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute nix run: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str("STDOUT:\n");
                        result.push_str(&stdout);
                        result.push('\n');
                    }
                    if !stderr.is_empty() {
                        result.push_str("STDERR:\n");
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = format!(
                            "Command completed successfully (exit code: {})",
                            output.status.code().unwrap_or(0)
                        );
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(
                            format!("nix run failed: {}", result),
                            None,
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
        description = "Run a command in a Nix development environment (from flake.nix devShell)",
        annotations(read_only_hint = false)
    )]
    pub async fn nix_develop(
        &self,
        Parameters(NixDevelopArgs {
            flake_ref,
            command,
            args,
        }): Parameters<NixDevelopArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate flake reference if provided
        if let Some(ref fref) = flake_ref {
            validate_flake_ref(fref).map_err(validation_error_to_mcp)?;
        }

        // Validate command
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_develop",
            Some(serde_json::json!({"flake_ref": &flake_ref, "command": &command, "args": &args})),
            || async {
                with_timeout(&self.audit, "nix_develop", 300, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("develop");

                    if let Some(ref fref) = flake_ref {
                        cmd.arg(fref);
                    }

                    cmd.arg("-c").arg(&command);

                    if let Some(command_args) = args {
                        for arg in command_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute nix develop: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str("STDOUT:\n");
                        result.push_str(&stdout);
                        result.push('\n');
                    }
                    if !stderr.is_empty() {
                        result.push_str("STDERR:\n");
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = format!(
                            "Command '{}' completed successfully in development environment",
                            command
                        );
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(
                            format!("nix develop failed: {}", result),
                            None,
                        ));
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }
}
