{
  description = "Statistical physics exercises in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {nixpkgs, ...}: let
    forAllSystems = nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
  in {
    devShells = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      run = pkgs.writeShellScriptBin "run" ''
        exec nu "$PWD/run.nu" "$@"
      '';

      pythonEnv = pkgs.python313.withPackages (ps:
        with ps; [
          numpy
          matplotlib
          scipy
        ]);
    in {
      default = pkgs.mkShell {
        name = "statphys-dev";
        buildInputs = with pkgs; [
          cargo clippy rustc rustfmt rust-analyzer
          cargo-expand typst
          run pythonEnv
        ];
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        shellHook = ''
          export LD_LIBRARY_PATH="${pythonEnv}/lib:$LD_LIBRARY_PATH"
        '';
      };
    });
  };
}
