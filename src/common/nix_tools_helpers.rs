//! Helper functions for nix tools that can be unit tested
//! This separates business logic from MCP handler code

/// Extract flake reference from a package string
/// Examples:
/// - ".#package" -> "."
/// - "github:owner/repo#pkg" -> "github:owner/repo"
/// - "." -> "."
#[allow(dead_code)]
pub fn extract_flake_ref(package: &str) -> String {
    if package == "." || package.starts_with("./") || package.starts_with("/") {
        package.to_string()
    } else if let Some(hash_pos) = package.find('#') {
        package[..hash_pos].to_string()
    } else {
        package.to_string()
    }
}

/// Format error message for missing package with available alternatives
#[allow(dead_code)]
pub fn format_missing_package_error(
    package: &str,
    available_packages: Vec<String>,
    original_error: &str,
) -> String {
    let mut error_msg = format!("Package '{}' not found in flake.\n\n", package);

    if !available_packages.is_empty() {
        error_msg.push_str("Available packages:\n");
        for pkg in available_packages.iter().take(10) {
            error_msg.push_str(&format!("  - {}\n", pkg));
        }
        if available_packages.len() > 10 {
            error_msg.push_str(&format!(
                "  ... and {} more\n",
                available_packages.len() - 10
            ));
        }
        error_msg.push_str("\nTry using one of these package references.");
    } else {
        error_msg
            .push_str("No packages found in flake. Run 'nix flake show' to see available outputs.");
    }

    error_msg.push_str(&format!("\n\nOriginal error:\n{}", original_error));
    error_msg
}

/// Simple URL encoding for NixOS option queries
pub fn encode_option_query(query: &str) -> String {
    query.replace(' ', "%20").replace('.', "%2E")
}

/// Format the search_options response message
pub fn format_option_search_response(query: &str) -> String {
    let encoded_query = encode_option_query(query);
    let search_url = format!(
        "https://search.nixos.org/options?channel=unstable&query={}",
        encoded_query
    );

    format!(
        "NixOS option search for '{}':\n\n\
        Search online:\n\
        - {}\n\
        - https://nixos.org/manual/nixos/stable/options.html\n\n\
        On NixOS systems, you can also use:\n\
        - nixos-option {}\n\
        - man configuration.nix",
        query, search_url, query
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_flake_ref() {
        assert_eq!(extract_flake_ref("."), ".");
        assert_eq!(extract_flake_ref("./foo"), "./foo");
        assert_eq!(extract_flake_ref("/abs/path"), "/abs/path");
        assert_eq!(extract_flake_ref(".#package"), ".");
        assert_eq!(
            extract_flake_ref("github:owner/repo#pkg"),
            "github:owner/repo"
        );
        assert_eq!(extract_flake_ref("nixpkgs#hello"), "nixpkgs");
    }

    #[test]
    fn test_format_missing_package_error_with_packages() {
        let packages = vec![
            ".#package1".to_string(),
            ".#package2".to_string(),
            ".#package3".to_string(),
        ];
        let error = format_missing_package_error(".", packages, "error: attribute not found");

        assert!(error.contains("Package '.' not found"));
        assert!(error.contains("Available packages:"));
        assert!(error.contains("  - .#package1"));
        assert!(error.contains("  - .#package2"));
        assert!(error.contains("  - .#package3"));
        assert!(error.contains("Original error:"));
        assert!(error.contains("error: attribute not found"));
    }

    #[test]
    fn test_format_missing_package_error_no_packages() {
        let packages = vec![];
        let error = format_missing_package_error(".", packages, "error: attribute not found");

        assert!(error.contains("Package '.' not found"));
        assert!(error.contains("No packages found"));
        assert!(!error.contains("Available packages:"));
    }

    #[test]
    fn test_format_missing_package_error_many_packages() {
        let packages: Vec<String> = (1..=15).map(|i| format!(".#package{}", i)).collect();
        let error = format_missing_package_error(".", packages, "error");

        assert!(error.contains("  - .#package1"));
        assert!(error.contains("  - .#package10"));
        assert!(!error.contains("  - .#package11")); // Should be truncated
        assert!(error.contains("... and 5 more"));
    }

    #[test]
    fn test_encode_option_query() {
        assert_eq!(
            encode_option_query("networking.hostName"),
            "networking%2EhostName"
        );
        assert_eq!(
            encode_option_query("services.nginx.enable"),
            "services%2Enginx%2Eenable"
        );
        assert_eq!(encode_option_query("boot loader"), "boot%20loader");
        assert_eq!(encode_option_query("simple"), "simple");
    }

    #[test]
    fn test_format_option_search_response() {
        let response = format_option_search_response("networking.hostName");

        assert!(response.contains("NixOS option search for 'networking.hostName'"));
        assert!(response.contains(
            "https://search.nixos.org/options?channel=unstable&query=networking%2EhostName"
        ));
        assert!(response.contains("nixos-option networking.hostName"));
        assert!(response.contains("man configuration.nix"));
    }
}
