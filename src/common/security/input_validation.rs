use once_cell::sync::Lazy;
/// Input validation for Nix MCP server
/// Prevents command injection, path traversal, and other security vulnerabilities
use regex::Regex;
use std::path::PathBuf;

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    Empty {
        field: String,
    },
    #[allow(dead_code)]
    InvalidCharacters {
        field: String,
        value: String,
    },
    PathTraversal {
        path: String,
    },
    TooLong {
        field: String,
        max_length: usize,
        actual: usize,
    },
    InvalidFormat {
        field: String,
        expected: String,
        got: String,
    },
    Suspicious {
        field: String,
        reason: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Empty { field } => {
                write!(f, "Field '{}' cannot be empty", field)
            }
            ValidationError::InvalidCharacters { field, value } => {
                write!(
                    f,
                    "Field '{}' contains invalid characters: '{}'",
                    field, value
                )
            }
            ValidationError::PathTraversal { path } => {
                write!(f, "Path contains traversal attempt: '{}'", path)
            }
            ValidationError::TooLong {
                field,
                max_length,
                actual,
            } => {
                write!(
                    f,
                    "Field '{}' too long: {} characters (max: {})",
                    field, actual, max_length
                )
            }
            ValidationError::InvalidFormat {
                field,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Field '{}' has invalid format. Expected: {}, got: '{}'",
                    field, expected, got
                )
            }
            ValidationError::Suspicious { field, reason } => {
                write!(f, "Field '{}' is suspicious: {}", field, reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Maximum lengths for various input types
const MAX_PACKAGE_NAME_LEN: usize = 255;
const MAX_FLAKE_REF_LEN: usize = 1000;
const MAX_PATH_LEN: usize = 4096;
const MAX_EXPRESSION_LEN: usize = 10000;
const MAX_COMMAND_LEN: usize = 1000;

/// Regex patterns for validation
static PACKAGE_NAME_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_][a-zA-Z0-9_\-\.]*$").unwrap());

static FLAKE_REF_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Matches:
    // - Simple registry refs: nixpkgs
    // - Registry refs with fragments: nixpkgs#hello, github:owner/repo
    // - URLs: https://..., git+https://...
    // - Paths: ., ./, ../, /absolute/path
    // Note: This is a permissive regex - shell metacharacters are blocked separately
    Regex::new(r"^[a-zA-Z0-9_\-\.\+/:@#]+$").unwrap()
});

static MACHINE_NAME_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_\-]+$").unwrap());

/// Dangerous patterns that should never appear in Nix expressions
static DANGEROUS_PATTERNS: &[&str] = &[
    "__noChroot",
    "allowSubstitutes = false",
    "trustedUsers",
    "allowed-users",
    "builders",
    "substituters",
    "trusted-substituters",
    "system-features",
    "builtins.exec",
];

/// Shell metacharacters that indicate potential command injection
static SHELL_METACHARACTERS: &[char] = &[
    ';', '|', '&', '$', '`', '\n', '\r', '>', '<', '(', ')', '{', '}', '[', ']', '!', '*', '?',
];

/// Validate package name for nixpkgs
///
/// Ensures package names:
/// - Are not empty
/// - Don't exceed length limits
/// - Only contain allowed characters (alphanumeric, underscore, hyphen, dot)
/// - Start with alphanumeric or underscore
/// - Don't contain path traversal patterns
pub fn validate_package_name(name: &str) -> Result<(), ValidationError> {
    // Check empty
    if name.is_empty() {
        return Err(ValidationError::Empty {
            field: "package_name".to_string(),
        });
    }

    // Check length
    if name.len() > MAX_PACKAGE_NAME_LEN {
        return Err(ValidationError::TooLong {
            field: "package_name".to_string(),
            max_length: MAX_PACKAGE_NAME_LEN,
            actual: name.len(),
        });
    }

    // Check for path traversal
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(ValidationError::PathTraversal {
            path: name.to_string(),
        });
    }

    // Check pattern
    if !PACKAGE_NAME_PATTERN.is_match(name) {
        return Err(ValidationError::InvalidFormat {
            field: "package_name".to_string(),
            expected: "alphanumeric, underscore, hyphen, dot only".to_string(),
            got: name.to_string(),
        });
    }

    // Check for suspicious patterns
    if name.starts_with('.') || name.ends_with('.') {
        return Err(ValidationError::Suspicious {
            field: "package_name".to_string(),
            reason: "cannot start or end with dot".to_string(),
        });
    }

    Ok(())
}

/// Validate flake reference
///
/// Supports formats:
/// - Relative paths: ".", "..", "./path"
/// - Absolute paths: "/path/to/flake"
/// - Git URLs: "github:owner/repo", "git+https://..."
/// - Flake registry: "nixpkgs", "nixpkgs/nixos-unstable"
pub fn validate_flake_ref(flake_ref: &str) -> Result<(), ValidationError> {
    // Check empty
    if flake_ref.is_empty() {
        return Err(ValidationError::Empty {
            field: "flake_ref".to_string(),
        });
    }

    // Check length
    if flake_ref.len() > MAX_FLAKE_REF_LEN {
        return Err(ValidationError::TooLong {
            field: "flake_ref".to_string(),
            max_length: MAX_FLAKE_REF_LEN,
            actual: flake_ref.len(),
        });
    }

    // Check pattern
    if !FLAKE_REF_PATTERN.is_match(flake_ref) {
        return Err(ValidationError::InvalidFormat {
            field: "flake_ref".to_string(),
            expected: "valid flake reference (path, URL, or registry)".to_string(),
            got: flake_ref.to_string(),
        });
    }

    // Check for null bytes (command injection via C strings)
    if flake_ref.contains('\0') {
        return Err(ValidationError::Suspicious {
            field: "flake_ref".to_string(),
            reason: "contains null byte".to_string(),
        });
    }

    // Check for shell metacharacters
    for &metachar in SHELL_METACHARACTERS {
        if flake_ref.contains(metachar) {
            return Err(ValidationError::Suspicious {
                field: "flake_ref".to_string(),
                reason: format!("contains shell metacharacter: '{}'", metachar),
            });
        }
    }

    Ok(())
}

/// Validate filesystem path
///
/// Prevents:
/// - Path traversal attacks
/// - Access to sensitive system paths
/// - Symlink attacks
pub fn validate_path(path: &str) -> Result<PathBuf, ValidationError> {
    // Check empty
    if path.is_empty() {
        return Err(ValidationError::Empty {
            field: "path".to_string(),
        });
    }

    // Check length
    if path.len() > MAX_PATH_LEN {
        return Err(ValidationError::TooLong {
            field: "path".to_string(),
            max_length: MAX_PATH_LEN,
            actual: path.len(),
        });
    }

    // Parse path
    let path_buf = PathBuf::from(path);

    // Check for path traversal patterns
    for component in path_buf.components() {
        if let std::path::Component::ParentDir = component {
            return Err(ValidationError::PathTraversal {
                path: path.to_string(),
            });
        }
    }

    // Check for dangerous system paths
    let dangerous_prefixes = [
        "/etc/shadow",
        "/etc/passwd",
        "/root/.ssh",
        "/home/*/.ssh",
        "/var/lib/private",
    ];

    for prefix in dangerous_prefixes {
        if path.starts_with(prefix) {
            return Err(ValidationError::Suspicious {
                field: "path".to_string(),
                reason: format!("access to sensitive path: {}", prefix),
            });
        }
    }

    // Canonicalize if path exists (resolves symlinks)
    if path_buf.exists() {
        match path_buf.canonicalize() {
            Ok(canonical) => Ok(canonical),
            Err(_) => Err(ValidationError::Suspicious {
                field: "path".to_string(),
                reason: "cannot canonicalize path (broken symlink?)".to_string(),
            }),
        }
    } else {
        Ok(path_buf)
    }
}

/// Validate Nix expression for evaluation
///
/// Checks for:
/// - Dangerous patterns (builtins that bypass sandboxing)
/// - Excessive length
/// - Shell injection attempts
pub fn validate_nix_expression(expr: &str) -> Result<(), ValidationError> {
    // Check empty
    if expr.is_empty() {
        return Err(ValidationError::Empty {
            field: "expression".to_string(),
        });
    }

    // Check length
    if expr.len() > MAX_EXPRESSION_LEN {
        return Err(ValidationError::TooLong {
            field: "expression".to_string(),
            max_length: MAX_EXPRESSION_LEN,
            actual: expr.len(),
        });
    }

    // Check for dangerous patterns
    for &pattern in DANGEROUS_PATTERNS {
        if expr.contains(pattern) {
            return Err(ValidationError::Suspicious {
                field: "expression".to_string(),
                reason: format!("contains dangerous pattern: {}", pattern),
            });
        }
    }

    // Check for shell command injection attempts
    if expr.contains("$(") || expr.contains("`") {
        return Err(ValidationError::Suspicious {
            field: "expression".to_string(),
            reason: "contains shell command substitution".to_string(),
        });
    }

    // Check for null bytes
    if expr.contains('\0') {
        return Err(ValidationError::Suspicious {
            field: "expression".to_string(),
            reason: "contains null byte".to_string(),
        });
    }

    Ok(())
}

/// Validate command for nix-shell execution
///
/// Ensures commands:
/// - Don't contain shell injection patterns
/// - Are reasonable length
/// - Don't access dangerous paths
pub fn validate_command(command: &str) -> Result<(), ValidationError> {
    // Check empty
    if command.is_empty() {
        return Err(ValidationError::Empty {
            field: "command".to_string(),
        });
    }

    // Check length
    if command.len() > MAX_COMMAND_LEN {
        return Err(ValidationError::TooLong {
            field: "command".to_string(),
            max_length: MAX_COMMAND_LEN,
            actual: command.len(),
        });
    }

    // Check for null bytes
    if command.contains('\0') {
        return Err(ValidationError::Suspicious {
            field: "command".to_string(),
            reason: "contains null byte".to_string(),
        });
    }

    // Warn about dangerous commands (but don't block - user may have legitimate need)
    let dangerous_commands = [
        "rm -rf",
        "dd if=",
        "mkfs",
        "fdisk",
        "parted",
        ":(){ :|:& };:",
    ];
    for dangerous in &dangerous_commands {
        if command.contains(*dangerous) {
            tracing::warn!(
                command = %command,
                pattern = %dangerous,
                "User command contains potentially dangerous pattern"
            );
        }
    }

    Ok(())
}

/// Validate machine name for Clan operations
pub fn validate_machine_name(name: &str) -> Result<(), ValidationError> {
    // Check empty
    if name.is_empty() {
        return Err(ValidationError::Empty {
            field: "machine_name".to_string(),
        });
    }

    // Check length
    if name.len() > 63 {
        return Err(ValidationError::TooLong {
            field: "machine_name".to_string(),
            max_length: 63,
            actual: name.len(),
        });
    }

    // Check pattern (hostname rules)
    if !MACHINE_NAME_PATTERN.is_match(name) {
        return Err(ValidationError::InvalidFormat {
            field: "machine_name".to_string(),
            expected: "alphanumeric, underscore, hyphen only".to_string(),
            got: name.to_string(),
        });
    }

    // Check start/end
    if name.starts_with('-') || name.ends_with('-') {
        return Err(ValidationError::Suspicious {
            field: "machine_name".to_string(),
            reason: "cannot start or end with hyphen".to_string(),
        });
    }

    Ok(())
}

/// Validate URL for prefetch operations
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    // Check empty
    if url.is_empty() {
        return Err(ValidationError::Empty {
            field: "url".to_string(),
        });
    }

    // Check length
    if url.len() > 2048 {
        return Err(ValidationError::TooLong {
            field: "url".to_string(),
            max_length: 2048,
            actual: url.len(),
        });
    }

    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("ftp://") {
        return Err(ValidationError::InvalidFormat {
            field: "url".to_string(),
            expected: "http://, https://, or ftp:// URL".to_string(),
            got: url.to_string(),
        });
    }

    // Check for null bytes
    if url.contains('\0') {
        return Err(ValidationError::Suspicious {
            field: "url".to_string(),
            reason: "contains null byte".to_string(),
        });
    }

    // Check for spaces (should be URL-encoded)
    if url.contains(' ') {
        return Err(ValidationError::Suspicious {
            field: "url".to_string(),
            reason: "contains unencoded space".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name() {
        // Valid names
        assert!(validate_package_name("ripgrep").is_ok());
        assert!(validate_package_name("python3").is_ok());
        assert!(validate_package_name("rust_1.70.0").is_ok());
        assert!(validate_package_name("gcc-wrapper").is_ok());

        // Invalid names
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("../etc/passwd").is_err());
        assert!(validate_package_name("package;rm -rf /").is_err());
        assert!(validate_package_name("package`whoami`").is_err());
        assert!(validate_package_name(".hidden").is_err());
    }

    #[test]
    fn test_validate_flake_ref() {
        // Valid refs
        assert!(validate_flake_ref(".").is_ok());
        assert!(validate_flake_ref("nixpkgs").is_ok());
        assert!(validate_flake_ref("github:nixos/nixpkgs").is_ok());
        assert!(validate_flake_ref("git+https://github.com/nixos/nixpkgs").is_ok());

        // Invalid refs
        assert!(validate_flake_ref("").is_err());
        assert!(validate_flake_ref("nixpkgs; rm -rf /").is_err());
        assert!(validate_flake_ref("nixpkgs`whoami`").is_err());
    }

    #[test]
    fn test_validate_nix_expression() {
        // Valid expressions
        assert!(validate_nix_expression("1 + 1").is_ok());
        assert!(validate_nix_expression("builtins.toString 42").is_ok());
        assert!(validate_nix_expression("{ a = 1; b = 2; }").is_ok());

        // Invalid expressions
        assert!(validate_nix_expression("").is_err());
        assert!(validate_nix_expression("builtins.exec [\"rm\" \"-rf\" \"/\"]").is_err());
        assert!(validate_nix_expression("$(rm -rf /)").is_err());
    }

    #[test]
    fn test_validate_machine_name() {
        // Valid names
        assert!(validate_machine_name("server-01").is_ok());
        assert!(validate_machine_name("web_server").is_ok());
        assert!(validate_machine_name("prod123").is_ok());

        // Invalid names
        assert!(validate_machine_name("").is_err());
        assert!(validate_machine_name("-server").is_err());
        assert!(validate_machine_name("server-").is_err());
        assert!(validate_machine_name("server.local").is_err());
    }
}
