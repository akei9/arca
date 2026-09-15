# Canonical mobile vault-session contract

Status: contract only; implementation and native-adapter work follow
separately.

This document defines the public Rust boundary for the full iOS and Android
applications. It implements the decisions in ADR-0002, ADR-0004, ADR-0014, and
ADR-0015 without implementing the operations themselves. The Rust types that
make this contract machine-checkable live in `vault_api::mobile_session`.

## Boundary and ownership

`vault-api` is the only crate exposed through the future mobile UniFFI facade.
Swift and Kotlin must not bind `vault-core`, call its KDBX functions, or define
parallel vault semantics. The implementation will privately delegate from the
`vault-api` session facade to `vault-core`.

One native session coordinator owns one Rust session object. A session object:

- is constructed once with immutable `ClientKind::IosApp` or
  `ClientKind::AndroidApp` identity;
- owns at most one unlocked document and one in-flight save;
- serializes operations so an unlock, mutation, save, and lock cannot race;
- never exposes a session identifier that another app process can redeem;
- never accepts a caller-selected client kind after construction; and
- is not shared with an iOS Credential Provider or Android Autofill Service.

Autofill clients require a separate restricted facade and are not covered by
this full-app contract.

`MobileSessionClient::authorize` is mandatory at the start of every public
session operation, before state inspection, KDBX work, secret lookup, mutation,
or save preparation. `MobileSessionClient::authorize_document` is additionally
required before accepting a document. The guard itself must be private session
state in the implementation, so native adapters cannot bypass it. Creating a
full-app session with any other `ClientKind` fails with `capability_denied`.

## Platform document access

Permission-scoped document locators remain owned by native code:

- iOS retains its security-scoped URL or security-scoped bookmark, calls
  `startAccessingSecurityScopedResource`, coordinates the read or write, and
  balances it with `stopAccessingSecurityScopedResource`.
- Android retains its Storage Access Framework `content://` URI and persisted
  grant, and performs reads and writes through `ContentResolver` or the
  provider's file descriptor while the grant is valid.
- App-scoped files use the same boundary even when the OS does not require an
  explicit grant.

A raw path, URL, bookmark, or document URI never crosses UniFFI and is never
stored by Rust. The native coordinator instead allocates a process-local opaque
`MobileDocumentHandle` and keeps the locator in its own handle table. Rust
accepts only bounded ASCII alphanumeric identifiers plus `-` and `_`, rejecting
values shaped like absolute or relative paths, URLs, or `content://` URIs.

For open, native code performs a coordinated read while permission is active,
captures an opaque `DocumentRevision` from stable platform metadata, and sends
the handle, revision, access mode, and encrypted KDBX bytes to Rust. Encrypted
bytes are held in `EncryptedVaultBytes`, are omitted from formatting and
serialization, and are cleared on drop as defense in depth.

For save, the native adapter re-reads the current revision immediately before
calling Rust. Rust compares it with the revision captured by the session. A
mismatch returns `external_file_changed` and no write payload. On a match, Rust
prepares `PreparedMobileVaultWrite`; native code performs a coordinated,
replacement-style write through the original permission surface and then sends
`CommitMobileVaultWriteRequest` with the new revision. The session becomes clean
only after this acknowledgement. Providers that cannot support a safe replace
must fail with `save_failed` or use an explicit user-selected duplicate-save;
they must not silently overwrite externally changed data.

Permission checks and native I/O happen outside Rust because possession of a URL
or URI string does not grant Rust the platform capability to use it. The native
adapter maps a revoked, stale, or unavailable grant to
`document_permission_lost`, invalidates the handle, and tells the Rust session
to lock and discard its in-memory state. A moved or deleted document maps to
`file_not_found` unless the platform reports a permission failure instead.

## Session surface

The session facade should expose the following logical operations. Names may be
adapted mechanically to UniFFI naming rules, but the arguments, results,
capability checks, and state transitions are normative.

| Operation | Contract input/result | Required capability | State rule |
| --- | --- | --- | --- |
| create | `CreateMobileVaultRequest` / `PreparedMobileVaultWrite` | `CreateVault` | Prepare an empty KDBX write; become unlocked-clean only after native commit acknowledgement. |
| open | `OpenMobileVaultRequest` / `MobileSessionStatus` | `Unlock` | Register encrypted bytes and become locked; do not decrypt. |
| unlock | `UnlockMobileVaultRequest` / `MobileVaultSummary` | `Unlock` | Decrypt into Rust-owned memory; the input password is consumed and zeroized. |
| lock | none / `MobileSessionStatus` | `Unlock` | Idempotently discard password, decrypted entries, pending secret results, encrypted staging bytes, and pending writes. |
| read summary | none / `MobileVaultSummary` | `ReadMeta` | Require unlocked; return no document locator or password. |
| list/read entry | id or none / `EntryView` values | `ReadMeta` | Require unlocked; current and revision passwords are absent. |
| search | query / `EntryView` values | `ReadMeta` | Require unlocked; search non-secret metadata in Rust. |
| reveal | entry id / `RevealedSecret` | `RevealSecret` | Require unlocked and explicit user action; return one short-lived secret. |
| copy | entry id / `SecretForCopy` | `CopySecret` | Require unlocked and explicit user action; native writes immediately to the clipboard and drops the response. |
| generate | `GeneratorParams` / `GeneratedSecret` | `RevealSecret` | Return one short-lived generated password; do not retain it unless a later mutation receives it. |
| create entry | `CreateEntryRequest` / `EntryView` | `MutateEntry` | Mutate only the in-memory session and mark it dirty. |
| update entry | id plus `EntryMutation` / `EntryView` | `MutateEntry` | Mutate only the in-memory session and mark it dirty. |
| delete entry | id / password-free completion | `MutateEntry` | Use the existing shared delete semantics, mark dirty, and do not add a mobile-only permanent-delete path. |
| save | `PrepareMobileVaultSaveRequest` / `PreparedMobileVaultWrite` | `MutateEntry` | Compare revisions, prepare encrypted bytes, and remain dirty until native commit acknowledgement. |

The operation-to-capability mapping is represented by `ApiOperation`, and
`MOBILE_VAULT_SESSION_OPERATIONS` is the complete first-session surface.
`IosApp` and `AndroidApp` must pass enforcement on every row. Both continue to
deny `ExportPlaintext`.

Create, mutate, delete, and save call `MobileSessionClient::authorize_write` so
the capability check and platform match happen before the access-mode check. A
read-only document returns `document_read_only` before any in-memory mutation;
the session remains clean and readable.

Mutation and save are deliberately separate. A successful mutation does not
claim durability. A failed save leaves the unlocked session dirty so the user
can retry or choose duplicate-save. Explicit lock, lifecycle lock, timeout,
permission loss, or external change discards dirty in-memory changes along with
all decrypted state; the native UI must warn about unsaved changes before a
user-initiated lock where interaction is possible, but lifecycle security locks
must not be delayed.

## DTO and plaintext rules

Password-free DTOs are `MobileDocument`, `MobileVaultSummary`,
`MobileSessionStatus`, `EntryView`, and `RevisionView`. They never contain a
master password, current password, revision password, generated password,
clipboard value, vault key, KDF output, raw document locator, or encrypted vault
payload. A password-free DTO may be serialized for generated bindings, but it
must not be treated as safe telemetry: vault and entry metadata can still be
private.

Secret-bearing inputs are `SecretString`, `UnlockMobileVaultRequest`,
`CreateMobileVaultRequest`, `CreateEntryRequest`, and the password member of
`EntryMutation`. Explicit plaintext outputs are only `RevealedSecret`,
`SecretForCopy`, and `GeneratedSecret`.

Rust secret owners use zeroizing storage and redact `Debug`; they do not
implement a plaintext `Display`. Only the three explicit output wrappers may
serialize plaintext, because serialization is the audited bridge transfer.
Bindings can create unavoidable Swift `String`, Kotlin `String`, or FFI buffer
copies; adapters must keep those copies inside the smallest callback or view,
must never format or log the containing request/response, and must clear or
release them immediately after reveal, clipboard write, generation, or
mutation. No response is cached by the session facade.

The Rust session owns the master password only while unlocked so it can save the
vault using existing KDBX behavior. Lock and all fail-closed transitions drop
that value and every decrypted `vault-core` object. Native code must not retain
the master password after the unlock call and must never persist it.

## Stable errors and transitions

Mobile errors cross the binding as `ApiError { code, message }`. Code is the
machine contract; message is the fixed value from `ErrorCode::safe_message`.
Native adapters and the implementation must use `ApiError::stable` for
these errors and must not include file locators, provider messages, parser
details, passwords, entry contents, or other caller-controlled values in the
message, `Debug`, `Display`, serialization, diagnostics, or logs.

| Condition | Code | Required result |
| --- | --- | --- |
| Invalid master password | `invalid_password` | Stay locked, discard the attempted password, retain only the encrypted staging bytes for retry. |
| Corrupted or unsupported vault bytes | `corrupted_vault` | Lock and discard all staged/decrypted state; require a fresh open. |
| Revoked, stale, or missing platform grant | `document_permission_lost` | Invalidate the handle, lock, discard state, and require the platform picker or grant recovery. |
| Selected document is read-only | `document_read_only` | Reject create, mutation, delete, and save before state changes; keep the current clean/readable state. |
| Document moved or deleted | `file_not_found` | Do not save; lock if the active handle can no longer identify the opened document. |
| Revision changed outside Arca | `external_file_changed` | Produce no write payload, lock and discard state, and require reopen. Never merge or overwrite. |
| Native or Rust save step fails | `save_failed` | Return no success acknowledgement; keep the unlocked session dirty for retry unless a lifecycle lock independently occurs. |
| Operation while not unlocked | `vault_locked` | Leave state locked and return no partial data. |
| Wrong client kind or denied operation | `capability_denied` | Perform no state inspection or mutation and return no partial data. |

Underlying `vault-core`, OS, filesystem, content-provider, and cryptography error
payloads remain internal. The contract does not change cryptographic algorithms,
KDF parameters, KDBX versions, or Arca vault semantics.

## Implementation verification

The implementation PR must add synthetic-fixture tests for the full state
machine, capability enforcement before every operation, invalid password,
corruption, permission loss, external revision mismatch, two-phase save success,
save failure retry, and zeroization on lock/failure. Generated Swift and Kotlin
bindings must prove they expose `vault-api` session types only and do not link a
public `vault-core` surface.
