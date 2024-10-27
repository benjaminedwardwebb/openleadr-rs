{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay/d687672";
  };
  outputs = { self, nixpkgs, rust-overlay }: let
    pname = "openleadr-rs";
    version =
      if self ? rev then self.shortRev
      else if self ? dirtyRev then self.dirtyShortRev
      else "unknown";
    system = "x86_64-linux";
    pkgs = let
      overlays = [ (import rust-overlay) ];
      in import nixpkgs {
          inherit system overlays;
      };
    package = import ./default.nix { inherit pkgs pname version; };
  in {
    packages.${system}.default = package;
    devShells.${system}.default =
      import ./shell.nix { inherit pkgs package; };
    apps.${system}.default = {
      type = "app";
      program = "${package}/bin/openadr-vtn";
    };
  };
}
