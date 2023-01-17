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
          GRAMMARS =
            with tree-sitter-grammars;
            let ext = hostPlatform.extensions.sharedLibrary; in
              stdenv.mkDerivation {
                name = "grammar-dir";
                nativeBuildInputs = [ fixDarwinDylibNames ];

                dontUnpack = true;

                buildPhase = ''
                  mkdir $out $out/lib
                  cp -a ${tree-sitter-typescript}/parser $out/lib/libtypescript${ext}
                  cp -a ${tree-sitter-tsx}/parser $out/lib/libtsx${ext}
                '';

                installPhase = ''
                  :
                '';
              };
        };
      });
}
