# OpenADR 3.0 in Rust

This is a work-in-progress implementation of the OpenADR 3.0 specification.
OpenADR is a protocol for automatic demand-response in electricity grids, like dynamic pricing or load shedding.

## Limitations

This repository contains only OpenADR 3.0, older versions are not supported.
Currently, only the `/programs`, `/reports`, `/events` endpoints are supported.
Also no authentication is supported yet.

## Database setup

Startup a postgres database. For example, using docker compose:

```bash
docker compose up db
```

Run the [migrations](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md):

```bash
cargo sqlx migrate run
```

## How to use

Running the VTN using cargo:

```bash
RUST_LOG=trace cargo run --bin vtn
```

Running the VTN using docker-compose:

```bash
docker compose up
```

Running the client

```bash
cargo run --bin openadr
```

## With [Nix][0]

Note: this section is part of a commit that is not intended to be merged upstream. I've packaged the VTN server in [nix][0] for my own convenience.

Nix usage is currently only supported on `x86_64-linux` hosts.

Running the VTN using nix:

```bash
docker compose up db # Start database for the VTN server to use.
RUST_LOG=trace nix run
```

Building the VTN executable with nix:

```bash
nix build
```

(The executable is available in a symlink named `result` placed in the current working directory. You can run it with `RUST_LOG=trace result/bin/openadr-vtn`)

Entering a shell with development dependencies installed by nix.

```bash
nix develop
```

[0]: https://nixos.org/guides/nix-pills/01-why-you-should-give-it-a-try
