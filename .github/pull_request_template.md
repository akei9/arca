# Description

<!-- What does this PR do? Link to the issue it closes. -->

## Security Checklist

- [ ] No secrets, keys, passwords, vault contents, or generated credentials are hardcoded or logged
- [ ] No new `unsafe` blocks without justification and human review
- [ ] Cryptographic changes reviewed against the threat model
- [ ] OSV/RustSec scan passes with no new vulnerabilities introduced
- [ ] Changes touching `packages/vault-core`, Tauri IPC, or secret display/copy flows are marked for human review

## Testing

- [ ] Unit tests added/updated
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `pnpm build`
- [ ] Tested locally on target platform when UI or Tauri behavior changed

## Agent-Assisted Changes

- [ ] AI-generated or AI-edited code was reviewed line by line by a human
- [ ] `AGENTS.md` files remain accurate after this change
