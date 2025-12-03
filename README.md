# Onix-MCP

Model Context Protocol (MCP) server providing 80+ tools for Nix ecosystem operations and Clan infrastructure management.

## Features

- **Nix Package Management**: Search, inspect, locate files, prefetch URLs
- **Build Operations**: Build packages, run shells, execute binaries, analyze logs
- **Code Quality**: Linters and formatters for Rust, Python, Nix, Shell, TOML
- **Dependency Analysis**: Trace dependencies, compare derivations, measure closure sizes
- **Flake Management**: Inspect metadata, create and migrate flakes
- **Clan Infrastructure**: Peer-to-peer NixOS deployment lifecycle (machines, backups, secrets, VMs)
- **Task Queue**: Async job management via Pueue
- **Process Control**: Interactive session management with pexpect

## Quick Start

### Build

```sh
nix build
```

### Run

```sh
nix run
```

### Development

```sh
nix develop
nix develop -c cargo build
nix develop -c cargo nextest run
```

## Getting Started: Your First Workflow

Let's get you productive with onix-mcp in under 5 minutes! This guide shows common workflows that demonstrate the server's capabilities.

### Step 1: Search for a Package (30 seconds)

Ask your MCP client (e.g., Claude):

> "Search for ripgrep in nixpkgs"

The server will use the `search_packages` tool and show:

```
Found packages matching 'ripgrep':

Package: nixpkgs.ripgrep
Version: 14.1.0
Description: A search tool that combines the usability of ag with the raw speed of grep
```

### Step 2: Get Package Details (30 seconds)

Ask:

> "Explain the ripgrep package"

The server will use the `explain_package` tool and show:

```
Package: ripgrep
Version: 14.1.0
License: MIT
Homepage: https://github.com/BurntSushi/ripgrep
Platforms: x86_64-linux, aarch64-linux, aarch64-darwin, x86_64-darwin
Description: A search tool that combines the usability of ag with the raw speed of grep
```

### Step 3: Run Without Installing (1 minute)

Ask:

> "Run ripgrep with --version using comma"

The server will use the `comma` tool:

```
ripgrep 14.1.0
```

What just happened? Nix downloaded ripgrep temporarily, ran it, and cleaned up - no permanent installation needed!

### Common Workflows

Here are the most useful workflows with onix-mcp:

| Goal | Tools Used | Example Query |
|------|------------|---------------|
| Find a package | `search_packages` → `explain_package` | "Find and explain the htop package" |
| Try before installing | `comma` or `nix_run` | "Run cowsay with comma" |
| Debug a build failure | `nix_build` (dry-run) → `get_build_log` | "Show what's needed to build firefox, then show build logs" |
| Understand dependencies | `why_depends` → `get_closure_size` | "Why does firefox depend on libx11? What's the total closure size?" |
| Set up dev environment | `run_in_shell` | "Run my Python script with numpy and pandas available" |
| Locate a file's package | `nix_locate` → `get_package_info` | "Which package provides bin/gcc?" |
| Format Nix code | `format_nix` → `lint_nix` | "Format and lint this Nix code: { }" |
| Create a flake | `generate_flake` | "Generate a flake.nix for a Rust project" |

### Tool Comparison: When to Use What?

Confused about which tool to use? These comparison matrices help you choose the right tool for your task.

#### Running Packages: Which Tool to Use?

Different tools for different needs - choose based on what you know and what you need:

| Scenario | Tool | When to Use | Speed | Example |
|----------|------|-------------|-------|---------|
| Run package's main binary | `nix_run` | Know exact package name (e.g., `nixpkgs#hello`) | Fast | `nix_run nixpkgs#cowsay Hello` |
| Run any command | `comma` | Don't know which package; auto-discovery needed | Medium | `comma cowsay Hello` (finds package automatically) |
| Command with dependencies | `run_in_shell` | Need multiple packages in environment | Medium | `run_in_shell ["python3", "numpy"] "python script.py"` |
| Quick one-off test | `comma` | Fastest for ad-hoc exploration | Medium | Try unfamiliar commands instantly |
| Reproducible automation | `run_in_shell` | CI/CD, scripts - explicit dependencies | Medium | Documented environment requirements |
| Find command location | `find_command` + `nix_run` | Locate which package, then run it | Slow | Two-step: find then execute |

**Decision Flow**: Know exact package? → `nix_run`. Need auto-discovery? → `comma`. Multiple packages? → `run_in_shell`.

#### Building vs Analyzing: Understanding Your Package

Choose based on whether you want to build or just understand:

| Goal | Tool | What It Does | Speed | Best For |
|------|------|--------------|-------|----------|
| Preview build plan | `nix_build` (dry_run) | Shows what would be built/downloaded | Fast | Check impact before building |
| Actually build | `nix_build` | Builds package, returns store path | Slow | Testing builds, getting binaries |
| Trace dependencies | `why_depends` | Shows full dependency chain A→B→C | Fast | Understanding why package X needs Y |
| Measure total size | `get_closure_size` | Size with ALL dependencies | Fast | Planning disk space, optimizing images |
| Compare packages | `diff_derivations` | Differences between two versions | Fast | Understanding version changes |
| Debug build failure | `get_build_log` | Complete build output & errors | Fast | Troubleshooting compilation issues |
| Inspect derivation | `show_derivation` | Raw derivation attributes & paths | Fast | Deep debugging, understanding builds |

**Decision Flow**: Build failed? → `get_build_log`. Want size? → `get_closure_size`. Why dependency X? → `why_depends`. Before building → `nix_build --dry-run`.

#### Code Quality: Formatting & Linting

Different tools for different quality checks:

| Task | Tool | What It Checks | Auto-Fix | When to Use |
|------|------|----------------|----------|-------------|
| Format Nix code | `format_nix` | Code style consistency | ✅ Yes | Before committing Nix code |
| Format entire project | `nix_fmt` | All Nix files in project | ✅ Yes | Batch formatting |
| Lint Nix code | `lint_nix` | Anti-patterns, dead code | ❌ No | Code review, quality checks |
| Validate syntax | `validate_nix` | Parse errors, basic syntax | ❌ No | Quick syntax verification |
| Run all checks | `pre_commit_run` | Format + lint + custom hooks | ✅ Partial | Before git commit |
| Check hooks status | `check_pre_commit_status` | Hook configuration health | N/A | Setup verification |

**Decision Flow**: Quick syntax check? → `validate_nix`. Find issues? → `lint_nix`. Fix formatting? → `format_nix` or `nix_fmt`. All at once? → `pre_commit_run`.

#### Package Discovery: Finding What You Need

Different strategies for finding packages:

| Goal | Tool | How It Works | Best For |
|------|------|--------------|----------|
| Search by name/description | `search_packages` | Full-text search in nixpkgs | Know roughly what you want |
| Find file provider | `nix_locate` | Which package provides `/bin/gcc`? | Have file path, need package |
| Get package details | `get_package_info` | Version, license, platforms, etc. | Deep dive on specific package |
| Explain package | `explain_package` | Human-friendly package summary | Quick overview |
| Find command | `find_command` | Which package has the `gcc` command? | Know command name only |

**Decision Flow**: Know name? → `search_packages`. Have file path? → `nix_locate`. Have command name? → `find_command`. Want details? → `get_package_info`.

#### Flake Operations: Modern Nix Workflows

Choose based on what you're doing with flakes:

| Task | Tool | Purpose | Output |
|------|------|---------|--------|
| Inspect flake | `flake_metadata` | Inputs, outputs, description | JSON metadata |
| Show flake structure | `flake_show` | All outputs (packages, apps, etc.) | Tree structure |
| Prefetch URL hash | `prefetch_url` | Get hash for fetchurl/fetchgit | SHA256/SRI hash |
| Evaluate expression | `nix_eval` | Test Nix expressions | Evaluation result |

**Decision Flow**: Understand flake? → `flake_metadata` or `flake_show`. Need URL hash? → `prefetch_url`. Test expression? → `nix_eval`.

### Quick Tips

**Caching is Automatic**: The server caches results to be fast. Package searches, metadata lookups, and build queries all benefit from intelligent caching.

**Validation is Built-in**: All inputs are validated to prevent command injection and other security issues. You can safely pass user input to tools.

**Timeouts are Enforced**: Long-running operations have automatic timeouts to prevent resource exhaustion.

**Audit Logging**: All operations are logged for security and debugging purposes.

## Installation

### Prerequisites

- Nix package manager with flakes enabled
- An MCP client that supports stdio transport (Claude Desktop, Continue, etc.)

### Install Locally from Clone

1. Build the project:
```sh
cd /path/to/onix-mcp
nix build
```

2. Configure your MCP client using the absolute path to the built binary:

```json
{
  "mcpServers": {
    "onix-mcp": {
      "command": "/absolute/path/to/onix-mcp/result/bin/onix-mcp"
    }
  }
}
```

**Note:** Replace `/absolute/path/to/onix-mcp` with the actual absolute path to your clone.

### MCP Client Configuration Locations

Common MCP client configuration file locations:

- **Claude Desktop**: `~/.config/claude/claude_desktop_config.json` (Linux/macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows)
- **Continue**: `.continue/config.json` in your project or home directory
- **Other clients**: Refer to your MCP client's documentation

### Verifying Installation

After configuring your MCP client, restart it to load the server. You should see `onix-mcp` with tools for:
- Nix package management and search
- Build and development utilities
- Clan infrastructure management
- Code quality tools

### Troubleshooting

If the server doesn't appear:

1. Verify Nix is installed: `nix --version`
2. Ensure flakes are enabled in your Nix configuration
3. Check the MCP client logs for connection errors
4. For local builds, verify the binary exists: `ls result/bin/onix-mcp`
5. Test the server manually: `nix run . -- --version` (from repo directory)

### Updating

Pull latest changes and rebuild:
```sh
git pull
nix build
```

## Available Tools

### Package Management

**search_packages** - Search for packages in nixpkgs
- `query` (string): Search query for package name or description
- `limit` (number, optional): Maximum results to return (default: 10)

**get_package_info** - Get detailed information about a package
- `package` (string): Package attribute path (e.g., "nixpkgs#ripgrep")

**nix_locate** - Find which package provides a file
- `path` (string): File path to search for
- `limit` (number, optional): Maximum results (default: 10)

**explain_package** - Get detailed explanation of a package
- `package` (string): Package attribute path

**prefetch_url** - Download URL and generate Nix hash
- `url` (string): URL to prefetch
- `hash_format` (string, optional): "sha256" or "sri" (default: "sri")

### Build and Development

**nix_build** - Build a Nix package
- `package` (string): Package or flake reference to build
- `show_trace` (boolean, optional): Show detailed error trace
- `keep_going` (boolean, optional): Continue building despite failures

**nix_develop** - Enter development shell for a flake
- `flake_ref` (string, optional): Flake reference (default: current directory)

**nix_run** - Run a package binary
- `package` (string): Package to run
- `args` (array, optional): Arguments to pass to the program

**run_in_shell** - Run command in nix-shell with packages
- `packages` (array): List of packages to make available
- `command` (string): Command to execute

**get_build_log** - Get the build log for a package
- `package` (string): Package to get build log for

**nix_log** - Search build logs with grep
- `package` (string): Package derivation or store path
- `pattern` (string, optional): Grep pattern to search for

### Code Quality

**format_nix** - Format Nix code
- `code` (string): Nix code to format

**validate_nix** - Validate Nix syntax
- `code` (string): Nix code to validate

**lint_nix** - Lint Nix code for issues
- `code` (string): Nix code to lint
- `linter` (string, optional): "statix", "deadnix", or "both" (default: "both")

### Flakes

**flake_metadata** - Show flake metadata
- `flake_ref` (string): Flake reference

**flake_show** - Show flake outputs
- `flake_ref` (string): Flake reference

**generate_flake** - Generate a new flake.nix
- `language` (string): Programming language
- `description` (string, optional): Project description

**migrate_to_flakes** - Migrate project to flakes
- `current_setup` (string): Description of current setup

### Analysis

**search_options** - Search NixOS options
- `query` (string): Search query

**nix_eval** - Evaluate a Nix expression
- `expression` (string): Nix expression to evaluate

**why_depends** - Show why a package depends on another
- `package` (string): Package to analyze
- `dependency` (string): Dependency to trace

**show_derivation** - Show derivation details
- `package` (string): Package attribute path

**get_closure_size** - Get total size of package closure
- `package` (string): Package to analyze
- `human_readable` (boolean, optional): Format size in human-readable form

**diff_derivations** - Compare two derivations
- `derivation1` (string): First derivation path
- `derivation2` (string): Second derivation path

**find_command** - Find nix commands by description
- `query` (string): Search query

### Clan.lol Tools

**clan_machine_create** - Create new Clan machine
- `machine_name` (string): Name for the machine
- `flake_dir` (string, optional): Flake directory path

**clan_machine_list** - List all Clan machines
- `flake_dir` (string, optional): Flake directory path

**clan_machine_update** - Update Clan machine configuration
- `machine_name` (string): Machine to update
- `flake_dir` (string, optional): Flake directory path

**clan_machine_delete** - Delete a Clan machine (destructive)
- `machine_name` (string): Machine to delete
- `flake_dir` (string, optional): Flake directory path

**clan_machine_install** - Install Clan machine to hardware (destructive)
- `machine_name` (string): Machine to install
- `target_host` (string, optional): Target host

**clan_backup_create** - Create backup for Clan machine
- `machine_name` (string): Machine to backup
- `provider` (string, optional): Backup provider

**clan_backup_list** - List backups for a machine
- `machine_name` (string): Machine name
- `provider` (string, optional): Backup provider

**clan_backup_restore** - Restore backup (destructive)
- `machine_name` (string): Machine to restore to
- `backup_id` (string): Backup to restore

**clan_flake_create** - Create new Clan flake
- `directory` (string): Directory to create flake in
- `template` (string, optional): Template to use

**clan_secrets_list** - List secrets for a machine
- `machine_name` (string): Machine name
- `flake_dir` (string, optional): Flake directory

**clan_vm_create** - Create and run VM for testing
- `machine_name` (string): Machine to create VM for
- `flake_dir` (string, optional): Flake directory

**clan_machine_build** - Build Clan machine configuration locally for testing
- `machine` (string): Machine name to build
- `flake` (string, optional): Flake directory path
- `use_nom` (boolean, optional): Use nom for better build output

**nixos_build** - Build NixOS machine configuration from flake
- `machine` (string): Machine configuration name
- `flake` (string, optional): Flake reference
- `use_nom` (boolean, optional): Use nom for better build output

**clan_analyze_secrets** - Analyze Clan secret (ACL) ownership across machines
- `flake` (string, optional): Flake directory path
- Note: Uses packages from local flake or falls back to github:onixcomputer/onix-core

**clan_analyze_vars** - Analyze Clan vars ownership across machines
- `flake` (string, optional): Flake directory path
- Note: Uses packages from local flake or falls back to github:onixcomputer/onix-core

**clan_analyze_tags** - Analyze Clan machine tags
- `flake` (string, optional): Flake directory path
- Note: Uses packages from local flake or falls back to github:onixcomputer/onix-core

**clan_analyze_roster** - Analyze Clan user roster configurations
- `flake` (string, optional): Flake directory path
- Note: Uses packages from local flake or falls back to github:onixcomputer/onix-core

### Task Queue (Pueue)

**pueue_add** - Add task to queue
- `command` (string): Command to queue
- `label` (string, optional): Task label

**pueue_status** - Get queue status

**pueue_log** - Get task logs
- `task_id` (number, optional): Specific task ID

**pueue_wait** - Wait for tasks to complete
- `task_ids` (array, optional): Task IDs to wait for

**pueue_remove** - Remove task from queue
- `task_id` (number): Task ID to remove

**pueue_clean** - Clean finished tasks

**pueue_pause** - Pause queue

**pueue_start** - Start/resume queue

### Interactive Processes (Pexpect)

**pexpect_start** - Start interactive process
- `command` (string): Command to run
- `timeout` (number, optional): Timeout in seconds

**pexpect_send** - Send input to process
- `session_id` (string): Session ID from pexpect_start
- `input` (string): Text to send

**pexpect_close** - Close interactive process
- `session_id` (string): Session ID to close

### Utilities

**setup_dev_environment** - Set up development environment
- `language` (string): Programming language
- `features` (array, optional): Additional features

**troubleshoot_build** - Diagnose build failures
- `error_message` (string): Build error output

**optimize_closure** - Suggest ways to reduce closure size
- `package` (string): Package to analyze

## Prompts

**complete** - Get code completion suggestions
- `code` (string): Code context
- `language` (string): Programming language
- `cursor_position` (number, optional): Cursor position

**initialize** - Initialize new project
- `language` (string): Programming language
- `features` (array, optional): Features to include

## Resources

**list_resources** - List available documentation resources

**read_resource** - Read a specific documentation resource
- `uri` (string): Resource URI

**list_resource_templates** - List available templates

## Security

All tools implement:
- Input validation to prevent command injection
- Timeout protection against resource exhaustion
- Audit logging of all operations
- Safety annotations (read-only, destructive, idempotent)

See [SECURITY.md](SECURITY.md) for detailed security documentation.

## Performance

The server includes aggressive caching for expensive operations:
- Package searches: 10 minute TTL
- Package info: 30 minute TTL
- nix_locate: 5 minute TTL
- URL prefetch: 24 hour TTL

See [PERFORMANCE.md](PERFORMANCE.md) for performance benchmarks.

## Architecture

- Written in Rust using the rmcp MCP SDK
- Uses tokio async runtime
- Wraps Nix CLI tools with validation and caching
- Includes Clan.lol tools for NixOS deployment

## License

AGPL-3.0
