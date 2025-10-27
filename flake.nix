{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      fenix,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rustpkgs = fenix.packages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            (rustpkgs.stable.withComponents [
              "cargo"
              "clippy"
              "rustc"
              "rust-src"
              "rust-docs"
              "rustfmt"
              "rust-analyzer"
            ])
          ];
        };
      }
    );
}
