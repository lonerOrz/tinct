{
  description = "A theme injector tool that applies Material Design 3 color palettes to various configuration files";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          lib = pkgs.lib;
        in
        {
          packages = {
            default = self'.packages.tinct;
            tinct = pkgs.rustPlatform.buildRustPackage {
              pname = "tinct";
              version = "0.1.0";
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              meta = {
                description = "A theme injector tool that applies Material Design 3 color palettes to various configuration files";
                homepage = "https://github.com/lonerOrz/tinct";
                mainProgram = "tinct";
                license = lib.licenses.bsd3;
                maintainers = with lib.maintainers; [ lonerOrz ];
                platforms = [
                  "x86_64-linux"
                  "aarch64-linux"
                  "x86_64-darwin"
                  "aarch64-darwin"
                ];
              };
            };
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [ self'.packages.default ];
            packages = with pkgs; [
              cargo
              rustc
              rust-analyzer
              rustfmt
              clippy
              cargo-watch
              cargo-criterion
            ];
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              rustfmt.enable = true;
              nixfmt.enable = true;
            };
          };
        };
    };
}
