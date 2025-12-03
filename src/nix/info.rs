use crate::common::security::audit::AuditLogger;
use crate::nix::types::{EcosystemToolArgs, NixCommandHelpArgs};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use std::sync::Arc;

/// Informational tools providing Nix documentation and ecosystem guidance.
///
/// This struct provides read-only informational tools that help users learn
/// about Nix commands, patterns, and ecosystem tools. These tools do not
/// execute commands or modify state - they only return helpful documentation.
///
/// # Available Operations
///
/// - **Command Help**: [`nix_command_help`](Self::nix_command_help)
/// - **Ecosystem Info**: [`ecosystem_tools`](Self::ecosystem_tools)
///
/// # Caching Strategy
///
/// No caching needed (static documentation content).
///
/// # Security
///
/// All operations are read-only and logged:
/// - No command execution
/// - No state modification
/// - Audit logging for usage tracking
///
/// # Coverage
///
/// **nix_command_help** covers:
/// - `nix develop` - Development shells
/// - `nix build` - Building packages
/// - `nix flake` - Flake management
/// - `nix run` - Running packages
/// - `nix shell` - Temporary shells
///
/// **ecosystem_tools** covers:
/// - comma - Run programs without installing
/// - disko - Declarative disk partitioning
/// - nixos-generators - Generate NixOS images
/// - alejandra - Nix code formatter
/// - statix - Nix linter
/// - And more...
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::nix::InfoTools;
/// use onix_mcp::nix::types::NixCommandHelpArgs;
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # fn example(tools: InfoTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Get help for nix develop command
/// let result = tools.nix_command_help(Parameters(NixCommandHelpArgs {
///     command: Some("develop".to_string()),
/// }))?;
/// # Ok(())
/// # }
/// ```
pub struct InfoTools {
    pub audit: Arc<AuditLogger>,
}

impl InfoTools {
    /// Creates a new `InfoTools` instance with audit logging.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for usage tracking
    ///
    /// # Note
    ///
    /// InfoTools contains only synchronous, read-only operations that
    /// return static documentation. No caching or timeouts are needed.
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl InfoTools {
    #[tool(
        description = "Get help with common Nix commands and patterns",
        annotations(read_only_hint = true)
    )]
    pub fn nix_command_help(
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
    pub fn ecosystem_tools(
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
}
