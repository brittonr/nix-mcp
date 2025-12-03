//! Common trait for all MCP tool modules.
//!
//! This module provides the [`ToolModule`] trait which defines shared functionality
//! across all tool implementations in the codebase. All tool structs (e.g., `PackageTools`,
//! `BuildTools`, `MachineTools`) implement this trait for consistency.
//!
//! # Benefits
//!
//! - **Consistency**: All tools expose the same base functionality
//! - **Testing**: Generic tests can work with any tool module
//! - **Maintenance**: Shared behavior implemented once, used everywhere
//! - **Extension**: Easy to add new cross-cutting concerns
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::common::tool_module::ToolModule;
//! use onix_mcp::nix::PackageTools;
//! use onix_mcp::common::security::audit_logger;
//! use onix_mcp::common::cache_registry::CacheRegistry;
//! use std::sync::Arc;
//!
//! let audit = audit_logger();
//! let caches = Arc::new(CacheRegistry::new());
//! let tools = PackageTools::new(audit, caches);
//!
//! // All tools implement ToolModule
//! println!("Module name: {}", tools.name());
//! tools.log_tool_call("search_packages");
//! ```

use crate::common::security::AuditLogger;
use std::sync::Arc;

/// Common trait for all MCP tool modules.
///
/// This trait defines the minimal interface that all tool modules must implement.
/// It provides access to the audit logger and module metadata, as well as
/// convenience methods for common operations.
///
/// # Implementers
///
/// The following tool modules implement `ToolModule`:
///
/// - **Nix tools**: `PackageTools`, `BuildTools`, `DevelopTools`, `FlakeTools`, `QualityTools`, `InfoTools`
/// - **Clan tools**: `MachineTools`, `BackupTools`, `AnalysisTools`
/// - **Process tools**: `PueueTools`, `PexpectTools`
/// - **Dev tools**: `PreCommitTools`
///
/// # Examples
///
/// Working with tools generically:
///
/// ```no_run
/// use onix_mcp::common::tool_module::ToolModule;
/// use onix_mcp::nix::PackageTools;
/// use onix_mcp::common::security::audit_logger;
/// use onix_mcp::common::cache_registry::CacheRegistry;
/// use std::sync::Arc;
///
/// fn log_module_start<T: ToolModule>(tool: &T) {
///     tool.log_tool_call("module_initialized");
///     println!("Started {}", tool.name());
/// }
///
/// let audit = audit_logger();
/// let caches = Arc::new(CacheRegistry::new());
/// let pkg_tools = PackageTools::new(audit, caches);
/// log_module_start(&pkg_tools);
/// ```
pub trait ToolModule {
    /// Returns a reference to the audit logger for this tool module.
    ///
    /// All tool operations should use this logger for security auditing.
    fn audit_logger(&self) -> &Arc<AuditLogger>;

    /// Returns the name of this tool module.
    ///
    /// Used for logging, debugging, and error messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use onix_mcp::common::tool_module::ToolModule;
    /// use onix_mcp::nix::PackageTools;
    /// # use onix_mcp::common::security::audit_logger;
    /// # use onix_mcp::common::cache_registry::CacheRegistry;
    /// # use std::sync::Arc;
    /// # let audit = audit_logger();
    /// # let caches = Arc::new(CacheRegistry::new());
    /// let tools = PackageTools::new(audit, caches);
    /// assert_eq!(tools.name(), "PackageTools");
    /// ```
    fn name(&self) -> &'static str;

    /// Log a tool invocation for audit purposes.
    ///
    /// This is a convenience method that logs to the audit logger with
    /// the module name prefix. Use this to track tool usage.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool being called
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use onix_mcp::common::tool_module::ToolModule;
    /// # use onix_mcp::nix::PackageTools;
    /// # use onix_mcp::common::security::audit_logger;
    /// # use onix_mcp::common::cache_registry::CacheRegistry;
    /// # use std::sync::Arc;
    /// # let audit = audit_logger();
    /// # let caches = Arc::new(CacheRegistry::new());
    /// # let tools = PackageTools::new(audit, caches);
    ///
    /// tools.log_tool_call("search_packages");
    /// // Logs: "PackageTools::search_packages invoked"
    /// ```
    fn log_tool_call(&self, tool_name: &str) {
        self.audit_logger()
            .log_tool_invocation(tool_name, None, true, None, 0);
    }

    /// Log successful completion of a tool operation.
    ///
    /// Use this to track successful operations for metrics and debugging.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool that completed
    /// * `detail` - Optional detail message about the success
    fn log_tool_success(&self, tool_name: &str, detail: Option<&str>) {
        let message = match detail {
            Some(d) => format!("{}::{} completed: {}", self.name(), tool_name, d),
            None => format!("{}::{} completed successfully", self.name(), tool_name),
        };
        tracing::debug!("{}", message);
    }

    /// Log a tool error for debugging.
    ///
    /// Use this to track errors for metrics and debugging.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool that failed
    /// * `error` - The error that occurred
    fn log_tool_error(&self, tool_name: &str, error: &str) {
        tracing::error!("{}::{} failed: {}", self.name(), tool_name, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::security::audit_logger;

    // Test implementation of ToolModule
    struct TestTool {
        audit: Arc<AuditLogger>,
    }

    impl ToolModule for TestTool {
        fn audit_logger(&self) -> &Arc<AuditLogger> {
            &self.audit
        }

        fn name(&self) -> &'static str {
            "TestTool"
        }
    }

    #[test]
    fn test_tool_module_name() {
        let tool = TestTool {
            audit: audit_logger(),
        };
        assert_eq!(tool.name(), "TestTool");
    }

    #[test]
    fn test_tool_module_audit_logger() {
        let audit = audit_logger();
        let tool = TestTool {
            audit: audit.clone(),
        };
        assert!(Arc::ptr_eq(tool.audit_logger(), &audit));
    }

    #[test]
    fn test_tool_module_log_methods() {
        let tool = TestTool {
            audit: audit_logger(),
        };

        // These should not panic
        tool.log_tool_call("test_operation");
        tool.log_tool_success("test_operation", Some("test detail"));
        tool.log_tool_success("test_operation", None);
        tool.log_tool_error("test_operation", "test error");
    }
}
