# SPDX-License-Identifier: GPL-3.0-only

{
  description = "dangreco/provident";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.git-hooks.flakeModule ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        let
          pkgs' = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          rust = rec {
            version = "1.99.1";
            package = pkgs'.rust-bin.stable.${version}.default.override {
              extensions = [
                "rust-src"
                "clippy"
                "rustfmt"
                "rust-analyzer"
              ];
            };
          };

          spdx = {
            license = "GPL-3.0-only";
            annotate = pkgs.writeShellApplication {
              name = "spdx-annotate";
              runtimeInputs = [ pkgs.reuse ];
              text = ''
                exec reuse annotate \
                  --license ${spdx.license} \
                  --skip-existing \
                  --skip-unrecognised \
                  "$@"
              '';
            };
          };
        in
        {
          _module.args.pkgs = pkgs';

          pre-commit.settings.hooks = {
            # Nix
            nixfmt.enable = true;
            nil.enable = true;
            statix.enable = true;
            deadnix.enable = true;
            flake-checker.enable = true;

            # Rust
            rustfmt = {
              enable = true;
              packageOverrides = {
                cargo = rust.package;
                rustfmt = rust.package;
              };
            };
            clippy = {
              enable = true;
              packageOverrides = {
                cargo = rust.package;
                clippy = rust.package;
              };
              settings = {
                offline = true;
                allFeatures = true;
                denyWarnings = true;
              };
            };
            cargo-sort.enable = true;

            # TOML
            taplo.enable = true;
            check-toml.enable = true;

            # Spelling
            typos.enable = true;

            # Utility
            check-merge-conflicts.enable = true;
            check-added-large-files.enable = true;
            check-case-conflicts.enable = true;
            check-symlinks.enable = true;
            check-executables-have-shebangs.enable = true;
            check-shebang-scripts-are-executable.enable = true;
            forbid-new-submodules.enable = true;
            fix-byte-order-marker.enable = true;
            mixed-line-endings.enable = true;
            end-of-file-fixer.enable = true;
            trim-trailing-whitespace.enable = true;
            detect-private-keys.enable = true;
            ripsecrets.enable = true;

            # Adds `SPDX-License-Identifier` to source files missing one.
            spdx-license-header = {
              enable = true;
              name = "spdx-license-header";
              description = "Add SPDX-License-Identifier header to source files";
              entry = "${spdx.annotate}/bin/spdx-annotate";
              types_or = [
                "rust"
                "nix"
              ];
            };
          };

          devShells = {
            default = pkgs.mkShellNoCC {
              inherit (config.pre-commit) shellHook;

              packages =
                with pkgs;
                [
                  nil
                  nixfmt
                ]
                ++ config.pre-commit.settings.enabledPackages;

              buildInputs = with pkgs; [ openssl ];
              nativeBuildInputs = with pkgs; [
                pkg-config
                rust.package
              ];
            };

            ci = config.devShells.default;
          };
        };
    };
}
