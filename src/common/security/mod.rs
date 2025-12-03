//! Security infrastructure for onix-mcp server.
//!
//! This module provides comprehensive security mechanisms to protect against
//! common vulnerabilities and ensure safe operation of the MCP server.
//!
//! # Modules
//!
//! - [`audit`] - Security event logging and audit trail management
//! - [`helpers`] - Security helper functions (timeouts, validation wrappers)
//! - [`input_validation`] - Input validation functions to prevent injection attacks
//!
//! # Security Features
//!
//! ## Input Validation
//!
//! All user inputs are validated before use in shell commands or system operations:
//!
//! - **Command Injection Prevention**: Blocks shell metacharacters (`;`, `$`, backticks, etc.)
//! - **Path Traversal Protection**: Prevents `../` sequences and dangerous system paths
//! - **Null Byte Filtering**: Blocks null bytes that could truncate commands
//! - **Length Limits**: Enforces maximum sizes to prevent DoS attacks
//! - **Pattern Matching**: Validates against expected formats (packages, flake refs, etc.)
//!
//! ## Audit Logging
//!
//! All security-relevant events are logged:
//!
//! - Tool invocations with parameters
//! - Validation failures
//! - Timeout events
//! - Success/failure status
//!
//! ## Timeout Protection
//!
//! All external command executions have configurable timeouts to prevent:
//!
//! - Resource exhaustion
//! - Hung processes
//! - Denial of service
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::common::security::{validate_package_name, audit_logger};
//!
//! // Input validation
//! let package = "ripgrep";
//! validate_package_name(package).expect("Invalid package name");
//!
//! // Audit logging
//! let logger = audit_logger();
//! logger.log_tool_invocation("search_packages", None, true, None, 0);
//! ```
//!
//! # Threat Model
//!
//! This module protects against:
//!
//! - **Command Injection** (OWASP A03:2021): Shell metacharacter filtering
//! - **Path Traversal** (OWASP A01:2021): Directory traversal prevention
//! - **Denial of Service**: Timeouts, length limits, resource controls
//! - **Information Disclosure**: Audit logging of security events
//!
//! # Validation Functions
//!
//! - [`validate_package_name`] - Nix package names (alphanumeric, -, _, .)
//! - [`validate_flake_ref`] - Flake references (paths, URLs, identifiers)
//! - [`validate_nix_expression`] - Nix expressions (dangerous patterns blocked)
//! - [`validate_command`] - Shell commands (null bytes, length checks)
//! - [`validate_machine_name`] - Clan machine names (RFC 1123 compliant)
//! - [`validate_url`] - HTTP(S)/FTP URLs (protocol whitelist)
//! - [`validate_path`] - File paths (traversal prevention, dangerous paths)

pub mod audit;
pub mod helpers;
pub mod input_validation;

pub use audit::{audit_logger, AuditLogger};
pub use helpers::validation_error_to_mcp;
pub use input_validation::{
    validate_command, validate_flake_ref, validate_machine_name, validate_nix_expression,
    validate_package_name, validate_path, validate_url, ValidationError,
};
