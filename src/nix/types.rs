//! Parameter types for Nix MCP tools.
//!
//! This module defines all parameter types used by the Nix tools. Each type
//! corresponds to a specific tool operation and includes field-level documentation
//! with examples.

use rmcp::schemars;

/// Parameters for getting help about Nix commands.
///
/// Used by [`InfoTools::nix_command_help`](crate::nix::InfoTools::nix_command_help).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixCommandHelpArgs;
///
/// // Get help for the 'nix develop' command
/// let args = NixCommandHelpArgs {
///     command: Some("develop".to_string()),
/// };
///
/// // Get general Nix help
/// let args = NixCommandHelpArgs {
///     command: None,
/// };
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NixCommandHelpArgs {
    /// Specific nix command to get help for (e.g., "develop", "build", "flake")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Parameters for getting information about Nix ecosystem tools.
///
/// Used by [`InfoTools::ecosystem_tools`](crate::nix::InfoTools::ecosystem_tools).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::EcosystemToolArgs;
///
/// // Get info about comma
/// let args = EcosystemToolArgs {
///     tool: Some("comma".to_string()),
/// };
///
/// // List all ecosystem tools
/// let args = EcosystemToolArgs {
///     tool: None,
/// };
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EcosystemToolArgs {
    /// Tool name to get info about (e.g., "comma", "disko", "alejandra"). Leave empty to list all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

/// Parameters for searching packages in nixpkgs.
///
/// Used by [`PackageTools::search_packages`](crate::nix::PackageTools::search_packages).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::SearchPackagesArgs;
///
/// // Search for packages matching "firefox"
/// let args = SearchPackagesArgs {
///     query: "firefox".to_string(),
///     limit: Some(10),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchPackagesArgs {
    /// Search query for package name or description
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Parameters for getting detailed package information.
///
/// Used by [`PackageTools::get_package_info`](crate::nix::PackageTools::get_package_info).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::GetPackageInfoArgs;
///
/// let args = GetPackageInfoArgs {
///     package: "nixpkgs#ripgrep".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetPackageInfoArgs {
    /// Package attribute path (e.g., "nixpkgs#ripgrep")
    pub package: String,
}

/// Parameters for explaining package metadata.
///
/// Used by [`PackageTools::explain_package`](crate::nix::PackageTools::explain_package).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::ExplainPackageArgs;
///
/// let args = ExplainPackageArgs {
///     package: "hello".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplainPackageArgs {
    /// Package attribute path (e.g., "nixpkgs#hello" or "hello")
    pub package: String,
}

/// Parameters for finding which package provides a command.
///
/// Used by [`PackageTools::find_command`](crate::nix::PackageTools::find_command).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::FindCommandArgs;
///
/// // Find which package provides 'gcc'
/// let args = FindCommandArgs {
///     command: "gcc".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindCommandArgs {
    /// Command name to find (e.g., "git", "python3", "gcc")
    pub command: String,
}

/// Parameters for locating files in nixpkgs packages.
///
/// Used by [`PackageTools::nix_locate`](crate::nix::PackageTools::nix_locate).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixLocateArgs;
///
/// // Find which packages contain bin/ip
/// let args = NixLocateArgs {
///     path: "bin/ip".to_string(),
///     limit: Some(20),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixLocateArgs {
    /// Path or pattern to search for (e.g., "bin/ip", "lib/libfoo.so")
    pub path: String,
    /// Show only top N results (default: 20)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Parameters for running commands without installation using comma.
///
/// Used by [`PackageTools::comma`](crate::nix::PackageTools::comma).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::CommaArgs;
///
/// // Run cowsay with an argument
/// let args = CommaArgs {
///     command: "cowsay".to_string(),
///     args: Some(vec!["Hello!".to_string()]),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CommaArgs {
    /// Command to find and run (e.g., "cowsay", "hello", "htop")
    pub command: String,
    /// Arguments to pass to the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Parameters for building Nix packages.
///
/// Used by [`BuildTools::nix_build`](crate::nix::BuildTools::nix_build).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixBuildArgs;
///
/// // Dry-run build to see what would be built
/// let args = NixBuildArgs {
///     package: "nixpkgs#hello".to_string(),
///     dry_run: Some(true),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixBuildArgs {
    /// Package to build (e.g., "nixpkgs#hello", ".#mypackage")
    pub package: String,
    /// Perform a dry-run build to show what would be built
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
}

/// Parameters for understanding dependency relationships.
///
/// Used by [`BuildTools::why_depends`](crate::nix::BuildTools::why_depends).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::WhyDependsArgs;
///
/// // Find why firefox depends on libx11
/// let args = WhyDependsArgs {
///     package: "nixpkgs#firefox".to_string(),
///     dependency: "nixpkgs#libx11".to_string(),
///     show_all: Some(false),
/// };
/// ```
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

/// Parameters for inspecting package derivations.
///
/// Used by [`BuildTools::show_derivation`](crate::nix::BuildTools::show_derivation).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::ShowDerivationArgs;
///
/// let args = ShowDerivationArgs {
///     package: "nixpkgs#hello".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ShowDerivationArgs {
    /// Package to inspect (e.g., "nixpkgs#hello")
    pub package: String,
}

/// Parameters for analyzing package closure sizes.
///
/// Used by [`BuildTools::get_closure_size`](crate::nix::BuildTools::get_closure_size).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::GetClosureSizeArgs;
///
/// // Get closure size in human-readable format
/// let args = GetClosureSizeArgs {
///     package: "nixpkgs#firefox".to_string(),
///     human_readable: Some(true),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetClosureSizeArgs {
    /// Package to analyze (e.g., "nixpkgs#firefox", ".#myapp")
    pub package: String,
    /// Show human-readable sizes (e.g., "1.2 GB" instead of bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_readable: Option<bool>,
}

/// Parameters for retrieving package build logs.
///
/// Used by [`BuildTools::get_build_log`](crate::nix::BuildTools::get_build_log).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::GetBuildLogArgs;
///
/// let args = GetBuildLogArgs {
///     package: "/nix/store/xxx-hello.drv".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetBuildLogArgs {
    /// Package or store path to get build log for (e.g., "nixpkgs#hello", "/nix/store/xxx-hello.drv")
    pub package: String,
}

/// Parameters for comparing two package derivations.
///
/// Used by [`BuildTools::diff_derivations`](crate::nix::BuildTools::diff_derivations).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::DiffDerivationsArgs;
///
/// // Compare firefox and firefox-esr
/// let args = DiffDerivationsArgs {
///     package_a: "nixpkgs#firefox".to_string(),
///     package_b: "nixpkgs#firefox-esr".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DiffDerivationsArgs {
    /// First package to compare (e.g., "nixpkgs#firefox")
    pub package_a: String,
    /// Second package to compare (e.g., "nixpkgs#firefox-esr")
    pub package_b: String,
}

/// Parameters for building NixOS system configurations.
///
/// Used by [`BuildTools::nixos_build`](crate::nix::BuildTools::nixos_build).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixosBuildArgs;
///
/// let args = NixosBuildArgs {
///     machine: "myserver".to_string(),
///     flake: Some(".".to_string()),
///     use_nom: Some(true),
/// };
/// ```
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

/// Parameters for searching NixOS configuration options.
///
/// Used by [`DevelopTools::search_options`](crate::nix::DevelopTools::search_options).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::SearchOptionsArgs;
///
/// let args = SearchOptionsArgs {
///     query: "services.nginx".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchOptionsArgs {
    /// Search query for NixOS options
    pub query: String,
}

/// Parameters for evaluating Nix expressions.
///
/// Used by [`DevelopTools::nix_eval`](crate::nix::DevelopTools::nix_eval).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixEvalArgs;
///
/// let args = NixEvalArgs {
///     expression: "1 + 2".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixEvalArgs {
    /// Nix expression to evaluate
    pub expression: String,
}

/// Parameters for running commands in a Nix shell with packages.
///
/// Used by [`DevelopTools::run_in_shell`](crate::nix::DevelopTools::run_in_shell).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::RunInShellArgs;
///
/// // Run python with numpy available
/// let args = RunInShellArgs {
///     packages: vec!["python3".to_string(), "python3Packages.numpy".to_string()],
///     command: "python -c 'import numpy; print(numpy.__version__)'".to_string(),
///     use_flake: Some(false),
/// };
/// ```
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

/// Parameters for retrieving Nix build logs from store paths.
///
/// Used by [`DevelopTools::nix_log`](crate::nix::DevelopTools::nix_log).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixLogArgs;
///
/// // Get logs filtered by pattern
/// let args = NixLogArgs {
///     store_path: "/nix/store/xxx-hello-1.0.drv".to_string(),
///     grep_pattern: Some("error".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixLogArgs {
    /// Nix store path to get logs for (e.g., "/nix/store/xxx-hello-1.0.drv")
    pub store_path: String,
    /// Optional grep pattern to filter log output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grep_pattern: Option<String>,
}

/// Parameters for running packages without installation.
///
/// Used by [`DevelopTools::nix_run`](crate::nix::DevelopTools::nix_run).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixRunArgs;
///
/// // Run cowsay with an argument
/// let args = NixRunArgs {
///     package: "nixpkgs#cowsay".to_string(),
///     args: Some(vec!["Hello!".to_string()]),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixRunArgs {
    /// Package to run (e.g., "nixpkgs#hello", "nixpkgs#cowsay")
    pub package: String,
    /// Arguments to pass to the program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Parameters for running commands in a Nix development environment.
///
/// Used by [`DevelopTools::nix_develop`](crate::nix::DevelopTools::nix_develop).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixDevelopArgs;
///
/// // Run cargo build in the project's dev shell
/// let args = NixDevelopArgs {
///     flake_ref: Some(".".to_string()),
///     command: "cargo".to_string(),
///     args: Some(vec!["build".to_string()]),
/// };
/// ```
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

/// Parameters for getting flake metadata.
///
/// Used by [`FlakeTools::flake_metadata`](crate::nix::FlakeTools::flake_metadata).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::FlakeMetadataArgs;
///
/// let args = FlakeMetadataArgs {
///     flake_ref: "github:nixos/nixpkgs".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FlakeMetadataArgs {
    /// Flake reference (e.g., ".", "github:owner/repo", "nixpkgs")
    pub flake_ref: String,
}

/// Parameters for showing flake outputs.
///
/// Used by [`FlakeTools::flake_show`](crate::nix::FlakeTools::flake_show).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::FlakeShowArgs;
///
/// // Show outputs of current flake
/// let args = FlakeShowArgs {
///     flake_ref: None,
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FlakeShowArgs {
    /// Flake reference to inspect (e.g., ".", "github:owner/repo")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake_ref: Option<String>,
}

/// Parameters for prefetching URLs and computing hashes.
///
/// Used by [`FlakeTools::prefetch_url`](crate::nix::FlakeTools::prefetch_url).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::PrefetchUrlArgs;
///
/// let args = PrefetchUrlArgs {
///     url: "https://example.com/file.tar.gz".to_string(),
///     hash_format: Some("sri".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PrefetchUrlArgs {
    /// URL to prefetch
    pub url: String,
    /// Expected hash format: "sha256" or "sri" (default: "sri")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_format: Option<String>,
}

/// Parameters for formatting Nix code with nixpkgs-fmt.
///
/// Used by [`QualityTools::format_nix`](crate::nix::QualityTools::format_nix).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::FormatNixArgs;
///
/// let args = FormatNixArgs {
///     code: "{ pkgs }: pkgs.hello".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatNixArgs {
    /// Nix code to format
    pub code: String,
}

/// Parameters for validating Nix code syntax.
///
/// Used by [`QualityTools::validate_nix`](crate::nix::QualityTools::validate_nix).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::ValidateNixArgs;
///
/// let args = ValidateNixArgs {
///     code: "{ pkgs }: pkgs.hello".to_string(),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateNixArgs {
    /// Nix code to validate
    pub code: String,
}

/// Parameters for linting Nix code with statix and deadnix.
///
/// Used by [`QualityTools::lint_nix`](crate::nix::QualityTools::lint_nix).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::LintNixArgs;
///
/// // Run both linters
/// let args = LintNixArgs {
///     code: "{ pkgs }: pkgs.hello".to_string(),
///     linter: Some("both".to_string()),
/// };
///
/// // Run only statix
/// let args = LintNixArgs {
///     code: "{ pkgs }: pkgs.hello".to_string(),
///     linter: Some("statix".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LintNixArgs {
    /// Nix code to lint
    pub code: String,
    /// Which linters to run: "statix", "deadnix", or "both" (default: "both")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<String>,
}

/// Parameters for formatting files/directories with nix fmt.
///
/// Used by [`QualityTools::nix_fmt`](crate::nix::QualityTools::nix_fmt).
///
/// # Examples
///
/// ```
/// use onix_mcp::nix::types::NixFmtArgs;
///
/// // Format current directory
/// let args = NixFmtArgs {
///     path: None,
/// };
///
/// // Format specific file
/// let args = NixFmtArgs {
///     path: Some("flake.nix".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NixFmtArgs {
    /// Path to format (file or directory, defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
