{
  description = "A theme injector tool that applies Material Design 3 color palettes to various configuration files";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

      mkPkgs =
        system:
        import nixpkgs {
          inherit system;
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;

          tinct = pkgs.rustPlatform.buildRustPackage {
            pname = "tinct";
            version = "0.1.0";

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
        in
        {
          default = tinct;
          inherit tinct;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rust-analyzer
              rustfmt
              clippy
              cargo-watch
              cargo-criterion
              nixfmt
              yamlfmt
            ];
          };
        }
      );

      formatter = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
        in
        pkgs.writeShellApplication {
          name = "format";

          runtimeInputs = with pkgs; [
            cargo
            nixfmt
            yamlfmt
            git
          ];

          text = ''
            set -euo pipefail

            # Rust
            if [ -f Cargo.toml ]; then
              cargo fmt --all
            fi

            # Nix
            git ls-files '*.nix' -z |
              while IFS= read -r -d "" file; do
                nixfmt "$file"
              done

            # YAML
            git ls-files '*.yaml' '*.yml' -z |
              while IFS= read -r -d "" file; do
                yamlfmt "$file"
              done
          '';
        }
      );
    };
}
