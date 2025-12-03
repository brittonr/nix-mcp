use crate::common::security::audit::AuditLogger;
use crate::common::security::helpers::{audit_tool_execution, with_timeout};
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use std::process::Output;
use std::sync::Arc;

/// Result of executing a command
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl CommandResult {
    /// Create a CommandResult from tokio Command output
    pub fn from_output(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }

    /// Convert to MCP CallToolResult (success case)
    pub fn to_tool_result(self) -> Result<CallToolResult, McpError> {
        if !self.success {
            return Err(McpError::internal_error(
                format!("Command failed:\n{}", self.stderr),
                None,
            ));
        }
        Ok(CallToolResult::success(vec![Content::text(self.stdout)]))
    }

    /// Convert to MCP CallToolResult with custom error message
    pub fn to_tool_result_with_error(self, error_prefix: &str) -> Result<CallToolResult, McpError> {
        if !self.success {
            return Err(McpError::internal_error(
                format!("{}:\n{}", error_prefix, self.stderr),
                None,
            ));
        }
        Ok(CallToolResult::success(vec![Content::text(self.stdout)]))
    }

    /// Get combined output (stderr + stdout)
    pub fn combined_output(&self) -> String {
        let mut result = String::new();
        if !self.stderr.is_empty() {
            result.push_str(&self.stderr);
        }
        if !self.stdout.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&self.stdout);
        }
        result
    }
}

/// Builder for executing commands with common patterns
pub struct CommandExecutor {
    audit: Arc<AuditLogger>,
}

impl CommandExecutor {
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }

    /// Execute a nix command with args and return processed output
    pub async fn execute_nix(
        &self,
        args: &[&str],
        context: &str,
    ) -> Result<CommandResult, McpError> {
        let output = tokio::process::Command::new("nix")
            .args(args)
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("{}: {}", context, e), None))?;

        Ok(CommandResult::from_output(output))
    }

    /// Execute a nix command with timeout, audit logging, and error handling
    pub async fn execute_nix_with_security(
        &self,
        tool_name: &str,
        args: Vec<String>,
        timeout_secs: u64,
        params: Option<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        let audit = self.audit.clone();
        let audit_inner = self.audit.clone();

        audit_tool_execution(&audit, tool_name, params, || async move {
            with_timeout(&audit_inner, tool_name, timeout_secs, || async {
                let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = tokio::process::Command::new("nix")
                    .args(&args_refs)
                    .output()
                    .await
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute nix {}: {}", tool_name, e),
                            None,
                        )
                    })?;

                let result = CommandResult::from_output(output);
                result.to_tool_result()
            })
            .await
        })
        .await
    }

    /// Execute a generic command (not nix) with timeout and audit
    pub async fn execute_command_with_security(
        &self,
        tool_name: &str,
        program: &str,
        args: Vec<String>,
        timeout_secs: u64,
        params: Option<serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        let audit = self.audit.clone();
        let audit_inner = self.audit.clone();
        let program = program.to_string();

        audit_tool_execution(&audit, tool_name, params, || async move {
            with_timeout(&audit_inner, tool_name, timeout_secs, || async {
                let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = tokio::process::Command::new(&program)
                    .args(&args_refs)
                    .output()
                    .await
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute {} {}: {}", program, tool_name, e),
                            None,
                        )
                    })?;

                let result = CommandResult::from_output(output);
                result.to_tool_result()
            })
            .await
        })
        .await
    }

    /// Execute with custom result processing
    pub async fn execute_nix_with_processor<F, Fut>(
        &self,
        tool_name: &str,
        args: Vec<String>,
        timeout_secs: u64,
        params: Option<serde_json::Value>,
        processor: F,
    ) -> Result<CallToolResult, McpError>
    where
        F: FnOnce(CommandResult) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<CallToolResult, McpError>> + Send,
    {
        let audit = self.audit.clone();
        let audit_inner = self.audit.clone();

        audit_tool_execution(&audit, tool_name, params, || async move {
            with_timeout(&audit_inner, tool_name, timeout_secs, || async {
                let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = tokio::process::Command::new("nix")
                    .args(&args_refs)
                    .output()
                    .await
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute nix {}: {}", tool_name, e),
                            None,
                        )
                    })?;

                let result = CommandResult::from_output(output);
                processor(result).await
            })
            .await
        })
        .await
    }
}
