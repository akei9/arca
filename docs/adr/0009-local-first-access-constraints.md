---
status: proposed
date: 2026-09-07
decision-makers: ["akei9"]
related: ["#88", "#138", "#210", "#211", "#216"]
---

# ADR-0009: Local-first access constraints before sync or new clients

## Status History

- 2026-09-07 - proposed by Codex for maintainer review.

## Context

Browser extension and mobile clients need access to vault data without turning
Arca into an account-backed or network-first system. The first cross-client
contract should preserve local vault ownership, keep `vault-core` offline, and
avoid designing sync before the local boundaries are stable.

This ADR defines access constraints for early browser and mobile work. It does
not approve a sync service, sharing protocol, account system, or remote storage
provider integration.

## Decision Drivers

- Preserve local-first vault ownership.
- Keep KDFs, KDBX parsing, and decrypted vault material inside Rust boundaries.
- Keep `vault-core` free of network access.
- Make browser and autofill clients depend on narrow, explicit bridges.
- Let mobile apps use OS-native file and credential surfaces without adding an
  Arca account model.
- Leave sync metadata and conflict resolution for a later ADR.

## Considered Options

- Desktop-bridge or native-host-backed access for browser extension clients.
- File-based local access for full mobile apps.
- Cloud-file-provider-backed local files on mobile.
- Direct browser extension vault-file access.
- Account-backed sync as the first multi-client access model.
- Network sync embedded in `vault-core`.

## Proposed Decision

Arca remains local-first for the first extension and mobile clients.

`vault-core` must not perform network I/O. It owns local KDBX parsing,
cryptography, password generation, audit primitives, and vault mutation. Any
future network sync or sharing component requires a separate ADR, a threat model,
and human review before implementation.

The desktop app continues to access local vault files directly through the Tauri
shell and Rust vault/session boundary.

The first browser extension should access vault data through native messaging to
the desktop app or a small native host. The extension must not parse KDBX, run
the full unlock path in a content script or service worker, directly read vault
files, persist unlocked vault material, or rely on cloud storage APIs as its
primary access path. Browser access is a bridge to an already installed local
component, not a standalone vault engine.

The first full mobile apps may use local file access through OS document
pickers, app-scoped storage, secure storage for platform-bound auxiliary
material, and cloud file providers only as local file providers exposed by the
operating system. The mobile app must still route vault semantics through
`vault-api` and `vault-core`; cloud provider SDKs, account-backed remote APIs,
and Arca-hosted sync are out of scope.

Mobile autofill extensions are separate restricted clients. They should request
only the smallest working set needed for an explicit autofill interaction and
must not retain decrypted vault contents beyond the platform extension
lifecycle.

All early clients are offline-first. A client must remain useful with a local
vault file and no network. If a vault file changes externally while a client has
it open, the first implementation may fail closed, prompt the user to close and
reopen, or use a tested single-writer handoff. Automatic merge, multi-device
conflict resolution, per-record sync clocks, and remote tombstones are out of
scope until a sync ADR exists.

Native messaging and device handoff boundaries must identify the calling client
kind, enforce `vault-api` capabilities, avoid logging plaintext secrets, and
treat message payloads as untrusted input. Pairing protocols, long-lived device
trust, push notifications, and background remote unlock are out of scope for the
first extension and mobile MVPs.

Sync metadata must not be added to persisted vault semantics unless a later ADR
defines it. `FutureSyncServer` remains ciphertext-only and must not receive
plaintext secret capabilities.

## Consequences

### Positive

- Keeps future clients aligned with Arca's local-first promise.
- Prevents sync assumptions from leaking into the first public API and client
  bootstraps.
- Gives extension and mobile work concrete access boundaries before
  implementation begins.
- Keeps network dependencies out of `vault-core`.

### Negative

- Browser extension usefulness depends on a native host or desktop app being
  installed.
- Mobile cross-device use relies on user-managed files or OS file providers
  until sync is designed.
- Conflict handling remains intentionally limited for the first client phase.

### Neutral

- A future sync service may still be designed later, but it starts from a
  ciphertext-only and threat-modeled boundary.

## Compliance

- [ ] Verify `vault-core` remains free of network dependencies and network I/O.
- [ ] Add a native messaging protocol contract before browser extension
  implementation.
- [ ] Add extension tests that identify the client kind and enforce restricted
  capabilities at the bridge.
- [ ] Add mobile bootstrap notes for OS file access, secure storage, and
  autofill lifecycle boundaries.
- [ ] Require a separate sync ADR before adding sync metadata, remote conflict
  resolution, account-backed access, device pairing, or Arca-hosted services.

## Revisit / Out Of Scope

- Account management.
- Shared vaults and secure sharing.
- Remote sync transport and conflict resolution.
- Device pairing or long-lived device trust.
- Browser extension direct KDBX parsing or standalone unlock.
- Cloud provider SDK integrations.

## References

- #88
- #138
- #210
- #211
- #216
