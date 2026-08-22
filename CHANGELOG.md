# Changelog

All notable changes to Arca are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-22

First public desktop release. A local-first, KDBX 4 password vault with a
terminal-inspired UI. Supported target: macOS on Apple Silicon
(`aarch64-apple-darwin`); other platforms are unverified.

### Added

- Local encrypted KDBX 4 vault - create, open, lock, and unlock with a master password.
- Entry management with title, username, password, URL, notes, collection, and tags, plus live search, collections, and tag filters.
- Password generator with configurable length and character sets, a passphrase mode, and an entropy readout.
- Bounded, encrypted entry revision history, with a panel to browse prior versions and reveal or copy a previous password.
- Auto-lock after a configurable inactivity timeout (or disabled).
- Clipboard auto-clear on a configurable timer after copying a secret.
- Audit view that surfaces weak or reused credentials without exposing plaintext.
- Full keyboard navigation across the app.
- Paper and Ink themes and an adjustable font size.
- Custom desktop window chrome and terminal-style vault-path autocomplete.

### Security

- At-rest cryptography pinned explicitly (Argon2id, 128 MiB / 3 iterations / parallelism 4; ChaCha20 outer cipher; ChaCha20 for protected fields) rather than relying on library defaults.
- Secrets are held in `Zeroizing` / `ZeroizeOnDrop` types and zeroized on lock; `Debug` never renders decrypted vault contents.
- No network access, accounts, sync, or telemetry.

### Notes

- Entries require a non-empty password; passwordless entries and password clearing are unsupported ([#74](https://github.com/akei9/arca/issues/74)).
- No remote sync - you are responsible for backing up your vault file.

[Unreleased]: https://github.com/akei9/arca/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/akei9/arca/releases/tag/v0.1.0
