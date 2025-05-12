{
  description = "rwpspread: Multi-Monitor Wallpaper Spanning Utility";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { nixpkgs, ... }: let
    pkgs = nixpkgs.legacyPackages;
  in {
    devShells.x86_64-linux.default = pkgs.x86_64-linux.mkShell {
      packages = builtins.attrValues {
        inherit (pkgs) cargo rustfmt rust-analyzer pkg-config libxkbcommon;
      };
    };
    devShells.aarch64-linux.default = pkgs.aarch64-linux.mkShell {
      packages = builtins.attrValues {
        inherit (pkgs) cargo rustfmt rust-analyzer pkg-config libxkbcommon;
      };
    };
    packages.x86_64-linux.default = pkgs.x86_64-linux.rustPlatform.buildRustPackage {
      pname = "rwpspread";
      version = "master";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [pkgs.libxkbcommon];
    };
    packages.aarch64-linux.default = pkgs.aarch64-linux.rustPlatform.buildRustPackage {
      pname = "rwpspread";
      version = "master";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [pkgs.libxkbcommon];
    };
  };
}
