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

## Pull Requests

- Use Conventional Commit titles such as `feat:`, `fix:`, `security:`, `test:`, and `chore:`.
- Keep each PR focused on one concern.
- Fill out the security checklist in the PR template.
- Mark AI-assisted work with the `agent-assisted` label when labels are available.
- If agent instructions become inaccurate, update the nearest `AGENTS.md` in the same PR.

## Vulnerability Reports

Do not report vulnerabilities in public issues. Use GitHub private vulnerability reporting if it is enabled for the repository, or contact the maintainers through the repository security policy.
