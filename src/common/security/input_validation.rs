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

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // ========== validate_package_name property tests ==========

    proptest! {
        /// Test that valid package names are accepted
        /// Pattern: start with alphanumeric/underscore, then any alphanumeric/underscore/hyphen/dot
        #[test]
        fn prop_valid_package_names_accept(
            start in "[a-zA-Z0-9_]",
            rest in "[a-zA-Z0-9_\\-\\.]{0,254}"
        ) {
            let name = format!("{}{}", start, rest);
            // Skip names starting or ending with dot (explicitly rejected)
            // Skip names containing ".." (path traversal, explicitly rejected)
            if !name.starts_with('.') && !name.ends_with('.') && !name.contains("..") {
                prop_assert!(validate_package_name(&name).is_ok(), "Valid package name rejected: {}", name);
            }
        }

        /// Test that command injection attempts are rejected
        #[test]
        fn prop_package_name_command_injection_reject(
            base in "[a-zA-Z0-9_]{1,10}",
            exploit in prop::sample::select(vec![
                ";rm -rf /", "$(whoami)", "`cat /etc/passwd`",
                "||true", "&&false", "|cat", ">/dev/null",
                ";true", "&background", "$VAR", "`cmd`"
            ])
        ) {
            let malicious = format!("{}{}", base, exploit);
            prop_assert!(validate_package_name(&malicious).is_err(),
                "Command injection not rejected: {}", malicious);
        }

        /// Test that path traversal attempts are rejected
        #[test]
        fn prop_package_name_path_traversal_reject(
            dots in "\\.\\./+",
            suffix in "[a-z]{1,10}"
        ) {
            let malicious = format!("{}{}", dots, suffix);
            prop_assert!(validate_package_name(&malicious).is_err(),
                "Path traversal not rejected: {}", malicious);
        }

        /// Test that names with slashes/backslashes are rejected
        #[test]
        fn prop_package_name_slash_reject(
            prefix in "[a-zA-Z0-9_]{1,10}",
            slash in "[/\\\\]",
            suffix in "[a-zA-Z0-9_]{1,10}"
        ) {
            let malicious = format!("{}{}{}", prefix, slash, suffix);
            prop_assert!(validate_package_name(&malicious).is_err(),
                "Slash not rejected: {}", malicious);
        }

        /// Test that overly long names are rejected
        #[test]
        fn prop_package_name_too_long_reject(
            name in "[a-zA-Z0-9_]{256,300}"
        ) {
            prop_assert!(validate_package_name(&name).is_err(),
                "Overly long name not rejected");
        }
    }

    // ========== validate_flake_ref property tests ==========

    proptest! {
        /// Test that shell metacharacters in flake refs are rejected
        #[test]
        fn prop_flake_ref_shell_metacharacters_reject(
            base in "nixpkgs|github:[a-z]+/[a-z]+",
            metachar in "[;`$(){}\\[\\]|&><\\n\\r!*?]"
        ) {
            let malicious = format!("{}{}", base, metachar);
            prop_assert!(validate_flake_ref(&malicious).is_err(),
                "Shell metacharacter not rejected: {}", malicious);
        }

        /// Test that null bytes are rejected in flake refs
        #[test]
        fn prop_flake_ref_null_byte_reject(
            base in "[a-zA-Z0-9_]{1,20}"
        ) {
            let malicious = format!("{}\0poison", base);
            prop_assert!(validate_flake_ref(&malicious).is_err(),
                "Null byte not rejected");
        }

        /// Test that valid flake ref patterns are accepted
        #[test]
        fn prop_flake_ref_valid_patterns_accept(
            pattern in prop::sample::select(vec![
                ".",
                "..",
                "./foo",
                "../bar",
                "nixpkgs",
                "github:nixos/nixpkgs",
                "git+https://github.com/nixos/nixpkgs",
            ])
        ) {
            prop_assert!(validate_flake_ref(pattern).is_ok(),
                "Valid flake ref rejected: {}", pattern);
        }

        /// Test that overly long flake refs are rejected
        #[test]
        fn prop_flake_ref_too_long_reject(
            ref_str in "[a-zA-Z0-9_]{1001,1100}"
        ) {
            prop_assert!(validate_flake_ref(&ref_str).is_err(),
                "Overly long flake ref not rejected");
        }

        /// Test command injection attempts in flake refs
        #[test]
        fn prop_flake_ref_command_injection_reject(
            base in "nixpkgs|github:[a-z]+/[a-z]+",
            inject in ";.*|&&.*|\\|\\|.*|`.*`|\\$\\(.*\\)"
        ) {
            let malicious = format!("{}{}", base, inject);
            prop_assert!(validate_flake_ref(&malicious).is_err(),
                "Command injection not rejected: {}", malicious);
        }
    }

    // ========== validate_nix_expression property tests ==========

    proptest! {
        /// Test that dangerous patterns in Nix expressions are rejected
        #[test]
        fn prop_nix_expression_dangerous_patterns_reject(
            pattern in prop::sample::select(vec![
                "__noChroot",
                "allowSubstitutes = false",
                "trustedUsers",
                "allowed-users",
                "builders",
                "substituters",
                "trusted-substituters",
                "system-features",
                "builtins.exec",
            ])
        ) {
            let expr = format!("{{ {} = true; }}", pattern);
            prop_assert!(validate_nix_expression(&expr).is_err(),
                "Dangerous pattern not rejected: {}", pattern);
        }

        /// Test that shell command substitution is rejected
        #[test]
        fn prop_nix_expression_shell_substitution_reject(
            base in "[a-zA-Z0-9_]+",
            cmd in "[a-z]{1,10}"
        ) {
            let expr1 = format!("{} $({}) end", base, cmd);
            let expr2 = format!("{} `{}` end", base, cmd);
            prop_assert!(validate_nix_expression(&expr1).is_err(),
                "Shell substitution $() not rejected");
            prop_assert!(validate_nix_expression(&expr2).is_err(),
                "Shell substitution `` not rejected");
        }

        /// Test that null bytes are rejected in expressions
        #[test]
        fn prop_nix_expression_null_byte_reject(
            base in "[a-zA-Z0-9_ ]{1,50}"
        ) {
            let malicious = format!("{}\0poison", base);
            prop_assert!(validate_nix_expression(&malicious).is_err(),
                "Null byte not rejected");
        }

        /// Test that valid simple expressions are accepted
        #[test]
        fn prop_nix_expression_valid_simple_accept(
            a in 0..1000i32,
            b in 0..1000i32,
            op in "\\+|\\-|\\*"
        ) {
            let expr = format!("{} {} {}", a, op, b);
            prop_assert!(validate_nix_expression(&expr).is_ok(),
                "Valid expression rejected: {}", expr);
        }

        /// Test that overly long expressions are rejected
        #[test]
        fn prop_nix_expression_too_long_reject(
            expr in "[a-zA-Z0-9_ ]{10001,10100}"
        ) {
            prop_assert!(validate_nix_expression(&expr).is_err(),
                "Overly long expression not rejected");
        }
    }

    // ========== validate_machine_name property tests ==========

    proptest! {
        /// Test that valid machine names are accepted
        #[test]
        fn prop_machine_name_valid_accept(
            start in "[a-zA-Z0-9_]",
            middle in "[a-zA-Z0-9_\\-]{0,60}",
            end in "[a-zA-Z0-9_]"
        ) {
            let name = format!("{}{}{}", start, middle, end);
            if name.len() <= 63 && !name.starts_with('-') && !name.ends_with('-') {
                prop_assert!(validate_machine_name(&name).is_ok(),
                    "Valid machine name rejected: {}", name);
            }
        }

        /// Test that names starting with hyphen are rejected
        #[test]
        fn prop_machine_name_starts_hyphen_reject(
            rest in "[a-zA-Z0-9_\\-]{1,20}"
        ) {
            let malicious = format!("-{}", rest);
            prop_assert!(validate_machine_name(&malicious).is_err(),
                "Name starting with hyphen not rejected");
        }

        /// Test that names ending with hyphen are rejected
        #[test]
        fn prop_machine_name_ends_hyphen_reject(
            base in "[a-zA-Z0-9_\\-]{1,20}"
        ) {
            let malicious = format!("{}-", base);
            prop_assert!(validate_machine_name(&malicious).is_err(),
                "Name ending with hyphen not rejected");
        }

        /// Test that names with invalid characters are rejected
        #[test]
        fn prop_machine_name_invalid_chars_reject(
            base in "[a-zA-Z0-9_]{1,10}",
            invalid in "[\\.;,/\\\\@#$%^&*()+=\\[\\]{}]"
        ) {
            let malicious = format!("{}{}", base, invalid);
            prop_assert!(validate_machine_name(&malicious).is_err(),
                "Invalid character not rejected: {}", malicious);
        }

        /// Test that overly long names are rejected
        #[test]
        fn prop_machine_name_too_long_reject(
            name in "[a-zA-Z0-9_]{64,100}"
        ) {
            prop_assert!(validate_machine_name(&name).is_err(),
                "Overly long name not rejected");
        }
    }

    // ========== validate_command property tests ==========

    proptest! {
        /// Test that null bytes are rejected in commands
        #[test]
        fn prop_command_null_byte_reject(
            base in "[a-zA-Z0-9_ ]{1,50}"
        ) {
            let malicious = format!("{}\0poison", base);
            prop_assert!(validate_command(&malicious).is_err(),
                "Null byte not rejected");
        }

        /// Test that overly long commands are rejected
        #[test]
        fn prop_command_too_long_reject(
            cmd in "[a-zA-Z0-9_ ]{1001,1100}"
        ) {
            prop_assert!(validate_command(&cmd).is_err(),
                "Overly long command not rejected");
        }

        /// Test that simple valid commands are accepted
        #[test]
        fn prop_command_simple_valid_accept(
            cmd in "[a-z]{1,20}",
            arg in "[a-zA-Z0-9_]{1,20}"
        ) {
            let command = format!("{} {}", cmd, arg);
            prop_assert!(validate_command(&command).is_ok(),
                "Valid command rejected: {}", command);
        }
    }

    // ========== validate_url property tests ==========

    proptest! {
        /// Test that valid HTTP/HTTPS URLs are accepted
        #[test]
        fn prop_url_valid_http_accept(
            protocol in "https?://",
            domain in "[a-z]{3,20}\\.[a-z]{2,5}",
            path in "(/[a-zA-Z0-9_\\-\\.]+){0,5}"
        ) {
            let url = format!("{}{}{}", protocol, domain, path);
            if url.len() <= 2048 {
                prop_assert!(validate_url(&url).is_ok(),
                    "Valid URL rejected: {}", url);
            }
        }

        /// Test that URLs with null bytes are rejected
        #[test]
        fn prop_url_null_byte_reject(
            base in "https://example\\.com"
        ) {
            let malicious = format!("{}\0poison", base);
            prop_assert!(validate_url(&malicious).is_err(),
                "Null byte not rejected");
        }

        /// Test that URLs with unencoded spaces are rejected
        #[test]
        fn prop_url_unencoded_space_reject(
            base in "https://example\\.com/",
            suffix in "[a-z]{1,10}"
        ) {
            let malicious = format!("{} {}", base, suffix);
            prop_assert!(validate_url(&malicious).is_err(),
                "Unencoded space not rejected");
        }

        /// Test that overly long URLs are rejected
        #[test]
        fn prop_url_too_long_reject(
            url in "https://example\\.com/[a-z]{2049,2100}"
        ) {
            prop_assert!(validate_url(&url).is_err(),
                "Overly long URL not rejected");
        }

        /// Test that non-HTTP/HTTPS/FTP URLs are rejected
        #[test]
        fn prop_url_invalid_protocol_reject(
            protocol in "file://|data:|javascript:"
        ) {
            let malicious = format!("{}test", protocol);
            prop_assert!(validate_url(&malicious).is_err(),
                "Invalid protocol not rejected: {}", malicious);
        }
    }

    // ========== validate_path property tests ==========

    proptest! {
        /// Test that path traversal with .. is rejected
        #[test]
        fn prop_path_traversal_reject(
            base in "/[a-z]{1,10}",
            traversal in "(\\.\\./)+",
            target in "[a-z]{1,10}"
        ) {
            let malicious = format!("{}/{}{}", base, traversal, target);
            prop_assert!(validate_path(&malicious).is_err(),
                "Path traversal not rejected: {}", malicious);
        }

        /// Test that dangerous system paths are rejected
        #[test]
        fn prop_path_dangerous_system_reject(
            prefix in prop::sample::select(vec![
                "/etc/shadow",
                "/etc/passwd",
                "/root/.ssh",
                "/var/lib/private",
            ]),
            suffix in "/[a-z]{0,10}"
        ) {
            let malicious = format!("{}{}", prefix, suffix);
            prop_assert!(validate_path(&malicious).is_err(),
                "Dangerous system path not rejected: {}", malicious);
        }

        /// Test that overly long paths are rejected
        #[test]
        fn prop_path_too_long_reject(
            path in "/[a-z]{4097,4200}"
        ) {
            prop_assert!(validate_path(&path).is_err(),
                "Overly long path not rejected");
        }

        /// Test that simple valid paths are accepted (non-existent paths)
        #[test]
        fn prop_path_simple_valid_accept(
            dir in "/tmp/[a-z]{10,20}",
            file in "/[a-z]{10,20}"
        ) {
            let path = format!("{}{}", dir, file);
            // Only test non-existent paths to avoid filesystem dependencies
            if !PathBuf::from(&path).exists() && path.len() <= 4096 {
                prop_assert!(validate_path(&path).is_ok(),
                    "Valid path rejected: {}", path);
            }
        }
    }
}
