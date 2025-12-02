/// Security helper functions for integrating validation and audit logging into tools
use super::{AuditLogger, ValidationError};
use rmcp::ErrorData as McpError;
use serde_json::json;
use std::time::Instant;

/// Convert ValidationError to McpError
pub fn validation_error_to_mcp(err: ValidationError) -> McpError {
    McpError::invalid_params(
        err.to_string(),
        Some(json!({
            "validation_error": format!("{:?}", err),
        })),
    )
}

/// Audit tool execution with timing
pub async fn audit_tool_execution<F, Fut, T>(
    audit: &AuditLogger,
    tool_name: &str,
    parameters: Option<serde_json::Value>,
    f: F,
) -> Result<T, McpError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, McpError>>,
{
    let start = Instant::now();
    let result = f().await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            audit.log_tool_invocation(tool_name, parameters, true, None, duration_ms);
        }
        Err(e) => {
            audit.log_tool_invocation(
                tool_name,
                parameters,
                false,
                Some(e.message.to_string()),
                duration_ms,
            );
        }
    }

    result
}

/// Execute with timeout
pub async fn with_timeout<F, Fut, T>(
    audit: &AuditLogger,
    operation_name: &str,
    timeout_secs: u64,
    f: F,
) -> Result<T, McpError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, McpError>>,
{
    match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), f()).await {
        Ok(result) => result,
        Err(_) => {
            audit.log_timeout(operation_name, timeout_secs);
            Err(McpError::internal_error(
                format!("Operation timed out after {} seconds", timeout_secs),
                Some(json!({
                    "operation": operation_name,
                    "timeout_seconds": timeout_secs,
                })),
            ))
        }
    }
}

/// Execute with cancellation support
#[allow(dead_code)]
pub async fn with_cancellation<F, Fut, T>(
    ct: &tokio_util::sync::CancellationToken,
    f: F,
) -> Result<T, McpError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, McpError>>,
{
    tokio::select! {
        result = f() => result,
        _ = ct.cancelled() => {
            Err(McpError::internal_error(
                "Operation cancelled by client".to_string(),
                None,
            ))
        }
    }
}

/// Macro to wrap tool execution with security features
#[macro_export]
macro_rules! secure_tool {
    (
        audit = $audit:expr,
        tool_name = $tool_name:expr,
        params = $params:expr,
        timeout = $timeout:expr,
        ct = $ct:expr,
        $body:block
    ) => {{
        use $crate::common::security::helpers::{
            audit_tool_execution, with_cancellation, with_timeout,
        };

        audit_tool_execution($audit, $tool_name, Some($params), || async {
            with_cancellation($ct, || async {
                with_timeout($audit, $tool_name, $timeout, || async { $body }).await
            })
            .await
        })
        .await
    }};
}
