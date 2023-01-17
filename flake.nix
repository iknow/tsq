{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo rustc rustfmt pre-commit rustPackages.clippy rust-analyzer
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          GRAMMAR_DIR =
            with tree-sitter-grammars;
            let ext = hostPlatform.extensions.sharedLibrary; in
              stdenv.mkDerivation {
                name = "grammar-dir";
                nativeBuildInputs = [ fixDarwinDylibNames ];

                dontUnpack = true;

                buildPhase = ''
                  mkdir $out
                  ln -s ${tree-sitter-typescript} $out/typescript
                  ln -s ${tree-sitter-tsx} $out/tsx
                '';

                installPhase = ''
                  :
                '';
              };
        };
      });
}
