use crate::common::cache::TtlCache;
use crate::common::security::{
    audit_logger, validate_command, validate_flake_ref, validate_package_name,
    validation_error_to_mcp, AuditLogger,
};
use crate::nix::{
    CommaArgs, DiffDerivationsArgs, EcosystemToolArgs, ExplainPackageArgs, FindCommandArgs,
    GetBuildLogArgs, GetClosureSizeArgs, GetPackageInfoArgs, NixBuildArgs, NixCommandHelpArgs,
    NixLocateArgs, NixosBuildArgs, SearchPackagesArgs, ShowDerivationArgs, WhyDependsArgs,
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
pub struct NixFmtArgs {
    /// Path to format (file or directory, defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

// Import pre-commit types from dev module
use crate::dev::{CheckPreCommitStatusArgs, PreCommitRunArgs, SetupPreCommitArgs};

// Import pexpect and pueue types from process module
use crate::process::{
    PexpectCloseArgs, PexpectSendArgs, PexpectStartArgs, PueueAddArgs, PueueCleanArgs,
    PueueLogArgs, PueuePauseArgs, PueueRemoveArgs, PueueStartArgs, PueueStatusArgs, PueueWaitArgs,
};

// Import prompt types from prompts module
use crate::prompts::{
    MigrateToFlakesArgs, OptimizeClosureArgs, SetupDevEnvironmentArgs, TroubleshootBuildArgs,
};

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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineBuildArgs {
    /// Machine name to build
    pub machine: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
    /// Use nom for better build output (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_nom: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeSecretsArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeVarsArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeTagsArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeRosterArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Clone)]
pub struct NixServer {
    tool_router: ToolRouter<NixServer>,
    prompt_router: PromptRouter<NixServer>,
    audit: Arc<AuditLogger>,
    // Modular tool implementations
    precommit_tools: Arc<crate::dev::PreCommitTools>,
    pexpect_tools: Arc<crate::process::PexpectTools>,
    pueue_tools: Arc<crate::process::PueueTools>,
    // Modular prompt implementations
    nix_prompts: Arc<crate::prompts::NixPrompts>,
    // Modular nix tool implementations
    info_tools: Arc<crate::nix::InfoTools>,
    package_tools: Arc<crate::nix::PackageTools>,
    build_tools: Arc<crate::nix::BuildTools>,
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
        let audit = audit_logger();

        // Create caches first so they can be shared
        let locate_cache = Arc::new(TtlCache::new(Duration::from_secs(300))); // 5 min TTL
        let search_cache = Arc::new(TtlCache::new(Duration::from_secs(600))); // 10 min TTL
        let package_info_cache = Arc::new(TtlCache::new(Duration::from_secs(1800))); // 30 min TTL
        let closure_size_cache = Arc::new(TtlCache::new(Duration::from_secs(1800))); // 30 min TTL
        let derivation_cache = Arc::new(TtlCache::new(Duration::from_secs(1800))); // 30 min TTL

        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            audit: audit.clone(),
            precommit_tools: Arc::new(crate::dev::PreCommitTools::new(audit.clone())),
            pexpect_tools: Arc::new(crate::process::PexpectTools::new(audit.clone())),
            pueue_tools: Arc::new(crate::process::PueueTools::new(audit.clone())),
            nix_prompts: Arc::new(crate::prompts::NixPrompts::new()),
            info_tools: Arc::new(crate::nix::InfoTools::new(audit.clone())),
            package_tools: Arc::new(crate::nix::PackageTools::new(
                audit.clone(),
                search_cache.clone(),
                package_info_cache.clone(),
                locate_cache.clone(),
            )),
            build_tools: Arc::new(crate::nix::BuildTools::new(
                audit.clone(),
                closure_size_cache.clone(),
                derivation_cache.clone(),
            )),
            locate_cache,
            search_cache,
            package_info_cache,
            eval_cache: Arc::new(TtlCache::new(Duration::from_secs(300))), // 5 min TTL
            prefetch_cache: Arc::new(TtlCache::new(Duration::from_secs(86400))), // 24 hour TTL
            closure_size_cache,
            derivation_cache,
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
        args: Parameters<SearchPackagesArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.package_tools.search_packages(args).await
    }

    #[tool(
        description = "Get detailed information about a specific package",
        annotations(read_only_hint = true)
    )]
    async fn get_package_info(
        &self,
        args: Parameters<GetPackageInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.package_tools.get_package_info(args).await
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
        audit_tool_execution(
            &self.audit,
            "search_options",
            Some(serde_json::json!({"query": &query})),
            || async {
                with_timeout(&self.audit, "search_options", 30, || async {
                    // Check if we're on NixOS and can query options directly
                    let nixos_check = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg("test -f /etc/NIXOS")
                        .output()
                        .await;

                    let on_nixos = nixos_check.map(|o| o.status.success()).unwrap_or(false);

                    if on_nixos {
                        // Try to search using nixos-option if available
                        let output = tokio::process::Command::new("nixos-option")
                            .arg(&query)
                            .output()
                            .await;

                        if let Ok(output) = output {
                            if output.status.success() {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                return Ok(CallToolResult::success(vec![Content::text(
                                    stdout.to_string(),
                                )]));
                            }
                        }
                    }

                    // Provide helpful information with web search links
                    use crate::common::nix_tools_helpers::format_option_search_response;
                    Ok(CallToolResult::success(vec![Content::text(
                        format_option_search_response(&query),
                    )]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Evaluate a Nix expression")]
    async fn nix_eval(
        &self,
        Parameters(NixEvalArgs { expression }): Parameters<NixEvalArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::caching::CachedExecutor;
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_nix_expression;

        // Validate Nix expression for dangerous patterns
        validate_nix_expression(&expression).map_err(validation_error_to_mcp)?;

        // Use cached executor for cache-check-execute-cache pattern
        let cached_executor = CachedExecutor::new(self.eval_cache.clone());
        let audit = self.audit.clone();
        let expression_clone = expression.clone();

        cached_executor
            .execute_with_string_cache(expression.clone(), || async move {
                let audit_inner = audit.clone();
                // Execute with security features (audit logging + 30s timeout for eval)
                audit_tool_execution(
                    &audit,
                    "nix_eval",
                    Some(serde_json::json!({"expression_length": expression_clone.len()})),
                    || async move {
                        with_timeout(&audit_inner, "nix_eval", 30, || async {
                            let output = tokio::process::Command::new("nix")
                                .args(["eval", "--expr", &expression_clone])
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

                            Ok(String::from_utf8_lossy(&output.stdout).to_string())
                        })
                        .await
                    },
                )
                .await
            })
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
        args: Parameters<NixCommandHelpArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.info_tools.nix_command_help(args)
    }

    #[tool(
        description = "Get information about useful Nix ecosystem tools and utilities",
        annotations(read_only_hint = true)
    )]
    fn ecosystem_tools(
        &self,
        args: Parameters<EcosystemToolArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.info_tools.ecosystem_tools(args)
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
        args: Parameters<ExplainPackageArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.package_tools.explain_package(args).await
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
        args: Parameters<FindCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.package_tools.find_command(args).await
    }

    #[tool(
        description = "Run a command without installing it using comma (automatically finds and runs commands from nixpkgs)"
    )]
    async fn comma(&self, args: Parameters<CommaArgs>) -> Result<CallToolResult, McpError> {
        self.package_tools.comma(args).await
    }

    #[tool(description = "Build a Nix package and show what will be built or the build output")]
    async fn nix_build(&self, args: Parameters<NixBuildArgs>) -> Result<CallToolResult, McpError> {
        self.build_tools.nix_build(args).await
    }

    #[tool(
        description = "Explain why one package depends on another (show dependency chain)",
        annotations(read_only_hint = true)
    )]
    async fn why_depends(
        &self,
        args: Parameters<WhyDependsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.build_tools.why_depends(args).await
    }

    #[tool(
        description = "Show the derivation details of a package (build inputs, environment, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn show_derivation(
        &self,
        args: Parameters<ShowDerivationArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.build_tools.show_derivation(args).await
    }

    #[tool(
        description = "Get the closure size of a package (total size including all dependencies)",
        annotations(read_only_hint = true)
    )]
    async fn get_closure_size(
        &self,
        args: Parameters<GetClosureSizeArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.build_tools.get_closure_size(args).await
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
                            if let serde_json::Value::Object(map) = value {
                                for (key, val) in map {
                                    if val.is_object()
                                        && val.as_object().unwrap().contains_key("type")
                                    {
                                        let type_str = val["type"].as_str().unwrap_or("unknown");
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
        args: Parameters<GetBuildLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.build_tools.get_build_log(args).await
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
        args: Parameters<DiffDerivationsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.build_tools.diff_derivations(args).await
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
        description = "Build a Clan machine configuration locally for testing without deployment"
    )]
    async fn clan_machine_build(
        &self,
        Parameters(ClanMachineBuildArgs {
            machine,
            flake,
            use_nom,
        }): Parameters<ClanMachineBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "clan_machine_build", Some(serde_json::json!({"machine": &machine, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_machine_build", 300, || async {
                let use_nom = use_nom.unwrap_or(false);
                let build_target = format!(".#nixosConfigurations.{}.config.system.build.toplevel", machine);

                let mut cmd = if use_nom {
                    // Check if nom is available
                    let nom_check = tokio::process::Command::new("which")
                        .arg("nom")
                        .output()
                        .await;

                    if nom_check.is_ok() && nom_check.unwrap().status.success() {
                        let mut c = tokio::process::Command::new("nom");
                        c.args(["build", &build_target]);
                        c
                    } else {
                        let mut c = tokio::process::Command::new("nix");
                        c.args(["build", &build_target]);
                        c
                    }
                } else {
                    let mut c = tokio::process::Command::new("nix");
                    c.args(["build", &build_target]);
                    c
                };

                cmd.current_dir(&flake_str);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute build command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Build failed for machine '{}':\n\n{}{}", machine, stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Successfully built machine '{}' configuration.\n\n{}{}\n\nThe build result is in ./result/", machine, stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Build a NixOS machine configuration from a flake")]
    async fn nixos_build(
        &self,
        args: Parameters<NixosBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.build_tools.nixos_build(args).await
    }

    #[tool(description = "Analyze Clan secret (ACL) ownership across machines")]
    async fn clan_analyze_secrets(
        &self,
        Parameters(ClanAnalyzeSecretsArgs { flake }): Parameters<ClanAnalyzeSecretsArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "clan_analyze_secrets", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_secrets", 60, || async {
                // Try local flake first, then fall back to onix-core
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#acl 2>/dev/null || nix run github:onixcomputer/onix-core#acl) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute acl command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("ACL analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan Secret (ACL) Ownership Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Analyze Clan vars ownership across machines")]
    async fn clan_analyze_vars(
        &self,
        Parameters(ClanAnalyzeVarsArgs { flake }): Parameters<ClanAnalyzeVarsArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "clan_analyze_vars", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_vars", 60, || async {
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#vars 2>/dev/null || nix run github:onixcomputer/onix-core#vars) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute vars command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Vars analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan Vars Ownership Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Analyze Clan machine tags across the infrastructure")]
    async fn clan_analyze_tags(
        &self,
        Parameters(ClanAnalyzeTagsArgs { flake }): Parameters<ClanAnalyzeTagsArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "clan_analyze_tags", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_tags", 60, || async {
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#tags 2>/dev/null || nix run github:onixcomputer/onix-core#tags) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute tags command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Tags analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan Machine Tags Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Analyze Clan user roster configurations")]
    async fn clan_analyze_roster(
        &self,
        Parameters(ClanAnalyzeRosterArgs { flake }): Parameters<ClanAnalyzeRosterArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "clan_analyze_roster", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_roster", 60, || async {
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#roster 2>/dev/null || nix run github:onixcomputer/onix-core#roster) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute roster command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Roster analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan User Roster Analysis:\n\n{}{}", stdout, stderr)
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
- clan_machine_build - Build machine configuration locally for testing

Backup Operations:
- clan_backup_create - Create backups for machines
- clan_backup_list - List available backups
- clan_backup_restore - Restore from backup

Flake & Project:
- clan_flake_create - Initialize new Clan project

Secrets:
- clan_secrets_list - View configured secrets

Testing & Building:
- clan_vm_create - Create VMs for testing configurations
- nixos_build - Build NixOS configurations from flakes

Analysis Tools:
- clan_analyze_secrets - Analyze secret (ACL) ownership across machines
- clan_analyze_vars - Analyze vars ownership across machines
- clan_analyze_tags - Analyze machine tags
- clan_analyze_roster - Analyze user roster configurations

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
        args: Parameters<NixLocateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.package_tools.nix_locate(args).await
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
                        result.push('\n');
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
                        result.push('\n');
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
                            result.push('\n');
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
    async fn pueue_add(&self, args: Parameters<PueueAddArgs>) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_add(args).await
    }

    #[tool(
        description = "Get the status of pueue tasks (all or specific task IDs)",
        annotations(read_only_hint = true)
    )]
    async fn pueue_status(
        &self,
        args: Parameters<PueueStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_status(args).await
    }

    #[tool(
        description = "Get logs for a specific pueue task",
        annotations(read_only_hint = true)
    )]
    async fn pueue_log(&self, args: Parameters<PueueLogArgs>) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_log(args).await
    }

    #[tool(
        description = "Wait for specific pueue tasks to complete",
        annotations(read_only_hint = true)
    )]
    async fn pueue_wait(
        &self,
        args: Parameters<PueueWaitArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_wait(args).await
    }

    #[tool(
        description = "Remove/kill specific pueue tasks",
        annotations(read_only_hint = false)
    )]
    async fn pueue_remove(
        &self,
        args: Parameters<PueueRemoveArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_remove(args).await
    }

    #[tool(
        description = "Clean up finished pueue tasks from the queue",
        annotations(read_only_hint = false)
    )]
    async fn pueue_clean(
        &self,
        args: Parameters<PueueCleanArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_clean(args).await
    }

    #[tool(
        description = "Pause specific pueue tasks or all tasks",
        annotations(read_only_hint = false)
    )]
    async fn pueue_pause(
        &self,
        args: Parameters<PueuePauseArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_pause(args).await
    }

    #[tool(
        description = "Start/resume specific pueue tasks or all tasks",
        annotations(read_only_hint = false)
    )]
    async fn pueue_start(
        &self,
        args: Parameters<PueueStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pueue_tools.pueue_start(args).await
    }

    #[tool(
        description = "Start a new pexpect-cli interactive session. Returns session ID.",
        annotations(read_only_hint = false)
    )]
    async fn pexpect_start(
        &self,
        args: Parameters<PexpectStartArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pexpect_tools.pexpect_start(args).await
    }

    #[tool(
        description = "Send Python pexpect code to an active session",
        annotations(read_only_hint = false)
    )]
    async fn pexpect_send(
        &self,
        args: Parameters<PexpectSendArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pexpect_tools.pexpect_send(args).await
    }

    #[tool(
        description = "Close an active pexpect-cli session",
        annotations(read_only_hint = false)
    )]
    async fn pexpect_close(
        &self,
        args: Parameters<PexpectCloseArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.pexpect_tools.pexpect_close(args).await
    }

    #[tool(
        description = "Run pre-commit hooks to check code quality (formatting, linting, etc.)",
        annotations(read_only_hint = false)
    )]
    async fn pre_commit_run(
        &self,
        args: Parameters<PreCommitRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.precommit_tools.pre_commit_run(args).await
    }

    #[tool(
        description = "Check if pre-commit hooks are installed and configured in the current repository",
        annotations(read_only_hint = true)
    )]
    async fn check_pre_commit_status(
        &self,
        args: Parameters<CheckPreCommitStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.precommit_tools.check_pre_commit_status(args).await
    }

    #[tool(
        description = "Set up pre-commit hooks for a project (creates config and installs hooks)",
        annotations(read_only_hint = false)
    )]
    async fn setup_pre_commit(
        &self,
        args: Parameters<SetupPreCommitArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.precommit_tools.setup_pre_commit(args).await
    }
}

#[prompt_router]
impl NixServer {
    /// Generate a nix flake template based on requirements
    #[prompt(name = "generate_flake")]
    async fn generate_flake(
        &self,
        args: Parameters<serde_json::Map<String, serde_json::Value>>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        self.nix_prompts.generate_flake(args, ctx).await
    }

    /// Guide for setting up a Nix development environment for a specific project type
    #[prompt(name = "setup_dev_environment")]
    async fn setup_dev_environment(
        &self,
        args: Parameters<SetupDevEnvironmentArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.nix_prompts.setup_dev_environment(args, ctx).await
    }

    /// Help troubleshoot Nix build failures with diagnostic guidance
    #[prompt(name = "troubleshoot_build")]
    async fn troubleshoot_build(
        &self,
        args: Parameters<TroubleshootBuildArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.nix_prompts.troubleshoot_build(args, ctx).await
    }

    /// Guide for migrating existing projects to Nix flakes
    #[prompt(name = "migrate_to_flakes")]
    async fn migrate_to_flakes(
        &self,
        args: Parameters<MigrateToFlakesArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.nix_prompts.migrate_to_flakes(args, ctx).await
    }

    /// Help optimize package closure size with actionable recommendations
    #[prompt(name = "optimize_closure")]
    async fn optimize_closure(
        &self,
        args: Parameters<OptimizeClosureArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.nix_prompts.optimize_closure(args, ctx).await
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
                \n\nBuild & Execution: nix_build, nix_run, comma, run_in_shell, get_closure_size, get_build_log \
                \n\nDependency Analysis: why_depends, show_derivation, diff_derivations \
                \n\nFlake Management: flake_metadata, flake_show \
                \n\nCode Quality: validate_nix, lint_nix, format_nix, pre_commit_run, check_pre_commit_status, setup_pre_commit \
                \n\nUtilities: nix_eval, prefetch_url, search_options, nix_command_help, ecosystem_tools \
                \n\n=== PROACTIVE CODE QUALITY CHECKS === \
                \n\nWhen working with a git repository, PROACTIVELY check if pre-commit hooks are set up using check_pre_commit_status. \
                If they are not configured, suggest setting them up with setup_pre_commit or by adding pre-commit-hooks.nix to the flake. \
                Pre-commit hooks enforce code quality standards (formatting, linting) before commits. \
                \n\nBest practice: Run check_pre_commit_status when starting work on a project to ensure quality tooling is in place. \
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
                - Enforce code quality with pre-commit hooks (check and run via MCP tools) \
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
                                        formatted.push('\n');
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
