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

- **Local encrypted vault** - one portable vault file, opened with your master password.
- **Entries** - title, username, password, URL, notes, collection, and tags, with fast search.
- **Password generator** - configurable length and character classes, with an entropy readout.
- **Reveal & copy discipline** - passwords stay masked until explicitly revealed; the clipboard auto-clears after a configurable timeout.
- **Auto-lock** - the vault locks after a configurable period of inactivity.
- **Revision history** - a bounded, encrypted history of prior entry versions; reveal or copy a previous password on demand.
- **Audit** - surfaces weak or reused credentials without exposing plaintext.

## Security model

- **Vault format:** KeePass KDBX 4, so vaults stay portable and interoperable. Persistence goes through the [`keepass`](https://crates.io/crates/keepass) crate.
- **At-rest encryption:** the current KDBX 4 defaults - **Argon2d** key derivation from your master password, an **AES-256** outer cipher, and a **ChaCha20** stream cipher for protected fields.
- **Secret handling:** while the vault is unlocked, decrypted values necessarily flow through the app to be shown or copied - the Rust session state, the Tauri IPC responses, UI strings, and the system clipboard during reveal/copy. Rust-side secrets use `Zeroizing`/`ZeroizeOnDrop` and are zeroized on lock, and secrets are never logged; clipboard contents clear on the configured timeout rather than on lock.
- **Local only:** no sync, accounts, or telemetry in this release. You own your vault file and are responsible for its backups.

To report a vulnerability, see [SECURITY.md](.github/SECURITY.md). Please do not
include real secrets in reports.

## Status & limitations

The first desktop target is **macOS on Apple Silicon** (`aarch64-apple-darwin`);
other platforms are currently unverified. See [RELEASE.md](RELEASE.md) for the
release process and supported targets.

- Entries require a non-empty password; passwordless entries and password clearing are not supported ([#74](https://github.com/akei9/arca/issues/74)).
- No remote sync - you are responsible for backing up your vault file.

## Build from source

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

- `apps/desktop` - Tauri v2 desktop app (Svelte 5 frontend + thin Rust command layer).
- `packages/vault-core` - Rust crate holding all cryptography, vault parsing, and secret handling.

## Contributing

Contributions are welcome - see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Changes touching crypto, vault
persistence, IPC secret transport, or secret display/copy behavior require human
review before merge.

## License

[MIT](LICENSE) © 2026 Adrian Kucharczyk
