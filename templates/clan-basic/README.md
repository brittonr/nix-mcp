# Basic Clan Infrastructure with Onix-MCP

This template provides a minimal Clan configuration with onix-mcp enabled for Nix ecosystem operations.

## Quick Start

1. Initialize a new Clan project:

```bash
nix flake init -t github:yourorg/onix-mcp#clan-basic
```

2. Customize the configuration:

- Edit `flake.nix` to:
  - Update `inputs.onix-mcp.url` with the correct repository URL
  - Add your SSH public key to `users.users.admin.openssh.authorizedKeys.keys`
  - Adjust machine name and hostname as needed

3. Build the machine configuration:

```bash
nix build .#clanInternals.machines.example-machine.config.system.build.toplevel
```

4. Deploy to a remote machine:

```bash
clan machines update example-machine
```

## What's Included

- **Onix-MCP Service**: Enabled by default with info-level logging
- **SSH Access**: Configured for secure remote management
- **Basic Firewall**: Enabled with SSH port 22 open
- **Admin User**: With sudo access (wheel group)
- **Clan Integration**: Full Clan.lol deployment lifecycle support

## Onix-MCP Features

With onix-mcp enabled, you get 80+ tools for:

- Nix package management and search
- Build operations and debugging
- Code quality (linters, formatters)
- Dependency analysis
- Flake management
- Task queue management (Pueue)
- Interactive process control (Pexpect)

## Customization

### Change Log Level

```nix
services.onix-mcp.logLevel = "debug"; # error, warn, info, debug, trace
```

### Add Environment Variables

```nix
services.onix-mcp.extraEnvironment = {
  RUST_BACKTRACE = "1";
};
```

### Change Socket Path

```nix
services.onix-mcp.socketPath = "/run/my-custom-path/socket";
```

### Disable Socket Activation

```nix
services.onix-mcp.socketActivation = false;
```

## Next Steps

1. **Add More Machines**: Copy the `example-machine` block in `flake.nix`
2. **Configure Secrets**: Use `clan secrets` commands for sensitive data
3. **Set Up Backups**: Configure `clan backup` for your machines
4. **Create VMs for Testing**: Use `clan vms create example-machine`

## Documentation

- Clan Documentation: https://docs.clan.lol
- Onix-MCP Documentation: https://github.com/yourorg/onix-mcp
- NixOS Manual: https://nixos.org/manual/nixos/stable/

## Troubleshooting

### Check Service Status

```bash
systemctl status onix-mcp
```

### View Logs

```bash
journalctl -u onix-mcp -f
```

### Test Configuration

```bash
nix flake check
```

### Verify Onix-MCP

```bash
# On the deployed machine
systemctl status onix-mcp
```
