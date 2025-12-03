/// Argument types for Nix informational tools
use rmcp::schemars;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NixCommandHelpArgs {
    /// Specific nix command to get help for (e.g., "develop", "build", "flake")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EcosystemToolArgs {
    /// Tool name to get info about (e.g., "comma", "disko", "alejandra"). Leave empty to list all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchPackagesArgs {
    /// Search query for package name or description
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetPackageInfoArgs {
    /// Package attribute path (e.g., "nixpkgs#ripgrep")
    pub package: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplainPackageArgs {
    /// Package attribute path (e.g., "nixpkgs#hello" or "hello")
    pub package: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindCommandArgs {
    /// Command name to find (e.g., "git", "python3", "gcc")
    pub command: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixLocateArgs {
    /// Path or pattern to search for (e.g., "bin/ip", "lib/libfoo.so")
    pub path: String,
    /// Show only top N results (default: 20)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CommaArgs {
    /// Command to find and run (e.g., "cowsay", "hello", "htop")
    pub command: String,
    /// Arguments to pass to the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixBuildArgs {
    /// Package to build (e.g., "nixpkgs#hello", ".#mypackage")
    pub package: String,
    /// Perform a dry-run build to show what would be built
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WhyDependsArgs {
    /// Package that has the dependency (e.g., "nixpkgs#firefox", ".#result")
    pub package: String,
    /// Dependency to explain (e.g., "nixpkgs#libx11")
    pub dependency: String,
    /// Show all dependency paths, not just the shortest one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_all: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ShowDerivationArgs {
    /// Package to inspect (e.g., "nixpkgs#hello")
    pub package: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetClosureSizeArgs {
    /// Package to analyze (e.g., "nixpkgs#firefox", ".#myapp")
    pub package: String,
    /// Show human-readable sizes (e.g., "1.2 GB" instead of bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_readable: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetBuildLogArgs {
    /// Package or store path to get build log for (e.g., "nixpkgs#hello", "/nix/store/xxx-hello.drv")
    pub package: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DiffDerivationsArgs {
    /// First package to compare (e.g., "nixpkgs#firefox")
    pub package_a: String,
    /// Second package to compare (e.g., "nixpkgs#firefox-esr")
    pub package_b: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixosBuildArgs {
    /// Machine configuration name to build
    pub machine: String,
    /// Optional flake reference (defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
    /// Use nom for better build output (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_nom: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchOptionsArgs {
    /// Search query for NixOS options
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixEvalArgs {
    /// Nix expression to evaluate
    pub expression: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunInShellArgs {
    /// Packages to include in the shell (e.g., ["python3", "nodejs"])
    pub packages: Vec<String>,
    /// Command to run in the shell
    pub command: String,
    /// Use nix develop instead of nix-shell (requires flake.nix)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_flake: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixLogArgs {
    /// Nix store path to get logs for (e.g., "/nix/store/xxx-hello-1.0.drv")
    pub store_path: String,
    /// Optional grep pattern to filter log output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grep_pattern: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixRunArgs {
    /// Package to run (e.g., "nixpkgs#hello", "nixpkgs#cowsay")
    pub package: String,
    /// Arguments to pass to the program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixDevelopArgs {
    /// Flake reference for development shell (e.g., ".", "github:owner/repo")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake_ref: Option<String>,
    /// Command to run in the development environment
    pub command: String,
    /// Additional arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FlakeMetadataArgs {
    /// Flake reference (e.g., ".", "github:owner/repo", "nixpkgs")
    pub flake_ref: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FlakeShowArgs {
    /// Flake reference to inspect (e.g., ".", "github:owner/repo")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake_ref: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PrefetchUrlArgs {
    /// URL to prefetch
    pub url: String,
    /// Expected hash format: "sha256" or "sri" (default: "sri")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_format: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatNixArgs {
    /// Nix code to format
    pub code: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateNixArgs {
    /// Nix code to validate
    pub code: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LintNixArgs {
    /// Nix code to lint
    pub code: String,
    /// Which linters to run: "statix", "deadnix", or "both" (default: "both")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixFmtArgs {
    /// Path to format (file or directory, defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
