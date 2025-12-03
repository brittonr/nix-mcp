{
  description = "Basic Clan infrastructure using onix-mcp";

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
    in
    {
      # Clan infrastructure configuration
      clanInternals = clan-core.lib.buildClan {
        directory = self;
        specialArgs = { inherit onix-mcp; };

        # Define your machines
        machines = {
          # Example machine configuration
          example-machine = { config, ... }: {
            # Import the onix-mcp NixOS module
            imports = [ onix-mcp.nixosModules.default ];

            # Enable onix-mcp service
            services.onix-mcp = {
              enable = true;
              logLevel = "info";
            };

            # Basic NixOS configuration
            networking.hostName = "example-machine";
            system.stateVersion = "24.05";

            # Minimal base system
            boot.loader.grub.enable = true;
            boot.loader.grub.device = "/dev/sda";

            # Enable SSH for remote management
            services.openssh = {
              enable = true;
              settings.PermitRootLogin = "no";
            };

            # Basic firewall
            networking.firewall = {
              enable = true;
              allowedTCPPorts = [ 22 ]; # SSH
            };

            # User account
            users.users.admin = {
              isNormalUser = true;
              extraGroups = [ "wheel" "networkmanager" ];
              openssh.authorizedKeys.keys = [
                # Add your SSH public key here
                # "ssh-ed25519 AAAA... user@host"
              ];
            };
          };
        };
      };

      # Development shell
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          clan-core.packages.${system}.clan-cli
          onix-mcp.packages.${system}.default
        ];
      };
    };
}
