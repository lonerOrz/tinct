{
  description = "A theme injector tool that applies Material Design 3 color palettes to various configuration files";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

      rustToolchain =
        pkgs:
        pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

      packageMeta = (fromTOML (builtins.readFile ./Cargo.toml)).package;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rust = rustToolchain pkgs;

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = packageMeta.name;
            version = packageMeta.version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "A theme injector tool that applies Material Design 3 color palettes to various configuration files";
              homepage = "https://github.com/lonerOrz/tinct";
              mainProgram = "tinct";
              license = pkgs.lib.licenses.bsd3;
              maintainers = [ pkgs.lib.maintainers.lonerOrz ];
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rust = rustToolchain pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rust
              rust-analyzer
              cargo-watch
              cargo-criterion
              nixfmt
              yamlfmt
            ];

            RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
          };
        }
      );

      formatter = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rust = rustToolchain pkgs;
        in
        pkgs.writeShellApplication {
          name = "format";

          runtimeInputs = with pkgs; [
            rust
            nixfmt
            yamlfmt
            git
          ];

          text = ''
            set -euo pipefail

            [ -f Cargo.toml ] && cargo fmt --all

            git ls-files '*.nix' -z |
              xargs -0 -r -n1 nixfmt

            git ls-files '*.yaml' '*.yml' -z |
              xargs -0 -r -n1 yamlfmt
          '';
        }
      );
    };
}
