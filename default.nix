{ pkgs ? import <nixpkgs> { }
, pname ? "openleadr-rs"
, version ? "unknown"
}:

let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = pkgs.rust-bin.stable.latest.default;
    rustc = pkgs.rust-bin.stable.latest.default;
  };
in rustPlatform.buildRustPackage {
  inherit pname version;
  src = with pkgs.lib.fileset; 
    let
      nixFiles = unions [ (maybeMissing ./result) ./flake.nix ./flake.lock ./default.nix ./shell.nix ./.envrc ];
      fileset = difference ./. nixFiles;
    in toSource { root = ./.; inherit fileset; };
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  # Disable compile-time checks of SQL queries against live database.
  #
  # The sqlx rust crate's query*! macros require a live database connection
  # during compilation (ugh!) in order to support compile-time checked queries
  # (wow!). We disable this feature for now within the nix build context. Not
  # doing so yields errors like
  #
  #   error: error communicating with database: Connection refused (os error 111)
  #
  # during compilation.
  #
  # See https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#force-building-in-offline-mode
  # and https://docs.rs/sqlx/latest/sqlx/macro.query.html#offline-mode
  SQLX_OFFLINE = "true";
  # Disable tests.
  # 
  # The basic_create_read test in the client requires a live database
  # connection. Testing without a database yields errors like
  #
  #   failed to connect to setup test database: PoolTimedOut
  #
  # and
  #
  #   error: test failed, to rerun pass `-p openadr-client --test basic-read`
  #
  # We avoid this by disabling all tests. This isn't ideal, and there is an
  # outstanding issue on the upstream project's board to avoid depending on
  # live database connections inside tests.
  #
  # See https://github.com/orgs/OpenLEADR/projects/1?pane=issue&itemId=83420564
  doCheck = false;
}
