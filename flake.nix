{
  description = "rwpspread: Multi-Monitor Wallpaper Spanning Utility";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { nixpkgs, ... }: let
    pkgs-aarch64 = nixpkgs.legacyPackages.aarch64-linux;
    pkgs-x86 = nixpkgs.legacyPackages.x86_64-linux;
  in {
    devShells.x86_64-linux.default = pkgs-x86.mkShell {
      packages = builtins.attrValues {
        inherit (pkgs-x86) cargo rustfmt rust-analyzer pkg-config libxkbcommon;
      };
    };
    devShells.aarch64-linux.default = pkgs-aarch64.mkShell {
      packages = builtins.attrValues {
        inherit (pkgs-aarch64) cargo rustfmt rust-analyzer pkg-config libxkbcommon;
      };
    };
    packages.x86_64-linux.default = pkgs-x86.rustPlatform.buildRustPackage {
      pname = "rwpspread";
      version = "master";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      nativeBuildInputs = [pkgs-x86.pkg-config];
      buildInputs = [pkgs-x86.libxkbcommon];
    };
    packages.aarch64-linux.default = pkgs-aarch64.rustPlatform.buildRustPackage {
      pname = "rwpspread";
      version = "master";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      nativeBuildInputs = [pkgs-aarch64.pkg-config];
      buildInputs = [pkgs-aarch64.libxkbcommon];
    };
  };
}
