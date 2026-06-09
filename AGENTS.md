# Arca Agent Guide

## Project Overview

Arca is an open-source, local-first desktop password manager. It is a pnpm monorepo with a Tauri v2 desktop app in `apps/desktop` and Rust vault logic in `packages/vault-core`.

All cryptography, vault parsing, password generation, and secret-handling logic belongs in `packages/vault-core`. The Svelte frontend and Tauri command layer should treat plaintext secrets as short-lived values and avoid logging them.

## Commands

- `pnpm dev` - run the desktop frontend through the workspace script.
- `pnpm build` - build the desktop frontend.
- `pnpm lint` - run the current frontend lint/type gate.
- `cargo fmt --all --check` - check Rust formatting.
- `cargo clippy --workspace --all-targets -- -D warnings` - run Rust lints as a hard gate.
- `cargo test --workspace` - run Rust tests.

## Security Rules

- Never log, print, snapshot, or expose plaintext passwords, vault keys, recovery material, or decrypted vault contents.
- Keep secrets inside `vault-core` or the narrow Tauri command/session boundary required to service UI actions.
- Do not add `unsafe` without an explanatory comment, tests, and explicit human review.
- Do not introduce or swap cryptographic algorithms, KDF parameters, vault formats, or key-management behavior without a linked issue and human approval.
- Prefer established crates and APIs already in the repo. Do not implement custom crypto.
- Do not manually edit `Cargo.lock` or `pnpm-lock.yaml`; update them through the relevant package manager.

## Code Style

- Rust uses edition 2021 and must pass rustfmt and Clippy with zero warnings.
- TypeScript should stay strict and use existing Svelte 5 patterns.
- Keep Tauri IPC handlers thin. Push reusable vault behavior into `vault-core`.
- Keep changes scoped to one concern per PR.

## Pull Requests

- Use Conventional Commits such as `feat:`, `fix:`, `security:`, `test:`, and `chore:`.
- Mark AI-assisted PRs with the `agent-assisted` label when available.
- Any change touching crypto, auth, vault persistence, IPC secret transport, or secret display/copy behavior needs human review before merge.
- Keep `AGENTS.md` files updated when project commands, constraints, or recurring agent mistakes change.
