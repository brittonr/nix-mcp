use crate::common::cache::TtlCache;
use crate::common::security::{
    audit_logger, validate_command, validate_flake_ref, validate_package_name,
    validation_error_to_mcp, AuditLogger,
};
use rmcp::{
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router, schemars,
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

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
pub struct FormatNixArgs {
    /// Nix code to format
    pub code: String,
}

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
pub struct ExplainPackageArgs {
    /// Package attribute path (e.g., "nixpkgs#hello" or "hello")
    pub package: String,
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
pub struct FlakeMetadataArgs {
    /// Flake reference (e.g., ".", "github:owner/repo", "nixpkgs")
    pub flake_ref: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindCommandArgs {
    /// Command name to find (e.g., "git", "python3", "gcc")
    pub command: String,
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
pub struct FlakeShowArgs {
    /// Flake reference to inspect (e.g., ".", "github:owner/repo")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake_ref: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetBuildLogArgs {
    /// Package or store path to get build log for (e.g., "nixpkgs#hello", "/nix/store/xxx-hello.drv")
    pub package: String,
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
pub struct DiffDerivationsArgs {
    /// First package to compare (e.g., "nixpkgs#firefox")
    pub package_a: String,
    /// Second package to compare (e.g., "nixpkgs#firefox-esr")
    pub package_b: String,
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
pub struct NixFmtArgs {
    /// Path to format (file or directory, defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

// Pueue process management argument types
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueAddArgs {
    /// Command to run
    pub command: String,
    /// Arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Working directory for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Label for the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueStatusArgs {
    /// Show only specific task IDs (comma-separated, e.g., "1,2,3")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueLogArgs {
    /// Task ID to get logs for
    pub task_id: u32,
    /// Number of lines to show from the end (like tail -n)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueWaitArgs {
    /// Task IDs to wait for (comma-separated, e.g., "1,2,3")
    pub task_ids: String,
    /// Timeout in seconds (default: 300)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueRemoveArgs {
    /// Task IDs to remove (comma-separated, e.g., "1,2,3")
    pub task_ids: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueuePauseArgs {
    /// Task IDs to pause (comma-separated, e.g., "1,2,3"). Leave empty to pause all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PueueStartArgs {
    /// Task IDs to start/resume (comma-separated, e.g., "1,2,3"). Leave empty to start all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_ids: Option<String>,
}

// Pexpect-cli interactive terminal automation argument types
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectStartArgs {
    /// Command to run interactively (e.g., "bash", "python", "ssh user@host")
    pub command: String,
    /// Arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectSendArgs {
    /// Session ID from pexpect_start
    pub session_id: String,
    /// Python pexpect code to execute (e.g., "child.sendline('ls'); child.expect('$'); print(child.before.decode())")
    pub code: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PexpectCloseArgs {
    /// Session ID to close
    pub session_id: String,
}

// Build and test runner argument types with timeout protection
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoBuildArgs {
    /// Build in release mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<bool>,
    /// Specific package to build (e.g., "mcp-basic-server")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Additional cargo arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
    /// Timeout in seconds (default: 600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoTestArgs {
    /// Test name pattern to filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_pattern: Option<String>,
    /// Specific package to test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Additional cargo test arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
    /// Timeout in seconds (default: 600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoNextestArgs {
    /// Test name pattern to filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_pattern: Option<String>,
    /// Specific package to test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Additional nextest arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
    /// Timeout in seconds (default: 600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoClippyArgs {
    /// Run on all targets (lib, bins, tests, benches, examples)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_targets: Option<bool>,
    /// Specific package to lint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Additional clippy arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
    /// Timeout in seconds (default: 300)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MakeBuildArgs {
    /// Target to build (e.g., "all", "install", "clean")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Working directory containing Makefile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Number of parallel jobs (e.g., 4 for -j4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<u32>,
    /// Additional make arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
    /// Timeout in seconds (default: 600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

// Code quality tool argument types
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ShellcheckArgs {
    /// Path to shell script file or directory
    pub path: String,
    /// Shell dialect: sh, bash, dash, ksh (default: auto-detect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Severity level: error, warning, info, style (default: style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RuffCheckArgs {
    /// Path to Python file or directory
    pub path: String,
    /// Fix issues automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<bool>,
    /// Additional ruff arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RuffFormatArgs {
    /// Path to Python file or directory
    pub path: String,
    /// Check if files are formatted without making changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MypyArgs {
    /// Path to Python file or directory
    pub path: String,
    /// Additional mypy arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoAuditArgs {
    /// Path to Cargo.toml directory (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Deny warnings (exit with error on warnings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_warnings: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CargoDenyArgs {
    /// Path to project directory (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Check type: advisories, bans, licenses, sources (default: all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_type: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaploArgs {
    /// Path to TOML file or directory
    pub path: String,
    /// Check formatting without making changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<bool>,
}

// Clan-specific argument types
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineCreateArgs {
    /// Name of the machine to create
    pub name: String,
    /// Optional template to use (default: "new-machine")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Optional target host address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_host: Option<String>,
    /// Optional flake directory path (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineListArgs {
    /// Optional flake directory path (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineUpdateArgs {
    /// Machines to update (empty for all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machines: Option<Vec<String>>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineDeleteArgs {
    /// Name of the machine to delete
    pub name: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineInstallArgs {
    /// Name of the machine to install
    pub machine: String,
    /// Target SSH host to install to
    pub target_host: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
    /// Confirm destructive operations (overwrites disk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanBackupCreateArgs {
    /// Machine name to backup
    pub machine: String,
    /// Optional backup provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanBackupListArgs {
    /// Machine name to list backups for
    pub machine: String,
    /// Optional backup provider to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanBackupRestoreArgs {
    /// Machine name to restore backup to
    pub machine: String,
    /// Backup provider
    pub provider: String,
    /// Backup name/identifier
    pub name: String,
    /// Optional service to restore (restore all if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanFlakeCreateArgs {
    /// Directory to create the Clan flake in
    pub directory: String,
    /// Optional template to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanSecretsListArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanVmCreateArgs {
    /// Machine name to create VM for
    pub machine: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

// Prompt argument types
#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SetupDevEnvironmentArgs {
    /// Project type (e.g., "rust", "python", "nodejs", "go", "c", "generic")
    pub project_type: String,
    /// Additional tools or dependencies needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    /// Whether to use flakes (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_flakes: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TroubleshootBuildArgs {
    /// The package or flake reference that's failing to build
    pub package: String,
    /// Error message or build log excerpt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MigrateToFlakesArgs {
    /// Current setup description (e.g., "using nix-shell", "using configuration.nix")
    pub current_setup: String,
    /// Project type if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OptimizeClosureArgs {
    /// Package to optimize
    pub package: String,
    /// Current closure size if known (in bytes or human-readable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_size: Option<String>,
    /// Target size or reduction goal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Clone)]
pub struct NixServer {
    tool_router: ToolRouter<NixServer>,
    prompt_router: PromptRouter<NixServer>,
    audit: Arc<AuditLogger>,
    // Cache for expensive nix-locate queries (TTL: 5 minutes)
    locate_cache: Arc<TtlCache<String, String>>,
    // Cache for package search results (TTL: 10 minutes)
    search_cache: Arc<TtlCache<String, String>>,
    // Cache for package info (TTL: 30 minutes, packages don't change often)
    package_info_cache: Arc<TtlCache<String, String>>,
    // Cache for nix eval results (TTL: 5 minutes)
    eval_cache: Arc<TtlCache<String, String>>,
    // Cache for URL prefetch results (TTL: 24 hours, URLs are immutable)
    prefetch_cache: Arc<TtlCache<String, String>>,
    // Cache for closure size calculations (TTL: 30 minutes)
    closure_size_cache: Arc<TtlCache<String, String>>,
    // Cache for derivation info (TTL: 30 minutes, derivations are immutable)
    derivation_cache: Arc<TtlCache<String, String>>,
}

#[tool_router]
impl NixServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            audit: audit_logger(),
            locate_cache: Arc::new(TtlCache::new(Duration::from_secs(300))), // 5 min TTL
            search_cache: Arc::new(TtlCache::new(Duration::from_secs(600))), // 10 min TTL
            package_info_cache: Arc::new(TtlCache::new(Duration::from_secs(1800))), // 30 min TTL
            eval_cache: Arc::new(TtlCache::new(Duration::from_secs(300))),   // 5 min TTL
            prefetch_cache: Arc::new(TtlCache::new(Duration::from_secs(86400))), // 24 hour TTL
            closure_size_cache: Arc::new(TtlCache::new(Duration::from_secs(1800))), // 30 min TTL
            derivation_cache: Arc::new(TtlCache::new(Duration::from_secs(1800))), // 30 min TTL
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    #[tool(
        description = "Search for packages in nixpkgs by name or description",
        annotations(read_only_hint = true)
    )]
    async fn search_packages(
        &self,
        Parameters(SearchPackagesArgs { query, limit }): Parameters<SearchPackagesArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate query input
        validate_package_name(&query).map_err(validation_error_to_mcp)?;

        // Create cache key including limit
        let cache_key = format!("{}:{}", query, limit.unwrap_or(10));

        // Check cache first
        if let Some(cached_result) = self.search_cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute with security features (audit logging + timeout)
        let search_cache = self.search_cache.clone();
        let cache_key_clone = cache_key.clone();

        audit_tool_execution(
            &self.audit,
            "search_packages",
            Some(serde_json::json!({"query": &query})),
            || async move {
                with_timeout(&self.audit, "search_packages", 30, || async {
                    let limit = limit.unwrap_or(10);

                    // Use nix search command
                    let output = tokio::process::Command::new("nix")
                        .args(["search", "nixpkgs", &query, "--json"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix search: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("nix search failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let results: serde_json::Value =
                        serde_json::from_str(&stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse search results: {}", e),
                                None,
                            )
                        })?;

                    // Format results nicely
                    let mut formatted_results = Vec::new();
                    if let Some(obj) = results.as_object() {
                        for (i, (pkg_path, info)) in obj.iter().enumerate() {
                            if i >= limit {
                                break;
                            }

                            let description =
                                info["description"].as_str().unwrap_or("No description");
                            let version = info["version"].as_str().unwrap_or("unknown");

                            formatted_results.push(format!(
                                "Package: {}\nVersion: {}\nDescription: {}\n",
                                pkg_path, version, description
                            ));
                        }
                    }

                    let result_text = if formatted_results.is_empty() {
                        format!("No packages found matching '{}'", query)
                    } else {
                        format!(
                            "Found {} packages matching '{}':\n\n{}",
                            formatted_results.len(),
                            query,
                            formatted_results.join("\n")
                        )
                    };

                    // Cache the result
                    search_cache.insert(cache_key_clone, result_text.clone());

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get detailed information about a specific package",
        annotations(read_only_hint = true)
    )]
    async fn get_package_info(
        &self,
        Parameters(GetPackageInfoArgs { package }): Parameters<GetPackageInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate package reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Check cache first
        if let Some(cached_result) = self.package_info_cache.get(&package) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute with security features (audit logging + timeout)
        let package_info_cache = self.package_info_cache.clone();
        let package_clone = package.clone();

        audit_tool_execution(
            &self.audit,
            "get_package_info",
            Some(serde_json::json!({"package": &package})),
            || async move {
                with_timeout(&self.audit, "get_package_info", 30, || async {
                    // Use nix eval to get package metadata
                    let output = tokio::process::Command::new("nix")
                        .args(["eval", &package, "--json"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix eval: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("nix eval failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                    // Cache the result
                    package_info_cache.insert(package_clone, stdout.clone());

                    Ok(CallToolResult::success(vec![Content::text(stdout)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Search NixOS configuration options",
        annotations(read_only_hint = true)
    )]
    async fn search_options(
        &self,
        Parameters(SearchOptionsArgs { query }): Parameters<SearchOptionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate query input
        validate_package_name(&query).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + timeout)
        audit_tool_execution(&self.audit, "search_options", Some(serde_json::json!({"query": &query})), || async {
            with_timeout(&self.audit, "search_options", 30, || async {
                    // Use nix-instantiate to search options if available, or provide helpful info
                    let output = tokio::process::Command::new("nix")
                        .args([
                            "search",
                            "--extra-experimental-features", "nix-command",
                            "--extra-experimental-features", "flakes",
                            &format!("nixos-options#{}", query)
                        ])
                        .output()
                        .await;

                    match output {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);

                            if !output.status.success() {
                                // Fallback to providing helpful information
                                Ok(CallToolResult::success(vec![Content::text(format!(
                                    "Note: Direct option search requires NixOS. Search for '{}' at:\n- https://search.nixos.org/options\n- https://nixos.org/manual/nixos/stable/options.html\n\nError: {}",
                                    query, stderr
                                ))]))
                            } else {
                                Ok(CallToolResult::success(vec![Content::text(stdout.to_string())]))
                            }
                        }
                        Err(_) => {
                            Ok(CallToolResult::success(vec![Content::text(format!(
                                "Search for NixOS options containing '{}':\n- https://search.nixos.org/options?query={}\n- https://nixos.org/manual/nixos/stable/options.html",
                                query, query
                            ))]))
                        }
                    }
            }).await
        }).await
    }

    #[tool(description = "Evaluate a Nix expression")]
    async fn nix_eval(
        &self,
        Parameters(NixEvalArgs { expression }): Parameters<NixEvalArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_nix_expression;

        // Validate Nix expression for dangerous patterns
        validate_nix_expression(&expression).map_err(validation_error_to_mcp)?;

        // Check cache first
        if let Some(cached_result) = self.eval_cache.get(&expression) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute with security features (audit logging + 30s timeout for eval)
        let eval_cache = self.eval_cache.clone();
        let expression_clone = expression.clone();

        audit_tool_execution(
            &self.audit,
            "nix_eval",
            Some(serde_json::json!({"expression_length": expression.len()})),
            || async move {
                with_timeout(&self.audit, "nix_eval", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["eval", "--expr", &expression])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix eval: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Evaluation failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                    // Cache the result
                    eval_cache.insert(expression_clone, stdout.clone());

                    Ok(CallToolResult::success(vec![Content::text(stdout)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Format Nix code using nixpkgs-fmt",
        annotations(idempotent_hint = true)
    )]
    async fn format_nix(
        &self,
        Parameters(FormatNixArgs { code }): Parameters<FormatNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_nix_expression;

        // Validate Nix code
        validate_nix_expression(&code).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(&self.audit, "format_nix", Some(serde_json::json!({"code_length": code.len()})), || async {
            with_timeout(&self.audit, "format_nix", 30, || async {
                // Try nixpkgs-fmt first, fallback to alejandra
                let child = tokio::process::Command::new("nixpkgs-fmt")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn();

                let mut child = match child {
                    Ok(c) => c,
                    Err(_) => {
                        // Try alejandra as fallback
                        tokio::process::Command::new("alejandra")
                            .args(["--quiet", "-"])
                            .stdin(std::process::Stdio::piped())
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .spawn()
                            .map_err(|e| McpError::internal_error(
                                format!("Neither nixpkgs-fmt nor alejandra found. Install with: nix-shell -p nixpkgs-fmt\nError: {}", e),
                                None
                            ))?
                    }
                };

                // Write code to stdin
                if let Some(ref mut stdin) = child.stdin {
                    use tokio::io::AsyncWriteExt;
                    stdin.write_all(code.as_bytes()).await
                        .map_err(|e| McpError::internal_error(format!("Failed to write to formatter: {}", e), None))?;
                }

                let output = child.wait_with_output().await
                    .map_err(|e| McpError::internal_error(format!("Formatter failed: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(format!("Formatting failed: {}", stderr), None));
                }

                let formatted = String::from_utf8_lossy(&output.stdout);
                Ok(CallToolResult::success(vec![Content::text(formatted.to_string())]))
            }).await
        }).await
    }

    #[tool(
        description = "Get help with common Nix commands and patterns",
        annotations(read_only_hint = true)
    )]
    fn nix_command_help(
        &self,
        Parameters(NixCommandHelpArgs { command }): Parameters<NixCommandHelpArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Audit logging for informational tool
        self.audit.log_tool_invocation(
            "nix_command_help",
            Some(serde_json::json!({"command": &command})),
            true,
            None,
            0,
        );

        let help_text = match command.as_deref() {
            Some("develop") => {
                r#"nix develop - Enter a development shell

Usage:
  nix develop              # Enter devShell from flake.nix
  nix develop .#myShell    # Enter specific shell
  nix develop -c <cmd>     # Run command in dev shell
  nix develop --impure     # Allow impure evaluation

Example flake.nix devShell:
  devShells.default = pkgs.mkShell {
    packages = [ pkgs.rustc pkgs.cargo ];
    shellHook = ''
      echo "Welcome to the dev shell!"
    '';
  };
"#
            }
            Some("build") => {
                r#"nix build - Build a package

Usage:
  nix build                # Build default package from flake
  nix build .#package      # Build specific package
  nix build nixpkgs#hello  # Build from nixpkgs
  nix build --json         # Output JSON metadata

Result: Creates 'result' symlink to build output
"#
            }
            Some("flake") => {
                r#"nix flake - Manage Nix flakes

Common commands:
  nix flake init           # Create new flake.nix
  nix flake update         # Update flake.lock
  nix flake check          # Check flake outputs
  nix flake show           # Show flake outputs
  nix flake metadata       # Show flake metadata

Templates:
  nix flake init -t templates#rust      # Rust template
  nix flake init -t templates#python    # Python template
"#
            }
            Some("shell") | Some("nix-shell") => {
                r#"Getting Packages in a Shell

Modern way (with flakes):
  nix shell nixpkgs#hello             # Add hello to PATH
  nix shell nixpkgs#hello nixpkgs#git # Multiple packages
  nix shell nixpkgs#python3 -c python # Run command in shell

Classic way (nix-shell -p):
  nix-shell -p hello                  # Quick temporary shell with package
  nix-shell -p python3 nodejs         # Multiple packages
  nix-shell -p gcc --run "gcc --version"  # Run command and exit

The -p flag is the FASTEST way to try any package from nixpkgs!
No configuration needed, just: nix-shell -p <package-name>

Note: Prefer 'nix develop' for project development environments with flake.nix
"#
            }
            Some("run") => {
                r#"nix run - Run a package

Usage:
  nix run nixpkgs#hello        # Run hello from nixpkgs
  nix run .#myapp              # Run app from local flake
  nix run github:user/repo     # Run from GitHub
"#
            }
            _ => {
                r#"Common Nix Commands:

Quick Package Access (MOST USEFUL):
  nix-shell -p <pkg>        - Instant shell with ANY nixpkgs package
  nix-shell -p pkg1 pkg2    - Multiple packages at once
  nix shell nixpkgs#<pkg>   - Flakes equivalent

Development:
  nix develop               - Enter development shell from flake.nix
  nix develop -c <cmd>      - Run command in dev environment

Building:
  nix build                 - Build package (creates 'result' symlink)
  nix build .#pkg           - Build specific package

Running:
  nix run nixpkgs#tool      - Run a package directly

Flakes:
  nix flake init            - Initialize new flake
  nix flake update          - Update dependencies
  nix flake check           - Validate flake outputs
  nix flake show            - Display flake structure

Searching:
  nix search nixpkgs query  - Search for packages

Other:
  nix eval --expr "1 + 1"   - Evaluate Nix expression
  nix fmt                   - Format Nix files

Use 'nix_command_help' with specific command for details.
Available: develop, build, flake, shell, nix-shell, run

TIP: 'nix-shell -p <package>' is the fastest way to try any nixpkgs package!

Ecosystem Tools:
  Use 'ecosystem_tools' to learn about useful tools like:
  - comma (,): Run programs without installing
  - noogle.dev: Search Nix functions
  - alejandra: Format Nix code
  - disko: Declarative disk setup
  - nixos-anywhere: Remote NixOS installation
  And many more!
"#
            }
        };

        Ok(CallToolResult::success(vec![Content::text(help_text)]))
    }

    #[tool(
        description = "Get information about useful Nix ecosystem tools and utilities",
        annotations(read_only_hint = true)
    )]
    fn ecosystem_tools(
        &self,
        Parameters(EcosystemToolArgs { tool }): Parameters<EcosystemToolArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Audit logging for informational tool
        self.audit.log_tool_invocation(
            "ecosystem_tools",
            Some(serde_json::json!({"tool": &tool})),
            true,
            None,
            0,
        );

        let info = match tool.as_deref() {
            Some("comma") | Some(",") => {
                r#"comma - Run programs without installing them
Repository: https://github.com/nix-community/comma
Install: nix-env -iA nixpkgs.comma

Usage:
  , cowsay hello    # Runs cowsay without installing it
  , python3 -c "print('hi')"  # Run Python scripts

Comma uses nix-index to locate and run any program from nixpkgs instantly.
First time may take a while to build the index, but then it's very fast!"#
            }

            Some("disko") => {
                r#"disko - Declarative disk partitioning and formatting
Repository: https://github.com/nix-community/disko

Declaratively define disk layouts in Nix, including partitions, filesystems,
LUKS encryption, LVM, RAID, and more. Great for automated NixOS installations.

Example use: Define your entire disk layout in configuration.nix
Can be used with nixos-anywhere for remote installations."#
            }

            Some("nixos-anywhere") => {
                r#"nixos-anywhere - Install NixOS remotely via SSH
Repository: https://github.com/nix-community/nixos-anywhere

Install NixOS on a remote machine from any Linux system via SSH.
Works great with disko for declarative disk setup.

Usage:
  nixos-anywhere --flake '.#my-server' root@192.168.1.10

Perfect for automated server deployments!"#
            }

            Some("terranix") => {
                r#"terranix - NixOS-like Terraform configurations
Repository: https://github.com/terranix/terranix

Write Terraform configurations in Nix instead of HCL.
Get Nix's module system, type checking, and code reuse for infrastructure.

Benefits:
- Use Nix functions and imports
- Type-safe infrastructure code
- Share modules across projects
- Generate complex Terraform configs programmatically"#
            }

            Some("noogle") | Some("noogle.dev") => {
                r#"noogle.dev - Search Nix functions and built-ins
Website: https://noogle.dev/

Interactive search for Nix language built-ins and nixpkgs lib functions.
Essential reference when writing Nix expressions.

Search examples:
- "map" - Find list mapping functions
- "filter" - Find filtering functions
- "mkDerivation" - Package building functions

Much faster than reading docs.nixos.org!"#
            }

            Some("microvm") | Some("microvm.nix") => {
                r#"microvm.nix - Lightweight NixOS VMs
Repository: https://github.com/microvm-nix/microvm.nix

Create ultra-lightweight NixOS VMs (MicroVMs) with minimal overhead.
Uses cloud-hypervisor, firecracker, or qemu.

Benefits:
- Boot in milliseconds
- Minimal memory footprint
- Declarative VM configuration
- Share /nix/store with host (saves space)

Great for development, testing, or running services in isolation."#
            }

            Some("alejandra") => {
                r#"alejandra - Opinionated Nix code formatter
Repository: https://github.com/kamadorueda/alejandra
Install: nix-shell -p alejandra

Usage:
  alejandra .           # Format all Nix files
  alejandra file.nix    # Format specific file

Alternative to nixpkgs-fmt with different style opinions.
Fast and deterministic formatting."#
            }

            Some("deadnix") => {
                r#"deadnix - Find and remove dead Nix code
Repository: https://github.com/astro/deadnix
Install: nix-shell -p deadnix

Usage:
  deadnix .                    # Find dead code
  deadnix --edit .             # Remove dead code automatically

Finds unused:
- Function arguments
- Let bindings
- Imports

Helps keep Nix code clean and maintainable."#
            }

            Some("nix-init") => {
                r#"nix-init - Generate Nix packages from URLs
Repository: https://github.com/nix-community/nix-init
Install: nix-shell -p nix-init

Usage:
  nix-init              # Interactive package generation
  nix-init <url>        # Generate from URL

Automatically creates Nix package definitions for:
- Rust crates (Cargo.toml)
- Python packages (PyPI)
- Go modules
- NPM packages
- And more!

Saves tons of time when packaging software."#
            }

            Some("statix") => {
                r#"statix - Lints and suggestions for Nix
Repository: https://github.com/oppiliappan/statix
Install: nix-shell -p statix

Usage:
  statix check .        # Check for issues
  statix fix .          # Auto-fix issues

Checks for:
- Anti-patterns
- Deprecated syntax
- Performance issues
- Code smells

Helps write better, more idiomatic Nix code."#
            }

            Some("nvd") => {
                r#"nvd - Nix version diff tool
Repository: https://git.sr.ht/~khumba/nvd
Install: nix-shell -p nvd

Usage:
  nvd diff /nix/var/nix/profiles/system-{42,43}-link

Shows what changed between NixOS generations:
- Added/removed packages
- Version upgrades/downgrades
- Size changes

Much more readable than plain nix-store diff!"#
            }

            Some("nixpkgs-review") => {
                r#"nixpkgs-review - Review nixpkgs pull requests
Repository: https://github.com/Mic92/nixpkgs-review
Install: nix-shell -p nixpkgs-review

Usage:
  nixpkgs-review pr 12345     # Review PR #12345
  nixpkgs-review rev HEAD     # Review local changes

Automatically builds packages affected by nixpkgs PRs.
Essential for nixpkgs contributors to test changes before merging.

Features:
- Builds all affected packages
- Creates a nix-shell with built packages
- Reports build failures
- Tests on multiple platforms"#
            }

            Some("crane") => {
                r#"crane - Nix library for building Cargo projects
Repository: https://github.com/ipetkov/crane
Install: Add to flake inputs

A Nix library focused on building Cargo (Rust) projects efficiently.

Benefits:
- Incremental builds with dependency caching
- Faster CI builds (cache dependencies separately)
- Cross-compilation support
- Minimal rebuilds when code changes

Example flake.nix:
  inputs.crane.url = "github:ipetkov/crane";
  craneLib = crane.mkLib pkgs;
  my-crate = craneLib.buildPackage {
    src = ./.;
  };

Much better than naersk for Rust projects!"#
            }

            Some("nil") => {
                r#"nil - Nix Language Server (LSP)
Repository: https://github.com/oxalica/nil
Install: nix-shell -p nil

A Nix language server providing IDE features:
- Syntax highlighting
- Auto-completion
- Go to definition
- Find references
- Diagnostics and error checking

Configure in your editor:
- VSCode: Use "nix-ide" extension
- Neovim: Configure with nvim-lspconfig
- Emacs: Use lsp-mode

Much faster and more accurate than other Nix LSPs!"#
            }

            Some("treefmt-nix") | Some("treefmt") => {
                r#"treefmt-nix - Multi-language formatter manager
Repository: https://github.com/numtide/treefmt-nix
Install: Add to flake inputs

One command to format all files in your project, regardless of language.

Example flake.nix:
  treefmt.config = {
    projectRootFile = "flake.nix";
    programs = {
      nixpkgs-fmt.enable = true;
      rustfmt.enable = true;
      prettier.enable = true;
    };
  };

Then just run: treefmt

Formats Nix, Rust, JS, Python, and more in one go!"#
            }

            Some("git-hooks.nix") | Some("pre-commit-hooks") | Some("pre-commit-hooks.nix") => {
                r#"git-hooks.nix - Pre-commit hooks for Nix projects
Repository: https://github.com/cachix/git-hooks.nix
Install: Add to flake inputs

Declaratively configure git pre-commit hooks in your flake.

Example flake.nix:
  pre-commit-check = pre-commit-hooks.lib.${system}.run {
    src = ./.;
    hooks = {
      nixpkgs-fmt.enable = true;
      statix.enable = true;
      deadnix.enable = true;
    };
  };

Automatically formats and lints code before commits.
Prevents bad code from being committed!"#
            }

            _ => {
                r#"Useful Nix Ecosystem Tools:

Quick Access & Discovery:
- comma (,)         - Run any program without installing (nix-shell -p comma)
- noogle.dev        - Search Nix functions and documentation online

Code Quality & Formatting:
- alejandra         - Opinionated Nix formatter (nix-shell -p alejandra)
- deadnix           - Find dead/unused code (nix-shell -p deadnix)
- statix            - Linter with auto-fixes (nix-shell -p statix)
- treefmt-nix       - Multi-language formatter manager
- git-hooks.nix     - Declarative pre-commit hooks

Development Tools:
- nil               - Nix Language Server / LSP (nix-shell -p nil)
- nixpkgs-review    - Review nixpkgs PRs (nix-shell -p nixpkgs-review)

Package Development:
- nix-init          - Generate Nix packages from URLs (nix-shell -p nix-init)
- crane             - Efficient Cargo/Rust builds

Infrastructure & Deployment:
- disko             - Declarative disk partitioning
- nixos-anywhere    - Remote NixOS installation via SSH
- terranix          - Write Terraform in Nix
- microvm.nix       - Lightweight NixOS VMs

System Management:
- nvd               - Diff NixOS generations (nix-shell -p nvd)

Use 'ecosystem_tools' with a specific tool name for detailed information.
Example: ecosystem_tools(tool="comma") or ecosystem_tools(tool="crane")"#
            }
        };

        Ok(CallToolResult::success(vec![Content::text(info)]))
    }

    #[tool(
        description = "Validate Nix code syntax and check for parse errors",
        annotations(idempotent_hint = true)
    )]
    async fn validate_nix(
        &self,
        Parameters(ValidateNixArgs { code }): Parameters<ValidateNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_nix_expression;

        // Validate Nix code for dangerous patterns
        validate_nix_expression(&code).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "validate_nix",
            Some(serde_json::json!({"code_length": code.len()})),
            || async {
                with_timeout(&self.audit, "validate_nix", 30, || async {
                    // Use nix-instantiate --parse to validate syntax
                    let child = tokio::process::Command::new("nix-instantiate")
                        .args(["--parse", "-E"])
                        .arg(&code)
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to spawn nix-instantiate: {}", e),
                                None,
                            )
                        })?;

                    let output = child.wait_with_output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to validate: {}", e), None)
                    })?;

                    if output.status.success() {
                        Ok(CallToolResult::success(vec![Content::text(
                            "✓ Nix code is valid! No syntax errors found.".to_string(),
                        )]))
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Ok(CallToolResult::success(vec![Content::text(format!(
                            "✗ Syntax errors found:\n\n{}",
                            stderr
                        ))]))
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Lint Nix code with statix and/or deadnix to find issues and anti-patterns",
        annotations(idempotent_hint = true)
    )]
    async fn lint_nix(
        &self,
        Parameters(LintNixArgs { code, linter }): Parameters<LintNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_nix_expression;

        // Validate Nix code for dangerous patterns
        validate_nix_expression(&code).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(&self.audit, "lint_nix", Some(serde_json::json!({"code_length": code.len(), "linter": &linter})), || async {
            with_timeout(&self.audit, "lint_nix", 30, || async {
                let linter = linter.unwrap_or_else(|| "both".to_string());
                let mut results = Vec::new();

                // Create a temporary file for the code
                let temp_dir = std::env::temp_dir();
                let temp_file = temp_dir.join(format!("nix_lint_{}.nix", std::process::id()));

                tokio::fs::write(&temp_file, &code).await
                    .map_err(|e| McpError::internal_error(format!("Failed to write temp file: {}", e), None))?;

        // Run statix if requested
        if linter == "statix" || linter == "both" {
            let output = tokio::process::Command::new("statix")
                .args(["check", temp_file.to_str().unwrap()])
                .output()
                .await;

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !stdout.is_empty() || !stderr.is_empty() {
                        results.push(format!("=== statix findings ===\n{}{}", stdout, stderr));
                    } else if output.status.success() {
                        results.push("=== statix findings ===\n✓ No issues found by statix".to_string());
                    }
                }
                Err(_) => {
                    results.push("=== statix findings ===\n(statix not installed - run: nix-shell -p statix)".to_string());
                }
            }
        }

        // Run deadnix if requested
        if linter == "deadnix" || linter == "both" {
            let output = tokio::process::Command::new("deadnix")
                .arg(temp_file.to_str().unwrap())
                .output()
                .await;

            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !stdout.is_empty() || !stderr.is_empty() {
                        results.push(format!("=== deadnix findings ===\n{}{}", stdout, stderr));
                    } else if output.status.success() {
                        results.push("=== deadnix findings ===\n✓ No dead code found".to_string());
                    }
                }
                Err(_) => {
                    results.push("=== deadnix findings ===\n(deadnix not installed - run: nix-shell -p deadnix)".to_string());
                }
            }
        }

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_file).await;

        let result_text = if results.is_empty() {
            "No linters were run. Use linter=\"statix\", \"deadnix\", or \"both\".".to_string()
        } else {
            results.join("\n\n")
        };

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }).await
        }).await
    }

    #[tool(
        description = "Get detailed information about a package (version, description, homepage, license, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn explain_package(
        &self,
        Parameters(ExplainPackageArgs { package }): Parameters<ExplainPackageArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate package name
        validate_package_name(&package).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "explain_package",
            Some(serde_json::json!({"package": &package})),
            || async {
                with_timeout(&self.audit, "explain_package", 30, || async {
                    // Normalize package reference
                    let pkg_ref = if package.contains('#') {
                        package.clone()
                    } else {
                        format!("nixpkgs#{}", package)
                    };

                    // Get package metadata using nix eval
                    let meta_attr = format!("{}.meta", pkg_ref);

                    let output = tokio::process::Command::new("nix")
                        .args(["eval", "--json", &meta_attr])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to get package info: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to evaluate package: {}", stderr),
                            None,
                        ));
                    }

                    let meta: serde_json::Value =
                        serde_json::from_slice(&output.stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse metadata: {}", e),
                                None,
                            )
                        })?;

                    let mut info = Vec::new();
                    info.push(format!("Package: {}", package));

                    if let Some(version) = meta.get("version").and_then(|v| v.as_str()) {
                        info.push(format!("Version: {}", version));
                    }

                    if let Some(description) = meta.get("description").and_then(|v| v.as_str()) {
                        info.push(format!("Description: {}", description));
                    }

                    if let Some(homepage) = meta.get("homepage").and_then(|v| v.as_str()) {
                        info.push(format!("Homepage: {}", homepage));
                    }

                    if let Some(license) = meta.get("license") {
                        if let Some(name) = license.get("spdxId").and_then(|v| v.as_str()) {
                            info.push(format!("License: {}", name));
                        } else if let Some(name) = license.get("fullName").and_then(|v| v.as_str())
                        {
                            info.push(format!("License: {}", name));
                        }
                    }

                    if let Some(platforms) = meta.get("platforms").and_then(|v| v.as_array()) {
                        let platform_list: Vec<String> = platforms
                            .iter()
                            .filter_map(|p| p.as_str().map(String::from))
                            .take(5)
                            .collect();
                        if !platform_list.is_empty() {
                            info.push(format!(
                                "Platforms: {} (showing first 5)",
                                platform_list.join(", ")
                            ));
                        }
                    }

                    if let Some(maintainers) = meta.get("maintainers").and_then(|v| v.as_array()) {
                        let maint_list: Vec<String> = maintainers
                            .iter()
                            .filter_map(|m| {
                                m.get("name").and_then(|n| n.as_str()).map(String::from)
                            })
                            .take(3)
                            .collect();
                        if !maint_list.is_empty() {
                            info.push(format!("Maintainers: {}", maint_list.join(", ")));
                        }
                    }

                    Ok(CallToolResult::success(vec![Content::text(
                        info.join("\n"),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Prefetch a URL and get its hash for use in Nix expressions")]
    async fn prefetch_url(
        &self,
        Parameters(PrefetchUrlArgs { url, hash_format }): Parameters<PrefetchUrlArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_url;

        // Validate URL
        validate_url(&url).map_err(validation_error_to_mcp)?;

        // Create cache key including format
        let cache_key = format!("{}:{}", url, hash_format.as_deref().unwrap_or("sri"));

        // Check cache first
        if let Some(cached_result) = self.prefetch_cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute with security features (audit logging + 60s timeout)
        let prefetch_cache = self.prefetch_cache.clone();
        let cache_key_clone = cache_key.clone();

        audit_tool_execution(&self.audit, "prefetch_url", Some(serde_json::json!({"url": &url})), || async move {
            with_timeout(&self.audit, "prefetch_url", 60, || async {
                let _format = hash_format.unwrap_or_else(|| "sri".to_string());

                let output = tokio::process::Command::new("nix")
                    .args(["store", "prefetch-file", &url])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to prefetch URL: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(format!("Prefetch failed: {}", stderr), None));
                }

                // Parse hash from stderr which contains: "Downloaded '...' to '...' (hash 'sha256-...')."
                let stderr = String::from_utf8_lossy(&output.stderr);
                let hash = if let Some(hash_start) = stderr.find("(hash '") {
                    let hash_part = &stderr[hash_start + 7..];
                    if let Some(hash_end) = hash_part.find("')") {
                        hash_part[..hash_end].to_string()
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                };

                let result = format!(
                    "URL: {}\nHash: {}\n\nUse in Nix:\nfetchurl {{\n  url = \"{}\";\n  hash = \"{}\";\n}}",
                    url, hash, url, hash
                );

                // Cache the result
                prefetch_cache.insert(cache_key_clone, result.clone());

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }).await
        }).await
    }

    #[tool(
        description = "Get metadata about a flake (inputs, outputs, description)",
        annotations(read_only_hint = true)
    )]
    async fn flake_metadata(
        &self,
        Parameters(FlakeMetadataArgs { flake_ref }): Parameters<FlakeMetadataArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate flake reference
        validate_flake_ref(&flake_ref).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "flake_metadata",
            Some(serde_json::json!({"flake_ref": &flake_ref})),
            || async {
                with_timeout(&self.audit, "flake_metadata", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["flake", "metadata", "--json", &flake_ref])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to get flake metadata: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to read flake: {}", stderr),
                            None,
                        ));
                    }

                    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse metadata: {}", e),
                                None,
                            )
                        })?;

                    let mut info = Vec::new();

                    if let Some(description) = metadata.get("description").and_then(|v| v.as_str())
                    {
                        info.push(format!("Description: {}", description));
                    }

                    if let Some(url) = metadata.get("url").and_then(|v| v.as_str()) {
                        info.push(format!("URL: {}", url));
                    }

                    if let Some(locked) = metadata.get("locked") {
                        if let Some(rev) = locked.get("rev").and_then(|v| v.as_str()) {
                            info.push(format!("Revision: {}", &rev[..12.min(rev.len())]));
                        }
                        if let Some(last_mod) = locked.get("lastModified").and_then(|v| v.as_u64())
                        {
                            info.push(format!("Last Modified: {}", last_mod));
                        }
                    }

                    if let Some(locks) = metadata.get("locks") {
                        if let Some(nodes) = locks.get("nodes").and_then(|v| v.as_object()) {
                            let inputs: Vec<String> = nodes
                                .keys()
                                .filter(|k| k.as_str() != "root")
                                .map(|k| k.to_string())
                                .collect();
                            if !inputs.is_empty() {
                                info.push(format!("\nInputs: {}", inputs.join(", ")));
                            }
                        }
                    }

                    Ok(CallToolResult::success(vec![Content::text(
                        info.join("\n"),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Find which package provides a command using nix-locate",
        annotations(read_only_hint = true)
    )]
    async fn find_command(
        &self,
        Parameters(FindCommandArgs { command }): Parameters<FindCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_command;

        // Validate command name
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "find_command", Some(serde_json::json!({"command": &command})), || async {
            with_timeout(&self.audit, "find_command", 30, || async {
                // Try nix-locate first
                let output = tokio::process::Command::new("nix-locate")
                    .args(["--top-level", "--whole-name", &format!("/bin/{}", command)])
                    .output()
                    .await;

                match output {
                    Ok(output) if output.status.success() => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let packages: Vec<&str> = stdout.lines()
                            .filter_map(|line| line.split_whitespace().next())
                            .take(10)
                            .collect();

                        if packages.is_empty() {
                            Ok(CallToolResult::success(vec![Content::text(
                                format!("Command '{}' not found in any package.\n\nTry:\n- nix search nixpkgs {}", command, command)
                            )]))
                        } else {
                            let result = format!(
                                "Command '{}' is provided by:\n\n{}\n\nInstall with:\n  nix-shell -p {}",
                                command,
                                packages.iter().map(|p| format!("  - {}", p)).collect::<Vec<_>>().join("\n"),
                                packages[0]
                            );
                            Ok(CallToolResult::success(vec![Content::text(result)]))
                        }
                    }
                    _ => {
                        // Fallback: provide instructions
                        Ok(CallToolResult::success(vec![Content::text(
                            format!(
                                "nix-locate not available. Install with: nix-shell -p nix-index\n\n\
                                To find command '{}' manually:\n\
                                1. nix search nixpkgs {}\n\
                                2. Try common packages: nix-shell -p {}\n\
                                3. Use https://search.nixos.org/packages to search",
                                command, command, command
                            )
                        )]))
                    }
                }
            }).await
        }).await
    }

    #[tool(description = "Build a Nix package and show what will be built or the build output")]
    async fn nix_build(
        &self,
        Parameters(NixBuildArgs { package, dry_run }): Parameters<NixBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate package reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 300s timeout for builds)
        audit_tool_execution(
            &self.audit,
            "nix_build",
            Some(serde_json::json!({"package": &package, "dry_run": dry_run})),
            || async {
                with_timeout(&self.audit, "nix_build", 300, || async {
                    let dry_run = dry_run.unwrap_or(false);

                    let mut args = vec!["build"];
                    if dry_run {
                        args.push("--dry-run");
                    }
                    args.push(&package);
                    args.push("--json");

                    let output = tokio::process::Command::new("nix")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix build: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        let error_msg = if dry_run {
                            format!("Dry-run build check failed:\n\n{}", stderr)
                        } else {
                            format!("Build failed:\n\n{}", stderr)
                        };

                        return Ok(CallToolResult::success(vec![Content::text(error_msg)]));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);

                    if dry_run {
                        // For dry-run, parse what would be built
                        let result = if let Ok(json_output) =
                            serde_json::from_str::<serde_json::Value>(&stdout)
                        {
                            format!(
                                "Dry-run completed successfully.\n\nBuild plan:\n{}",
                                serde_json::to_string_pretty(&json_output)
                                    .unwrap_or_else(|_| stdout.to_string())
                            )
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            format!("Dry-run completed successfully.\n\n{}", stderr)
                        };
                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    } else {
                        // For actual build, show the result
                        if let Ok(json_output) = serde_json::from_str::<serde_json::Value>(&stdout)
                        {
                            let mut result = String::from("Build completed successfully!\n\n");

                            if let Some(arr) = json_output.as_array() {
                                for item in arr {
                                    if let Some(drv_path) =
                                        item.get("drvPath").and_then(|v| v.as_str())
                                    {
                                        result.push_str(&format!("Derivation: {}\n", drv_path));
                                    }
                                    if let Some(out_paths) =
                                        item.get("outputs").and_then(|v| v.as_object())
                                    {
                                        result.push_str("Outputs:\n");
                                        for (name, path) in out_paths {
                                            if let Some(path_str) = path.as_str() {
                                                result.push_str(&format!(
                                                    "  {}: {}\n",
                                                    name, path_str
                                                ));
                                            }
                                        }
                                    }
                                }
                            }

                            result.push_str("\nResult symlink created: ./result\n");
                            Ok(CallToolResult::success(vec![Content::text(result)]))
                        } else {
                            Ok(CallToolResult::success(vec![Content::text(format!(
                                "Build completed!\n\n{}",
                                stdout
                            ))]))
                        }
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Explain why one package depends on another (show dependency chain)",
        annotations(read_only_hint = true)
    )]
    async fn why_depends(
        &self,
        Parameters(WhyDependsArgs {
            package,
            dependency,
            show_all,
        }): Parameters<WhyDependsArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_package_name;

        // Validate package names
        validate_package_name(&package).map_err(validation_error_to_mcp)?;
        validate_package_name(&dependency).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "why_depends",
            Some(serde_json::json!({"package": &package, "dependency": &dependency})),
            || async {
                with_timeout(&self.audit, "why_depends", 60, || async {
                    let show_all = show_all.unwrap_or(false);

                    // First, build the package to get its store path
                    let build_output = tokio::process::Command::new("nix")
                        .args(["build", &package, "--json", "--no-link"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to build package: {}", e),
                                None,
                            )
                        })?;

                    if !build_output.status.success() {
                        let stderr = String::from_utf8_lossy(&build_output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to build package: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&build_output.stdout);
                    let build_json: serde_json::Value =
                        serde_json::from_str(&stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse build output: {}", e),
                                None,
                            )
                        })?;

                    let package_path = build_json
                        .as_array()
                        .and_then(|arr| arr.get(0))
                        .and_then(|item| item.get("outputs"))
                        .and_then(|outputs| outputs.get("out"))
                        .and_then(|out| out.as_str())
                        .ok_or_else(|| {
                            McpError::internal_error(
                                "Failed to get package output path".to_string(),
                                None,
                            )
                        })?;

                    // Build dependency to get its store path
                    let dep_build_output = tokio::process::Command::new("nix")
                        .args(["build", &dependency, "--json", "--no-link"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to build dependency: {}", e),
                                None,
                            )
                        })?;

                    if !dep_build_output.status.success() {
                        let stderr = String::from_utf8_lossy(&dep_build_output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to build dependency: {}", stderr),
                            None,
                        ));
                    }

                    let dep_stdout = String::from_utf8_lossy(&dep_build_output.stdout);
                    let dep_json: serde_json::Value =
                        serde_json::from_str(&dep_stdout).map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to parse dependency build output: {}", e),
                                None,
                            )
                        })?;

                    let dependency_path = dep_json
                        .as_array()
                        .and_then(|arr| arr.get(0))
                        .and_then(|item| item.get("outputs"))
                        .and_then(|outputs| outputs.get("out"))
                        .and_then(|out| out.as_str())
                        .ok_or_else(|| {
                            McpError::internal_error(
                                "Failed to get dependency output path".to_string(),
                                None,
                            )
                        })?;

                    // Now run nix why-depends
                    let mut args = vec!["why-depends", package_path, dependency_path];
                    if show_all {
                        args.push("--all");
                    }

                    let output = tokio::process::Command::new("nix")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix why-depends: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        // Check if it's because there's no dependency
                        if stderr.contains("does not depend on") {
                            return Ok(CallToolResult::success(vec![Content::text(format!(
                                "{} does not depend on {}",
                                package, dependency
                            ))]));
                        }

                        return Err(McpError::internal_error(
                            format!("why-depends failed: {}", stderr),
                            None,
                        ));
                    }

                    let result = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        result.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Show the derivation details of a package (build inputs, environment, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn show_derivation(
        &self,
        Parameters(ShowDerivationArgs { package }): Parameters<ShowDerivationArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_flake_ref;

        // Validate package/flake reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Create cache key (package is the only parameter)
        let cache_key = package.clone();

        // Check cache first
        if let Some(cached_result) = self.derivation_cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Clone cache and key for use in async closure
        let derivation_cache = self.derivation_cache.clone();
        let cache_key_clone = cache_key.clone();

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "show_derivation",
            Some(serde_json::json!({"package": &package})),
            || async move {
                with_timeout(&self.audit, "show_derivation", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["derivation", "show", &package])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix derivation show: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to show derivation: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Try to parse and format nicely
                    if let Ok(drv_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let mut result = String::from("Derivation Details:\n\n");

                        // Get the first (and usually only) derivation
                        if let Some(obj) = drv_json.as_object() {
                            if let Some((drv_path, drv_info)) = obj.iter().next() {
                                result.push_str(&format!("Path: {}\n\n", drv_path));

                                if let Some(outputs) =
                                    drv_info.get("outputs").and_then(|v| v.as_object())
                                {
                                    result.push_str("Outputs:\n");
                                    for (name, info) in outputs {
                                        result.push_str(&format!("  - {}\n", name));
                                        if let Some(path) =
                                            info.get("path").and_then(|v| v.as_str())
                                        {
                                            result.push_str(&format!("    Path: {}\n", path));
                                        }
                                    }
                                    result.push('\n');
                                }

                                if let Some(inputs) =
                                    drv_info.get("inputDrvs").and_then(|v| v.as_object())
                                {
                                    result.push_str(&format!(
                                        "Build Dependencies: {} derivations\n",
                                        inputs.len()
                                    ));
                                }

                                if let Some(env) = drv_info.get("env").and_then(|v| v.as_object()) {
                                    result.push_str("\nKey Environment Variables:\n");
                                    for key in
                                        ["name", "version", "src", "builder", "system", "outputs"]
                                            .iter()
                                    {
                                        if let Some(value) = env.get(*key).and_then(|v| v.as_str())
                                        {
                                            result.push_str(&format!("  {}: {}\n", key, value));
                                        }
                                    }
                                }

                                result.push_str("\nFull JSON available for detailed inspection.");
                                // Only show first derivation in formatted view
                            }
                        }

                        // Cache the result
                        derivation_cache.insert(cache_key_clone.clone(), result.clone());

                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    } else {
                        let result = stdout.to_string();

                        // Cache the result
                        derivation_cache.insert(cache_key_clone, result.clone());

                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get the closure size of a package (total size including all dependencies)",
        annotations(read_only_hint = true)
    )]
    async fn get_closure_size(
        &self,
        Parameters(GetClosureSizeArgs {
            package,
            human_readable,
        }): Parameters<GetClosureSizeArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_flake_ref;

        // Validate package/flake reference
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Create cache key including human_readable flag
        let cache_key = format!("{}:{}", package, human_readable.unwrap_or(true));

        // Check cache first
        if let Some(cached_result) = self.closure_size_cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Clone cache and key for use in async closure
        let closure_size_cache = self.closure_size_cache.clone();
        let cache_key_clone = cache_key.clone();

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "get_closure_size", Some(serde_json::json!({"package": &package})), || async move {
            with_timeout(&self.audit, "get_closure_size", 60, || async {
                let human_readable = human_readable.unwrap_or(true);

                // First build the package to get its store path
                let build_output = tokio::process::Command::new("nix")
                    .args(["build", &package, "--json", "--no-link"])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to build package: {}", e), None))?;

                if !build_output.status.success() {
                    let stderr = String::from_utf8_lossy(&build_output.stderr);
                    return Err(McpError::internal_error(format!("Failed to build package: {}", stderr), None));
                }

                let stdout = String::from_utf8_lossy(&build_output.stdout);
                let build_json: serde_json::Value = serde_json::from_str(&stdout)
                    .map_err(|e| McpError::internal_error(format!("Failed to parse build output: {}", e), None))?;

                let package_path = build_json
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|item| item.get("outputs"))
                    .and_then(|outputs| outputs.get("out"))
                    .and_then(|out| out.as_str())
                    .ok_or_else(|| McpError::internal_error("Failed to get package output path".to_string(), None))?;

                // Get closure size using nix path-info
                let mut args = vec!["path-info", "-S", package_path];
                if !human_readable {
                    args.push("--json");
                }

                let output = tokio::process::Command::new("nix")
                    .args(&args)
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to get path info: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(format!("Failed to get closure size: {}", stderr), None));
                }

                let result_text = if human_readable {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Parse the output which is in format: /nix/store/... \t closure_size
                    if let Some(line) = stdout.lines().next() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let closure_size: u64 = parts[1].parse().unwrap_or(0);
                            let size_gb = closure_size as f64 / (1024.0 * 1024.0 * 1024.0);
                            let size_mb = closure_size as f64 / (1024.0 * 1024.0);

                            let human_size = if size_gb >= 1.0 {
                                format!("{:.2} GB", size_gb)
                            } else {
                                format!("{:.2} MB", size_mb)
                            };

                            format!(
                                "Package: {}\nClosure Size: {} ({} bytes)\n\nThis includes the package and all its dependencies.",
                                package, human_size, closure_size
                            )
                        } else {
                            stdout.to_string()
                        }
                    } else {
                        "No size information available".to_string()
                    }
                } else {
                    String::from_utf8_lossy(&output.stdout).to_string()
                };

                // Cache the result
                closure_size_cache.insert(cache_key_clone, result_text.clone());

                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }).await
        }).await
    }

    #[tool(description = "Run a command in a Nix shell with specified packages available")]
    async fn run_in_shell(
        &self,
        Parameters(RunInShellArgs {
            packages,
            command,
            use_flake,
        }): Parameters<RunInShellArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_command;

        // Validate command for dangerous patterns
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Validate package names if provided
        for package in &packages {
            validate_package_name(package).map_err(validation_error_to_mcp)?;
        }

        // Log potentially dangerous operation
        self.audit.log_dangerous_operation(
            "run_in_shell",
            true,
            &format!("Running command: {}", command),
        );

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(
            &self.audit,
            "run_in_shell",
            Some(serde_json::json!({"command": &command, "packages": &packages})),
            || async {
                with_timeout(&self.audit, "run_in_shell", 120, || async {
                    let use_flake = use_flake.unwrap_or(false);

                    let output = if use_flake {
                        // Use nix develop -c
                        tokio::process::Command::new("nix")
                            .args(["develop", "-c", "sh", "-c", &command])
                            .output()
                            .await
                            .map_err(|e| {
                                McpError::internal_error(
                                    format!("Failed to run in dev shell: {}", e),
                                    None,
                                )
                            })?
                    } else {
                        // Use nix-shell -p
                        let package_args: Vec<String> = packages
                            .iter()
                            .flat_map(|pkg| vec!["-p".to_string(), pkg.clone()])
                            .collect();

                        let mut args = package_args;
                        args.push("--run".to_string());
                        args.push(command.clone());

                        tokio::process::Command::new("nix-shell")
                            .args(&args)
                            .output()
                            .await
                            .map_err(|e| {
                                McpError::internal_error(
                                    format!("Failed to run in shell: {}", e),
                                    None,
                                )
                            })?
                    };

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let result_text = if output.status.success() {
                        format!(
                            "Command executed successfully!\n\nOutput:\n{}{}",
                            stdout, stderr
                        )
                    } else {
                        format!(
                            "Command failed with exit code: {:?}\n\nOutput:\n{}\n\nError:\n{}",
                            output.status.code(),
                            stdout,
                            stderr
                        )
                    };

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Show the outputs available in a flake (packages, apps, devShells, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn flake_show(
        &self,
        Parameters(FlakeShowArgs { flake_ref }): Parameters<FlakeShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        let flake_ref = flake_ref.unwrap_or_else(|| ".".to_string());

        // Validate flake reference
        validate_flake_ref(&flake_ref).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "flake_show",
            Some(serde_json::json!({"flake_ref": &flake_ref})),
            || async {
                with_timeout(&self.audit, "flake_show", 30, || async {
                    let output = tokio::process::Command::new("nix")
                        .args(["flake", "show", &flake_ref, "--json"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix flake show: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to show flake: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Parse and format the flake structure
                    if let Ok(flake_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let mut result = format!("Flake Outputs for: {}\n\n", flake_ref);

                        fn format_outputs(
                            value: &serde_json::Value,
                            prefix: String,
                            result: &mut String,
                        ) {
                            match value {
                                serde_json::Value::Object(map) => {
                                    for (key, val) in map {
                                        if val.is_object()
                                            && val.as_object().unwrap().contains_key("type")
                                        {
                                            let type_str =
                                                val["type"].as_str().unwrap_or("unknown");
                                            result.push_str(&format!(
                                                "{}  {}: {}\n",
                                                prefix, key, type_str
                                            ));
                                        } else if val.is_object() {
                                            result.push_str(&format!("{}{}:\n", prefix, key));
                                            format_outputs(val, format!("{}  ", prefix), result);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        format_outputs(&flake_json, String::new(), &mut result);

                        Ok(CallToolResult::success(vec![Content::text(result)]))
                    } else {
                        Ok(CallToolResult::success(vec![Content::text(
                            stdout.to_string(),
                        )]))
                    }
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get the build log for a package (useful for debugging build failures)",
        annotations(read_only_hint = true)
    )]
    async fn get_build_log(
        &self,
        Parameters(GetBuildLogArgs { package }): Parameters<GetBuildLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_package_name;

        // Validate package name
        validate_package_name(&package).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "get_build_log", Some(serde_json::json!({"package": &package})), || async {
            with_timeout(&self.audit, "get_build_log", 30, || async {
                // nix log can take either a package reference or a store path
                let output = tokio::process::Command::new("nix")
                    .args(["log", &package])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute nix log: {}", e), None))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    // Check if it's because the package hasn't been built
                    if stderr.contains("does not have a known build log") || stderr.contains("no build logs available") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            format!("No build log available for '{}'.\n\nThis could mean:\n- The package hasn't been built yet (use nix_build first)\n- The build was done by a different user/system\n- The log has been garbage collected\n\nTry building the package first: nix_build(package=\"{}\")", package, package)
                        )]));
                    }

                    return Err(McpError::internal_error(format!("Failed to get build log: {}", stderr), None));
                }

                let log = String::from_utf8_lossy(&output.stdout);

                // Truncate very long logs
                let result = if log.len() > 50000 {
                    let truncated = &log[..50000];
                    format!("{}\n\n... [Log truncated - showing first 50KB of {} KB total]",
                        truncated, log.len() / 1024)
                } else {
                    log.to_string()
                };

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }).await
        }).await
    }

    #[tool(
        description = "Get Nix build logs directly from store path, optionally filtered with grep pattern",
        annotations(read_only_hint = true)
    )]
    async fn nix_log(
        &self,
        Parameters(NixLogArgs {
            store_path,
            grep_pattern,
        }): Parameters<NixLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate store path
        validate_path(&store_path).map_err(validation_error_to_mcp)?;

        // Validate grep pattern if provided
        if let Some(ref pattern) = grep_pattern {
            if pattern.contains('\0') || pattern.is_empty() {
                return Err(McpError::invalid_params(
                    "Invalid grep pattern".to_string(),
                    Some(serde_json::json!({"pattern": pattern})),
                ));
            }
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_log",
            Some(serde_json::json!({"store_path": &store_path, "grep_pattern": &grep_pattern})),
            || async {
                with_timeout(&self.audit, "nix_log", 30, || async {
                    // Use nix log with store path
                    let output = tokio::process::Command::new("nix")
                        .args(["log", &store_path])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix log: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to get log: {}", stderr),
                            None,
                        ));
                    }

                    let log = String::from_utf8_lossy(&output.stdout);

                    // Apply grep filter if provided
                    let result = if let Some(ref pattern) = grep_pattern {
                        let filtered_lines: Vec<&str> = log
                            .lines()
                            .filter(|line| line.contains(pattern.as_str()))
                            .collect();

                        if filtered_lines.is_empty() {
                            format!(
                                "No lines matching '{}' found in log for {}",
                                pattern, store_path
                            )
                        } else {
                            format!(
                                "Lines matching '{}' in {}:\n\n{}",
                                pattern,
                                store_path,
                                filtered_lines.join("\n")
                            )
                        }
                    } else {
                        // Return full log, truncate if too long
                        if log.len() > 50000 {
                            let truncated = &log[..50000];
                            format!(
                                "{}\n\n... [Log truncated - showing first 50KB of {} KB total]",
                                truncated,
                                log.len() / 1024
                            )
                        } else {
                            log.to_string()
                        }
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Compare two derivations to understand what differs between packages (uses nix-diff)",
        annotations(read_only_hint = true)
    )]
    async fn diff_derivations(
        &self,
        Parameters(DiffDerivationsArgs {
            package_a,
            package_b,
        }): Parameters<DiffDerivationsArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_package_name;

        // Validate package names
        validate_package_name(&package_a).map_err(validation_error_to_mcp)?;
        validate_package_name(&package_b).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "diff_derivations", Some(serde_json::json!({"package_a": &package_a, "package_b": &package_b})), || async {
            with_timeout(&self.audit, "diff_derivations", 60, || async {
                // First, try to use nix-diff if available
                let nix_diff_check = tokio::process::Command::new("nix-diff")
                    .arg("--version")
                    .output()
                    .await;

                if nix_diff_check.is_err() {
                    // nix-diff not available, provide installation instructions
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("nix-diff is not installed.\n\nInstall with:\n  nix-shell -p nix-diff\n\nOr add to your flake devShell:\n  buildInputs = [ pkgs.nix-diff ];\n\nAlternatively, you can use show_derivation to inspect each package separately:\n- show_derivation(package=\"{}\")\n- show_derivation(package=\"{}\")", package_a, package_b)
                    )]));
                }

                // Build both packages to get their derivation paths
                let build_a = tokio::process::Command::new("nix")
                    .args(["build", &package_a, "--json", "--no-link", "--dry-run"])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to build package A: {}", e), None))?;

                if !build_a.status.success() {
                    let stderr = String::from_utf8_lossy(&build_a.stderr);
                    return Err(McpError::internal_error(format!("Failed to build package A: {}", stderr), None));
                }

                let build_b = tokio::process::Command::new("nix")
                    .args(["build", &package_b, "--json", "--no-link", "--dry-run"])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to build package B: {}", e), None))?;

                if !build_b.status.success() {
                    let stderr = String::from_utf8_lossy(&build_b.stderr);
                    return Err(McpError::internal_error(format!("Failed to build package B: {}", stderr), None));
                }

                // Parse derivation paths from JSON output
                let json_a: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&build_a.stdout))
                    .map_err(|e| McpError::internal_error(format!("Failed to parse build output A: {}", e), None))?;
                let json_b: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&build_b.stdout))
                    .map_err(|e| McpError::internal_error(format!("Failed to parse build output B: {}", e), None))?;

                let drv_a = json_a
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|item| item.get("drvPath"))
                    .and_then(|drv| drv.as_str())
                    .ok_or_else(|| McpError::internal_error("Failed to get derivation path A".to_string(), None))?;

                let drv_b = json_b
                    .as_array()
                    .and_then(|arr| arr.get(0))
                    .and_then(|item| item.get("drvPath"))
                    .and_then(|drv| drv.as_str())
                    .ok_or_else(|| McpError::internal_error("Failed to get derivation path B".to_string(), None))?;

                // Run nix-diff
                let output = tokio::process::Command::new("nix-diff")
                    .args([drv_a, drv_b])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to run nix-diff: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = if !stdout.is_empty() {
                    format!("Differences between {} and {}:\n\n{}", package_a, package_b, stdout)
                } else if !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    format!("Packages {} and {} have identical derivations (no differences found).", package_a, package_b)
                };

                Ok(CallToolResult::success(vec![Content::text(result)]))
            }).await
        }).await
    }

    // Clan integration tools

    #[tool(description = "Create a new Clan machine configuration")]
    async fn clan_machine_create(
        &self,
        Parameters(ClanMachineCreateArgs {
            name,
            template,
            target_host,
            flake,
        }): Parameters<ClanMachineCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&name).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 60s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_create",
            Some(serde_json::json!({"name": &name, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_create", 60, || async {
                    let mut args = vec!["machines", "create", &name];

                    let template_str = template.unwrap_or_else(|| "new-machine".to_string());
                    args.push("-t");
                    args.push(&template_str);

                    args.push("--flake");
                    args.push(&flake_str);

                    let target_host_str;
                    if let Some(ref host) = target_host {
                        target_host_str = host.clone();
                        args.push("--target-host");
                        args.push(&target_host_str);
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to create machine '{}':\n\n{}{}",
                            name, stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully created machine '{}'.\n\n{}{}",
                        name, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "List all Clan machines in the flake",
        annotations(read_only_hint = true)
    )]
    async fn clan_machine_list(
        &self,
        Parameters(ClanMachineListArgs { flake }): Parameters<ClanMachineListArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_list",
            Some(serde_json::json!({"flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_list", 30, || async {
                    let output = tokio::process::Command::new("clan")
                        .args(["machines", "list", "--flake", &flake_str])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to list machines:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    let result = if stdout.trim().is_empty() {
                        "No machines configured in this Clan flake.".to_string()
                    } else {
                        format!("Clan Machines:\n\n{}", stdout)
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Update Clan machine(s) - rebuilds and deploys configuration",
        annotations(destructive_hint = true)
    )]
    async fn clan_machine_update(
        &self,
        Parameters(ClanMachineUpdateArgs { machines, flake }): Parameters<ClanMachineUpdateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Validate machine names if provided
        if let Some(ref m) = machines {
            for machine in m {
                validate_machine_name(machine).map_err(validation_error_to_mcp)?;
            }
        }

        // Log dangerous operation
        let machines_desc = machines
            .as_ref()
            .map(|m| m.join(", "))
            .unwrap_or_else(|| "all machines".to_string());
        self.audit.log_dangerous_operation(
            "clan_machine_update",
            true,
            &format!("Updating machines: {}", machines_desc),
        );

        // Execute with security features (audit logging + 300s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_update",
            Some(serde_json::json!({"machines": &machines, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_update", 300, || async {
                    let mut args = vec!["machines", "update"];

                    args.push("--flake");
                    args.push(&flake_str);

                    let machine_names: Vec<String>;
                    if let Some(ref m) = machines {
                        machine_names = m.clone();
                        for machine in &machine_names {
                            args.push(machine);
                        }
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Machine update failed:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Machine update completed.\n\n{}{}",
                        stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Delete a Clan machine configuration",
        annotations(destructive_hint = true)
    )]
    async fn clan_machine_delete(
        &self,
        Parameters(ClanMachineDeleteArgs { name, flake }): Parameters<ClanMachineDeleteArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&name).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Log dangerous operation
        self.audit.log_dangerous_operation(
            "clan_machine_delete",
            true,
            &format!("Deleting machine: {}", name),
        );

        // Execute with security features (audit logging + 60s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_delete",
            Some(serde_json::json!({"name": &name, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_delete", 60, || async {
                    let output = tokio::process::Command::new("clan")
                        .args(["machines", "delete", &name, "--flake", &flake_str])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to delete machine '{}':\n\n{}{}",
                            name, stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully deleted machine '{}'.\n\n{}{}",
                        name, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Install Clan machine to a target host via SSH (WARNING: Destructive - overwrites disk)",
        annotations(destructive_hint = true)
    )]
    async fn clan_machine_install(
        &self,
        Parameters(ClanMachineInstallArgs {
            machine,
            target_host,
            flake,
            confirm,
        }): Parameters<ClanMachineInstallArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Require user confirmation for this destructive operation
        if !confirm.unwrap_or(false) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "WARNING: Installing machine '{}' to '{}' will OVERWRITE THE DISK!\n\n\
                    This is a destructive operation that will:\n\
                    - Partition and format the target disk\n\
                    - Install NixOS\n\
                    - Deploy the Clan configuration\n\n\
                    To proceed, call this function again with confirm=true",
                machine, target_host
            ))]));
        }

        // Log dangerous operation approval
        self.audit.log_dangerous_operation(
            "clan_machine_install",
            true,
            &format!(
                "Installing machine '{}' to '{}' (user confirmed)",
                machine, target_host
            ),
        );

        // Execute with security features (audit logging + 600s timeout for install)
        audit_tool_execution(&self.audit, "clan_machine_install", Some(serde_json::json!({"machine": &machine, "target_host": &target_host, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_machine_install", 600, || async {
                let output = tokio::process::Command::new("clan")
                    .args(["machines", "install", &machine, &target_host, "--flake", &flake_str])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Machine installation failed:\n\n{}{}", stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Machine '{}' successfully installed to '{}'.\n\n{}{}", machine, target_host, stdout, stderr)
        )]))
            }).await
        }).await
    }

    #[tool(description = "Create a backup for a Clan machine")]
    async fn clan_backup_create(
        &self,
        Parameters(ClanBackupCreateArgs {
            machine,
            provider,
            flake,
        }): Parameters<ClanBackupCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_backup_create",
            Some(serde_json::json!({"machine": &machine, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_backup_create", 120, || async {
                    let mut args = vec!["backups", "create", &machine];

                    args.push("--flake");
                    args.push(&flake_str);

                    let provider_str;
                    if let Some(ref p) = provider {
                        provider_str = p.clone();
                        args.push("--provider");
                        args.push(&provider_str);
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Backup creation failed:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Backup created for machine '{}'.\n\n{}{}",
                        machine, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "List backups for a Clan machine",
        annotations(read_only_hint = true)
    )]
    async fn clan_backup_list(
        &self,
        Parameters(ClanBackupListArgs {
            machine,
            provider,
            flake,
        }): Parameters<ClanBackupListArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_backup_list",
            Some(serde_json::json!({"machine": &machine, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_backup_list", 30, || async {
                    let mut args = vec!["backups", "list", &machine];

                    args.push("--flake");
                    args.push(&flake_str);

                    let provider_str;
                    if let Some(ref p) = provider {
                        provider_str = p.clone();
                        args.push("--provider");
                        args.push(&provider_str);
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to list backups:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    let result = if stdout.trim().is_empty() {
                        format!("No backups found for machine '{}'.", machine)
                    } else {
                        format!("Backups for machine '{}':\n\n{}", machine, stdout)
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Restore a backup for a Clan machine",
        annotations(destructive_hint = true)
    )]
    async fn clan_backup_restore(
        &self,
        Parameters(ClanBackupRestoreArgs {
            machine,
            provider,
            name,
            service,
            flake,
        }): Parameters<ClanBackupRestoreArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Validate backup name (basic alphanumeric check)
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(McpError::invalid_params(
                "Invalid backup name: must be non-empty alphanumeric with dashes, underscores, or dots",
                None,
            ));
        }

        // Log dangerous operation
        self.audit.log_dangerous_operation(
            "clan_backup_restore",
            true,
            &format!("Restoring backup '{}' for machine '{}'", name, machine),
        );

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_backup_restore",
            Some(serde_json::json!({"machine": &machine, "backup": &name, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_backup_restore", 120, || async {
                    let mut args = vec!["backups", "restore", &machine, &provider, &name];

                    args.push("--flake");
                    args.push(&flake_str);

                    let service_str;
                    if let Some(ref s) = service {
                        service_str = s.clone();
                        args.push("--service");
                        args.push(&service_str);
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Backup restore failed:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Backup '{}' restored for machine '{}'.\n\n{}{}",
                        name, machine, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Create a new Clan flake from a template")]
    async fn clan_flake_create(
        &self,
        Parameters(ClanFlakeCreateArgs {
            directory,
            template,
        }): Parameters<ClanFlakeCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate directory path
        validate_path(&directory).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 60s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_flake_create",
            Some(serde_json::json!({"directory": &directory})),
            || async {
                with_timeout(&self.audit, "clan_flake_create", 60, || async {
                    let mut args = vec!["flakes", "create", &directory];

                    let template_str;
                    if let Some(ref t) = template {
                        template_str = t.clone();
                        args.push("--template");
                        args.push(&template_str);
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to create Clan flake:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Clan flake created in '{}'.\n\n{}{}",
                        directory, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "List secrets in a Clan flake",
        annotations(read_only_hint = true)
    )]
    async fn clan_secrets_list(
        &self,
        Parameters(ClanSecretsListArgs { flake }): Parameters<ClanSecretsListArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_secrets_list",
            Some(serde_json::json!({"flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_secrets_list", 30, || async {
                    let output = tokio::process::Command::new("clan")
                        .args(["secrets", "list", "--flake", &flake_str])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to list secrets:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    let result = if stdout.trim().is_empty() {
                        "No secrets configured.".to_string()
                    } else {
                        format!("Clan Secrets:\n\n{}", stdout)
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Create and run a VM for a Clan machine (useful for testing)")]
    async fn clan_vm_create(
        &self,
        Parameters(ClanVmCreateArgs { machine, flake }): Parameters<ClanVmCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(&self.audit, "clan_vm_create", Some(serde_json::json!({"machine": &machine, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_vm_create", 120, || async {
                let output = tokio::process::Command::new("clan")
                    .args(["vms", "create", &machine, "--flake", &flake_str])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("VM creation failed:\n\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("VM created for machine '{}'.\n\n{}{}\n\nNote: This creates a VM configuration. Use 'clan vms run {}' to start it.", machine, stdout, stderr, machine)
                )]))
            }).await
        }).await
    }

    #[tool(
        description = "Get help and information about Clan - the peer-to-peer NixOS management framework"
    )]
    fn clan_help(
        &self,
        Parameters(_args): Parameters<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let help_text = r#"Clan - Peer-to-Peer NixOS Management Framework

Clan is a framework built on NixOS that enables declarative, collaborative management
of distributed systems. It provides tools for managing machines, backups, secrets, and more.

KEY CONCEPTS:

1. Clan Flake
   A Git repository containing your infrastructure as code:
   - Machine configurations
   - Shared services and modules
   - Secrets and variables
   - Network topology

2. Machines
   Individual systems managed by Clan. Each machine has:
   - Hardware configuration
   - NixOS configuration
   - Service definitions
   - Access to shared secrets

3. Services
   Modular components that add functionality:
   - Networking (VPN, mesh networks)
   - Backups (automated, versioned)
   - Monitoring and observability
   - Custom application stacks

AVAILABLE TOOLS:

Machine Management:
- clan_machine_create - Create new machine configurations
- clan_machine_list - List all machines in flake
- clan_machine_update - Update/deploy machine configurations
- clan_machine_delete - Remove machine configurations
- clan_machine_install - Install NixOS to a remote host (destructive!)

Backup Operations:
- clan_backup_create - Create backups for machines
- clan_backup_list - List available backups
- clan_backup_restore - Restore from backup

Flake & Project:
- clan_flake_create - Initialize new Clan project

Secrets:
- clan_secrets_list - View configured secrets

Testing:
- clan_vm_create - Create VMs for testing configurations

COMMON WORKFLOWS:

1. Creating a New Clan Project:
   clan_flake_create(directory="my-infrastructure")

2. Adding a Machine:
   clan_machine_create(name="webserver", target_host="192.168.1.10")

3. Deploying to Production:
   clan_machine_install(machine="webserver", target_host="192.168.1.10", confirm=true)

4. Regular Updates:
   clan_machine_update(machines=["webserver"])

5. Backup & Restore:
   clan_backup_create(machine="webserver")
   clan_backup_list(machine="webserver")
   clan_backup_restore(machine="webserver", provider="borgbackup", name="2024-12-01")

DOCUMENTATION:
- Main docs: https://docs.clan.lol
- Repository: https://git.clan.lol/clan/clan-core
- Option search: https://docs.clan.lol/option-search/

BENEFITS:
- Declarative infrastructure (everything in Git)
- Peer-to-peer collaboration
- Reproducible builds (Nix)
- Integrated backups and secrets
- Testing with VMs before deployment
"#;

        Ok(CallToolResult::success(vec![Content::text(help_text)]))
    }

    #[tool(
        description = "Find which package provides a specific file path using nix-locate",
        annotations(read_only_hint = true)
    )]
    async fn nix_locate(
        &self,
        Parameters(NixLocateArgs { path, limit }): Parameters<NixLocateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Basic validation
        if path.is_empty() || path.contains('\0') {
            return Err(McpError::invalid_params(
                "Invalid path".to_string(),
                Some(serde_json::json!({"path": path})),
            ));
        }

        // Create cache key including limit
        let cache_key = format!("{}:{}", path, limit.unwrap_or(20));

        // Check cache first
        if let Some(cached_result) = self.locate_cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Wrap tool logic with security
        let locate_cache = self.locate_cache.clone();
        let cache_key_clone = cache_key.clone();

        audit_tool_execution(
            &self.audit,
            "nix_locate",
            Some(serde_json::json!({"path": &path, "limit": &limit})),
            || async move {
                with_timeout(&self.audit, "nix_locate", 60, || async {
                    // Try local nix-locate first (needs pre-built database)
                    let output = tokio::process::Command::new("nix-locate")
                        .arg("--whole-name")
                        .arg(&path)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to execute nix-locate: {}. Install with: nix-shell -p nix-index\nThen build database: nix-index", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("command not found") || stderr.contains("No such file") {
                            return Ok(CallToolResult::success(vec![Content::text(
                                "nix-locate is not available. Install it with: nix-shell -p nix-index\n\
                                Then build the database with: nix-index\n\
                                This may take several minutes on first run.".to_string()
                            )]));
                        }
                        return Err(McpError::internal_error(
                            format!("nix-locate failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<&str> = stdout.lines().collect();

                    let result = if lines.is_empty() {
                        format!("No packages found providing '{}'", path)
                    } else {
                        let limit = limit.unwrap_or(20);
                        let results: Vec<&str> = lines.iter().take(limit).copied().collect();
                        let total = lines.len();

                        let mut output =
                            format!("Found {} package(s) providing '{}':\n\n", total, path);
                        output.push_str(&results.join("\n"));

                        if total > limit {
                            output.push_str(&format!(
                                "\n\n... and {} more results (showing top {})",
                                total - limit,
                                limit
                            ));
                        }
                        output
                    };

                    // Cache the result
                    locate_cache.insert(cache_key_clone, result.clone());

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Run an application from nixpkgs without installing it",
        annotations(read_only_hint = false)
    )]
    async fn nix_run(
        &self,
        Parameters(NixRunArgs { package, args }): Parameters<NixRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate package/flake reference (accepts nixpkgs#hello format)
        validate_flake_ref(&package).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_run",
            Some(serde_json::json!({"package": &package, "args": &args})),
            || async {
                with_timeout(&self.audit, "nix_run", 300, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg(&package);

                    if let Some(program_args) = args {
                        cmd.arg("--");
                        for arg in program_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute nix run: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str("STDOUT:\n");
                        result.push_str(&stdout);
                        result.push_str("\n");
                    }
                    if !stderr.is_empty() {
                        result.push_str("STDERR:\n");
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = format!(
                            "Command completed successfully (exit code: {})",
                            output.status.code().unwrap_or(0)
                        );
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(
                            format!("nix run failed: {}", result),
                            None,
                        ));
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Run a command in a Nix development environment (from flake.nix devShell)",
        annotations(read_only_hint = false)
    )]
    async fn nix_develop(
        &self,
        Parameters(NixDevelopArgs {
            flake_ref,
            command,
            args,
        }): Parameters<NixDevelopArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate flake reference if provided
        if let Some(ref fref) = flake_ref {
            validate_flake_ref(fref).map_err(validation_error_to_mcp)?;
        }

        // Validate command
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_develop",
            Some(serde_json::json!({"flake_ref": &flake_ref, "command": &command, "args": &args})),
            || async {
                with_timeout(&self.audit, "nix_develop", 300, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("develop");

                    if let Some(ref fref) = flake_ref {
                        cmd.arg(fref);
                    }

                    cmd.arg("-c").arg(&command);

                    if let Some(command_args) = args {
                        for arg in command_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute nix develop: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str("STDOUT:\n");
                        result.push_str(&stdout);
                        result.push_str("\n");
                    }
                    if !stderr.is_empty() {
                        result.push_str("STDERR:\n");
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = format!(
                            "Command '{}' completed successfully in development environment",
                            command
                        );
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(
                            format!("nix develop failed: {}", result),
                            None,
                        ));
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Format Nix code using the project's formatter (typically nix fmt)",
        annotations(read_only_hint = false)
    )]
    async fn nix_fmt(
        &self,
        Parameters(NixFmtArgs { path }): Parameters<NixFmtArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate path if provided
        if let Some(ref p) = path {
            validate_path(p).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "nix_fmt",
            Some(serde_json::json!({"path": &path})),
            || async {
                with_timeout(&self.audit, "nix_fmt", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("fmt");

                    if let Some(p) = path {
                        cmd.arg(p);
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute nix fmt: {}", e), None)
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("nix fmt failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = "Code formatted successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Add a command to the pueue task queue for async execution. Returns task ID.",
        annotations(read_only_hint = false)
    )]
    async fn pueue_add(
        &self,
        Parameters(PueueAddArgs {
            command,
            args,
            working_directory,
            label,
        }): Parameters<PueueAddArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate command
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Validate working directory if provided
        if let Some(ref wd) = working_directory {
            use crate::common::security::validate_path;
            validate_path(wd).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_add",
            Some(serde_json::json!({"command": &command, "args": &args, "working_directory": &working_directory, "label": &label})),
            || async {
                with_timeout(&self.audit, "pueue_add", 30, || async {
                    // Use nix run to ensure pueue is available
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("add");

                    if let Some(wd) = working_directory {
                        cmd.arg("--working-directory").arg(wd);
                    }

                    if let Some(lbl) = label {
                        cmd.arg("--label").arg(lbl);
                    }

                    cmd.arg("--");
                    cmd.arg(&command);

                    if let Some(command_args) = args {
                        for arg in command_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue add via nix run: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue add failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        stdout.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get the status of pueue tasks (all or specific task IDs)",
        annotations(read_only_hint = true)
    )]
    async fn pueue_status(
        &self,
        Parameters(PueueStatusArgs { task_ids }): Parameters<PueueStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_status",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_status", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("status");

                    if let Some(ids) = task_ids {
                        for id in ids.split(',') {
                            cmd.arg(id.trim());
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue status: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue status failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        stdout.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Get logs for a specific pueue task",
        annotations(read_only_hint = true)
    )]
    async fn pueue_log(
        &self,
        Parameters(PueueLogArgs { task_id, lines }): Parameters<PueueLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_log",
            Some(serde_json::json!({"task_id": &task_id, "lines": &lines})),
            || async {
                with_timeout(&self.audit, "pueue_log", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#pueue")
                        .arg("--")
                        .arg("log")
                        .arg(task_id.to_string());

                    if let Some(n) = lines {
                        cmd.arg("--lines").arg(n.to_string());
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue log: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue log failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(
                        stdout.to_string(),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Wait for specific pueue tasks to complete",
        annotations(read_only_hint = true)
    )]
    async fn pueue_wait(
        &self,
        Parameters(PueueWaitArgs { task_ids, timeout }): Parameters<PueueWaitArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Validate task IDs format
        if task_ids.is_empty() || task_ids.contains('\0') {
            return Err(McpError::invalid_params(
                "Invalid task_ids".to_string(),
                Some(serde_json::json!({"task_ids": task_ids})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_wait",
            Some(serde_json::json!({"task_ids": &task_ids, "timeout": &timeout})),
            || async {
                // Use custom timeout for wait command
                let wait_timeout = timeout.unwrap_or(300);

                let timeout_duration = tokio::time::Duration::from_secs(wait_timeout);
                let result = tokio::time::timeout(timeout_duration, async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("wait");

                    for id in task_ids.split(',') {
                        cmd.arg(id.trim());
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue wait: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue wait failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = format!("Task(s) {} completed successfully", task_ids);
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("pueue wait timed out after {} seconds", wait_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Remove/kill specific pueue tasks",
        annotations(read_only_hint = false)
    )]
    async fn pueue_remove(
        &self,
        Parameters(PueueRemoveArgs { task_ids }): Parameters<PueueRemoveArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate task IDs format
        if task_ids.is_empty() || task_ids.contains('\0') {
            return Err(McpError::invalid_params(
                "Invalid task_ids".to_string(),
                Some(serde_json::json!({"task_ids": task_ids})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_remove",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_remove", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("remove");

                    for id in task_ids.split(',') {
                        cmd.arg(id.trim());
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue remove: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue remove failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = format!("Task(s) {} removed successfully", task_ids);
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Clean up finished pueue tasks from the queue",
        annotations(read_only_hint = false)
    )]
    async fn pueue_clean(
        &self,
        Parameters(_): Parameters<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(&self.audit, "pueue_clean", None, || async {
            with_timeout(&self.audit, "pueue_clean", 30, || async {
                let output = tokio::process::Command::new("nix")
                    .arg("run")
                    .arg("nixpkgs#pueue")
                    .arg("--")
                    .arg("clean")
                    .output()
                    .await
                    .map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue clean: {}", e),
                            None,
                        )
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(McpError::internal_error(
                        format!("pueue clean failed: {}", stderr),
                        None,
                    ));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut result_text = stdout.to_string();
                if result_text.is_empty() {
                    result_text = "Finished tasks cleaned successfully".to_string();
                }

                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            })
            .await
        })
        .await
    }

    #[tool(
        description = "Pause specific pueue tasks or all tasks",
        annotations(read_only_hint = false)
    )]
    async fn pueue_pause(
        &self,
        Parameters(PueuePauseArgs { task_ids }): Parameters<PueuePauseArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_pause",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_pause", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("pause");

                    if let Some(ids) = task_ids {
                        for id in ids.split(',') {
                            cmd.arg(id.trim());
                        }
                    } else {
                        cmd.arg("--all");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue pause: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue pause failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = "Task(s) paused successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Start/resume specific pueue tasks or all tasks",
        annotations(read_only_hint = false)
    )]
    async fn pueue_start(
        &self,
        Parameters(PueueStartArgs { task_ids }): Parameters<PueueStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pueue_start",
            Some(serde_json::json!({"task_ids": &task_ids})),
            || async {
                with_timeout(&self.audit, "pueue_start", 30, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#pueue").arg("--").arg("start");

                    if let Some(ids) = task_ids {
                        for id in ids.split(',') {
                            cmd.arg(id.trim());
                        }
                    } else {
                        cmd.arg("--all");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pueue start: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pueue start failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut result_text = stdout.to_string();
                    if result_text.is_empty() {
                        result_text = "Task(s) started successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result_text)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Start a new pexpect-cli interactive session. Returns session ID.",
        annotations(read_only_hint = false)
    )]
    async fn pexpect_start(
        &self,
        Parameters(PexpectStartArgs { command, args }): Parameters<PexpectStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate command
        validate_command(&command).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pexpect_start",
            Some(serde_json::json!({"command": &command, "args": &args})),
            || async {
                with_timeout(&self.audit, "pexpect_start", 30, || async {
                    // Use nix run to ensure pexpect-cli is available
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#python3Packages.pexpect-cli")
                        .arg("--")
                        .arg("--start")
                        .arg(&command);

                    if let Some(command_args) = args {
                        for arg in command_args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pexpect-cli via nix run: {}", e),
                            None,
                        )
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("pexpect-cli failed: {}", stderr),
                            None,
                        ));
                    }

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Session started successfully. Session ID: {}",
                        stdout.trim()
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Send Python pexpect code to an active session",
        annotations(read_only_hint = false)
    )]
    async fn pexpect_send(
        &self,
        Parameters(PexpectSendArgs { session_id, code }): Parameters<PexpectSendArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate session ID format (should be alphanumeric)
        if session_id.is_empty()
            || session_id.contains('\0')
            || !session_id.chars().all(|c| c.is_alphanumeric())
        {
            return Err(McpError::invalid_params(
                "Invalid session_id".to_string(),
                Some(serde_json::json!({"session_id": session_id})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pexpect_send",
            Some(serde_json::json!({"session_id": &session_id, "code": &code})),
            || async {
                with_timeout(&self.audit, "pexpect_send", 60, || async {
                    // Use nix run via shell to pipe input to pexpect-cli
                    let mut cmd = tokio::process::Command::new("sh");
                    cmd.arg("-c");
                    cmd.arg(format!(
                        "echo '{}' | nix run nixpkgs#python3Packages.pexpect-cli -- {}",
                        code, session_id
                    ));

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute pexpect-cli via nix run: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str("STDERR:\n");
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = "Command sent successfully (no output)".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Close an active pexpect-cli session",
        annotations(read_only_hint = false)
    )]
    async fn pexpect_close(
        &self,
        Parameters(PexpectCloseArgs { session_id }): Parameters<PexpectCloseArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate session ID format
        if session_id.is_empty()
            || session_id.contains('\0')
            || !session_id.chars().all(|c| c.is_alphanumeric())
        {
            return Err(McpError::invalid_params(
                "Invalid session_id".to_string(),
                Some(serde_json::json!({"session_id": session_id})),
            ));
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "pexpect_close",
            Some(serde_json::json!({"session_id": &session_id})),
            || async {
                with_timeout(&self.audit, "pexpect_close", 30, || async {
                    // Use nix run via shell to close pexpect session
                    let output = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!(
                            "echo 'child.close()' | nix run nixpkgs#python3Packages.pexpect-cli -- {}",
                            session_id
                        ))
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to close pexpect session via nix run: {}", e),
                                None,
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(McpError::internal_error(
                            format!("Failed to close session: {}", stderr),
                            None,
                        ));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Session {} closed successfully",
                        session_id
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Build a Rust project with cargo build (with timeout protection)",
        annotations(read_only_hint = false)
    )]
    async fn cargo_build(
        &self,
        Parameters(CargoBuildArgs {
            release,
            package,
            extra_args,
            timeout,
        }): Parameters<CargoBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "cargo_build",
            Some(serde_json::json!({"release": &release, "package": &package, "extra_args": &extra_args, "timeout": &timeout})),
            || async {
                let build_timeout = timeout.unwrap_or(600);
                let timeout_duration = tokio::time::Duration::from_secs(build_timeout);

                let result = tokio::time::timeout(timeout_duration, async {
                    let mut cmd = tokio::process::Command::new("cargo");
                    cmd.arg("build");

                    if release.unwrap_or(false) {
                        cmd.arg("--release");
                    }

                    if let Some(pkg) = package {
                        cmd.arg("-p").arg(pkg);
                    }

                    if let Some(args) = extra_args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute cargo build: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(format!("Build failed:\n{}", result), None));
                    }

                    if result.is_empty() {
                        result = "Build completed successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("cargo build timed out after {} seconds", build_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Run Rust tests with cargo test (with timeout protection)",
        annotations(read_only_hint = false)
    )]
    async fn cargo_test(
        &self,
        Parameters(CargoTestArgs {
            test_pattern,
            package,
            extra_args,
            timeout,
        }): Parameters<CargoTestArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "cargo_test",
            Some(serde_json::json!({"test_pattern": &test_pattern, "package": &package, "extra_args": &extra_args, "timeout": &timeout})),
            || async {
                let test_timeout = timeout.unwrap_or(600);
                let timeout_duration = tokio::time::Duration::from_secs(test_timeout);

                let result = tokio::time::timeout(timeout_duration, async {
                    let mut cmd = tokio::process::Command::new("cargo");
                    cmd.arg("test");

                    if let Some(pkg) = package {
                        cmd.arg("-p").arg(pkg);
                    }

                    if let Some(pattern) = test_pattern {
                        cmd.arg(pattern);
                    }

                    if let Some(args) = extra_args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute cargo test: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(format!("Tests failed:\n{}", result), None));
                    }

                    if result.is_empty() {
                        result = "All tests passed".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("cargo test timed out after {} seconds", test_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Run Rust tests with cargo nextest (with timeout protection)",
        annotations(read_only_hint = false)
    )]
    async fn cargo_nextest(
        &self,
        Parameters(CargoNextestArgs {
            test_pattern,
            package,
            extra_args,
            timeout,
        }): Parameters<CargoNextestArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "cargo_nextest",
            Some(serde_json::json!({"test_pattern": &test_pattern, "package": &package, "extra_args": &extra_args, "timeout": &timeout})),
            || async {
                let test_timeout = timeout.unwrap_or(600);
                let timeout_duration = tokio::time::Duration::from_secs(test_timeout);

                let result = tokio::time::timeout(timeout_duration, async {
                    // Use nix shell to ensure cargo-nextest is available
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("shell")
                        .arg("nixpkgs#cargo-nextest")
                        .arg("-c")
                        .arg("cargo")
                        .arg("nextest")
                        .arg("run");

                    if let Some(pkg) = package {
                        cmd.arg("-p").arg(pkg);
                    }

                    if let Some(pattern) = test_pattern {
                        cmd.arg(pattern);
                    }

                    if let Some(args) = extra_args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute cargo nextest via nix shell: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(format!("Tests failed:\n{}", result), None));
                    }

                    if result.is_empty() {
                        result = "All tests passed".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("cargo nextest timed out after {} seconds", test_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Run Rust linter with cargo clippy (with timeout protection)",
        annotations(read_only_hint = true)
    )]
    async fn cargo_clippy(
        &self,
        Parameters(CargoClippyArgs {
            all_targets,
            package,
            extra_args,
            timeout,
        }): Parameters<CargoClippyArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "cargo_clippy",
            Some(serde_json::json!({"all_targets": &all_targets, "package": &package, "extra_args": &extra_args, "timeout": &timeout})),
            || async {
                let clippy_timeout = timeout.unwrap_or(300);
                let timeout_duration = tokio::time::Duration::from_secs(clippy_timeout);

                let result = tokio::time::timeout(timeout_duration, async {
                    let mut cmd = tokio::process::Command::new("cargo");
                    cmd.arg("clippy");

                    if all_targets.unwrap_or(false) {
                        cmd.arg("--all-targets");
                    }

                    if let Some(pkg) = package {
                        cmd.arg("-p").arg(pkg);
                    }

                    if let Some(args) = extra_args {
                        cmd.arg("--");
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute cargo clippy: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(format!("Clippy found issues:\n{}", result), None));
                    }

                    if result.is_empty() {
                        result = "No clippy warnings found".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("cargo clippy timed out after {} seconds", clippy_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Build a project with make (with timeout protection)",
        annotations(read_only_hint = false)
    )]
    async fn make_build(
        &self,
        Parameters(MakeBuildArgs {
            target,
            working_directory,
            jobs,
            extra_args,
            timeout,
        }): Parameters<MakeBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::audit_tool_execution;

        // Validate working directory if provided
        if let Some(ref wd) = working_directory {
            use crate::common::security::validate_path;
            validate_path(wd).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "make_build",
            Some(serde_json::json!({"target": &target, "working_directory": &working_directory, "jobs": &jobs, "extra_args": &extra_args, "timeout": &timeout})),
            || async {
                let make_timeout = timeout.unwrap_or(600);
                let timeout_duration = tokio::time::Duration::from_secs(make_timeout);

                let result = tokio::time::timeout(timeout_duration, async {
                    let mut cmd = tokio::process::Command::new("make");

                    if let Some(wd) = working_directory {
                        cmd.current_dir(wd);
                    }

                    if let Some(j) = jobs {
                        cmd.arg(format!("-j{}", j));
                    }

                    if let Some(args) = extra_args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    if let Some(tgt) = target {
                        cmd.arg(tgt);
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(format!("Failed to execute make: {}", e), None)
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if !output.status.success() {
                        return Err(McpError::internal_error(format!("Make failed:\n{}", result), None));
                    }

                    if result.is_empty() {
                        result = "Make completed successfully".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await;

                match result {
                    Ok(r) => r,
                    Err(_) => Err(McpError::internal_error(
                        format!("make timed out after {} seconds", make_timeout),
                        None,
                    )),
                }
            },
        )
        .await
    }

    #[tool(
        description = "Check shell scripts for issues with shellcheck",
        annotations(read_only_hint = true)
    )]
    async fn shellcheck(
        &self,
        Parameters(ShellcheckArgs {
            path,
            shell,
            severity,
        }): Parameters<ShellcheckArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate path
        validate_path(&path).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "shellcheck",
            Some(serde_json::json!({"path": &path, "shell": &shell, "severity": &severity})),
            || async {
                with_timeout(&self.audit, "shellcheck", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#shellcheck")
                        .arg("--")
                        .arg(&path);

                    if let Some(s) = shell {
                        cmd.arg("--shell").arg(s);
                    }

                    if let Some(sev) = severity {
                        cmd.arg("--severity").arg(sev);
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute shellcheck via nix run: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if result.is_empty() {
                        result = "No issues found".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Check Python code with ruff linter",
        annotations(read_only_hint = false)
    )]
    async fn ruff_check(
        &self,
        Parameters(RuffCheckArgs {
            path,
            fix,
            extra_args,
        }): Parameters<RuffCheckArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate path
        validate_path(&path).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "ruff_check",
            Some(serde_json::json!({"path": &path, "fix": &fix, "extra_args": &extra_args})),
            || async {
                with_timeout(&self.audit, "ruff_check", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#ruff")
                        .arg("--")
                        .arg("check")
                        .arg(&path);

                    if fix.unwrap_or(false) {
                        cmd.arg("--fix");
                    }

                    if let Some(args) = extra_args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute ruff check via nix run: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if result.is_empty() {
                        result = "All checks passed".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Format Python code with ruff formatter",
        annotations(read_only_hint = false)
    )]
    async fn ruff_format(
        &self,
        Parameters(RuffFormatArgs { path, check }): Parameters<RuffFormatArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate path
        validate_path(&path).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "ruff_format",
            Some(serde_json::json!({"path": &path, "check": &check})),
            || async {
                with_timeout(&self.audit, "ruff_format", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run")
                        .arg("nixpkgs#ruff")
                        .arg("--")
                        .arg("format")
                        .arg(&path);

                    if check.unwrap_or(false) {
                        cmd.arg("--check");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute ruff format via nix run: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stdout);
                    }

                    if result.is_empty() {
                        result = if check.unwrap_or(false) {
                            "Code is properly formatted".to_string()
                        } else {
                            "Code formatted successfully".to_string()
                        };
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Check Python types with mypy",
        annotations(read_only_hint = true)
    )]
    async fn mypy(
        &self,
        Parameters(MypyArgs { path, extra_args }): Parameters<MypyArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate path
        validate_path(&path).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "mypy",
            Some(serde_json::json!({"path": &path, "extra_args": &extra_args})),
            || async {
                with_timeout(&self.audit, "mypy", 120, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#mypy").arg("--").arg(&path);

                    if let Some(args) = extra_args {
                        for arg in args {
                            cmd.arg(arg);
                        }
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute mypy via nix run: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = "No type errors found".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Audit Rust dependencies for security vulnerabilities",
        annotations(read_only_hint = true)
    )]
    async fn cargo_audit(
        &self,
        Parameters(CargoAuditArgs {
            path,
            deny_warnings,
        }): Parameters<CargoAuditArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate path if provided
        if let Some(ref p) = path {
            use crate::common::security::validate_path;
            validate_path(p).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "cargo_audit",
            Some(serde_json::json!({"path": &path, "deny_warnings": &deny_warnings})),
            || async {
                with_timeout(&self.audit, "cargo_audit", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("shell")
                        .arg("nixpkgs#cargo-audit")
                        .arg("-c")
                        .arg("cargo")
                        .arg("audit");

                    if let Some(p) = path {
                        cmd.current_dir(p);
                    }

                    if deny_warnings.unwrap_or(false) {
                        cmd.arg("--deny").arg("warnings");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute cargo audit via nix shell: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = "No vulnerabilities found".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Check Rust dependencies for license compliance and bans",
        annotations(read_only_hint = true)
    )]
    async fn cargo_deny(
        &self,
        Parameters(CargoDenyArgs { path, check_type }): Parameters<CargoDenyArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};

        // Validate path if provided
        if let Some(ref p) = path {
            use crate::common::security::validate_path;
            validate_path(p).map_err(validation_error_to_mcp)?;
        }

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "cargo_deny",
            Some(serde_json::json!({"path": &path, "check_type": &check_type})),
            || async {
                with_timeout(&self.audit, "cargo_deny", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("shell")
                        .arg("nixpkgs#cargo-deny")
                        .arg("-c")
                        .arg("cargo")
                        .arg("deny")
                        .arg("check");

                    if let Some(p) = path {
                        cmd.current_dir(p);
                    }

                    if let Some(ct) = check_type {
                        cmd.arg(ct);
                    }

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute cargo deny via nix shell: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = "All checks passed".to_string();
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Format TOML files with taplo",
        annotations(read_only_hint = false)
    )]
    async fn taplo(
        &self,
        Parameters(TaploArgs { path, check }): Parameters<TaploArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_path;

        // Validate path
        validate_path(&path).map_err(validation_error_to_mcp)?;

        // Wrap tool logic with security
        audit_tool_execution(
            &self.audit,
            "taplo",
            Some(serde_json::json!({"path": &path, "check": &check})),
            || async {
                with_timeout(&self.audit, "taplo", 60, || async {
                    let mut cmd = tokio::process::Command::new("nix");
                    cmd.arg("run").arg("nixpkgs#taplo").arg("--").arg("fmt");

                    if check.unwrap_or(false) {
                        cmd.arg("--check");
                    }

                    cmd.arg(&path);

                    let output = cmd.output().await.map_err(|e| {
                        McpError::internal_error(
                            format!("Failed to execute taplo via nix run: {}", e),
                            None,
                        )
                    })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str(&stdout);
                    }
                    if !stderr.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n");
                        }
                        result.push_str(&stderr);
                    }

                    if result.is_empty() {
                        result = if check.unwrap_or(false) {
                            "TOML files are properly formatted".to_string()
                        } else {
                            "TOML files formatted successfully".to_string()
                        };
                    }

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }
}

#[prompt_router]
impl NixServer {
    /// Generate a nix flake template based on requirements
    #[prompt(name = "generate_flake")]
    async fn generate_flake(
        &self,
        Parameters(args): Parameters<serde_json::Map<String, serde_json::Value>>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let project_type = args
            .get("project_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        let prompt = format!(
            "Generate a Nix flake.nix file for a {} project. Include appropriate buildInputs, development shell, and package definition.",
            project_type
        );

        Ok(vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::text(prompt),
        }])
    }

    /// Guide for setting up a Nix development environment for a specific project type
    #[prompt(name = "setup_dev_environment")]
    async fn setup_dev_environment(
        &self,
        Parameters(args): Parameters<SetupDevEnvironmentArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let use_flakes = args.use_flakes.unwrap_or(true);
        let deps = args
            .dependencies
            .as_ref()
            .map(|d| d.join(", "))
            .unwrap_or_else(|| "none specified".to_string());

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I need to set up a Nix development environment for a {} project.\n\
                    Additional dependencies: {}\n\
                    Use flakes: {}\n\n\
                    Please provide:\n\
                    1. A complete flake.nix (if using flakes) or shell.nix file\n\
                    2. Explanation of the key components\n\
                    3. Commands to enter and use the development environment\n\
                    4. Best practices for this project type with Nix",
                args.project_type, deps, use_flakes
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!(
                "Setup {} development environment",
                args.project_type
            )),
            messages,
        })
    }

    /// Help troubleshoot Nix build failures with diagnostic guidance
    #[prompt(name = "troubleshoot_build")]
    async fn troubleshoot_build(
        &self,
        Parameters(args): Parameters<TroubleshootBuildArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let error_context = args
            .error_message
            .as_ref()
            .map(|e| format!("\n\nError message:\n{}", e))
            .unwrap_or_default();

        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "I'm having trouble building: {}{}\n\n\
                    Please help me:\n\
                    1. Identify the root cause of the build failure\n\
                    2. Suggest specific debugging commands to run (like nix log, nix why-depends, etc.)\n\
                    3. Provide potential solutions or workarounds\n\
                    4. Explain common patterns that might cause this issue\n\
                    5. Recommend preventive measures for the future",
                    args.package, error_context
                ),
            ),
        ];

        Ok(GetPromptResult {
            description: Some(format!("Troubleshoot build failure for {}", args.package)),
            messages,
        })
    }

    /// Guide for migrating existing projects to Nix flakes
    #[prompt(name = "migrate_to_flakes")]
    async fn migrate_to_flakes(
        &self,
        Parameters(args): Parameters<MigrateToFlakesArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let project_context = args
            .project_type
            .as_ref()
            .map(|p| format!(" for a {} project", p))
            .unwrap_or_default();

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I want to migrate to Nix flakes{}.\n\
                    Current setup: {}\n\n\
                    Please provide:\n\
                    1. Step-by-step migration plan\n\
                    2. Example flake.nix based on my current setup\n\
                    3. How to handle inputs and lock files\n\
                    4. Common pitfalls to avoid\n\
                    5. Benefits I'll gain from using flakes\n\
                    6. Backward compatibility considerations",
                project_context, args.current_setup
            ),
        )];

        Ok(GetPromptResult {
            description: Some("Migrate to Nix flakes".to_string()),
            messages,
        })
    }

    /// Help optimize package closure size with actionable recommendations
    #[prompt(name = "optimize_closure")]
    async fn optimize_closure(
        &self,
        Parameters(args): Parameters<OptimizeClosureArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let size_context = args
            .current_size
            .as_ref()
            .map(|s| format!("\nCurrent closure size: {}", s))
            .unwrap_or_default();
        let target_context = args
            .target
            .as_ref()
            .map(|t| format!("\nTarget: {}", t))
            .unwrap_or_default();

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I need to optimize the closure size for: {}{}{}\n\n\
                    Please help me:\n\
                    1. Analyze dependency tree to identify large dependencies\n\
                    2. Suggest specific packages or features to remove or replace\n\
                    3. Provide Nix expressions to create minimal variants\n\
                    4. Recommend build flags or overrides to reduce size\n\
                    5. Explain trade-offs between size and functionality\n\
                    6. Show how to measure and verify improvements",
                args.package, size_context, target_context
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!("Optimize closure for {}", args.package)),
            messages,
        })
    }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for NixServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .enable_completions()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides comprehensive Nix package management, development tools, and Clan infrastructure management. \
                \n\n=== NIX TOOLS === \
                \n\nPackage Discovery: search_packages, explain_package, get_package_info, find_command \
                \n\nBuild & Execution: nix_build, run_in_shell, get_closure_size, get_build_log \
                \n\nDependency Analysis: why_depends, show_derivation, diff_derivations \
                \n\nFlake Management: flake_metadata, flake_show \
                \n\nCode Quality: validate_nix, lint_nix, format_nix \
                \n\nUtilities: nix_eval, prefetch_url, search_options, nix_command_help, ecosystem_tools \
                \n\n=== CLAN TOOLS === \
                \n\nClan is a peer-to-peer NixOS management framework for declarative infrastructure. \
                \n\nMachine Management: \
                - clan_machine_create - Create new machine configurations \
                - clan_machine_list - List all machines \
                - clan_machine_update - Deploy configurations \
                - clan_machine_delete - Remove machines \
                - clan_machine_install - Install NixOS to remote hosts (DESTRUCTIVE) \
                \n\nBackup Operations: \
                - clan_backup_create - Create backups \
                - clan_backup_list - List backups \
                - clan_backup_restore - Restore from backup \
                \n\nProject & Infrastructure: \
                - clan_flake_create - Initialize new Clan project \
                - clan_secrets_list - View secrets \
                - clan_vm_create - Create VMs for testing \
                - clan_help - Comprehensive Clan documentation \
                \n\n=== KEY CAPABILITIES === \
                - Build packages with nix_build (supports dry-run) \
                - Debug builds with get_build_log \
                - Execute commands in isolated environments with run_in_shell \
                - Analyze package sizes with get_closure_size \
                - Understand dependencies with why_depends and show_derivation \
                - Compare packages with diff_derivations \
                - Manage distributed NixOS infrastructure with Clan \
                - Declarative machine deployment and configuration \
                - Automated backup and restore for Clan machines \
                \n\nIMPORTANT: You can use 'nix-shell -p <package>' to get any nixpkgs package in a temporary shell, \
                or 'nix shell nixpkgs#<package>' with flakes. Use run_in_shell to execute commands in these environments. \
                \n\nFor Clan: All tools support --flake parameter to specify the Clan directory (defaults to current directory)."
                    .to_string(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                self._create_resource_text("nix://commands/common", "Common Nix Commands"),
                self._create_resource_text("nix://ecosystem/tools", "Ecosystem Tools"),
                self._create_resource_text("nix://flake/template", "Flake Template"),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "nix://commands/common" => {
                let content = r#"Common Nix Commands Reference

QUICKEST WAY TO GET ANY PACKAGE:
- nix-shell -p <package>         Get ANY nixpkgs package instantly!
- nix-shell -p <pkg1> <pkg2>     Multiple packages at once
- nix-shell -p gcc --run "gcc --version"  Run command and exit

Examples:
  nix-shell -p python3           # Python in a temp shell
  nix-shell -p nodejs python3    # Node and Python together
  nix-shell -p ripgrep fd bat    # Multiple CLI tools

Package Management:
- nix search nixpkgs <query>     Search for packages
- nix shell nixpkgs#<pkg>        Temporary shell (flakes way)
- nix run nixpkgs#<pkg>          Run package directly

Development:
- nix develop                    Enter development shell from flake
- nix develop -c <command>       Run command in dev environment
- nix develop --impure           Allow impure evaluation

Building:
- nix build                      Build default package
- nix build .#<package>          Build specific package
- nix build --json               Output build metadata

Flakes:
- nix flake init                 Create new flake.nix
- nix flake update               Update flake.lock
- nix flake check                Validate flake
- nix flake show                 Show flake structure

Utilities:
- nix eval --expr "<expr>"       Evaluate Nix expression
- nix fmt                        Format Nix files
- nixpkgs-fmt <file>            Format specific file
"#;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, uri)],
                })
            }
            "nix://flake/template" => {
                let content = r#"{
  description = "A basic Nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          name = "my-package";
          src = ./.;
          buildInputs = [ ];
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Add your development tools here
          ];

          shellHook = ''
            echo "Development environment ready!"
          '';
        };
      }
    );
}
"#;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, uri)],
                })
            }
            "nix://ecosystem/tools" => {
                let content = r#"Nix Ecosystem Tools

Essential Tools for Nix Development:

comma (,) - Run without installing
  Repository: https://github.com/nix-community/comma
  Install: nix-shell -p comma
  Usage: , cowsay hello
  Run any program from nixpkgs without installing it!

noogle.dev - Search Nix functions
  Website: https://noogle.dev/
  Interactive search for Nix built-ins and nixpkgs lib functions.
  Essential reference when writing Nix code.

Code Quality Tools:

alejandra - Nix code formatter
  Repository: https://github.com/kamadorueda/alejandra
  Install: nix-shell -p alejandra
  Usage: alejandra .

deadnix - Find dead code
  Repository: https://github.com/astro/deadnix
  Install: nix-shell -p deadnix
  Finds unused function arguments, let bindings, and imports.

statix - Nix linter with auto-fixes
  Repository: https://github.com/oppiliappan/statix
  Install: nix-shell -p statix
  Usage: statix check . ; statix fix .

treefmt-nix - Multi-language formatter
  Repository: https://github.com/numtide/treefmt-nix
  One command to format all files (Nix, Rust, JS, Python, etc.)

git-hooks.nix - Pre-commit hooks
  Repository: https://github.com/cachix/git-hooks.nix
  Declaratively configure pre-commit hooks in flake.nix

Development Tools:

nil - Nix Language Server (LSP)
  Repository: https://github.com/oxalica/nil
  Install: nix-shell -p nil
  Provides IDE features: completion, diagnostics, go-to-definition

nixpkgs-review - Review nixpkgs PRs
  Repository: https://github.com/Mic92/nixpkgs-review
  Install: nix-shell -p nixpkgs-review
  Usage: nixpkgs-review pr 12345

Package Development:

nix-init - Generate Nix packages from URLs
  Repository: https://github.com/nix-community/nix-init
  Install: nix-shell -p nix-init
  Automatically creates package definitions for Rust, Python, Go, etc.

crane - Efficient Cargo/Rust builds
  Repository: https://github.com/ipetkov/crane
  Nix library for building Rust projects with incremental builds

Infrastructure & Deployment:

disko - Declarative disk partitioning
  Repository: https://github.com/nix-community/disko
  Define disk layouts, partitions, filesystems, LUKS, LVM in Nix.

nixos-anywhere - Remote NixOS installation
  Repository: https://github.com/nix-community/nixos-anywhere
  Install NixOS on remote machines via SSH.
  Usage: nixos-anywhere --flake '.#my-server' root@192.168.1.10

terranix - Terraform in Nix
  Repository: https://github.com/terranix/terranix
  Write Terraform configurations using Nix instead of HCL.

microvm.nix - Lightweight NixOS VMs
  Repository: https://github.com/microvm-nix/microvm.nix
  Ultra-lightweight VMs that boot in milliseconds.

System Management:

nvd - Nix version diff tool
  Repository: https://git.sr.ht/~khumba/nvd
  Install: nix-shell -p nvd
  Usage: nvd diff /nix/var/nix/profiles/system-{42,43}-link
  Shows what changed between NixOS generations.

Use the 'ecosystem_tools' tool to get detailed information about any of these tools.
"#;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, uri)],
                })
            }
            _ => {
                // Handle dynamic resource templates
                if let Some(package_name) = uri.strip_prefix("nix://package/") {
                    // Get package information
                    let output = tokio::process::Command::new("nix")
                        .args(["search", "nixpkgs", package_name, "--json"])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to search package: {}", e),
                                None,
                            )
                        })?;

                    let content = if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        match serde_json::from_str::<serde_json::Value>(&stdout) {
                            Ok(results) => {
                                if let Some(obj) = results.as_object() {
                                    let mut formatted =
                                        format!("Package Information: {}\n\n", package_name);
                                    for (pkg_path, info) in obj.iter().take(5) {
                                        formatted.push_str(&format!("Package: {}\n", pkg_path));
                                        if let Some(desc) =
                                            info.get("description").and_then(|v| v.as_str())
                                        {
                                            formatted.push_str(&format!("Description: {}\n", desc));
                                        }
                                        if let Some(version) =
                                            info.get("version").and_then(|v| v.as_str())
                                        {
                                            formatted.push_str(&format!("Version: {}\n", version));
                                        }
                                        formatted.push_str("\n");
                                    }
                                    formatted
                                } else {
                                    format!("No package found matching '{}'", package_name)
                                }
                            }
                            Err(_) => format!("No results found for package '{}'", package_name),
                        }
                    } else {
                        format!("Failed to search for package '{}'", package_name)
                    };

                    return Ok(ReadResourceResult {
                        contents: vec![ResourceContents::text(content, uri)],
                    });
                }

                if let Some(rest) = uri.strip_prefix("nix://flake/") {
                    if let Some(flake_ref) = rest.strip_suffix("/show") {
                        // Show flake outputs
                        let output = tokio::process::Command::new("nix")
                            .args(["flake", "show", flake_ref, "--json"])
                            .output()
                            .await
                            .map_err(|e| {
                                McpError::internal_error(
                                    format!("Failed to show flake: {}", e),
                                    None,
                                )
                            })?;

                        let content = if output.status.success() {
                            format!(
                                "Flake outputs for: {}\n\n{}",
                                flake_ref,
                                String::from_utf8_lossy(&output.stdout)
                            )
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            format!("Failed to show flake '{}': {}", flake_ref, stderr)
                        };

                        return Ok(ReadResourceResult {
                            contents: vec![ResourceContents::text(content, uri)],
                        });
                    }
                }

                if let Some(option_path) = uri.strip_prefix("nix://option/") {
                    // Search for NixOS option documentation
                    let output = tokio::process::Command::new("nix")
                        .args([
                            "eval",
                            "--expr",
                            &format!("(import <nixpkgs/nixos> {{}}).options.{}.description or \"Option not found\"", option_path)
                        ])
                        .output()
                        .await
                        .map_err(|e| McpError::internal_error(format!("Failed to query option: {}", e), None))?;

                    let content = if output.status.success() {
                        format!(
                            "NixOS Option: {}\n\n{}",
                            option_path,
                            String::from_utf8_lossy(&output.stdout)
                        )
                    } else {
                        format!("Option '{}' not found or not available", option_path)
                    };

                    return Ok(ReadResourceResult {
                        contents: vec![ResourceContents::text(content, uri)],
                    });
                }

                if let Some(package) = uri.strip_prefix("nix://derivation/") {
                    // Show derivation details
                    let output = tokio::process::Command::new("nix")
                        .args(["show-derivation", package])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to show derivation: {}", e),
                                None,
                            )
                        })?;

                    let content = if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        format!("Failed to get derivation for '{}': {}", package, stderr)
                    };

                    return Ok(ReadResourceResult {
                        contents: vec![ResourceContents::text(content, uri)],
                    });
                }

                Err(McpError::resource_not_found(
                    "resource_not_found",
                    Some(json!({
                        "uri": uri
                    })),
                ))
            }
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        let templates = vec![
            RawResourceTemplate {
                uri_template: "nix://package/{name}".to_string(),
                name: "package-info".to_string(),
                title: Some("Package Information".to_string()),
                description: Some("Get detailed information about any Nix package by name (e.g., nix://package/ripgrep)".to_string()),
                mime_type: Some("text/plain".to_string()),
            }.no_annotation(),
            RawResourceTemplate {
                uri_template: "nix://flake/{ref}/show".to_string(),
                name: "flake-show".to_string(),
                title: Some("Flake Outputs".to_string()),
                description: Some("Show outputs for any flake reference (e.g., nix://flake/nixpkgs/show, nix://flake/github:owner/repo/show)".to_string()),
                mime_type: Some("text/plain".to_string()),
            }.no_annotation(),
            RawResourceTemplate {
                uri_template: "nix://option/{path}".to_string(),
                name: "nixos-option".to_string(),
                title: Some("NixOS Option".to_string()),
                description: Some("Look up NixOS option documentation by path (e.g., nix://option/services.nginx.enable)".to_string()),
                mime_type: Some("text/plain".to_string()),
            }.no_annotation(),
            RawResourceTemplate {
                uri_template: "nix://derivation/{package}".to_string(),
                name: "derivation-info".to_string(),
                title: Some("Derivation Details".to_string()),
                description: Some("Show derivation details for a package (e.g., nix://derivation/nixpkgs#hello)".to_string()),
                mime_type: Some("application/json".to_string()),
            }.no_annotation(),
        ];

        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: templates,
        })
    }

    async fn complete(
        &self,
        request: CompleteRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        let candidates = match &request.r#ref {
            Reference::Prompt(prompt_ref) => {
                // Handle prompt argument completion
                match (prompt_ref.name.as_str(), request.argument.name.as_str()) {
                    ("setup_dev_environment", "project_type") => {
                        vec![
                            "rust", "python", "nodejs", "go", "c", "c++", "java", "haskell",
                            "generic",
                        ]
                    }
                    ("generate_flake", "project_type") => {
                        vec!["rust", "python", "nodejs", "go", "c", "generic"]
                    }
                    _ => vec![],
                }
            }
            _ => {
                // Could also handle tool or resource template argument completion here
                vec![]
            }
        };

        // Filter candidates based on the current input value
        let filtered: Vec<String> = if request.argument.value.is_empty() {
            candidates.into_iter().map(String::from).collect()
        } else {
            let query_lower = request.argument.value.to_lowercase();
            candidates
                .into_iter()
                .filter(|c| c.to_lowercase().contains(&query_lower))
                .map(String::from)
                .collect()
        };

        Ok(CompleteResult {
            completion: CompletionInfo {
                values: filtered,
                total: None,
                has_more: Some(false),
            },
        })
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
}
