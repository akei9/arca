# Contributing to Arca

Arca is a password manager, so all contributions should optimize for user safety, auditability, and clear human review.

## Development Checks

Run the relevant checks before opening a PR:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `pnpm build`

## Security Expectations

- Do not include real passwords, vault files, API keys, private keys, or recovery material in issues, PRs, tests, screenshots, or logs.
- Do not log plaintext secrets or decrypted vault contents.
- Changes to `packages/vault-core`, Tauri IPC secret transport, vault locking/unlocking, clipboard behavior, or password display/copy flows need human review.
- Do not add cryptographic algorithms, KDF changes, vault format changes, or `unsafe` blocks without a linked issue and explicit maintainer approval.

## Fuzzing

`vault-core` has `cargo-fuzz` targets for security-sensitive parser paths. Install the runner with:

```sh
cargo install cargo-fuzz --version 0.13.2 --locked
```

The GitHub fuzzing workflow pins the same reviewed `cargo-fuzz` version so
scheduled runs do not change behavior when a new runner release is published.

Run the KDBX parser target from `packages/vault-core`:

```sh
cargo fuzz run open_vault_bytes
```

For a short local smoke run:

```sh
cargo fuzz run open_vault_bytes -- -max_total_time=60
```

Fuzz corpora must use generated data only. Do not add real vault files, real passwords, private keys, or recovery material.

Maintainers can also run the `Vault Fuzzing` GitHub Actions workflow manually.
The workflow runs `open_vault_bytes` for 300 seconds by default, validates the
requested target and runtime, and caps manual runs at 1800 seconds. It also runs
weekly on `main` as a bounded maintenance check. Treat crash artifacts and
reproducers as security-sensitive until reviewed; do not paste them into public
issues before triage.

## Pull Requests

- Use Conventional Commit titles such as `feat:`, `fix:`, `security:`, `test:`, and `chore:`.
- Keep each PR focused on one concern.
- Fill out the security checklist in the PR template.
- Mark AI-assisted work with the `agent-assisted` label when labels are available.
- If agent instructions become inaccurate, update the nearest `AGENTS.md` in the same PR.

## Vulnerability Reports

Do not report vulnerabilities in public issues. Use GitHub private vulnerability reporting if it is enabled for the repository, or contact the maintainers through the repository security policy.
