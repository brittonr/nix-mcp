use crate::common::cache_registry::CacheRegistry;
use crate::common::security::{audit_logger, AuditLogger};
use crate::common::tool_registry::ToolRegistry;
use crate::nix::{
    CommaArgs, DiffDerivationsArgs, EcosystemToolArgs, ExplainPackageArgs, FindCommandArgs,
    FlakeMetadataArgs, FlakeShowArgs, FormatNixArgs, GetBuildLogArgs, GetClosureSizeArgs,
    GetPackageInfoArgs, LintNixArgs, NixBuildArgs, NixCommandHelpArgs, NixDevelopArgs, NixEvalArgs,
    NixFmtArgs, NixLocateArgs, NixLogArgs, NixRunArgs, NixosBuildArgs, PrefetchUrlArgs,
    RunInShellArgs, SearchOptionsArgs, SearchPackagesArgs, ShowDerivationArgs, ValidateNixArgs,
    WhyDependsArgs,
};
use rmcp::{
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler,
};
use serde_json::json;
use std::sync::Arc;

// Import pre-commit types from dev module
use crate::dev::{CheckPreCommitStatusArgs, PreCommitRunArgs, SetupPreCommitArgs};

// Import pexpect and pueue types from process module
use crate::process::{
    PexpectCloseArgs, PexpectSendArgs, PexpectStartArgs, PueueAddArgs, PueueCleanArgs,
    PueueLogArgs, PueuePauseArgs, PueueRemoveArgs, PueueStartArgs, PueueStatusArgs, PueueWaitArgs,
};

// Import clan types from clan module
use crate::clan::{
    ClanAnalyzeRosterArgs, ClanAnalyzeSecretsArgs, ClanAnalyzeTagsArgs, ClanAnalyzeVarsArgs,
    ClanBackupCreateArgs, ClanBackupListArgs, ClanBackupRestoreArgs, ClanFlakeCreateArgs,
    ClanMachineBuildArgs, ClanMachineCreateArgs, ClanMachineDeleteArgs, ClanMachineInstallArgs,
    ClanMachineListArgs, ClanMachineUpdateArgs, ClanSecretsListArgs, ClanVmCreateArgs,
};

// Import prompt types from prompts module
use crate::prompts::{
    MigrateToFlakesArgs, OptimizeClosureArgs, SetupDevEnvironmentArgs, TroubleshootBuildArgs,
};

#[derive(Clone)]
pub struct NixServer {
    tool_router: ToolRouter<NixServer>,
    prompt_router: PromptRouter<NixServer>,
    audit: Arc<AuditLogger>,
    // Centralized tool registry for all tool implementations
    tools: Arc<ToolRegistry>,
    // Centralized cache registry for all caching needs
    caches: Arc<CacheRegistry>,
}

#[tool_router]
impl NixServer {
    pub fn new() -> Self {
        let audit = audit_logger();
        let caches = Arc::new(CacheRegistry::new());
        let tools = Arc::new(ToolRegistry::new(audit.clone(), caches.clone()));

        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            audit,
            tools,
            caches,
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
        self.tools.package.search_packages(args).await
    }

    #[tool(
        description = "Get detailed information about a specific package",
        annotations(read_only_hint = true)
    )]
    async fn get_package_info(
        &self,
        args: Parameters<GetPackageInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.package.get_package_info(args).await
    }

    #[tool(
        description = "Search NixOS configuration options",
        annotations(read_only_hint = true)
    )]
    async fn search_options(
        &self,
        args: Parameters<SearchOptionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.develop.search_options(args).await
    }

    #[tool(description = "Evaluate a Nix expression")]
    async fn nix_eval(&self, args: Parameters<NixEvalArgs>) -> Result<CallToolResult, McpError> {
        self.tools.develop.nix_eval(args).await
    }

    #[tool(
        description = "Format Nix code using nixpkgs-fmt",
        annotations(idempotent_hint = true)
    )]
    async fn format_nix(
        &self,
        args: Parameters<FormatNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.quality.format_nix(args).await
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
        self.tools.info.nix_command_help(args)
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
        self.tools.info.ecosystem_tools(args)
    }

    #[tool(
        description = "Validate Nix code syntax and check for parse errors",
        annotations(idempotent_hint = true)
    )]
    async fn validate_nix(
        &self,
        args: Parameters<ValidateNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.quality.validate_nix(args).await
    }

    #[tool(
        description = "Lint Nix code with statix and/or deadnix to find issues and anti-patterns",
        annotations(idempotent_hint = true)
    )]
    async fn lint_nix(&self, args: Parameters<LintNixArgs>) -> Result<CallToolResult, McpError> {
        self.tools.quality.lint_nix(args).await
    }

    #[tool(
        description = "Get detailed information about a package (version, description, homepage, license, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn explain_package(
        &self,
        args: Parameters<ExplainPackageArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.package.explain_package(args).await
    }

    #[tool(description = "Prefetch a URL and get its hash for use in Nix expressions")]
    async fn prefetch_url(
        &self,
        args: Parameters<PrefetchUrlArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.flake.prefetch_url(args).await
    }

    #[tool(
        description = "Get metadata about a flake (inputs, outputs, description)",
        annotations(read_only_hint = true)
    )]
    async fn flake_metadata(
        &self,
        args: Parameters<FlakeMetadataArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.flake.flake_metadata(args).await
    }

    #[tool(
        description = "Find which package provides a command using nix-locate",
        annotations(read_only_hint = true)
    )]
    async fn find_command(
        &self,
        args: Parameters<FindCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.package.find_command(args).await
    }

    #[tool(
        description = "Run a command without installing it using comma (automatically finds and runs commands from nixpkgs)"
    )]
    async fn comma(&self, args: Parameters<CommaArgs>) -> Result<CallToolResult, McpError> {
        self.tools.package.comma(args).await
    }

    #[tool(description = "Build a Nix package and show what will be built or the build output")]
    async fn nix_build(&self, args: Parameters<NixBuildArgs>) -> Result<CallToolResult, McpError> {
        self.tools.build.nix_build(args).await
    }

    #[tool(
        description = "Explain why one package depends on another (show dependency chain)",
        annotations(read_only_hint = true)
    )]
    async fn why_depends(
        &self,
        args: Parameters<WhyDependsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.build.why_depends(args).await
    }

    #[tool(
        description = "Show the derivation details of a package (build inputs, environment, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn show_derivation(
        &self,
        args: Parameters<ShowDerivationArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.build.show_derivation(args).await
    }

    #[tool(
        description = "Get the closure size of a package (total size including all dependencies)",
        annotations(read_only_hint = true)
    )]
    async fn get_closure_size(
        &self,
        args: Parameters<GetClosureSizeArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.build.get_closure_size(args).await
    }

    #[tool(description = "Run a command in a Nix shell with specified packages available")]
    async fn run_in_shell(
        &self,
        args: Parameters<RunInShellArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.develop.run_in_shell(args).await
    }

    #[tool(
        description = "Show the outputs available in a flake (packages, apps, devShells, etc.)",
        annotations(read_only_hint = true)
    )]
    async fn flake_show(
        &self,
        args: Parameters<FlakeShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.flake.flake_show(args).await
    }

    #[tool(
        description = "Get the build log for a package (useful for debugging build failures)",
        annotations(read_only_hint = true)
    )]
    async fn get_build_log(
        &self,
        args: Parameters<GetBuildLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.build.get_build_log(args).await
    }

    #[tool(
        description = "Get Nix build logs directly from store path, optionally filtered with grep pattern",
        annotations(read_only_hint = true)
    )]
    async fn nix_log(&self, args: Parameters<NixLogArgs>) -> Result<CallToolResult, McpError> {
        self.tools.develop.nix_log(args).await
    }

    #[tool(
        description = "Compare two derivations to understand what differs between packages (uses nix-diff)",
        annotations(read_only_hint = true)
    )]
    async fn diff_derivations(
        &self,
        args: Parameters<DiffDerivationsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.build.diff_derivations(args).await
    }

    // Clan integration tools

    #[tool(description = "Create a new Clan machine configuration")]
    async fn clan_machine_create(
        &self,
        args: Parameters<ClanMachineCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.machine.clan_machine_create(args).await
    }

    #[tool(
        description = "List all Clan machines in the flake",
        annotations(read_only_hint = true)
    )]
    async fn clan_machine_list(
        &self,
        args: Parameters<ClanMachineListArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.machine.clan_machine_list(args).await
    }

    #[tool(
        description = "Update Clan machine(s) - rebuilds and deploys configuration",
        annotations(destructive_hint = true)
    )]
    async fn clan_machine_update(
        &self,
        args: Parameters<ClanMachineUpdateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.machine.clan_machine_update(args).await
    }

    #[tool(
        description = "Delete a Clan machine configuration",
        annotations(destructive_hint = true)
    )]
    async fn clan_machine_delete(
        &self,
        args: Parameters<ClanMachineDeleteArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.machine.clan_machine_delete(args).await
    }

    #[tool(
        description = "Install Clan machine to a target host via SSH (WARNING: Destructive - overwrites disk)",
        annotations(destructive_hint = true)
    )]
    async fn clan_machine_install(
        &self,
        args: Parameters<ClanMachineInstallArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.machine.clan_machine_install(args).await
    }

    #[tool(description = "Create a backup for a Clan machine")]
    async fn clan_backup_create(
        &self,
        args: Parameters<ClanBackupCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.backup.clan_backup_create(args).await
    }

    #[tool(
        description = "List backups for a Clan machine",
        annotations(read_only_hint = true)
    )]
    async fn clan_backup_list(
        &self,
        args: Parameters<ClanBackupListArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.backup.clan_backup_list(args).await
    }

    #[tool(
        description = "Restore a backup for a Clan machine",
        annotations(destructive_hint = true)
    )]
    async fn clan_backup_restore(
        &self,
        args: Parameters<ClanBackupRestoreArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.backup.clan_backup_restore(args).await
    }

    #[tool(description = "Create a new Clan flake from a template")]
    async fn clan_flake_create(
        &self,
        args: Parameters<ClanFlakeCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_flake_create(args).await
    }

    #[tool(
        description = "List secrets in a Clan flake",
        annotations(read_only_hint = true)
    )]
    async fn clan_secrets_list(
        &self,
        args: Parameters<ClanSecretsListArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_secrets_list(args).await
    }

    #[tool(description = "Create and run a VM for a Clan machine (useful for testing)")]
    async fn clan_vm_create(
        &self,
        args: Parameters<ClanVmCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_vm_create(args).await
    }

    #[tool(
        description = "Build a Clan machine configuration locally for testing without deployment"
    )]
    async fn clan_machine_build(
        &self,
        args: Parameters<ClanMachineBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.machine.clan_machine_build(args).await
    }

    #[tool(description = "Build a NixOS machine configuration from a flake")]
    async fn nixos_build(
        &self,
        args: Parameters<NixosBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.build.nixos_build(args).await
    }

    #[tool(description = "Analyze Clan secret (ACL) ownership across machines")]
    async fn clan_analyze_secrets(
        &self,
        args: Parameters<ClanAnalyzeSecretsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_analyze_secrets(args).await
    }

    #[tool(description = "Analyze Clan vars ownership across machines")]
    async fn clan_analyze_vars(
        &self,
        args: Parameters<ClanAnalyzeVarsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_analyze_vars(args).await
    }

    #[tool(description = "Analyze Clan machine tags across the infrastructure")]
    async fn clan_analyze_tags(
        &self,
        args: Parameters<ClanAnalyzeTagsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_analyze_tags(args).await
    }

    #[tool(description = "Analyze Clan user roster configurations")]
    async fn clan_analyze_roster(
        &self,
        args: Parameters<ClanAnalyzeRosterArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_analyze_roster(args).await
    }

    #[tool(
        description = "Get help and information about Clan - the peer-to-peer NixOS management framework"
    )]
    fn clan_help(
        &self,
        args: Parameters<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.analysis.clan_help(args)
    }

    #[tool(
        description = "Find which package provides a specific file path using nix-locate",
        annotations(read_only_hint = true)
    )]
    async fn nix_locate(
        &self,
        args: Parameters<NixLocateArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.package.nix_locate(args).await
    }

    #[tool(
        description = "Run an application from nixpkgs without installing it",
        annotations(read_only_hint = false)
    )]
    async fn nix_run(&self, args: Parameters<NixRunArgs>) -> Result<CallToolResult, McpError> {
        self.tools.develop.nix_run(args).await
    }

    #[tool(
        description = "Run a command in a Nix development environment (from flake.nix devShell)",
        annotations(read_only_hint = false)
    )]
    async fn nix_develop(
        &self,
        args: Parameters<NixDevelopArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools.develop.nix_develop(args).await
    }

    #[tool(
        description = "Format Nix code using the project's formatter (typically nix fmt)",
        annotations(read_only_hint = false)
    )]
    async fn nix_fmt(&self, args: Parameters<NixFmtArgs>) -> Result<CallToolResult, McpError> {
        self.tools.quality.nix_fmt(args).await
    }

    #[tool(
        description = "Add a command to the pueue task queue for async execution. Returns task ID.",
        annotations(read_only_hint = false)
    )]
    async fn pueue_add(&self, args: Parameters<PueueAddArgs>) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.tools.pueue.pueue_add(args).await
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
        self.tools.pueue.pueue_status(args).await
    }

    #[tool(
        description = "Get logs for a specific pueue task",
        annotations(read_only_hint = true)
    )]
    async fn pueue_log(&self, args: Parameters<PueueLogArgs>) -> Result<CallToolResult, McpError> {
        // Delegate to modular implementation
        self.tools.pueue.pueue_log(args).await
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
        self.tools.pueue.pueue_wait(args).await
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
        self.tools.pueue.pueue_remove(args).await
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
        self.tools.pueue.pueue_clean(args).await
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
        self.tools.pueue.pueue_pause(args).await
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
        self.tools.pueue.pueue_start(args).await
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
        self.tools.pexpect.pexpect_start(args).await
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
        self.tools.pexpect.pexpect_send(args).await
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
        self.tools.pexpect.pexpect_close(args).await
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
        self.tools.precommit.pre_commit_run(args).await
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
        self.tools.precommit.check_pre_commit_status(args).await
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
        self.tools.precommit.setup_pre_commit(args).await
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
        self.tools.prompts.generate_flake(args, ctx).await
    }

    /// Guide for setting up a Nix development environment for a specific project type
    #[prompt(name = "setup_dev_environment")]
    async fn setup_dev_environment(
        &self,
        args: Parameters<SetupDevEnvironmentArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.tools.prompts.setup_dev_environment(args, ctx).await
    }

    /// Help troubleshoot Nix build failures with diagnostic guidance
    #[prompt(name = "troubleshoot_build")]
    async fn troubleshoot_build(
        &self,
        args: Parameters<TroubleshootBuildArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.tools.prompts.troubleshoot_build(args, ctx).await
    }

    /// Guide for migrating existing projects to Nix flakes
    #[prompt(name = "migrate_to_flakes")]
    async fn migrate_to_flakes(
        &self,
        args: Parameters<MigrateToFlakesArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.tools.prompts.migrate_to_flakes(args, ctx).await
    }

    /// Help optimize package closure size with actionable recommendations
    #[prompt(name = "optimize_closure")]
    async fn optimize_closure(
        &self,
        args: Parameters<OptimizeClosureArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.tools.prompts.optimize_closure(args, ctx).await
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
