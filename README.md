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
