{
  description = "Elerem";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-bin-custom = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
        };

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.pkg-config
            rust-bin-custom
            pkgs.openssl
            pkgs.sqlx-cli
            pkgs.stripe-cli
            pkgs.cargo-watch
            pkgs.cargo-expand
          ];
        };

        packages = {
          check = pkgs.writeShellScriptBin "check" ''
            cargo test --all
            cargo clippy -- -Dclippy::all -Dclippy::pedantic -D warnings
            cargo fmt --all -- --check
          '';

          restart-db = pkgs.writeShellScriptBin "restart-db" ''
            sqlx database drop -y -f
            sqlx database create
            sqlx migrate run --source migrations/principal
            cargo sqlx prepare -- --all-targets --all-features
          '';

          create-db = pkgs.writeShellScriptBin "create-db" ''
            sqlx database create
            sqlx migrate run --source migrations/principal
          '';

        };
      }
    );
}
