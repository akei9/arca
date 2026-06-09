# vault-core Agent Guide

`vault-core` is the security-critical Rust library for Arca. It owns vault encryption/decryption, KDBX persistence, entry modeling, password generation, and secret-related data structures.

## Required Checks

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Security Rules

- Do not implement custom cryptographic primitives or protocols.
- Do not introduce new crypto crates, KDF settings, vault formats, or key derivation behavior without a linked issue and human approval.
- Prefer existing established crates in the workspace. Current secret-clearing behavior uses `zeroize`.
- Public APIs that accept, return, persist, or transform plaintext secrets require human review before merging.
- Never add `println!`, `eprintln!`, `dbg!`, or tracing of secret-bearing values.
- Avoid cloning or formatting secrets. If cloning is unavoidable for API compatibility, keep the lifetime narrow and document why.
- No `unsafe` blocks without an explicit justification comment, tests, and human review.

## Testing Rules

- Every new public function should have a focused unit test or integration test.
- Crypto and vault persistence changes need negative tests, including wrong-password or tamper cases where applicable.
- Tests may use fixture passwords, but they must not encourage logging or exposing real secrets.
