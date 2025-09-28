{
  description = "Development environment for sewer56-archives-nx";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];
    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (
        system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
      );
  in {
    overlays.default = final: prev: {
      rustToolchain = with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
        combine (
          with latest;
            [
              clippy
              rustc
              cargo
              rustfmt
              rust-src
            ]
            ++ [
              # Cross targets
              targets.powerpc64-unknown-linux-gnu.latest.rust-std
              targets.powerpc-unknown-linux-gnu.latest.rust-std
            ]
        );
    };

    devShells = forEachSupportedSystem (
      {pkgs}: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain

            # For building the native dependencies
            openssl
            pkg-config
            gcc
            clang

            # Development Tools used in configs, e.g. VSCode tasks.
            cargo-watch

            # Python for some of the research scripts
            python3
          ];

          env = {
            # This will use the `cross` targets above, else stuff will be kinda broken.
            CROSS_CUSTOM_TOOLCHAIN = "1";
          };

          shellHook = ''
            echo "🦀 Rust development environment loaded"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
          '';
        };
      }
    );
  };
}
