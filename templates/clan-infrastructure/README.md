# Multi-Machine Clan Infrastructure with Onix-MCP

This template provides a production-ready multi-machine Clan configuration with onix-mcp enabled on all machines.

## Architecture

This template includes four machines:

1. **web-01**: Web server with nginx
2. **db-01**: Database server with PostgreSQL
3. **monitor-01**: Monitoring with Grafana + Prometheus
4. **dev-01**: Development machine with common tools

All machines include:
- Onix-MCP service for Nix operations
- SSH with key-based authentication
- Firewall configuration
- Automatic system updates
- Common security hardening

## Quick Start

1. Initialize the infrastructure:

```bash
nix flake init -t github:yourorg/onix-mcp#clan-infrastructure
```

2. Customize the configuration:

Edit `flake.nix` to:
- Update `inputs.onix-mcp.url`
- Set correct `targetHost` for each machine
- Add SSH public keys
- Adjust machine configurations

3. Enter development shell:

```bash
nix develop
```

4. List machines:

```bash
clan machines list
```

5. Build a specific machine:

```bash
nix build .#clanInternals.machines.web-01.config.system.build.toplevel
```

6. Deploy a machine:

```bash
clan machines update web-01
```

## Machine Details

### web-01 (Web Server)

**Ports**: 80, 443
**Services**: nginx
**Tags**: web, production

Configuration:
- Optimized nginx settings
- TLS/SSL ready
- Gzip compression enabled

### db-01 (Database Server)

**Ports**: 5432
**Services**: PostgreSQL
**Tags**: database, production

Configuration:
- PostgreSQL with app database
- Network authentication configured
- Ready for application connections

### monitor-01 (Monitoring)

**Ports**: 3000 (Grafana), 9090 (Prometheus)
**Services**: Grafana, Prometheus
**Tags**: monitoring, production

Configuration:
- Grafana dashboard on port 3000
- Prometheus metrics on port 9090
- Node exporter enabled

### dev-01 (Development)

**Ports**: 22 (SSH only)
**Services**: Standard development tools
**Tags**: development

Configuration:
- vim, git, tmux, htop installed
- Minimal footprint
- Suitable for testing and development

## Deployment Workflows

### Initial Deployment

```bash
# Deploy all machines
clan machines update

# Or deploy specific machines
clan machines update web-01
clan machines update db-01
```

### Update Single Machine

```bash
clan machines update web-01
```

### Create Backups

```bash
# Backup a specific machine
clan backup create db-01

# List backups
clan backup list db-01
```

### Test in VM

```bash
# Create VM for testing
clan vms create web-01

# The VM will start with your configuration
```

## Customization

### Add a New Machine

1. Define the machine in `flake.nix`:

```nix
machines = {
  # ... existing machines ...

  app-01 = { config, ... }: {
    imports = [ commonConfig ];
    networking.hostName = "app-01";
    clan.networking.targetHost = "app-01.example.com";
    clan.tags = [ "application" "production" ];

    # Add your application-specific config
  };
};
```

2. Deploy:

```bash
clan machines update app-01
```

### Modify Common Configuration

Edit `commonConfig` in `flake.nix` to change settings applied to all machines:

```nix
commonConfig = { config, ... }: {
  # Your common settings
  services.onix-mcp.logLevel = "debug";  # Change log level
  # ... other settings
};
```

### Add Machine-Specific Configuration

Create a new configuration module like `webServerConfig`:

```nix
myServiceConfig = { config, ... }: {
  # Your service configuration
  services.myapp.enable = true;
};
```

Then import it in your machine definition.

## Clan Features

### Secrets Management

```bash
# Add a secret
clan secrets set db-password --machine db-01

# List secrets
clan secrets list
```

### Tags and Filtering

Update machines by tag:

```bash
# Update all production machines
clan machines update --tag production

# Update all web servers
clan machines update --tag web
```

### Backup and Restore

```bash
# Create backup
clan backup create db-01

# List backups
clan backup list db-01

# Restore from backup
clan backup restore db-01 <backup-id>
```

## Onix-MCP on Each Machine

Each machine runs onix-mcp, providing 80+ tools for:

- Package management and search
- Build operations and debugging
- Code quality tools
- Dependency analysis
- Flake management

Access via MCP client connected to each machine's socket.

## Monitoring and Maintenance

### Check Service Status

```bash
# SSH to machine
ssh admin@web-01.example.com

# Check onix-mcp
systemctl status onix-mcp

# View logs
journalctl -u onix-mcp -f
```

### System Updates

Automatic updates are enabled but reboot is manual:

```bash
# Check for updates
nixos-rebuild dry-build --flake .#web-01

# Apply updates
clan machines update web-01
```

## Security Considerations

All machines include:

- ✅ SSH key-based authentication only
- ✅ Firewall enabled with minimal ports
- ✅ No root password login
- ✅ Onix-MCP runs as unprivileged user
- ✅ Systemd security hardening
- ✅ Automatic security updates

Review `commonConfig` and adjust firewall rules for your needs.

## Troubleshooting

### Machine Won't Deploy

```bash
# Check configuration
nix flake check

# Dry-run build
nix build .#clanInternals.machines.web-01.config.system.build.toplevel --dry-run
```

### Service Won't Start

```bash
# On the machine
systemctl status onix-mcp
journalctl -u onix-mcp -n 50
```

### Network Issues

Verify `clan.networking.targetHost` is correct and reachable:

```bash
ping web-01.example.com
ssh admin@web-01.example.com
```

## Documentation

- [Clan Documentation](https://docs.clan.lol)
- [Onix-MCP Documentation](https://github.com/yourorg/onix-mcp)
- [NixOS Manual](https://nixos.org/manual/nixos/stable/)

## Next Steps

1. Customize machine configurations for your needs
2. Set up secrets management with `clan secrets`
3. Configure backups with `clan backup`
4. Add monitoring and alerting
5. Set up CI/CD for automated deployments
