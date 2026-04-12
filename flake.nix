{
  description = "Statistical physics exercises in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default rust-overlay.overlays.default];
        };

        rustTools = {
          stable = pkgs.rust-bin.stable."1.89.0".default.override {
            extensions = ["rust-src"];
          };
          analyzer = pkgs.rust-bin.stable."1.89.0".rust-analyzer;
        };

        run = pkgs.writeShellScriptBin "run" ''
          exec nu "$PWD/run.nu" "$@"
        '';

        devTools = with pkgs; [
          cargo-expand
          typst
        ];
        pythonEnv = pkgs.python313.withPackages (ps:
          with ps; [
            numpy
            matplotlib
          ]);
      in {
        devShells.default = pkgs.mkShell {
          name = "statphys-dev";
          buildInputs =
            [
              rustTools.stable
              rustTools.analyzer
              run
              pythonEnv
            ]
            ++ devTools;

          shellHook = ''
            echo "Using Rust toolchain: $(rustc --version)"
            export LD_LIBRARY_PATH="${pythonEnv}/lib:$LD_LIBRARY_PATH"
            export CARGO_HOME="$HOME/.cargo"
            export RUSTUP_HOME="$HOME/.rustup"
            mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
          '';
        };
      }
    );
}
