{ config, lib, pkgs, ... }:

let
  cfg = config.services.onix-mcp;
in
{
  options.services.onix-mcp = {
    enable = lib.mkEnableOption "onix-mcp MCP server for Nix ecosystem operations";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.onix-mcp or (throw "onix-mcp package not available. Add the onix-mcp flake as an input.");
      defaultText = lib.literalExpression "pkgs.onix-mcp";
      description = "The onix-mcp package to use.";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "onix-mcp";
      description = "User account under which onix-mcp runs.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "onix-mcp";
      description = "Group under which onix-mcp runs.";
    };

    socketPath = lib.mkOption {
      type = lib.types.str;
      default = "/run/onix-mcp/socket";
      description = "Path to the Unix socket for onix-mcp communication.";
    };

    socketActivation = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Whether to use systemd socket activation.";
    };

    logLevel = lib.mkOption {
      type = lib.types.enum [ "error" "warn" "info" "debug" "trace" ];
      default = "info";
      description = "Logging level for onix-mcp.";
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional environment variables to set for the service.";
      example = lib.literalExpression ''
        {
          RUST_BACKTRACE = "1";
        }
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    # Create user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "Onix MCP server user";
      home = "/var/lib/onix-mcp";
      createHome = true;
    };

    users.groups.${cfg.group} = { };

    # Systemd service
    systemd.services.onix-mcp = {
      description = "Onix MCP Server for Nix Ecosystem Operations";
      documentation = [ "https://github.com/onix-project/onix-mcp" ];
      after = [ "network.target" ];
      wants = [ "network-online.target" ];
      wantedBy = lib.mkIf (!cfg.socketActivation) [ "multi-user.target" ];

      environment = {
        RUST_LOG = cfg.logLevel;
      } // cfg.extraEnvironment;

      serviceConfig = {
        Type = if cfg.socketActivation then "notify" else "simple";
        ExecStart = "${cfg.package}/bin/onix-mcp";
        User = cfg.user;
        Group = cfg.group;
        Restart = "on-failure";
        RestartSec = "5s";

        # Working directory
        WorkingDirectory = "/var/lib/onix-mcp";

        # Security hardening
        # Based on systemd.exec(5) security recommendations

        # Filesystem protection
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ "/var/lib/onix-mcp" ];
        PrivateDevices = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        ProtectProc = "invisible";
        ProcSubset = "pid";

        # Network restrictions
        RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" ];

        # Privilege restrictions
        NoNewPrivileges = true;
        PrivateUsers = true;
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        RemoveIPC = true;

        # Capability restrictions
        CapabilityBoundingSet = "";
        AmbientCapabilities = "";

        # System call filtering
        SystemCallFilter = [ "@system-service" "~@privileged" "~@resources" ];
        SystemCallErrorNumber = "EPERM";
        SystemCallArchitectures = "native";

        # Resource limits
        MemoryDenyWriteExecute = true;
        LimitNOFILE = 65536;
        TasksMax = 4096;
      };
    };

    # Systemd socket (if socket activation is enabled)
    systemd.sockets.onix-mcp = lib.mkIf cfg.socketActivation {
      description = "Onix MCP Server Socket";
      documentation = [ "https://github.com/onix-project/onix-mcp" ];
      wantedBy = [ "sockets.target" ];

      socketConfig = {
        ListenStream = cfg.socketPath;
        Accept = false;
        SocketMode = "0660";
        SocketUser = cfg.user;
        SocketGroup = cfg.group;
        RemoveOnStop = true;
        DirectoryMode = "0750";
      };
    };

    # Ensure runtime directory exists
    systemd.tmpfiles.rules = [
      "d ${dirOf cfg.socketPath} 0750 ${cfg.user} ${cfg.group} -"
    ];

    # Add onix-mcp user to necessary groups for Nix operations
    # This allows the service to interact with the Nix daemon
    users.users.${cfg.user}.extraGroups = [ "nixbld" ];
  };
}
