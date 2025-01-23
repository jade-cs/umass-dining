{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk }:
    flake-utils.lib.eachDefaultSystem
      (system:
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
          ];
          buildInputs = [ ];
        in
        rec {
          packages.default = packages.umass-dining;
          packages.umass-dining = naersk'.buildPackage {
            inherit nativeBuildInputs buildInputs;
            src = ./.;
          };

          devShells.default = pkgs.mkShell {
            inherit buildInputs;
            nativeBuildInputs = nativeBuildInputs ++ [
              (pkgs.python3.withPackages (ps: with ps; [ statistics aiohttp ]))
              (rust_toolchain.default.override {
                extensions = [ "rust-src" "rustfmt" "rust-analyzer" "clippy" ];
              })
            ];
          };
        }) // {
      nixosModules.default = { config, pkgs, lib, ... }:
        {
          options.services.umass-dining = with lib; {
            enable = mkEnableOption "UMass Dining service";
            port = mkOption {
              type = types.port;
              default = 9999;
              description = "Port to listen on";
            };
            address = mkOption {
              type = types.str;
              default = "127.0.0.1";
              description = "Address to bind to";
            };
          };

          config = lib.mkIf config.services.umass-dining.enable {
            systemd.services.umass-dining = {
              enable = true;
              description = "UMass Dining Service";

              serviceConfig =
                let
                  cfg = config.services.umass-dining;
                  umass-dining = self.packages.${config.nixpkgs.system}.umass-dining;
                in
                {
                  ExecStart = "${umass-dining}/bin/umass-dining --port ${toString cfg.port} --ip ${cfg.address}";
                  Restart = "always";
                  RestartSec = "5s";
                };

              wantedBy = [ "multi-user.target" ];
            };
          };
        };
    };
}
