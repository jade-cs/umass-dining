{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rust_toolchain = pkgs.rust-bin.stable.latest;
        naersk' = pkgs.callPackage naersk {
          rustc = rust_toolchain.minimal;
          cargo = rust_toolchain.minimal;
        };
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          (pkgs.python3.withPackages (ps: with ps; [ statistics aiohttp ]))
        ];
        buildInputs = [ ];
      in
      rec {
        defaultPackage = umass-dining;
        umass-dining = naersk'.buildPackage {
          inherit nativeBuildInputs buildInputs;
          src = ./.;
        };

        devShell = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [
            (rust_toolchain.default.override {
              extensions = [ "rust-src" "rustfmt" "rust-analyzer" "clippy" ];
            })
          ];
        };
      }
    );
}
