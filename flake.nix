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
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer

            # C/C++ build environment
            gcc
            clang
            cmake

            # C standard library headers
            glibc.dev

            # Additional build tools
            libiconv

            # Git for version control
            git

            # Python for scripts
            python3
          ];

          env = {
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

            # Environment variables for C compilation
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            C_INCLUDE_PATH = "${pkgs.glibc.dev}/include";
            CPLUS_INCLUDE_PATH = "${pkgs.glibc.dev}/include";
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
