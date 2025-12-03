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
