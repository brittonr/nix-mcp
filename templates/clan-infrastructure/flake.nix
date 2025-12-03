{
  description = "Multi-machine Clan infrastructure with onix-mcp";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    clan-core.url = "git+https://git.clan.lol/clan/clan-core";
    onix-mcp = {
      url = "github:yourorg/onix-mcp";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, clan-core, onix-mcp, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      # Common configuration shared across machines
      commonConfig = { config, ... }: {
        # Import onix-mcp module
        imports = [ onix-mcp.nixosModules.default ];

        # Enable onix-mcp service
        services.onix-mcp = {
          enable = true;
          logLevel = "info";
        };

        # Common system configuration
        system.stateVersion = "24.05";

        # SSH configuration
        services.openssh = {
          enable = true;
          settings = {
            PermitRootLogin = "no";
            PasswordAuthentication = false;
          };
        };

        # Firewall
        networking.firewall.enable = true;

        # Automatic updates
        system.autoUpgrade = {
          enable = true;
          allowReboot = false;
        };
      };

      # Web server specific configuration
      webServerConfig = { config, ... }: {
        networking.firewall.allowedTCPPorts = [ 80 443 ];

        # Example: nginx
        services.nginx = {
          enable = true;
          recommendedGzipSettings = true;
          recommendedOptimisation = true;
          recommendedProxySettings = true;
          recommendedTlsSettings = true;
        };
      };

      # Database server specific configuration
      dbServerConfig = { config, ... }: {
        networking.firewall.allowedTCPPorts = [ 5432 ];

        # Example: PostgreSQL
        services.postgresql = {
          enable = true;
          ensureDatabases = [ "app" ];
          authentication = ''
            host all all 10.0.0.0/8 md5
          '';
        };
      };

      # Monitoring server configuration
      monitoringConfig = { config, ... }: {
        networking.firewall.allowedTCPPorts = [ 3000 9090 ];

        # Example: Grafana + Prometheus
        services.grafana = {
          enable = true;
          settings.server.http_addr = "0.0.0.0";
        };

        services.prometheus = {
          enable = true;
          exporters.node.enable = true;
        };
      };
    in
    {
      clanInternals = clan-core.lib.buildClan {
        directory = self;
        specialArgs = { inherit onix-mcp; };

        machines = {
          # Web server
          web-01 = { config, ... }: {
            imports = [ commonConfig webServerConfig ];
            networking.hostName = "web-01";

            # Machine-specific configuration
            clan.networking.targetHost = "web-01.example.com";
            clan.tags = [ "web" "production" ];
          };

          # Database server
          db-01 = { config, ... }: {
            imports = [ commonConfig dbServerConfig ];
            networking.hostName = "db-01";

            clan.networking.targetHost = "db-01.example.com";
            clan.tags = [ "database" "production" ];
          };

          # Monitoring server
          monitor-01 = { config, ... }: {
            imports = [ commonConfig monitoringConfig ];
            networking.hostName = "monitor-01";

            clan.networking.targetHost = "monitor-01.example.com";
            clan.tags = [ "monitoring" "production" ];
          };

          # Development machine (optional)
          dev-01 = { config, ... }: {
            imports = [ commonConfig ];
            networking.hostName = "dev-01";

            # Development-specific packages
            environment.systemPackages = with pkgs; [
              vim
              git
              tmux
              htop
            ];

            clan.networking.targetHost = "dev-01.example.com";
            clan.tags = [ "development" ];
          };
        };
      };

      # Development shell with Clan CLI and Onix-MCP
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          clan-core.packages.${system}.clan-cli
          onix-mcp.packages.${system}.default
          nixpkgs-fmt
        ];

        shellHook = ''
          echo "Clan Infrastructure Environment"
          echo "Available commands:"
          echo "  clan machines list          - List all machines"
          echo "  clan machines update <name> - Update a machine"
          echo "  clan backup create <name>   - Create backup"
          echo "  clan vms create <name>      - Create test VM"
        '';
      };
    };
}
