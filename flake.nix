{
  description = "lwn-sub-snoozer";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages = {
        default = self.packages.${system}.myapp;
      };
      # $ nix develop
      devShells.default = pkgs.mkShell {
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.openssl];
        packages = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.nixpkgs-fmt
          pkgs.nil
          pkgs.postgresql

          # Rust
          fenix.packages.${system}.default.toolchain
        ];
      };
    });
}
