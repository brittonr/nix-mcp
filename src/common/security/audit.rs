/// Audit logging infrastructure for security events
/// Provides structured logging of security-relevant operations
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Security levels for audit events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Informational security event (normal operation)
    Info,
    /// Warning - suspicious but allowed
    Warning,
    /// Error - security violation or failure
    Error,
    /// Critical - serious security breach attempt
    Critical,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEvent {
    /// Tool invocation
    ToolInvoked {
        tool_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        duration_ms: u64,
    },

    /// Input validation failure
    ValidationFailed {
        field: String,
        value: String,
        reason: String,
    },

    /// Suspicious activity detected
    SuspiciousActivity {
        description: String,
        details: serde_json::Value,
    },

    /// Rate limit exceeded
    RateLimitExceeded {
        operation: String,
        limit: u32,
        actual: u32,
    },

    /// Operation timeout
    OperationTimeout {
        operation: String,
        timeout_secs: u64,
    },

    /// Authentication/authorization event
    AuthEvent { success: bool, reason: String },

    /// Dangerous operation attempted
    DangerousOperation {
        operation: String,
        approved: bool,
        reason: String,
    },
}

/// Audit logger implementation
#[derive(Clone)]
pub struct AuditLogger {
    // In future, could add structured log output, remote logging, etc.
    _marker: std::marker::PhantomData<()>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Log an audit event with security level
    pub fn log(&self, level: SecurityLevel, event: AuditEvent) {
        let event_json = serde_json::to_string(&event)
            .unwrap_or_else(|e| format!("{{\"error\": \"failed to serialize event: {}\"}}", e));

        match level {
            SecurityLevel::Info => {
                info!(
                    security_level = "info",
                    event = %event_json,
                    "Security audit event"
                );
            }
            SecurityLevel::Warning => {
                warn!(
                    security_level = "warning",
                    event = %event_json,
                    "Security audit warning"
                );
            }
            SecurityLevel::Error => {
                error!(
                    security_level = "error",
                    event = %event_json,
                    "Security audit error"
                );
            }
            SecurityLevel::Critical => {
                error!(
                    security_level = "critical",
                    event = %event_json,
                    "CRITICAL security audit event"
                );
            }
        }
    }

    /// Log tool invocation
    pub fn log_tool_invocation(
        &self,
        tool_name: &str,
        parameters: Option<serde_json::Value>,
        success: bool,
        error: Option<String>,
        duration_ms: u64,
    ) {
        let event = AuditEvent::ToolInvoked {
            tool_name: tool_name.to_string(),
            parameters,
            success,
            error,
            duration_ms,
        };

        let level = if success {
            SecurityLevel::Info
        } else {
            SecurityLevel::Warning
        };

        self.log(level, event);
    }

    /// Log validation failure
    #[allow(dead_code)]
    pub fn log_validation_failure(&self, field: &str, value: &str, reason: &str) {
        let event = AuditEvent::ValidationFailed {
            field: field.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        };

        self.log(SecurityLevel::Warning, event);
    }

    /// Log suspicious activity
    #[allow(dead_code)]
    pub fn log_suspicious_activity(&self, description: &str, details: serde_json::Value) {
        let event = AuditEvent::SuspiciousActivity {
            description: description.to_string(),
            details,
        };

        self.log(SecurityLevel::Error, event);
    }

    /// Log rate limit exceeded
    #[allow(dead_code)]
    pub fn log_rate_limit_exceeded(&self, operation: &str, limit: u32, actual: u32) {
        let event = AuditEvent::RateLimitExceeded {
            operation: operation.to_string(),
            limit,
            actual,
        };

        self.log(SecurityLevel::Warning, event);
    }

    /// Log operation timeout
    pub fn log_timeout(&self, operation: &str, timeout_secs: u64) {
        let event = AuditEvent::OperationTimeout {
            operation: operation.to_string(),
            timeout_secs,
        };

        self.log(SecurityLevel::Warning, event);
    }

    /// Log authentication/authorization event
    #[allow(dead_code)]
    pub fn log_auth_event(&self, success: bool, reason: &str) {
        let event = AuditEvent::AuthEvent {
            success,
            reason: reason.to_string(),
        };

        let level = if success {
            SecurityLevel::Info
        } else {
            SecurityLevel::Error
        };

        self.log(level, event);
    }

    /// Log dangerous operation
    pub fn log_dangerous_operation(&self, operation: &str, approved: bool, reason: &str) {
        let event = AuditEvent::DangerousOperation {
            operation: operation.to_string(),
            approved,
            reason: reason.to_string(),
        };

        let level = if approved {
            SecurityLevel::Warning
        } else {
            SecurityLevel::Error
        };

        self.log(level, event);
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global audit logger instance
static AUDIT_LOGGER: once_cell::sync::Lazy<Arc<AuditLogger>> =
    once_cell::sync::Lazy::new(|| Arc::new(AuditLogger::new()));

/// Get global audit logger
pub fn audit_logger() -> Arc<AuditLogger> {
    Arc::clone(&AUDIT_LOGGER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        logger.log_tool_invocation("test_tool", None, true, None, 100);
    }

    #[test]
    fn test_global_audit_logger() {
        let logger = audit_logger();
        logger.log_validation_failure("test_field", "test_value", "test_reason");
    }
}
