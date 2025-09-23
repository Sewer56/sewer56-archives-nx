{
  description = "Development environment for sewer56-archives-nx";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self, # Not unused, is required for flake.
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Read rust-toolchain.toml to get the correct Rust version
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # C/C++ build environment
            gcc
            clang
            cmake
            pkg-config

            # C standard library headers
            glibc.dev

            # Additional build tools
            libiconv

            # Git for version control
            git

            # Python for scripts
            python3
          ];

          shellHook = ''
            echo "🦀 Rust development environment loaded"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
          '';

          # Environment variables for C compilation
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          C_INCLUDE_PATH = "${pkgs.glibc.dev}/include";
          CPLUS_INCLUDE_PATH = "${pkgs.glibc.dev}/include";
        };
      }
    );
}
