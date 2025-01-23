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
          # Define the custom option
          options.services.umass-dining = lib.mkenableOption "UMass Dining service";

          # Conditionally enable the service if the option is true
          config = lib.mkIf config.services.umass-dining.enable {
            systemd.services.umass-dining = {
              enable = true;
              description = "UMass Dining Service";

              # Command to start the service
              serviceConfig =
                let
                  umass-dining = self.packages.${config.nixpkgs.system}.umass-dining;
                in
                {
                  ExecStart = "${umass-dining}/bin/umass-dining"; # Replace with your binary path
                  Restart = "always";
                  RestartSec = "5s";
                  Environment = "PORT=8000"; # Example environment variables
                };

              # Add dependencies and target
              wantedBy = [ "multi-user.target" ];
            };
          };
        };
    };
}
