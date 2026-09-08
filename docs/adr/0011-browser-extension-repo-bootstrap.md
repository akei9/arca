---
status: proposed
date: 2026-09-08
decision-makers: ["akei9"]
related: ["#138", "#207", "#208", "#209", "#210", "#211", "#213"]
---

# ADR-0011: Browser extension repository bootstrap

## Status History

- 2026-09-08 - proposed by Codex for maintainer review.

## Context

ADR-0007 accepts the first browser extension boundary: the extension is a thin
UI and autofill client, while unlock, KDF work, KDBX parsing, vault persistence,
and long-lived unlocked material stay outside the browser. ADR-0009 keeps the
first extension local-first through native messaging to the desktop app or a
small native host. ADR-0010 defers creating `arca-extension` until browser
extension implementation starts.

This ADR turns that accepted boundary into the implementation and repository
bootstrap plan needed before generating TypeScript bindings or creating the
extension repository.

## Decision Drivers

- Preserve the Arca client constitution and `vault-api` capability policy.
- Avoid KDBX, KDF, vault persistence, and unlocked vault sessions in browser
  extension contexts.
- Support useful autofill without broad browser permissions or hidden secret
  movement.
- Make the native messaging surface explicit before implementation.
- Keep browser-specific release, signing, and store workflows outside
  `akei9/arca` once implementation starts.

## Considered Options

- Bootstrap `arca-extension` immediately with planning docs only.
- Keep extension implementation in `akei9/arca`.
- Bootstrap `arca-extension` when implementation starts, after this ADR and the
  native messaging contract are reviewed.
- Use a standalone WASM vault engine in the extension.
- Use native messaging to the desktop app or a small native host.

## Proposed Decision

Keep all #211 planning work in `akei9/arca`. Create `arca-extension` only when
implementation starts and the bootstrap checklist in this ADR is satisfied.

The first implementation target is Chromium-family browsers using a
Manifest-V3-compatible architecture and native messaging. Chrome is the first
store/package target. Microsoft Edge may reuse the Chromium package after Chrome
support is stable and store metadata is ready.

Firefox is an MVP compatibility target, but not the first implementation target.
Before claiming Firefox support, the native messaging host registration,
permissions, background lifecycle behavior, and automated compatibility checks
must pass against Firefox. Safari is out of scope for the first extension MVP
because its packaging and native app extension model need a separate spike.

The extension is a `BrowserExtension` client in the `vault-api` capability
matrix. It may request only `Unlock`, `ReadMeta`, and `CopySecret` operations
through the bridge. It must not receive `RevealSecret`, `MutateEntry`,
`CreateVault`, `ChangeKdf`, `ExportPlaintext`, `ExportKdbx`, `ReadHistory`, or
`DeletePermanent` unless a later ADR expands the capability matrix with tests.

Unlock stays native-owned. The extension may ask the native side to start or
focus an unlock flow, but it must not collect or transmit the master password.
The native side owns KDF execution, vault file access, session lifetime, and any
OS clipboard integration used for copy operations.

## Native Messaging Protocol Surface

Define the first native messaging contract in `akei9/arca` before
implementation. The source of truth is Rust `vault-api`; generated TypeScript
bindings in `arca-extension` consume the contract and must be checked for drift.

The first protocol version should expose only these message families:

- `hello`: negotiate client protocol version, extension build metadata, native
  host version, and granted `ClientKind::BrowserExtension` capabilities.
- `status`: report whether a vault is available, locked, or unlocked without
  returning plaintext secrets or a full vault snapshot.
- `requestUnlock`: ask the native side to show or focus its unlock UI. The
  response is status-only.
- `searchEntries`: return redacted entry metadata needed to choose a fill
  candidate.
- `getEntryMeta`: return redacted metadata for one entry, excluding password
  and revision passwords.
- `copySecret`: copy one selected secret through the native side after explicit
  user action and capability enforcement. Prefer host-owned clipboard writes.
- `fillSecret`: request a one-time fill payload for one selected entry and
  origin after explicit user approval. This maps to `CopySecret`, not
  `RevealSecret`, and the extension must drop the payload immediately after the
  content script fills the selected fields.
- `fillApproved`: record that the user approved filling one selected origin and
  entry, without widening the extension's capability set or returning a secret.
- `lock`: ask the native side to lock the session.
- `error`: return typed `vault-api` errors, including capability denial.

Every request must identify the protocol version, client kind, browser
extension instance, active tab URL or origin when relevant, and a correlation ID
that is safe to log. The bridge must treat all browser-provided fields as
untrusted input and must never log plaintext secrets.

The bridge must fail closed on unknown protocol versions, unknown message
types, missing client kind, capability mismatch, malformed URLs, or host/extension
origin mismatch.

## Extension Surface

The extension repository should start with these user-visible surfaces only:

- browser action popup for connection, locked/unlocked state, search, and copy
  or fill actions;
- content script that detects login fields and requests user approval before
  filling;
- background service worker that brokers messages between popup, content
  scripts, and the native messaging port;
- options page for host connection diagnostics and permissions explanation.

The extension must not keep decrypted vault snapshots in `chrome.storage`,
`browser.storage`, IndexedDB, localStorage, sessionStorage, or cache storage. It
may cache non-secret metadata for a bounded session only when invalidated on
lock, host disconnect, tab origin change, or protocol version change.

Autofill requires explicit user action for the MVP. Fill payloads are scoped to
one selected origin and entry, are not cached, and must not be reused for later
page events. Silent page-driven fill, automatic submit, broad credential
scraping, and injected password generation are out of scope.

## WASM Boundary

WASM is not the primary unlock path and must not parse KDBX, run KDFs, persist
vaults, hold the unlocked vault session, or implement independent vault
semantics in the extension.

A later ADR may allow WASM for constrained already-unlocked working-set helpers
such as local filtering of redacted metadata, URL matching, or non-secret
validation. Any WASM helper must consume `vault-api` DTOs, avoid plaintext
secrets unless a capability-specific ADR allows it, and pass the same fixture
drift checks as native bindings.

## Repository Bootstrap Checklist

Create `arca-extension` when implementation starts and all of the following are
true:

- [ ] ADR-0007, ADR-0009, ADR-0010, and this ADR are accepted.
- [ ] `vault-api` exposes the browser extension operations or adapter contract
  needed by the MVP.
- [ ] Native messaging message schemas or generated bindings are committed in
  `akei9/arca` with fixture drift tests.
- [ ] Capability denial tests prove `BrowserExtension` cannot reveal, mutate,
  export, read history, change KDF settings, or delete permanently.
- [ ] Permission rationale is documented for `nativeMessaging`, active tab or
  host permissions, content scripts, storage, and clipboard behavior.
- [ ] Chrome package and store submission flow are documented.
- [ ] Firefox compatibility requirements are documented before support is
  advertised.
- [ ] CI covers TypeScript linting, tests, build, dependency scanning, secret
  scanning, browser compatibility checks, and generated contract drift.
- [ ] Security issue routing points shared contract, vault, native messaging,
  and secret-lifetime issues back to `akei9/arca`.

## Consequences

### Positive

- Keeps browser code out of the vault engine and KDF path.
- Gives extension implementation a narrow, testable protocol surface.
- Lets browser packaging and store review move independently once
  implementation begins.
- Aligns extension MVP work with the already enforced capability matrix.

### Negative

- Extension usefulness depends on installing or running the desktop app or
  native host.
- Chrome ships before complete browser coverage.
- Native messaging adds OS-specific install and diagnostics work.

### Neutral

- Edge and Firefox support can follow without changing Arca's vault contract if
  their platform behavior satisfies the same bridge requirements.

## Compliance

- [ ] Add the native messaging protocol contract in `akei9/arca`.
- [ ] Add browser extension generated binding drift tests.
- [ ] Add bridge-level capability denial tests for `BrowserExtension`.
- [ ] Add permission and store-review documentation to `arca-extension` before
  first release.
- [ ] Create `arca-extension` only when implementation starts.

## Revisit / Out Of Scope

- Creating `arca-extension` in this PR.
- Safari extension support.
- Browser-side standalone unlock.
- Direct browser vault-file access.
- Sync, pairing, remote unlock, accounts, or shared vaults.
- Silent autofill or submit automation.

## References

- #138
- #207
- #208
- #209
- #210
- #211
- #213
- Chrome Extensions native messaging documentation:
  https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging
- MDN WebExtensions native messaging documentation:
  https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging
