# Arca Release Checklist

This checklist is for the first local-first desktop releases. Arca handles
passwords and vault files, so release work should favor repeatability, explicit
limitations, and human review over speed.

## Supported Targets

For v0.1/v0.2 desktop releases, the supported target is:

- macOS on Apple Silicon (`aarch64-apple-darwin`)

Other targets remain unverified until they have a maintainer-owned build and
smoke-test path:

- macOS on Intel (`x86_64-apple-darwin`)
- Windows
- Linux

The Tauri configuration currently builds the macOS `.app` bundle. DMG
distribution is deferred until signing and notarization are configured.

## Required Checks

Run these before creating a release candidate:

```sh
pnpm typecheck
pnpm build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @arca/desktop tauri build
```

If a change only touches documentation, record which checks were intentionally
skipped in the PR or release notes.

## Desktop Smoke Test

Use a disposable test vault. Never use or attach a real personal vault during
release testing.

1. Launch the built app from `target/release/bundle/macos/Arca.app`.
2. Create a new local vault with a non-secret test master password.
3. Add an entry with title, username, password, URL, notes, category, and tags.
4. Lock and unlock the vault.
5. Quit and relaunch the app, then reopen the same vault.
6. Confirm the entry persisted and the password remains masked until explicitly revealed.
7. Edit the entry and confirm the updated fields persist after another relaunch.
8. Delete the entry and confirm it remains deleted after another relaunch.
9. Generate a password, copy it, and confirm clipboard clearing follows the configured setting.
10. Confirm auto-lock follows the configured inactivity timeout.
11. Confirm audit findings do not passively reveal plaintext passwords.

## Vault Files

Arca stores vault data in local vault files selected by the user. The path
autocomplete treats `.arca` and `.kdbx` files as vault candidates.

Expectations:

- Vault files are local files; Arca does not provide remote sync in v0.1/v0.2.
- Users are responsible for backups.
- Vault files should not be committed, attached to public issues, or shared in logs.
- Test fixtures must use non-secret fixture passwords and non-real vault contents.
- Opening a vault requires the correct master password.

Troubleshooting:

- `Invalid master password`: confirm the selected file and password match.
- Missing file: confirm the path still exists and the app has permission to read it.
- Corrupt or unsupported file: keep a backup copy and do not overwrite the original while debugging.
- Permission errors: move the test vault to a user-writable directory and retry.

## macOS Signing And Notarization

Current decision for v0.1/v0.2 internal testing:

- Local release-candidate builds may be unsigned.
- Unsigned builds are for maintainer smoke testing only.
- Do not present unsigned builds as public production releases.

Before a public macOS release:

- Configure Developer ID signing.
- Configure notarization credentials in the release environment.
- Re-enable and verify DMG packaging.
- Verify Gatekeeper behavior on a clean macOS account or machine.
- Document the exact signing and notarization commands or CI workflow.

## Current Build Verification

Last verified locally:

- Date: 2026-06-22
- Host: macOS 26.5.1, Apple Silicon (`aarch64-apple-darwin`)
- Rust: `rustc 1.91.0`
- Command: `pnpm --filter @arca/desktop tauri build`

Expected local artifacts after a successful macOS build:

- `target/release/arca-desktop`
- `target/release/bundle/macos/Arca.app`

Record release-candidate build hashes, attached artifacts, and smoke-test notes
in the GitHub release or release PR.
