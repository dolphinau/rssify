{
  description = "lwn-sub-snoozer";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
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
          pkgs.postgresql

          # Nix
          pkgs.nixpkgs-fmt
          pkgs.nil
          pkgs.nixd
          pkgs.alejandra

          # Rust
          pkgs.rustfmt
          pkgs.rustc
          pkgs.cargo
          pkgs.rust-analyzer
        ];
      };
    });
}
