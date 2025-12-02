use rmcp::{
    ErrorData as McpError,
    RoleServer,
    ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt,
    prompt_handler,
    prompt_router,
    schemars,
    service::RequestContext,
    tool,
    tool_handler,
    tool_router,
};
use serde_json::json;

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
pub struct DiffDerivationsArgs {
    /// First package to compare (e.g., "nixpkgs#firefox")
    pub package_a: String,
    /// Second package to compare (e.g., "nixpkgs#firefox-esr")
    pub package_b: String,
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

#[derive(Clone)]
pub struct NixServer {
    tool_router: ToolRouter<NixServer>,
    prompt_router: PromptRouter<NixServer>,
}

#[tool_router]
impl NixServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    #[tool(description = "Search for packages in nixpkgs by name or description")]
    async fn search_packages(
        &self,
        Parameters(SearchPackagesArgs { query, limit }): Parameters<SearchPackagesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let limit = limit.unwrap_or(10);

        // Use nix search command
        let output = tokio::process::Command::new("nix")
            .args(["search", "nixpkgs", &query, "--json"])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix search: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("nix search failed: {}", stderr), None));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| McpError::internal_error(format!("Failed to parse search results: {}", e), None))?;

        // Format results nicely
        let mut formatted_results = Vec::new();
        if let Some(obj) = results.as_object() {
            for (i, (pkg_path, info)) in obj.iter().enumerate() {
                if i >= limit {
                    break;
                }

                let description = info["description"].as_str().unwrap_or("No description");
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

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }

    #[tool(description = "Get detailed information about a specific package")]
    async fn get_package_info(
        &self,
        Parameters(GetPackageInfoArgs { package }): Parameters<GetPackageInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Use nix eval to get package metadata
        let output = tokio::process::Command::new("nix")
            .args(["eval", &package, "--json"])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix eval: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("nix eval failed: {}", stderr), None));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(CallToolResult::success(vec![Content::text(stdout.to_string())]))
    }

    #[tool(description = "Search NixOS configuration options")]
    async fn search_options(
        &self,
        Parameters(SearchOptionsArgs { query }): Parameters<SearchOptionsArgs>,
    ) -> Result<CallToolResult, McpError> {
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
    }

    #[tool(description = "Evaluate a Nix expression")]
    async fn nix_eval(
        &self,
        Parameters(NixEvalArgs { expression }): Parameters<NixEvalArgs>,
    ) -> Result<CallToolResult, McpError> {
        let output = tokio::process::Command::new("nix")
            .args(["eval", "--expr", &expression])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix eval: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("Evaluation failed: {}", stderr), None));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(CallToolResult::success(vec![Content::text(stdout.to_string())]))
    }

    #[tool(description = "Format Nix code using nixpkgs-fmt")]
    async fn format_nix(
        &self,
        Parameters(FormatNixArgs { code }): Parameters<FormatNixArgs>,
    ) -> Result<CallToolResult, McpError> {
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
    }

    #[tool(description = "Get help with common Nix commands and patterns")]
    fn nix_command_help(
        &self,
        Parameters(NixCommandHelpArgs { command }): Parameters<NixCommandHelpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let help_text = match command.as_deref() {
            Some("develop") => r#"nix develop - Enter a development shell

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
"#,
            Some("build") => r#"nix build - Build a package

Usage:
  nix build                # Build default package from flake
  nix build .#package      # Build specific package
  nix build nixpkgs#hello  # Build from nixpkgs
  nix build --json         # Output JSON metadata

Result: Creates 'result' symlink to build output
"#,
            Some("flake") => r#"nix flake - Manage Nix flakes

Common commands:
  nix flake init           # Create new flake.nix
  nix flake update         # Update flake.lock
  nix flake check          # Check flake outputs
  nix flake show           # Show flake outputs
  nix flake metadata       # Show flake metadata

Templates:
  nix flake init -t templates#rust      # Rust template
  nix flake init -t templates#python    # Python template
"#,
            Some("shell") | Some("nix-shell") => r#"Getting Packages in a Shell

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
"#,
            Some("run") => r#"nix run - Run a package

Usage:
  nix run nixpkgs#hello        # Run hello from nixpkgs
  nix run .#myapp              # Run app from local flake
  nix run github:user/repo     # Run from GitHub
"#,
            _ => r#"Common Nix Commands:

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
"#,
        };

        Ok(CallToolResult::success(vec![Content::text(help_text)]))
    }

    #[tool(description = "Get information about useful Nix ecosystem tools and utilities")]
    fn ecosystem_tools(
        &self,
        Parameters(EcosystemToolArgs { tool }): Parameters<EcosystemToolArgs>,
    ) -> Result<CallToolResult, McpError> {
        let info = match tool.as_deref() {
            Some("comma") | Some(",") => r#"comma - Run programs without installing them
Repository: https://github.com/nix-community/comma
Install: nix-env -iA nixpkgs.comma

Usage:
  , cowsay hello    # Runs cowsay without installing it
  , python3 -c "print('hi')"  # Run Python scripts

Comma uses nix-index to locate and run any program from nixpkgs instantly.
First time may take a while to build the index, but then it's very fast!"#,

            Some("disko") => r#"disko - Declarative disk partitioning and formatting
Repository: https://github.com/nix-community/disko

Declaratively define disk layouts in Nix, including partitions, filesystems,
LUKS encryption, LVM, RAID, and more. Great for automated NixOS installations.

Example use: Define your entire disk layout in configuration.nix
Can be used with nixos-anywhere for remote installations."#,

            Some("nixos-anywhere") => r#"nixos-anywhere - Install NixOS remotely via SSH
Repository: https://github.com/nix-community/nixos-anywhere

Install NixOS on a remote machine from any Linux system via SSH.
Works great with disko for declarative disk setup.

Usage:
  nixos-anywhere --flake '.#my-server' root@192.168.1.10

Perfect for automated server deployments!"#,

            Some("terranix") => r#"terranix - NixOS-like Terraform configurations
Repository: https://github.com/terranix/terranix

Write Terraform configurations in Nix instead of HCL.
Get Nix's module system, type checking, and code reuse for infrastructure.

Benefits:
- Use Nix functions and imports
- Type-safe infrastructure code
- Share modules across projects
- Generate complex Terraform configs programmatically"#,

            Some("noogle") | Some("noogle.dev") => r#"noogle.dev - Search Nix functions and built-ins
Website: https://noogle.dev/

Interactive search for Nix language built-ins and nixpkgs lib functions.
Essential reference when writing Nix expressions.

Search examples:
- "map" - Find list mapping functions
- "filter" - Find filtering functions
- "mkDerivation" - Package building functions

Much faster than reading docs.nixos.org!"#,

            Some("microvm") | Some("microvm.nix") => r#"microvm.nix - Lightweight NixOS VMs
Repository: https://github.com/microvm-nix/microvm.nix

Create ultra-lightweight NixOS VMs (MicroVMs) with minimal overhead.
Uses cloud-hypervisor, firecracker, or qemu.

Benefits:
- Boot in milliseconds
- Minimal memory footprint
- Declarative VM configuration
- Share /nix/store with host (saves space)

Great for development, testing, or running services in isolation."#,

            Some("alejandra") => r#"alejandra - Opinionated Nix code formatter
Repository: https://github.com/kamadorueda/alejandra
Install: nix-shell -p alejandra

Usage:
  alejandra .           # Format all Nix files
  alejandra file.nix    # Format specific file

Alternative to nixpkgs-fmt with different style opinions.
Fast and deterministic formatting."#,

            Some("deadnix") => r#"deadnix - Find and remove dead Nix code
Repository: https://github.com/astro/deadnix
Install: nix-shell -p deadnix

Usage:
  deadnix .                    # Find dead code
  deadnix --edit .             # Remove dead code automatically

Finds unused:
- Function arguments
- Let bindings
- Imports

Helps keep Nix code clean and maintainable."#,

            Some("nix-init") => r#"nix-init - Generate Nix packages from URLs
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

Saves tons of time when packaging software."#,

            Some("statix") => r#"statix - Lints and suggestions for Nix
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

Helps write better, more idiomatic Nix code."#,

            Some("nvd") => r#"nvd - Nix version diff tool
Repository: https://git.sr.ht/~khumba/nvd
Install: nix-shell -p nvd

Usage:
  nvd diff /nix/var/nix/profiles/system-{42,43}-link

Shows what changed between NixOS generations:
- Added/removed packages
- Version upgrades/downgrades
- Size changes

Much more readable than plain nix-store diff!"#,

            Some("nixpkgs-review") => r#"nixpkgs-review - Review nixpkgs pull requests
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
- Tests on multiple platforms"#,

            Some("crane") => r#"crane - Nix library for building Cargo projects
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

Much better than naersk for Rust projects!"#,

            Some("nil") => r#"nil - Nix Language Server (LSP)
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

Much faster and more accurate than other Nix LSPs!"#,

            Some("treefmt-nix") | Some("treefmt") => r#"treefmt-nix - Multi-language formatter manager
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

Formats Nix, Rust, JS, Python, and more in one go!"#,

            Some("git-hooks.nix") | Some("pre-commit-hooks") | Some("pre-commit-hooks.nix") => r#"git-hooks.nix - Pre-commit hooks for Nix projects
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
Prevents bad code from being committed!"#,

            _ => r#"Useful Nix Ecosystem Tools:

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
Example: ecosystem_tools(tool="comma") or ecosystem_tools(tool="crane")"#,
        };

        Ok(CallToolResult::success(vec![Content::text(info)]))
    }

    #[tool(description = "Validate Nix code syntax and check for parse errors")]
    async fn validate_nix(
        &self,
        Parameters(ValidateNixArgs { code }): Parameters<ValidateNixArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Use nix-instantiate --parse to validate syntax
        let child = tokio::process::Command::new("nix-instantiate")
            .args(["--parse", "-E"])
            .arg(&code)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| McpError::internal_error(
                format!("Failed to spawn nix-instantiate: {}", e),
                None
            ))?;

        let output = child.wait_with_output().await
            .map_err(|e| McpError::internal_error(format!("Failed to validate: {}", e), None))?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                "✓ Nix code is valid! No syntax errors found.".to_string()
            )]))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(CallToolResult::success(vec![Content::text(
                format!("✗ Syntax errors found:\n\n{}", stderr)
            )]))
        }
    }

    #[tool(description = "Lint Nix code with statix and/or deadnix to find issues and anti-patterns")]
    async fn lint_nix(
        &self,
        Parameters(LintNixArgs { code, linter }): Parameters<LintNixArgs>,
    ) -> Result<CallToolResult, McpError> {
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
    }

    #[tool(description = "Get detailed information about a package (version, description, homepage, license, etc.)")]
    async fn explain_package(
        &self,
        Parameters(ExplainPackageArgs { package }): Parameters<ExplainPackageArgs>,
    ) -> Result<CallToolResult, McpError> {
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
            .map_err(|e| McpError::internal_error(format!("Failed to get package info: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("Failed to evaluate package: {}", stderr), None));
        }

        let meta: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| McpError::internal_error(format!("Failed to parse metadata: {}", e), None))?;

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
            } else if let Some(name) = license.get("fullName").and_then(|v| v.as_str()) {
                info.push(format!("License: {}", name));
            }
        }

        if let Some(platforms) = meta.get("platforms").and_then(|v| v.as_array()) {
            let platform_list: Vec<String> = platforms.iter()
                .filter_map(|p| p.as_str().map(String::from))
                .take(5)
                .collect();
            if !platform_list.is_empty() {
                info.push(format!("Platforms: {} (showing first 5)", platform_list.join(", ")));
            }
        }

        if let Some(maintainers) = meta.get("maintainers").and_then(|v| v.as_array()) {
            let maint_list: Vec<String> = maintainers.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(String::from))
                .take(3)
                .collect();
            if !maint_list.is_empty() {
                info.push(format!("Maintainers: {}", maint_list.join(", ")));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(info.join("\n"))]))
    }

    #[tool(description = "Prefetch a URL and get its hash for use in Nix expressions")]
    async fn prefetch_url(
        &self,
        Parameters(PrefetchUrlArgs { url, hash_format }): Parameters<PrefetchUrlArgs>,
    ) -> Result<CallToolResult, McpError> {
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

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get metadata about a flake (inputs, outputs, description)")]
    async fn flake_metadata(
        &self,
        Parameters(FlakeMetadataArgs { flake_ref }): Parameters<FlakeMetadataArgs>,
    ) -> Result<CallToolResult, McpError> {
        let output = tokio::process::Command::new("nix")
            .args(["flake", "metadata", "--json", &flake_ref])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to get flake metadata: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("Failed to read flake: {}", stderr), None));
        }

        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| McpError::internal_error(format!("Failed to parse metadata: {}", e), None))?;

        let mut info = Vec::new();

        if let Some(description) = metadata.get("description").and_then(|v| v.as_str()) {
            info.push(format!("Description: {}", description));
        }

        if let Some(url) = metadata.get("url").and_then(|v| v.as_str()) {
            info.push(format!("URL: {}", url));
        }

        if let Some(locked) = metadata.get("locked") {
            if let Some(rev) = locked.get("rev").and_then(|v| v.as_str()) {
                info.push(format!("Revision: {}", &rev[..12.min(rev.len())]));
            }
            if let Some(last_mod) = locked.get("lastModified").and_then(|v| v.as_u64()) {
                info.push(format!("Last Modified: {}", last_mod));
            }
        }

        if let Some(locks) = metadata.get("locks") {
            if let Some(nodes) = locks.get("nodes").and_then(|v| v.as_object()) {
                let inputs: Vec<String> = nodes.keys()
                    .filter(|k| k.as_str() != "root")
                    .map(|k| k.to_string())
                    .collect();
                if !inputs.is_empty() {
                    info.push(format!("\nInputs: {}", inputs.join(", ")));
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(info.join("\n"))]))
    }

    #[tool(description = "Find which package provides a command using nix-locate")]
    async fn find_command(
        &self,
        Parameters(FindCommandArgs { command }): Parameters<FindCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
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
    }

    #[tool(description = "Build a Nix package and show what will be built or the build output")]
    async fn nix_build(
        &self,
        Parameters(NixBuildArgs { package, dry_run }): Parameters<NixBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix build: {}", e), None))?;

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
            let result = if let Ok(json_output) = serde_json::from_str::<serde_json::Value>(&stdout) {
                format!("Dry-run completed successfully.\n\nBuild plan:\n{}",
                    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| stdout.to_string()))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("Dry-run completed successfully.\n\n{}", stderr)
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            // For actual build, show the result
            if let Ok(json_output) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let mut result = String::from("Build completed successfully!\n\n");

                if let Some(arr) = json_output.as_array() {
                    for item in arr {
                        if let Some(drv_path) = item.get("drvPath").and_then(|v| v.as_str()) {
                            result.push_str(&format!("Derivation: {}\n", drv_path));
                        }
                        if let Some(out_paths) = item.get("outputs").and_then(|v| v.as_object()) {
                            result.push_str("Outputs:\n");
                            for (name, path) in out_paths {
                                if let Some(path_str) = path.as_str() {
                                    result.push_str(&format!("  {}: {}\n", name, path_str));
                                }
                            }
                        }
                    }
                }

                result.push_str("\nResult symlink created: ./result\n");
                Ok(CallToolResult::success(vec![Content::text(result)]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(format!("Build completed!\n\n{}", stdout))]))
            }
        }
    }

    #[tool(description = "Explain why one package depends on another (show dependency chain)")]
    async fn why_depends(
        &self,
        Parameters(WhyDependsArgs { package, dependency, show_all }): Parameters<WhyDependsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let show_all = show_all.unwrap_or(false);

        // First, build the package to get its store path
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

        // Build dependency to get its store path
        let dep_build_output = tokio::process::Command::new("nix")
            .args(["build", &dependency, "--json", "--no-link"])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to build dependency: {}", e), None))?;

        if !dep_build_output.status.success() {
            let stderr = String::from_utf8_lossy(&dep_build_output.stderr);
            return Err(McpError::internal_error(format!("Failed to build dependency: {}", stderr), None));
        }

        let dep_stdout = String::from_utf8_lossy(&dep_build_output.stdout);
        let dep_json: serde_json::Value = serde_json::from_str(&dep_stdout)
            .map_err(|e| McpError::internal_error(format!("Failed to parse dependency build output: {}", e), None))?;

        let dependency_path = dep_json
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|item| item.get("outputs"))
            .and_then(|outputs| outputs.get("out"))
            .and_then(|out| out.as_str())
            .ok_or_else(|| McpError::internal_error("Failed to get dependency output path".to_string(), None))?;

        // Now run nix why-depends
        let mut args = vec!["why-depends", package_path, dependency_path];
        if show_all {
            args.push("--all");
        }

        let output = tokio::process::Command::new("nix")
            .args(&args)
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix why-depends: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check if it's because there's no dependency
            if stderr.contains("does not depend on") {
                return Ok(CallToolResult::success(vec![Content::text(
                    format!("{} does not depend on {}", package, dependency)
                )]));
            }

            return Err(McpError::internal_error(format!("why-depends failed: {}", stderr), None));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Show the derivation details of a package (build inputs, environment, etc.)")]
    async fn show_derivation(
        &self,
        Parameters(ShowDerivationArgs { package }): Parameters<ShowDerivationArgs>,
    ) -> Result<CallToolResult, McpError> {
        let output = tokio::process::Command::new("nix")
            .args(["derivation", "show", &package])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix derivation show: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("Failed to show derivation: {}", stderr), None));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try to parse and format nicely
        if let Ok(drv_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            let mut result = String::from("Derivation Details:\n\n");

            // Get the first (and usually only) derivation
            if let Some(obj) = drv_json.as_object() {
                for (drv_path, drv_info) in obj {
                    result.push_str(&format!("Path: {}\n\n", drv_path));

                    if let Some(outputs) = drv_info.get("outputs").and_then(|v| v.as_object()) {
                        result.push_str("Outputs:\n");
                        for (name, info) in outputs {
                            result.push_str(&format!("  - {}\n", name));
                            if let Some(path) = info.get("path").and_then(|v| v.as_str()) {
                                result.push_str(&format!("    Path: {}\n", path));
                            }
                        }
                        result.push('\n');
                    }

                    if let Some(inputs) = drv_info.get("inputDrvs").and_then(|v| v.as_object()) {
                        result.push_str(&format!("Build Dependencies: {} derivations\n", inputs.len()));
                    }

                    if let Some(env) = drv_info.get("env").and_then(|v| v.as_object()) {
                        result.push_str("\nKey Environment Variables:\n");
                        for key in ["name", "version", "src", "builder", "system", "outputs"].iter() {
                            if let Some(value) = env.get(*key).and_then(|v| v.as_str()) {
                                result.push_str(&format!("  {}: {}\n", key, value));
                            }
                        }
                    }

                    result.push_str("\nFull JSON available for detailed inspection.");
                    break; // Only show first derivation in formatted view
                }
            }

            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(stdout.to_string())]))
        }
    }

    #[tool(description = "Get the closure size of a package (total size including all dependencies)")]
    async fn get_closure_size(
        &self,
        Parameters(GetClosureSizeArgs { package, human_readable }): Parameters<GetClosureSizeArgs>,
    ) -> Result<CallToolResult, McpError> {
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

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }

    #[tool(description = "Run a command in a Nix shell with specified packages available")]
    async fn run_in_shell(
        &self,
        Parameters(RunInShellArgs { packages, command, use_flake }): Parameters<RunInShellArgs>,
    ) -> Result<CallToolResult, McpError> {
        let use_flake = use_flake.unwrap_or(false);

        let output = if use_flake {
            // Use nix develop -c
            tokio::process::Command::new("nix")
                .args(["develop", "-c", "sh", "-c", &command])
                .output()
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to run in dev shell: {}", e), None))?
        } else {
            // Use nix-shell -p
            let package_args: Vec<String> = packages.iter()
                .flat_map(|pkg| vec!["-p".to_string(), pkg.clone()])
                .collect();

            let mut args = package_args;
            args.push("--run".to_string());
            args.push(command.clone());

            tokio::process::Command::new("nix-shell")
                .args(&args)
                .output()
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to run in shell: {}", e), None))?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result_text = if output.status.success() {
            format!("Command executed successfully!\n\nOutput:\n{}{}", stdout, stderr)
        } else {
            format!("Command failed with exit code: {:?}\n\nOutput:\n{}\n\nError:\n{}",
                output.status.code(), stdout, stderr)
        };

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }

    #[tool(description = "Show the outputs available in a flake (packages, apps, devShells, etc.)")]
    async fn flake_show(
        &self,
        Parameters(FlakeShowArgs { flake_ref }): Parameters<FlakeShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_ref = flake_ref.unwrap_or_else(|| ".".to_string());

        let output = tokio::process::Command::new("nix")
            .args(["flake", "show", &flake_ref, "--json"])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute nix flake show: {}", e), None))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::internal_error(format!("Failed to show flake: {}", stderr), None));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse and format the flake structure
        if let Ok(flake_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            let mut result = format!("Flake Outputs for: {}\n\n", flake_ref);

            fn format_outputs(value: &serde_json::Value, prefix: String, result: &mut String) {
                match value {
                    serde_json::Value::Object(map) => {
                        for (key, val) in map {
                            if val.is_object() && val.as_object().unwrap().contains_key("type") {
                                let type_str = val["type"].as_str().unwrap_or("unknown");
                                result.push_str(&format!("{}  {}: {}\n", prefix, key, type_str));
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
            Ok(CallToolResult::success(vec![Content::text(stdout.to_string())]))
        }
    }

    #[tool(description = "Get the build log for a package (useful for debugging build failures)")]
    async fn get_build_log(
        &self,
        Parameters(GetBuildLogArgs { package }): Parameters<GetBuildLogArgs>,
    ) -> Result<CallToolResult, McpError> {
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
    }

    #[tool(description = "Compare two derivations to understand what differs between packages (uses nix-diff)")]
    async fn diff_derivations(
        &self,
        Parameters(DiffDerivationsArgs { package_a, package_b }): Parameters<DiffDerivationsArgs>,
    ) -> Result<CallToolResult, McpError> {
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
    }

    // Clan integration tools

    #[tool(description = "Create a new Clan machine configuration")]
    async fn clan_machine_create(
        &self,
        Parameters(ClanMachineCreateArgs { name, template, target_host, flake }): Parameters<ClanMachineCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["machines", "create", &name];

        let template_str = template.unwrap_or_else(|| "new-machine".to_string());
        args.push("-t");
        args.push(&template_str);

        let flake_str = flake.unwrap_or_else(|| ".".to_string());
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Failed to create machine '{}':\n\n{}{}", name, stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Successfully created machine '{}'.\n\n{}{}", name, stdout, stderr)
        )]))
    }

    #[tool(description = "List all Clan machines in the flake")]
    async fn clan_machine_list(
        &self,
        Parameters(ClanMachineListArgs { flake }): Parameters<ClanMachineListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        let output = tokio::process::Command::new("clan")
            .args(["machines", "list", "--flake", &flake_str])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Failed to list machines:\n\n{}{}", stdout, stderr)
            )]));
        }

        let result = if stdout.trim().is_empty() {
            "No machines configured in this Clan flake.".to_string()
        } else {
            format!("Clan Machines:\n\n{}", stdout)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Update Clan machine(s) - rebuilds and deploys configuration")]
    async fn clan_machine_update(
        &self,
        Parameters(ClanMachineUpdateArgs { machines, flake }): Parameters<ClanMachineUpdateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["machines", "update"];

        let flake_str = flake.unwrap_or_else(|| ".".to_string());
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Machine update failed:\n\n{}{}", stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Machine update completed.\n\n{}{}", stdout, stderr)
        )]))
    }

    #[tool(description = "Delete a Clan machine configuration")]
    async fn clan_machine_delete(
        &self,
        Parameters(ClanMachineDeleteArgs { name, flake }): Parameters<ClanMachineDeleteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        let output = tokio::process::Command::new("clan")
            .args(["machines", "delete", &name, "--flake", &flake_str])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Failed to delete machine '{}':\n\n{}{}", name, stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Successfully deleted machine '{}'.\n\n{}{}", name, stdout, stderr)
        )]))
    }

    #[tool(description = "Install Clan machine to a target host via SSH (WARNING: Destructive - overwrites disk)")]
    async fn clan_machine_install(
        &self,
        Parameters(ClanMachineInstallArgs { machine, target_host, flake, confirm }): Parameters<ClanMachineInstallArgs>,
    ) -> Result<CallToolResult, McpError> {
        if !confirm.unwrap_or(false) {
            return Ok(CallToolResult::success(vec![Content::text(
                format!(
                    "WARNING: Installing machine '{}' to '{}' will OVERWRITE THE DISK!\n\n\
                    This is a destructive operation that will:\n\
                    - Partition and format the target disk\n\
                    - Install NixOS\n\
                    - Deploy the Clan configuration\n\n\
                    To proceed, call this function again with confirm=true",
                    machine, target_host
                )
            )]));
        }

        let flake_str = flake.unwrap_or_else(|| ".".to_string());

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
    }

    #[tool(description = "Create a backup for a Clan machine")]
    async fn clan_backup_create(
        &self,
        Parameters(ClanBackupCreateArgs { machine, provider, flake }): Parameters<ClanBackupCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["backups", "create", &machine];

        let flake_str = flake.unwrap_or_else(|| ".".to_string());
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Backup creation failed:\n\n{}{}", stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Backup created for machine '{}'.\n\n{}{}", machine, stdout, stderr)
        )]))
    }

    #[tool(description = "List backups for a Clan machine")]
    async fn clan_backup_list(
        &self,
        Parameters(ClanBackupListArgs { machine, provider, flake }): Parameters<ClanBackupListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["backups", "list", &machine];

        let flake_str = flake.unwrap_or_else(|| ".".to_string());
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Failed to list backups:\n\n{}{}", stdout, stderr)
            )]));
        }

        let result = if stdout.trim().is_empty() {
            format!("No backups found for machine '{}'.", machine)
        } else {
            format!("Backups for machine '{}':\n\n{}", machine, stdout)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Restore a backup for a Clan machine")]
    async fn clan_backup_restore(
        &self,
        Parameters(ClanBackupRestoreArgs { machine, provider, name, service, flake }): Parameters<ClanBackupRestoreArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["backups", "restore", &machine, &provider, &name];

        let flake_str = flake.unwrap_or_else(|| ".".to_string());
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Backup restore failed:\n\n{}{}", stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Backup '{}' restored for machine '{}'.\n\n{}{}", name, machine, stdout, stderr)
        )]))
    }

    #[tool(description = "Create a new Clan flake from a template")]
    async fn clan_flake_create(
        &self,
        Parameters(ClanFlakeCreateArgs { directory, template }): Parameters<ClanFlakeCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
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
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Failed to create Clan flake:\n\n{}{}", stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Clan flake created in '{}'.\n\n{}{}", directory, stdout, stderr)
        )]))
    }

    #[tool(description = "List secrets in a Clan flake")]
    async fn clan_secrets_list(
        &self,
        Parameters(ClanSecretsListArgs { flake }): Parameters<ClanSecretsListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        let output = tokio::process::Command::new("clan")
            .args(["secrets", "list", "--flake", &flake_str])
            .output()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Failed to list secrets:\n\n{}{}", stdout, stderr)
            )]));
        }

        let result = if stdout.trim().is_empty() {
            "No secrets configured.".to_string()
        } else {
            format!("Clan Secrets:\n\n{}", stdout)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Create and run a VM for a Clan machine (useful for testing)")]
    async fn clan_vm_create(
        &self,
        Parameters(ClanVmCreateArgs { machine, flake }): Parameters<ClanVmCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

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
    }

    #[tool(description = "Get help and information about Clan - the peer-to-peer NixOS management framework")]
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
        let project_type = args.get("project_type")
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
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
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
