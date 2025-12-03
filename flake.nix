{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    clan-core = {
      url = "git+https://git.clan.lol/clan/clan-core";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , crane
    , flake-utils
    , advisory-db
    , clan-core
    , pre-commit-hooks
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        # Pre-commit hooks configuration
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # Rust formatting
            rustfmt = {
              enable = true;
              description = "Format Rust code with cargo fmt";
            };

            # Rust linting
            # Disabled in pre-commit-check to avoid sandbox network access issues
            # Clippy still runs via checks.onix-mcp-clippy and locally on commits
            clippy = {
              enable = false;
              description = "Lint Rust code with clippy";
              entry = lib.mkForce "${pkgs.cargo}/bin/cargo clippy --all-targets -- --deny warnings";
            };

            # TOML formatting
            taplo = {
              enable = true;
              description = "Format TOML files with taplo";
            };

            # Nix formatting
            nixpkgs-fmt = {
              enable = true;
              description = "Format Nix code with nixpkgs-fmt";
            };
          };
        };

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        onix-mcp-unwrapped = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        # Wrap the binary to include nix-index and other Nix tools in PATH
        onix-mcp =
          pkgs.runCommand "onix-mcp"
            {
              buildInputs = [ pkgs.makeWrapper ];
            }
            ''
              mkdir -p $out/bin
              makeWrapper ${onix-mcp-unwrapped}/bin/onix-mcp $out/bin/onix-mcp \
                --prefix PATH : ${
                  lib.makeBinPath [
                    pkgs.nix
                    pkgs.nix-index
                    pkgs.comma
                    pkgs.nix-diff
                    pkgs.nixpkgs-fmt
                    pkgs.alejandra
                    pkgs.statix
                    pkgs.deadnix
                    clan-core.packages.${system}.clan-cli
                  ]
                }
            '';
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          onix-mcp = onix-mcp;

          # Pre-commit hooks check
          pre-commit-check = pre-commit-check;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          onix-mcp-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          onix-mcp-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              # This can be commented out or tweaked as necessary, e.g. set to
              # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          # Check formatting
          onix-mcp-fmt = craneLib.cargoFmt {
            inherit src;
          };

          onix-mcp-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
          };

          # Audit dependencies
          onix-mcp-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          onix-mcp-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages = {
          default = onix-mcp;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = onix-mcp;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            # pkgs.ripgrep
            pkgs.gemini-cli
            pkgs.codex
            pkgs.nix-index
            pkgs.comma
          ];

          # Install pre-commit hooks when entering the shell
          inherit (pre-commit-check) shellHook;
        };
      }
    );
}
