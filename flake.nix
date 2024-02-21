{
  description = "rwpspread: Wallpaper Utility written in Rust";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = {nixpkgs, ...}: let
    # skipping aarch64-linux as I don't have the hardware, please add it if you and can test.
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      packages = builtins.attrValues {
        inherit (pkgs) cargo rustfmt rust-analyzer pkg-config libxkbcommon;
      };
    };
    packages.x86_64-linux.default = pkgs.rustPlatform.buildRustPackage {
      pname = "rwpspread";
      version = "master";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [pkgs.libxkbcommon];
    };
  };
}
