{ pkgs ? import <nixpkgs> { }
, package ? import ./default.nix { inherit pkgs; }
}:

pkgs.mkShell {
  packages = package.nativeBuildInputs ++ [
    # Dependencies required for `cargo install sqlx-cli`
    pkgs.openssl.dev
    pkgs.pkg-config
  ];
  shellHook = ''
    cargo install sqlx-cli
  '';
}
