{
  description = "";
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/25.11;
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }@inputs: let
    lib = nixpkgs.lib;
    systems = lib.systems.flakeExposed;
    forEachSystem = lib.genAttrs systems;
    pkgsFor = system: import nixpkgs { inherit system; };
    packages = (pkgs: let
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          # For rust-analyzer and others.  See
          # https://nixos.wiki/wiki/Rust#Shell.nix_example for some details.
          "rust-src"
          "rust-analyzer"
          "rustfmt-preview"
        ];
      };
    in [
      pkgs.cargo-sweep
      pkgs.clang
      pkgs.cargo
      pkgs.openssl
      pkgs.libssh2
      # To help with finding openssl.
      pkgs.pkg-config
      rust
      pkgs.rustfmt
      pkgs.rustup
    ]);
  in {
    darwinModules.default = ./nix/nix-darwin.nix;

    devShells.aarch64-darwin.default = let
      system = "aarch64-darwin";
      overlays = [
        (import rust-overlay)
      ];
      pkgs = import nixpkgs {
        inherit overlays system;
      };
    in pkgs.mkShell {
      buildInputs = (packages pkgs);
      shellHook = ''
      '';
    };

    overlays.default = final: prev: {
      sytter = final.callPackage ./nix/package.nix { };
    };

    packages = forEachSystem (system:
      let pkgs = pkgsFor system;
      in {
        default = pkgs.callPackage ./nix/package.nix { };
        sytter = self.packages.${system}.default;
      });

  };
}
