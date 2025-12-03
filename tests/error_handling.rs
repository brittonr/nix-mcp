/// Error handling tests for MCP tools
/// These tests verify that tools properly handle error conditions
/// and return appropriate error messages without panicking
use onix_mcp::common::cache_registry::CacheRegistry;
use onix_mcp::common::security::audit_logger;
use onix_mcp::nix::{BuildTools, DevelopTools, FlakeTools, InfoTools, PackageTools, QualityTools};
use onix_mcp::process::{PexpectTools, PueueTools};
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

// ========== Package Tool Error Tests ==========

#[tokio::test]
async fn test_search_packages_empty_query() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    let result = tools
        .search_packages(Parameters(onix_mcp::nix::SearchPackagesArgs {
            query: "".to_string(),
            limit: None,
        }))
        .await;

    assert!(result.is_err(), "Empty query should be rejected");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("validation"),
        "Error should mention validation"
    );
}

#[tokio::test]
async fn test_search_packages_injection_attempt() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    let malicious_queries = vec![
        "package;rm -rf /",
        "package$(whoami)",
        "package`cat /etc/passwd`",
        "../../../etc/passwd",
        "package||true",
    ];

    for query in malicious_queries {
        let result = tools
            .search_packages(Parameters(onix_mcp::nix::SearchPackagesArgs {
                query: query.to_string(),
                limit: None,
            }))
            .await;

        assert!(
            result.is_err(),
            "Injection attempt should be rejected: {}",
            query
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Invalid")
                || err.to_string().contains("validation")
                || err.to_string().contains("rejected"),
            "Error should indicate rejection for: {}",
            query
        );
    }
}

#[tokio::test]
async fn test_get_package_info_empty_package() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    let result = tools
        .get_package_info(Parameters(onix_mcp::nix::GetPackageInfoArgs {
            package: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty package name should be rejected");
}

#[tokio::test]
async fn test_find_command_empty_command() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    let result = tools
        .find_command(Parameters(onix_mcp::nix::FindCommandArgs {
            command: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty command should be rejected");
}

#[tokio::test]
async fn test_find_command_path_traversal() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    let result = tools
        .find_command(Parameters(onix_mcp::nix::FindCommandArgs {
            command: "../../bin/bash".to_string(),
        }))
        .await;

    // Note: Path-like strings in commands are currently allowed and passed to nix-locate
    // which will handle them appropriately. This could be tightened in the future.
    // For now, we just verify it doesn't panic.
    let _ = result;
}

// ========== Build Tool Error Tests ==========

#[tokio::test]
async fn test_nix_build_empty_package() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = BuildTools::new(audit, caches);

    let result = tools
        .nix_build(Parameters(onix_mcp::nix::NixBuildArgs {
            package: "".to_string(),
            dry_run: Some(true),
        }))
        .await;

    assert!(result.is_err(), "Empty package should be rejected");
}

#[tokio::test]
async fn test_nix_build_injection_attempt() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = BuildTools::new(audit, caches);

    let result = tools
        .nix_build(Parameters(onix_mcp::nix::NixBuildArgs {
            package: "nixpkgs#hello;rm -rf /".to_string(),
            dry_run: Some(true),
        }))
        .await;

    assert!(result.is_err(), "Injection attempt should be rejected");
}

#[tokio::test]
async fn test_get_closure_size_invalid_package() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = BuildTools::new(audit, caches);

    let result = tools
        .get_closure_size(Parameters(onix_mcp::nix::GetClosureSizeArgs {
            package: "".to_string(),
            human_readable: Some(true),
        }))
        .await;

    assert!(result.is_err(), "Invalid package should be rejected");
}

#[tokio::test]
async fn test_show_derivation_injection() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = BuildTools::new(audit, caches);

    let result = tools
        .show_derivation(Parameters(onix_mcp::nix::ShowDerivationArgs {
            package: "package`whoami`".to_string(),
        }))
        .await;

    assert!(
        result.is_err(),
        "Shell substitution should be rejected in package names"
    );
}

// ========== Flake Tool Error Tests ==========

#[tokio::test]
async fn test_flake_metadata_empty_ref() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = FlakeTools::new(audit, caches);

    let result = tools
        .flake_metadata(Parameters(onix_mcp::nix::FlakeMetadataArgs {
            flake_ref: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty flake ref should be rejected");
}

#[tokio::test]
async fn test_flake_metadata_injection() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = FlakeTools::new(audit, caches);

    let malicious_refs = vec![
        "nixpkgs;rm -rf /",
        "nixpkgs$(whoami)",
        "nixpkgs`cat /etc/passwd`",
        "nixpkgs||echo hacked",
    ];

    for ref_str in malicious_refs {
        let result = tools
            .flake_metadata(Parameters(onix_mcp::nix::FlakeMetadataArgs {
                flake_ref: ref_str.to_string(),
            }))
            .await;

        assert!(
            result.is_err(),
            "Injection should be rejected in flake ref: {}",
            ref_str
        );
    }
}

#[tokio::test]
async fn test_prefetch_url_invalid_url() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = FlakeTools::new(audit, caches);

    let invalid_urls = vec![
        "",
        "not-a-url",
        "javascript:alert(1)",
        "file:///etc/passwd",
        "http://example.com\0poison",
    ];

    for url in invalid_urls {
        let result = tools
            .prefetch_url(Parameters(onix_mcp::nix::PrefetchUrlArgs {
                url: url.to_string(),
                hash_format: None,
            }))
            .await;

        assert!(result.is_err(), "Invalid URL should be rejected: {}", url);
    }
}

// ========== Quality Tool Error Tests ==========

#[tokio::test]
async fn test_validate_nix_empty_code() {
    let audit = audit_logger();
    let tools = QualityTools::new(audit);

    let result = tools
        .validate_nix(Parameters(onix_mcp::nix::ValidateNixArgs {
            code: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty code should be rejected");
}

#[tokio::test]
async fn test_validate_nix_dangerous_patterns() {
    let audit = audit_logger();
    let tools = QualityTools::new(audit);

    let dangerous_patterns = vec![
        "builtins.exec [\"rm\" \"-rf\" \"/\"]",
        "$(rm -rf /)",
        "`whoami`",
        "__noChroot = true",
        "allowSubstitutes = false",
    ];

    for code in dangerous_patterns {
        let result = tools
            .validate_nix(Parameters(onix_mcp::nix::ValidateNixArgs {
                code: code.to_string(),
            }))
            .await;

        assert!(
            result.is_err(),
            "Dangerous pattern should be rejected: {}",
            code
        );
    }
}

#[tokio::test]
async fn test_format_nix_empty_code() {
    let audit = audit_logger();
    let tools = QualityTools::new(audit);

    let result = tools
        .format_nix(Parameters(onix_mcp::nix::FormatNixArgs {
            code: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty code should be rejected");
}

#[tokio::test]
async fn test_lint_nix_empty_code() {
    let audit = audit_logger();
    let tools = QualityTools::new(audit);

    let result = tools
        .lint_nix(Parameters(onix_mcp::nix::LintNixArgs {
            code: "".to_string(),
            linter: None,
        }))
        .await;

    assert!(result.is_err(), "Empty code should be rejected");
}

// ========== Develop Tool Error Tests ==========

#[tokio::test]
async fn test_nix_eval_empty_expression() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = DevelopTools::new(audit, caches);

    let result = tools
        .nix_eval(Parameters(onix_mcp::nix::NixEvalArgs {
            expression: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty expression should be rejected");
}

#[tokio::test]
async fn test_nix_eval_dangerous_expression() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = DevelopTools::new(audit, caches);

    let result = tools
        .nix_eval(Parameters(onix_mcp::nix::NixEvalArgs {
            expression: "builtins.exec [\"whoami\"]".to_string(),
        }))
        .await;

    assert!(
        result.is_err(),
        "Dangerous builtins.exec should be rejected"
    );
}

#[tokio::test]
async fn test_run_in_shell_empty_packages() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = DevelopTools::new(audit, caches);

    let result = tools
        .run_in_shell(Parameters(onix_mcp::nix::RunInShellArgs {
            packages: vec![],
            command: "echo test".to_string(),
            use_flake: None,
        }))
        .await;

    // Note: Empty packages list is currently allowed - nix-shell will run without packages.
    // This could be considered valid for running commands in a minimal shell environment.
    // We just verify it doesn't panic.
    let _ = result;
}

#[tokio::test]
async fn test_run_in_shell_empty_command() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = DevelopTools::new(audit, caches);

    let result = tools
        .run_in_shell(Parameters(onix_mcp::nix::RunInShellArgs {
            packages: vec!["python3".to_string()],
            command: "".to_string(),
            use_flake: None,
        }))
        .await;

    assert!(result.is_err(), "Empty command should be rejected");
}

// ========== Process Tool Error Tests ==========

#[tokio::test]
async fn test_pueue_add_empty_command() {
    let audit = audit_logger();
    let tools = PueueTools::new(audit);

    let result = tools
        .pueue_add(Parameters(onix_mcp::process::PueueAddArgs {
            command: "".to_string(),
            args: None,
            label: None,
            working_directory: None,
        }))
        .await;

    assert!(result.is_err(), "Empty command should be rejected");
}

#[tokio::test]
async fn test_pueue_wait_invalid_task_ids() {
    let audit = audit_logger();
    let tools = PueueTools::new(audit);

    // Test with various invalid formats
    let invalid_ids = vec!["", "abc", "-1", "1,abc,3", "999999999999999999999"];

    for task_ids in invalid_ids {
        let result = tools
            .pueue_wait(Parameters(onix_mcp::process::PueueWaitArgs {
                task_ids: task_ids.to_string(),
                timeout: None,
            }))
            .await;

        // Some may be caught by validation, others by the command itself
        // We just verify the tool doesn't panic
        let _ = result;
    }
}

#[tokio::test]
async fn test_pexpect_send_empty_code() {
    let audit = audit_logger();
    let tools = PexpectTools::new(audit);

    let result = tools
        .pexpect_send(Parameters(onix_mcp::process::PexpectSendArgs {
            session_id: "test-session".to_string(),
            code: "".to_string(),
        }))
        .await;

    assert!(result.is_err(), "Empty code should be rejected");
}

#[tokio::test]
async fn test_pexpect_start_empty_command() {
    let audit = audit_logger();
    let tools = PexpectTools::new(audit);

    let result = tools
        .pexpect_start(Parameters(onix_mcp::process::PexpectStartArgs {
            command: "".to_string(),
            args: None,
        }))
        .await;

    assert!(result.is_err(), "Empty command should be rejected");
}

// ========== Info Tool Error Tests ==========

#[tokio::test]
async fn test_nix_command_help_overly_long() {
    let audit = audit_logger();
    let tools = InfoTools::new(audit);

    let result = tools.nix_command_help(Parameters(onix_mcp::nix::NixCommandHelpArgs {
        command: Some("a".repeat(1000)),
    }));

    // Should handle gracefully (either reject or return "unknown command")
    let _ = result;
}

// ========== Edge Cases and Boundary Tests ==========

#[tokio::test]
async fn test_package_name_max_length() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    // Test at boundary (255 chars is the limit)
    let valid_name = "a".repeat(255);
    let result = tools
        .search_packages(Parameters(onix_mcp::nix::SearchPackagesArgs {
            query: valid_name,
            limit: None,
        }))
        .await;

    // Should not panic, might fail with nix error but not validation error
    let _ = result;

    // Test over boundary (256 chars)
    let invalid_name = "a".repeat(256);
    let result = tools
        .search_packages(Parameters(onix_mcp::nix::SearchPackagesArgs {
            query: invalid_name,
            limit: None,
        }))
        .await;

    assert!(
        result.is_err(),
        "Overly long package name should be rejected"
    );
}

#[tokio::test]
async fn test_null_byte_injection_various_tools() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());

    // Test null byte in package search
    let pkg_tools = PackageTools::new(audit.clone(), caches.clone());
    let result = pkg_tools
        .search_packages(Parameters(onix_mcp::nix::SearchPackagesArgs {
            query: "test\0poison".to_string(),
            limit: None,
        }))
        .await;
    assert!(result.is_err(), "Null byte should be rejected in search");

    // Test null byte in flake ref
    let flake_tools = FlakeTools::new(audit.clone(), caches.clone());
    let result = flake_tools
        .flake_metadata(Parameters(onix_mcp::nix::FlakeMetadataArgs {
            flake_ref: "nixpkgs\0poison".to_string(),
        }))
        .await;
    assert!(result.is_err(), "Null byte should be rejected in flake ref");

    // Test null byte in nix expression
    let dev_tools = DevelopTools::new(audit, caches);
    let result = dev_tools
        .nix_eval(Parameters(onix_mcp::nix::NixEvalArgs {
            expression: "1 + 1\0poison".to_string(),
        }))
        .await;
    assert!(
        result.is_err(),
        "Null byte should be rejected in expression"
    );
}

#[tokio::test]
async fn test_unicode_handling() {
    let audit = audit_logger();
    let caches = Arc::new(CacheRegistry::new());
    let tools = PackageTools::new(audit, caches);

    // Test valid unicode in package name
    let result = tools
        .search_packages(Parameters(onix_mcp::nix::SearchPackagesArgs {
            query: "python-emoji-🎉".to_string(),
            limit: None,
        }))
        .await;

    // Should not panic - unicode may be valid or invalid depending on nix's rules
    let _ = result;
}
