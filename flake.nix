{
  description = "ros flake";

  inputs = {
    nix-ros-overlay.url = "github:lopsided98/nix-ros-overlay/master";
    nixpkgs.follows = "nix-ros-overlay/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    nixgl.url = "github:nix-community/nixGL";
  };

  outputs =
    { self, nixpkgs, rust-overlay, flake-utils, nixgl, nix-ros-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) nix-ros-overlay.overlays.default ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        roswsShell = import ./.rosws.nix { inherit pkgs; };

        nvidiaVersion = builtins.getEnv "NIXGL_NVIDIA_VERSION";
        nvidiaHash = builtins.getEnv "NIXGL_NVIDIA_HASH";

        nixglShell = import ./.nixgl.nix {
          inherit pkgs nixgl nvidiaVersion nvidiaHash;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            [
              (pkgs.rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
              pkgs.clang
              pkgs.mold
              pkgs.cmake
              pkgs.pkg-config
              pkgs.colcon
              (with pkgs.rosPackages.jazzy; buildEnv {
                paths = [
                  ros-core
                  desktop-full
                  cyclonedds
                  rmw-cyclonedds-cpp
                  ackermann-msgs
                  moveit
                ];
              })
            ]
            ++ nixglShell.packages
            ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              pkgs.alsa-lib
              pkgs.alsa-plugins
              pkgs.pipewire
              pkgs.vulkan-loader
              pkgs.vulkan-tools
              pkgs.libudev-zero
              pkgs.libx11
              pkgs.libxcursor
              pkgs.libxrandr
              pkgs.libxkbcommon
              pkgs.wayland
            ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.vulkan-loader
            pkgs.libx11
            pkgs.libxcursor
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.alsa-lib
            pkgs.alsa-plugins
            pkgs.pipewire
          ];

          shellHook = ''
            ${roswsShell.shellHook}
            ${nixglShell.shellHook}
          '';
        };
      }
    );

  nixConfig = {
    extra-substituters = [ "https://ros.cachix.org" ];
    extra-trusted-public-keys = [ "ros.cachix.org-1:dSyZxI8geDCJrwgvCOHDoAfOm5sV1wCPjBkKL+38Rvo=" ];
  };
}
