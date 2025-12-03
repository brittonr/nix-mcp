use crate::common::security::audit::AuditLogger;
use crate::common::security::{validate_command, validation_error_to_mcp};
use crate::process::types::{
    PueueAddArgs, PueueCleanArgs, PueueLogArgs, PueuePauseArgs, PueueRemoveArgs, PueueStartArgs,
    PueueStatusArgs, PueueWaitArgs,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

/// Tools for managing background tasks with the Pueue task queue.
///
/// This struct provides operations for adding commands to a background task queue,
/// monitoring their execution, and managing their lifecycle. Pueue enables long-running
/// commands to execute asynchronously without blocking the MCP server.
///
/// # Available Operations
///
/// - **Task Management**: [`pueue_add`](Self::pueue_add), [`pueue_remove`](Self::pueue_remove), [`pueue_clean`](Self::pueue_clean)
/// - **Task Control**: [`pueue_start`](Self::pueue_start), [`pueue_pause`](Self::pueue_pause)
/// - **Monitoring**: [`pueue_status`](Self::pueue_status), [`pueue_log`](Self::pueue_log), [`pueue_wait`](Self::pueue_wait)
///
/// # Caching Strategy
///
/// No caching for task queue operations (task state changes in real-time).
///
/// # Timeouts
///
/// Most operations have 30-second timeouts (quick pueue commands):
/// - `pueue_add`: 30 seconds
/// - `pueue_status`: 30 seconds
/// - `pueue_log`: 30 seconds
/// - `pueue_wait`: 300 seconds (5 minutes - waits for task completion)
/// - `pueue_remove`, `pueue_clean`, `pueue_pause`, `pueue_start`: 30 seconds
///
/// # Security
///
/// All commands are validated before execution:
/// - Commands checked for null bytes and length limits
/// - Working directories validated for path traversal
/// - All operations audited with parameters
///
/// # Pueue Integration
///
/// This tool uses `nix run nixpkgs#pueue` to ensure pueue is available
/// without requiring it to be installed globally. The pueue daemon must
/// be running for these tools to work.
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::process::PueueTools;
/// use onix_mcp::process::types::PueueAddArgs;
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # async fn example(tools: PueueTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Add a long-running build task to the queue
/// let result = tools.pueue_add(Parameters(PueueAddArgs {
///     command: "nix build .#mypackage".to_string(),
///     args: None,
///     working_directory: Some("/home/user/project".to_string()),
///     label: Some("build-mypackage".to_string()),
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct PueueTools {
    pub audit: Arc<AuditLogger>,
}

impl PueueTools {
    /// Creates a new `PueueTools` instance with audit logging.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    ///
    /// # Note
    ///
    /// PueueTools does not use caching as task queue state changes
    /// in real-time and must reflect current execution status.
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl PueueTools {
    #[tool(
        description = "Add a command to the pueue task queue for async execution. Returns task ID.",
        annotations(read_only_hint = false)
    )]
    pub async fn pueue_add(
        &self,
        Parameters(PueueAddArgs {
            command,
            args,
            working_directory,
            label,
        }): Parameters<PueueAddArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate command
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Validate working directory if provided
        if let Some(ref wd) = working_directory {
            use crate::common::security::validate_path;
            validate_path(wd).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_add",
            Some(serde_json::json!({"command": &command, "args": &args, "working_directory": &working_directory, "label": &label})),
            || async {
                with_timeout(&self.audit, "pueue_add", 30, || async {
                    // Use nix run to ensure pueue is available
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("add");

                    if let Some(wd) = working_directory {
                        cmd.arg("--working-directory").arg(wd);
                    }

                    if let Some(lbl) = label {
                        cmd.arg("--label").arg(lbl);
                    }

                    cmd.arg("--");
                    cmd.arg(&command);

                    if let Some(command_args) = args {
                        for arg in command_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue add via nix run: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue add failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        stdout.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get the status of pueue tasks (all or specific task IDs)",
        annotations(read_only_hint = true)
    )]
    pub async fn pueue_status(
        &self,
        Parameters(PueueStatusArgs { task_ids }): Parameters<PueueStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_status",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_status", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("status");

                    if let Some(ids) = task_ids {
                        for id in ids.split(',') {
                            cmd.arg(id.trim());
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue status: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue status failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        stdout.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get logs for a specific pueue task",
        annotations(read_only_hint = true)
    )]
    pub async fn pueue_log(
        &self,
        Parameters(PueueLogArgs { task_id, lines }): Parameters<PueueLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_log",
            Some(serde_json::json!({"task_id": &task_id, "lines": &lines})),
            || async {
                with_timeout(&self.audit, "pueue_log", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#pueue")
                        .arg("--")
                        .arg("log")
                        .arg(task_id.to_string());

                    if let Some(n) = lines {
                        cmd.arg("--lines").arg(n.to_string());
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue log: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue log failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        stdout.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Wait for specific pueue tasks to complete",
        annotations(read_only_hint = true)
    )]
    pub async fn pueue_wait(
        &self,
        Parameters(PueueWaitArgs { task_ids, timeout }): Parameters<PueueWaitArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Validate task IDs format
        if task_ids.is_empty() || task_ids.contains('\0') {
            return Err(McpError::invalid_params(
                "Invalid task_ids".to_string(),
                Some(serde_json::json!({"task_ids": task_ids})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_wait",
            Some(serde_json::json!({"task_ids": &task_ids, "timeout": &timeout})),
            || async {
                // Use custom timeout for wait command
                let wait_timeout = timeout.unwrap_or(300);

                let timeout_duration = tokio::time::Duration::from_secs(wait_timeout);
                let result = tokio::time::timeout(timeout_duration, async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("wait");

                    for id in task_ids.split(',') {
                        cmd.arg(id.trim());
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue wait: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue wait failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = format!("Task(s) {} completed successfully", task_ids);
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("pueue wait timed out after {} seconds", wait_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Remove/kill specific pueue tasks",
        annotations(read_only_hint = false)
    )]
    pub async fn pueue_remove(
        &self,
        Parameters(PueueRemoveArgs { task_ids }): Parameters<PueueRemoveArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate task IDs format
        if task_ids.is_empty() || task_ids.contains('\0') {
            return Err(McpError::invalid_params(
                "Invalid task_ids".to_string(),
                Some(serde_json::json!({"task_ids": task_ids})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_remove",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_remove", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("remove");

                    for id in task_ids.split(',') {
                        cmd.arg(id.trim());
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue remove: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue remove failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = format!("Task(s) {} removed successfully", task_ids);
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Clean up finished pueue tasks from the queue",
        annotations(read_only_hint = false)
    )]
    pub async fn pueue_clean(
        &self,
        Parameters(_): Parameters<PueueCleanArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "pueue_clean", None, || async {
            with_timeout(&self.audit, "pueue_clean", 30, || async {
                let output = tokio::process::Command::new("nix")
                    .arg("run")
                    .arg("nixpkgs#pueue")
                    .arg("--")
                    .arg("clean")
                    .output()
                    .await
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue clean: {}", e),
                            None,
                        )
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(
                        format!("pueue clean failed: {}", stderr),
                        None,
                    ));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut result_text = stdout.to_string();
                if result_text.is_empty() {
                    result_text = "Finished tasks cleaned successfully".to_string();
                }

                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            })
            .await
        })
        .await
    }

    #[tool(
        description = "Pause specific pueue tasks or all tasks",
        annotations(read_only_hint = false)
    )]
    pub async fn pueue_pause(
        &self,
        Parameters(PueuePauseArgs { task_ids }): Parameters<PueuePauseArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_pause",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_pause", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("pause");

                    if let Some(ids) = task_ids {
                        for id in ids.split(',') {
                            cmd.arg(id.trim());
                        }
                    } else {
                        cmd.arg("--all");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue pause: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue pause failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = "Task(s) paused successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Start/resume specific pueue tasks or all tasks",
        annotations(read_only_hint = false)
    )]
    pub async fn pueue_start(
        &self,
        Parameters(PueueStartArgs { task_ids }): Parameters<PueueStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_start",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_start", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("start");

                    if let Some(ids) = task_ids {
                        for id in ids.split(',') {
                            cmd.arg(id.trim());
                        }
                    } else {
                        cmd.arg("--all");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue start: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue start failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = "Task(s) started successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }
}
