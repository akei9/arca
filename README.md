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

> **Pre-1.0 / pre-release.** Arca has not yet had a tagged release or an
> independent security audit. Review the code and threat model before trusting
> it with real secrets. See the [v0.1 milestone](https://github.com/akei9/arca/milestone/1).

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

## Security model

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/13eadf72-0119-4aa8-bdfa-cf580d1e1520">
  <img alt="security-model" src="https://github.com/user-attachments/assets/0a82f493-2bc9-4ff7-9d49-658f01556a50" width="100%">
</picture>

- **Vault format:** KeePass KDBX 4, so vaults stay portable and interoperable. Persistence goes through the [`keepass`](https://crates.io/crates/keepass) crate.
- **At-rest encryption:** the current KDBX 4 defaults - **Argon2d** key derivation from your master password, an **AES-256** outer cipher, and a **ChaCha20** stream cipher for protected fields.
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

## Build from source

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/dc6e24e9-d045-40f8-8069-1347a8ff220e">
  <img alt="build-from-source" src="https://github.com/user-attachments/assets/09803697-4549-41a6-a08e-cb53c7446cc1" width="100%">
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
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/a99fbe08-e6e0-425e-ac0c-68e12efd6f53">
  <img alt="project-layout" src="https://github.com/user-attachments/assets/49cfcaa1-8a63-472b-9b9a-5a7ded0bff53" width="100%">
</picture>

- `apps/desktop` - Tauri v2 desktop app (Svelte 5 frontend + thin Rust command layer).
- `packages/vault-core` - Rust crate holding all cryptography, vault parsing, and secret handling.

## Contributing

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/521ce1de-abd7-4bd8-90d9-621559d66157">
  <img alt="contributing" src="https://github.com/user-attachments/assets/01a6e9f3-f080-4e62-85f7-5ad8010d1573" width="100%">
</picture>

Contributions are welcome - see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Changes touching crypto, vault
persistence, IPC secret transport, or secret display/copy behavior require human
review before merge.

## License

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/89a4422c-bd14-44da-b352-24fe4138a982">
  <img alt="license" src="https://github.com/user-attachments/assets/c6391763-c359-4b82-b5ac-bc5e4e6bbb38" width="100%">
</picture>

[MIT](LICENSE) © 2026 Adrian Kucharczyk
