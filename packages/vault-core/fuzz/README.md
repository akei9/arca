# vault-core fuzz targets

These targets exercise security-sensitive parsing paths with generated input only.
Do not use real vault files or real passwords as corpus material.

## Setup

```sh
cargo install cargo-fuzz
```

## Run

From `packages/vault-core`:

```sh
cargo fuzz run open_vault_bytes
```

For a short local smoke run:

```sh
cargo fuzz run open_vault_bytes -- -max_total_time=60
```

`open_vault_bytes` writes each fuzz input to a temporary `.kdbx` path, calls the
normal `vault_core::vault::open_vault` path with a generated non-secret
credential, and ignores expected parse/authentication errors. Crashes, panics,
or sanitizer findings should be treated as security-sensitive until triaged.
