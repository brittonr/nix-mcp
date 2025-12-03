//! Nix package management and development tools module.
//!
//! This module provides MCP tools for interacting with the Nix ecosystem, organized
//! into specialized submodules for different aspects of Nix operations.
//!
//! # Module Organization
//!
//! - [`packages`] - Package discovery, search, and information retrieval
//! - [`build`] - Building packages, analyzing dependencies, and understanding derivations
//! - [`develop`] - Development environments, nix-shell, and nix develop operations
//! - [`flakes`] - Flake metadata, prefetching, and flake-specific operations
//! - [`quality`] - Code quality tools (formatting, linting, validation)
//! - [`info`] - General Nix information and help (commands, ecosystem tools)
//!
//! # Caching Strategy
//!
//! All tools use TTL-based caching to balance freshness with performance:
//! - Package searches: 10-minute TTL
//! - Package info: 30-minute TTL
//! - File location: 5-minute TTL
//! - Nix evaluation: 5-minute TTL
//! - URL prefetch: 24-hour TTL
//! - Closure sizes: 30-minute TTL
//! - Derivations: 30-minute TTL
//!
//! # Security
//!
//! All tools implement comprehensive input validation:
//! - Package names validated against injection attacks
//! - Flake references checked for shell metacharacters
//! - Nix expressions scanned for dangerous patterns
//! - All operations use audit logging for security tracking
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::nix::{PackageTools, SearchPackagesArgs};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create package tools with caching
//! let audit = Arc::new(/* audit logger */);
//! let caches = Arc::new(/* cache registry */);
//! let tools = PackageTools::new(audit, caches);
//!
//! // Search for packages
//! // let result = tools.search_packages(Parameters(SearchPackagesArgs {
//! //     query: "ripgrep".to_string(),
//! //     limit: Some(10),
//! // })).await?;
//! # Ok(())
//! # }
//! ```

pub mod build;
pub mod develop;
pub mod flakes;
pub mod info;
pub mod packages;
pub mod quality;
pub mod types;

pub use build::BuildTools;
pub use develop::DevelopTools;
pub use flakes::FlakeTools;
pub use info::InfoTools;
pub use packages::PackageTools;
pub use quality::QualityTools;
pub use types::{
    CommaArgs, DiffDerivationsArgs, EcosystemToolArgs, ExplainPackageArgs, FindCommandArgs,
    FlakeMetadataArgs, FlakeShowArgs, FormatNixArgs, GetBuildLogArgs, GetClosureSizeArgs,
    GetPackageInfoArgs, LintNixArgs, NixBuildArgs, NixCommandHelpArgs, NixDevelopArgs, NixEvalArgs,
    NixFmtArgs, NixLocateArgs, NixLogArgs, NixRunArgs, NixosBuildArgs, PrefetchUrlArgs,
    RunInShellArgs, SearchOptionsArgs, SearchPackagesArgs, ShowDerivationArgs, ValidateNixArgs,
    WhyDependsArgs,
};
