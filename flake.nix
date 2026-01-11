{
  description = "nRF Microbit Learning";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { flake-utils, nixpkgs, rust-overlay, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "llvm-tools" "clippy" ];
      };
    in
    {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo-generate
          cargo-binutils
          rust
          rust-analyzer
          gdb
        ];
        shellHook = ''
          export SHELL=/run/current-system/sw/bin/bash
        '';
      };
    });
}
