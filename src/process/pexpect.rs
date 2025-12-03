use crate::common::security::audit::AuditLogger;
use crate::common::security::{validate_command, validation_error_to_mcp};
use crate::process::types::{PexpectCloseArgs, PexpectSendArgs, PexpectStartArgs};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

/// Tools for managing interactive sessions with pexpect-cli.
///
/// This struct provides operations for automating interactive programs like shells,
/// REPLs, SSH sessions, and other command-line tools that expect user input. Using
/// pexpect-cli, you can start sessions, send commands, and close sessions programmatically.
///
/// # Available Operations
///
/// - **Session Management**: [`pexpect_start`](Self::pexpect_start), [`pexpect_close`](Self::pexpect_close)
/// - **Interaction**: [`pexpect_send`](Self::pexpect_send)
///
/// # Caching Strategy
///
/// No caching for interactive sessions (sessions are stateful and ephemeral).
///
/// # Timeouts
///
/// All operations have 30-second timeouts:
/// - `pexpect_start`: 30 seconds (session initialization)
/// - `pexpect_send`: 30 seconds (send code and wait for response)
/// - `pexpect_close`: 30 seconds (graceful session closure)
///
/// # Security
///
/// All inputs are validated:
/// - Commands checked for null bytes and length limits
/// - Session IDs validated as alphanumeric
/// - Python code is not validated (trusts user input)
/// - All operations audited with parameters
///
/// # Pexpect Integration
///
/// This tool uses `nix run nixpkgs#python3Packages.pexpect-cli` to ensure
/// pexpect-cli is available without requiring global installation.
///
/// # Use Cases
///
/// - Automating SSH sessions
/// - Interacting with Python/Node/Ruby REPLs
/// - Testing interactive CLIs
/// - Scripting terminal-based tools
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::process::PexpectTools;
/// use onix_mcp::process::types::{PexpectStartArgs, PexpectSendArgs};
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # async fn example(tools: PexpectTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Start a Python REPL session
/// let start_result = tools.pexpect_start(Parameters(PexpectStartArgs {
///     command: "python3".to_string(),
///     args: Some(vec!["-i".to_string()]),
/// })).await?;
///
/// // Send code to the session
/// let send_result = tools.pexpect_send(Parameters(PexpectSendArgs {
///     session_id: "abc123".to_string(),
///     code: "print('Hello from pexpect!')".to_string(),
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct PexpectTools {
    pub audit: Arc<AuditLogger>,
}

impl PexpectTools {
    /// Creates a new `PexpectTools` instance with audit logging.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    ///
    /// # Note
    ///
    /// PexpectTools does not use caching as interactive sessions are
    /// stateful and ephemeral, requiring real-time interaction.
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
                    use std::process::Stdio;
                    use tokio::io::AsyncWriteExt;

                    // Use nix run with stdin piping to avoid shell injection
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#python3Packages.pexpect-cli")
                        .arg("--")
                        .arg(&session_id)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                    let mut child = cmd.spawn().map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to spawn pexpect-cli via nix run: {}", e),
                            None,
                        )
                    })?;

                    // Write code to stdin safely without shell interpolation
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(code.as_bytes()).await.map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to write code to pexpect-cli stdin: {}", e),
                                None,
                            )
                        })?;
                        drop(stdin); // Close stdin to signal EOF
                    }

                    let output = child.wait_with_output().await.map_err(|e| {
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
                    use std::process::Stdio;
                    use tokio::io::AsyncWriteExt;

                    // Use nix run with stdin piping to avoid shell injection
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#python3Packages.pexpect-cli")
                        .arg("--")
                        .arg(&session_id)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                    let mut child = cmd.spawn().map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to spawn pexpect-cli via nix run: {}", e),
                            None,
                        )
                    })?;

                    // Write close command to stdin safely
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(b"child.close()").await.map_err(|e| {
                            McpError::internal_error(
                                format!(
                                    "Failed to write close command to pexpect-cli stdin: {}",
                                    e
                                ),
                                None,
                            )
                        })?;
                        drop(stdin); // Close stdin to signal EOF
                    }

                    let output = child.wait_with_output().await.map_err(|e| {
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
