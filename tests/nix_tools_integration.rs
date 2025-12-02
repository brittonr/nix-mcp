/// Integration tests for Nix MCP tools
/// These tests interact with real nix commands and only run in development (not in nix build sandbox)
use tokio;

#[tokio::test]
async fn test_url_encoding_for_options() {
    // Test that URL encoding works correctly for search_options
    let test_cases = vec![
        ("networking.hostName", "networking%2EhostName"),
        ("services.nginx.enable", "services%2Enginx%2Eenable"),
        ("boot loader", "boot%20loader"),
    ];

    for (input, expected) in test_cases {
        let encoded = input.replace(' ', "%20").replace('.', "%2E");
        assert_eq!(encoded, expected, "Failed for input: {}", input);
    }
}

// Integration tests that require nix commands
// These are ignored in nix build (sandboxed) but run in nix develop

#[tokio::test]
#[cfg_attr(not(debug_assertions), ignore = "requires nix commands, skip in release/sandbox builds")]
async fn test_nix_build_dry_run() {
    // Test that nix build --dry-run works with a valid package
    // This is fast and verifies our nix environment is working
    let output = tokio::process::Command::new("nix")
        .args(["build", "nixpkgs#hello", "--dry-run"])
        .output()
        .await;

    assert!(output.is_ok(), "nix build command should execute");
    let out = output.unwrap();
    assert!(
        out.status.success(),
        "nix build --dry-run should succeed. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[tokio::test]
#[cfg_attr(not(debug_assertions), ignore = "requires nix commands, skip in release/sandbox builds")]
async fn test_nix_eval_simple_expression() {
    // Test that nix eval works for basic expressions
    let output = tokio::process::Command::new("nix")
        .args(["eval", "--expr", "1 + 1"])
        .output()
        .await;

    assert!(output.is_ok(), "nix eval should execute");
    let out = output.unwrap();
    assert!(out.status.success(), "nix eval should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains('2'), "nix eval 1+1 should return 2");
}

#[tokio::test]
#[cfg_attr(not(debug_assertions), ignore = "requires nix commands, skip in release/sandbox builds")]
async fn test_pueue_available_via_nix_shell() {
    // Verify we can run pueue via nix shell (even if daemon isn't running)
    let output = tokio::process::Command::new("nix")
        .args(["shell", "nixpkgs#pueue", "-c", "pueue", "--version"])
        .output()
        .await;

    assert!(output.is_ok(), "Should be able to run pueue via nix shell");
    let out = output.unwrap();
    assert!(
        out.status.success(),
        "pueue --version should work. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
