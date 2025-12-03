use crate::common::security::audit::AuditLogger;
use crate::common::security::{validate_command, validation_error_to_mcp};
use crate::process::types::{PexpectCloseArgs, PexpectSendArgs, PexpectStartArgs};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

/// Pexpect interactive session management tools
pub struct PexpectTools {
    pub audit: Arc<AuditLogger>,
}

impl PexpectTools {
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl PexpectTools {
    #[tool(
        description = "Start a new pexpect-cli interactive session. Returns session ID.",
        annotations(read_only_hint = false)
    )]
    pub async fn pexpect_start(
        &self,
        Parameters(PexpectStartArgs { command, args }): Parameters<PexpectStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate command
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pexpect_start",
            Some(serde_json::json!({"command": &command, "args": &args})),
            || async {
                with_timeout(&self.audit, "pexpect_start", 30, || async {
                    // Use nix run to ensure pexpect-cli is available
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#python3Packages.pexpect-cli")
                        .arg("--")
                        .arg("--start")
                        .arg(&command);

                    if let Some(command_args) = args {
                        for arg in command_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pexpect-cli via nix run: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pexpect-cli failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Session started successfully. Session ID: {}",
                        stdout.trim()
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Send Python pexpect code to an active session",
        annotations(read_only_hint = false)
    )]
    pub async fn pexpect_send(
        &self,
        Parameters(PexpectSendArgs { session_id, code }): Parameters<PexpectSendArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate session ID format (should be alphanumeric)
        if session_id.is_empty()
            || session_id.contains('\0')
            || !session_id.chars().all(|c| c.is_alphanumeric())
        {
            return Err(McpError::invalid_params(
                "Invalid session_id".to_string(),
                Some(serde_json::json!({"session_id": session_id})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pexpect_send",
            Some(serde_json::json!({"session_id": &session_id, "code": &code})),
            || async {
                with_timeout(&self.audit, "pexpect_send", 60, || async {
                    // Use nix run via shell to pipe input to pexpect-cli
                    let mut cmd = tokio::process::Command::new("sh");
                    cmd.arg("-c");
                    cmd.arg(format!(
                        "echo '{}' | nix run nixpkgs#python3Packages.pexpect-cli -- {}",
                        code, session_id
                    ));

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pexpect-cli via nix run: {}", e),
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
                        result = "Command sent successfully (no output)".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Close an active pexpect-cli session",
        annotations(read_only_hint = false)
    )]
    pub async fn pexpect_close(
        &self,
        Parameters(PexpectCloseArgs { session_id }): Parameters<PexpectCloseArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate session ID format
        if session_id.is_empty()
            || session_id.contains('\0')
            || !session_id.chars().all(|c| c.is_alphanumeric())
        {
            return Err(McpError::invalid_params(
                "Invalid session_id".to_string(),
                Some(serde_json::json!({"session_id": session_id})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pexpect_close",
            Some(serde_json::json!({"session_id": &session_id})),
            || async {
                with_timeout(&self.audit, "pexpect_close", 30, || async {
                    // Use nix run via shell to close pexpect session
                    let output = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!(
                            "echo 'child.close()' | nix run nixpkgs#python3Packages.pexpect-cli -- {}",
                            session_id
                        ))
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to close pexpect session via nix run: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to close session: {}", stderr),
                            None,
                        ));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Session {} closed successfully",
                        session_id
                    ))]))
                })
                .await
            },
        )
        .await
    }
}
