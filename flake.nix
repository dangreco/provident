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
          lib,
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
            # `minimal` = rustc, cargo, rust-std; no rust-docs (~700 MiB) to keep CI's Nix cache small.
            ci = pkgs'.rust-bin.stable.${version}.minimal.override {
              extensions = [
                "clippy"
                "rustfmt"
                # Coverage instrumentation for cargo-llvm-cov.
                "llvm-tools"
              ];
            };
            package = ci.override {
              extensions = [
                "rust-src"
                "clippy"
                "rustfmt"
                "llvm-tools"
                "rust-analyzer"
              ];
            };
          };

          # Interpreter for the bindings' tests and type checks (`make py-*`). The venv there
          # inherits these packages; maturin installs the extension into it.
          python = pkgs'.python3.withPackages (
            ps: with ps; [
              pytest
              mypy
            ]
          );

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
              # Same rustfmt as `rust.package`, without dragging rust-docs into the CI shell.
              packageOverrides = {
                cargo = rust.ci;
                rustfmt = rust.ci;
              };
            };
            # Not a hook: `async`/`blocking` are mutually exclusive, so `--all-features` can never
            # compile. Clippy runs per mode via `make lint MODE=…` (locally and in CI).
            clippy.enable = false;
            cargo-sort.enable = true;

            # Python (bindings in `python/`; ruff reads `python/pyproject.toml`)
            ruff.enable = true;
            ruff-format.enable = true;

            # TOML
            taplo.enable = true;
            check-toml.enable = true;

            # Spelling
            typos.enable = true;

            # Commit messages: Conventional Commits (config in `.cz.toml`), which release-please
            # turns into version bumps and CHANGELOG entries.
            commitizen.enable = true;

            # GitHub Actions (workflows + the local composite action)
            actionlint.enable = true;
            zizmor = {
              enable = true;
              files = "^\\.github/(workflows|actions)/";
              # Network-free so the hook is deterministic in CI and offline.
              entry = "${pkgs.zizmor}/bin/zizmor --offline";
            };
            pinact = {
              enable = true;
              name = "pinact";
              description = "Ensure GitHub Actions are pinned to full commit SHAs";
              entry = "${pkgs.pinact}/bin/pinact run --check";
              files = "^\\.github/(workflows|actions)/";
              types = [ "yaml" ];
              pass_filenames = false;
            };

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
                "python"
              ];
            };
          };

          devShells = {
            default = pkgs.mkShellNoCC {
              inherit (config.pre-commit) shellHook;
              # Other tools put a bare `python3` on PATH too; `make py-*` must use this one.
              PYTHON = lib.getExe python;

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
                cargo-llvm-cov
                python
                maturin
              ];
            };

            # Lean shell for GitHub Actions: CI toolchain + hook tools; no editor tooling, and no
            # git-hooks shellHook (it would install hooks). Exports the hook config instead so CI
            # runs exactly the hooks developers do: `pre-commit run -c "$PRE_COMMIT_CONFIG" …`.
            ci = pkgs.mkShellNoCC {
              PRE_COMMIT_CONFIG = config.pre-commit.settings.configFile;
              PYTHON = lib.getExe python;

              packages = [
                config.pre-commit.settings.package
                pkgs.gnumake
              ]
              ++ config.pre-commit.settings.enabledPackages;

              buildInputs = with pkgs; [ openssl ];
              nativeBuildInputs = with pkgs; [
                pkg-config
                rust.ci
                cargo-llvm-cov
                python
                maturin
              ];
            };
          };
        };
    };
}
