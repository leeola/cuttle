{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = [ "rust-analysis" ];
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config
            binutils
            gcc
            python3
            rust-analyzer
            # using a hardcoded rustfmt version to support nightly rustfmt features.
            rust-bin.nightly."2025-06-26".rustfmt
            rust-toolchain
          ];
        };
      }
    );
}
