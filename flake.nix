{
  description = "netidentity — network identity snapshot";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        netidentity = pkgs.rustPlatform.buildRustPackage {
          pname = "netidentity";
          version = "0.1.0";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          meta = with pkgs.lib; {
            description = "Network identity snapshot";
            license = licenses.mit;
            mainProgram = "netidentity";
          };
        };
      in {
        packages.default = netidentity;
        apps.default = flake-utils.lib.mkApp { drv = netidentity; };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ rustc cargo rustfmt clippy ];
        };
      });
}
