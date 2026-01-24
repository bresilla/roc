{
  description = "ROC development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell.override {
        stdenv = pkgs.stdenvAdapters.keepSystem (pkgs.stdenv);
      } {
        packages = with pkgs; [
          # Build tools
          clang
          cmake
          ninja
          gdb

          # Rust toolchain
          rustc
          cargo
          rustfmt
          rust-analyzer

          # nix-ld to keep system libraries accessible
          pkgs.nix-ld
        ];

        # Set up Rust environment
        shellHook = ''
          # Set RUSTUP_HOME to avoid conflicts with system rustup
          export RUSTUP_HOME=$(pwd)/.direnv/rustup

          # Add Cargo bin directory to PATH
          export PATH=$PATH:$(pwd)/target/debug:$(pwd)/target/release

          # Source ROS setup
          source /opt/ros/jazzy/setup.sh
        '';
      };
    };
}
