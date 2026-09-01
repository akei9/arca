# Arca Client Constitution

Status: proposed
Version: 0.1.0
Date: 2026-09-01
Related: #207, #208, #209, #210, #223

This constitution defines the invariants every Arca client must follow. It is
upstream of ADRs, implementation details, and client repositories. ADRs may
interpret or extend these principles, but they should not contradict them.

The constitution is intentionally short. It describes what must remain true
across desktop, browser extension, mobile, autofill, and future clients.

## Scope

This document governs:

- persisted vault compatibility
- public client API boundaries
- client capability policy
- secret lifetime expectations
- extension and mobile client constraints
- future repository split decisions

This document does not define:

- a sync service
- account management
- a browser extension implementation
- a mobile implementation
- a complete API reference

Those belong in ADRs and implementation issues.

## Principles

| ID | Principle | Enforcement |
| --- | --- | --- |
| C1 | `vault-core` owns KDBX I/O, cryptography, password generation, audit primitives, and secret lifetime behavior. | Rust module boundaries, code review, `vault-core` tests |
| C2 | Clients are adapters. They must not implement KDFs, parse KDBX, or persist unlocked vault material. | Client capability checks, generated bindings, security review |
| C3 | Arca stores vaults as KDBX 4 with pinned write-path parameters. KDBX format version, Arca semantics version, and client protocol version are separate version lines. | ADR-0001, ADR-0005, fixture tests |
| C4 | The public machine contract is a Rust API layer, expected to be `vault-api`, not `vault-core` internals and not JSON Schema. | `vault-api` crate, generated bindings, CI drift checks |
| C5 | Client privileges are encoded as `ClientKind` and `Capability` in Rust. Markdown may describe the matrix, but code must enforce it. | Capability denial tests |
| C6 | Plaintext secrets are explicit, short-lived, and user-driven. Non-secret list, search, and detail DTOs must not carry passwords or revision passwords. | DTO tests, Debug/Display tests, IPC review |
| C7 | Clipboard clearing, reveal, copy, lock, and zeroization are security behavior, not presentation details. | Core policy tests and desktop integration checks |
| C8 | `vault-core` has no network access. Future sync or sharing requires a separate ADR and human review. | Dependency review and ADR gate |
| C9 | Breaking contract changes require an ADR, version bump, fixtures, and migration or refusal tests. | Fixture suite and CI contract drift checks |
| C10 | Browser extension, iOS autofill, and Android autofill are distinct clients with narrower capabilities than the full desktop app. | Capability matrix and boundary ADRs |

## Version Lines

Arca tracks three compatibility lines independently:

- KDBX format version: the industry vault container format Arca reads and writes.
- Arca semantics version: Arca-owned fields and behavior such as collections,
  revisions, archive semantics, audit interpretation, and future migrations.
- Client protocol or binding version: the IPC, FFI, native messaging, or WASM
  interface a specific client uses.

A reader may be newer than a writer. A writer must not write an unknown Arca
semantics version. Unknown future semantics must fail closed unless an ADR
explicitly defines read-only fallback behavior.

## Initial Client Kinds

The public API should model at least these clients:

- `DesktopApp`
- `BrowserExtension`
- `IosApp`
- `AndroidApp`
- `IosAutofillExtension`
- `AndroidAutofillService`
- `FutureSyncServer`

`FutureSyncServer` is ciphertext-only. It must not receive plaintext secrets.

## Initial Capabilities

The public API should model at least these capabilities:

- `Unlock`
- `ReadMeta`
- `RevealSecret`
- `CopySecret`
- `MutateEntry`
- `CreateVault`
- `ChangeKdf`
- `ExportPlaintext`
- `ExportKdbx`
- `ReadHistory`
- `DeletePermanent`

Each client kind receives only the capabilities it needs. Autofill and browser
clients should be narrower than the desktop app.

## ADR Set

The contract phase starts with these proposed ADRs:

- ADR-0001: KDBX 4 canonical on-disk format
- ADR-0002: Rust `vault-api` is the machine contract
- ADR-0003: Client capability matrix
- ADR-0004: Secret lifetime
- ADR-0005: Versioning and migrations
- ADR-0006: Repository layout
- ADR-0007: Browser extension boundary
- ADR-0008: Mobile UniFFI and native UI

All ADRs start as proposed. Human review is required before treating them as
accepted architecture.

## Amendment Process

Changing this constitution requires:

1. A dedicated PR.
2. Human approval.
3. A version bump in this file.
4. A note explaining which ADRs, issues, or compatibility tests need follow-up.
