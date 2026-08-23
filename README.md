# arca

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/77654f67-5294-4d34-94d2-e79786ffc99c">
  <img alt="Arca - a vault for what you can't lose." src="https://github.com/user-attachments/assets/c44544b8-8737-41c1-8d1a-9aa8583bb7a5">
</picture>

Open-source, local-first password vault with a terminal-inspired UI.

Arca keeps your credentials in a single encrypted vault file on your own
machine - no accounts, no sync, no telemetry. The vault is decrypted only while
it is unlocked, and locking zeroizes the decrypted vault held in Arca's Rust
core.

> **Pre-1.0.** The first tagged release is
> [v0.1.0](https://github.com/akei9/arca/releases/latest); Arca has not had an
> independent security audit. Review the code and threat model before trusting
> it with real secrets.

## Features

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/35735290-74fb-486e-ba75-b432ecc132ab">
  <img alt="Features" src="https://github.com/user-attachments/assets/d2e7f429-7d23-4b91-b44b-a70197ed50d9" width="100%">
</picture>

- **Local encrypted vault** - one portable vault file, opened with your master password.
- **Entries** - title, username, password, URL, notes, collection, and tags, with fast search.
- **Password generator** - configurable length and character classes, with an entropy readout.
- **Reveal & copy discipline** - passwords stay masked until explicitly revealed; the clipboard auto-clears after a configurable timeout.
- **Auto-lock** - the vault locks after a configurable period of inactivity.
- **Revision history** - a bounded, encrypted history of prior entry versions; reveal or copy a previous password on demand.
- **Audit** - surfaces weak or reused credentials without exposing plaintext.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/54aba77e-d7b5-48b3-aa25-d6f994053f68">
  <img alt="arca-tour" src="https://github.com/user-attachments/assets/10544b79-2c65-4571-b5fb-b195d8bfd645" width="100%">
</picture>

## Security model

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/13eadf72-0119-4aa8-bdfa-cf580d1e1520">
  <img alt="security-model" src="https://github.com/user-attachments/assets/0a82f493-2bc9-4ff7-9d49-658f01556a50" width="100%">
</picture>

- **Vault format:** KeePass KDBX 4, so vaults stay portable and interoperable. Persistence goes through the [`keepass`](https://crates.io/crates/keepass) crate.
- **At-rest encryption:** Arca pins the KDBX 4 parameters rather than relying on library defaults - **Argon2id** key derivation from your master password (128 MiB, 3 iterations, parallelism 4), a **ChaCha20** outer cipher, and a **ChaCha20** stream cipher for protected fields.
- **Secret handling:** while the vault is unlocked, decrypted values necessarily flow through the app to be shown or copied - the Rust session state, the Tauri IPC responses, UI strings, and the system clipboard during reveal/copy. Rust-side secrets use `Zeroizing`/`ZeroizeOnDrop` and are zeroized on lock, and secrets are never logged; clipboard contents clear on the configured timeout rather than on lock.
- **Local only:** no sync, accounts, or telemetry in this release. You own your vault file and are responsible for its backups.

To report a vulnerability, see [SECURITY.md](.github/SECURITY.md). Please do not
include real secrets in reports.

## Status & limitations

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/705da9a9-8ce2-4703-95b2-18abf8bcdad5">
  <img alt="status-limitations" src="https://github.com/user-attachments/assets/09faa24b-22cc-4af1-8de7-5b635d14a2c4" width="100%">
</picture>

The first desktop target is **macOS on Apple Silicon** (`aarch64-apple-darwin`);
other platforms are currently unverified. See [RELEASE.md](RELEASE.md) for the
release process and supported targets.

- Entries require a non-empty password; passwordless entries and password clearing are not supported ([#74](https://github.com/akei9/arca/issues/74)).
- No remote sync - you are responsible for backing up your vault file.

## Install

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/7aaa3fd1-24a6-4899-a548-c4ef1d6a2c60">
  <img alt="status-limitations" src="https://github.com/user-attachments/assets/85516f53-c08f-42f8-a839-162234a13e93" width="100%">
</picture>

Download the latest signed and notarized macOS build from the
[releases page](https://github.com/akei9/arca/releases/latest):

1. Download `Arca_<version>_aarch64.dmg`.
2. Open it and drag **Arca** into your Applications folder.
3. Launch it - the build is signed with a Developer ID certificate and notarized
   by Apple, so it opens without a Gatekeeper prompt.

Only macOS on Apple Silicon (`aarch64-apple-darwin`) is published today; for any
other platform, build from source below.

## Build from source

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/e96722f7-0573-44d1-b2b8-aa3afb17f7e6">
  <img alt="build-from-source" src="https://github.com/user-attachments/assets/55c4c87a-b50c-4ffd-a5c3-fdb3f2868a8c" width="100%">
</picture>

Prerequisites: [Node.js](https://nodejs.org) + [pnpm](https://pnpm.io), a
[Rust](https://rustup.rs) toolchain, and the
[Tauri v2 system prerequisites](https://v2.tauri.app/start/prerequisites/) for
your OS.

```sh
pnpm install
pnpm dev                                  # run the app in development
pnpm --filter @arca/desktop tauri build   # build the desktop bundle
```

Checks (see [RELEASE.md](RELEASE.md) for the full pre-release gate):

```sh
pnpm lint            # svelte-check
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Project layout

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/44fe12ff-3d71-4a1c-b501-589ed819ce0f">
  <img alt="project-layout" src="https://github.com/user-attachments/assets/02237161-de75-471d-94c3-470ecf8d4865" width="100%">
</picture>

- `apps/desktop` - Tauri v2 desktop app (Svelte 5 frontend + thin Rust command layer).
- `packages/vault-core` - Rust crate holding all cryptography, vault parsing, and secret handling.

## Contributing

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/7692aa5b-9b5a-438d-beb7-d12045f8813b"">
  <img alt="contributing" src="https://github.com/user-attachments/assets/e5995fb6-3af6-4bc3-a812-47d70b049a4d" width="100%">
</picture>

Contributions are welcome - see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Changes touching crypto, vault
persistence, IPC secret transport, or secret display/copy behavior require human
review before merge.

## License

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/82019149-74d9-42e1-a4c4-4060c087c943">
  <img alt="license" src="https://github.com/user-attachments/assets/f4ac5ee0-5f90-456e-8df7-7b26c07da863" width="100%">
</picture>

[MIT](LICENSE) © 2026 Adrian Kucharczyk
