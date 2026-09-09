---
status: proposed
date: 2026-09-09
decision-makers: ["akei9"]
related: ["#138", "#207", "#210", "#214", "#215", "#216"]
---

# ADR-0014: Mobile security model for storage, biometrics, and autofill

## Status History

- 2026-09-09 - proposed by Codex for maintainer review.

## Context

Mobile clients add platform security surfaces that do not exist in the desktop
Tauri app: OS secure storage, biometric prompts, app switcher previews,
backgrounding, screenshot and screen recording surfaces, clipboard behavior,
notifications, inter-app intents, and native autofill extensions.

ADR-0008 accepts native iOS and Android apps using `vault-api` through UniFFI.
ADR-0009 keeps early mobile clients local-first and offline-first. ADR-0012
defers creating `arca-mobile` until implementation starts and defines the first
mobile app and autofill client capability sets. This ADR defines the mobile
security model that must be satisfied before implementation begins.

## Decision Drivers

- Keep master passwords, vault keys, decrypted vault contents, and plaintext
  secrets out of platform storage, logs, notifications, crash reports, and
  unnecessary UI or bridge memory.
- Treat biometric authentication as an OS gate, not as a replacement for the
  master password or Rust vault unlock policy.
- Keep full mobile apps useful while still narrower than the desktop app.
- Keep iOS and Android autofill extensions separate from full app clients and
  restricted to the smallest working surface.
- Preserve Arca's local-first model without adding cloud SDKs, account-backed
  sync, pairing, push unlock, or remote vault access.
- Make every boundary testable through `ClientKind`, `Capability`, fixtures, and
  release checks.

## Assets And Trust Boundaries

The protected assets are:

- the master password and any equivalent user-entered unlock secret;
- derived keys, vault session keys, decrypted entries, revealed fields, and
  generated passwords;
- KDBX vault files, export artifacts, backup artifacts, and their metadata;
- autofill candidate lists that can reveal account ownership or service usage;
- local vault handles, user preferences, biometric policy state, and other
  auxiliary metadata.

The mobile trust boundaries are:

- Rust `vault-api` and `vault-core`, which own vault semantics, KDBX parsing,
  KDFs, password generation, and capability enforcement;
- the UniFFI facade, which exposes only audited DTOs and explicit secret-bearing
  operations;
- native app UI processes, which may display or copy short-lived plaintext only
  after explicit user intent;
- iOS Keychain, Secure Enclave, LocalAuthentication, and app lifecycle APIs;
- Android Keystore, BiometricPrompt, app-scoped storage, lifecycle callbacks, and
  autofill APIs;
- iOS Credential Provider extensions and Android Autofill Services, which run as
  restricted clients with smaller surfaces than the full mobile apps;
- OS document providers and file pickers, which expose local files but must not
  become direct cloud SDK integrations in Arca.

## Proposed Decision

Adopt a mobile security model with four client kinds:

- `IosApp`
- `AndroidApp`
- `IosAutofillExtension`
- `AndroidAutofillService`

Full mobile apps may use the mobile app capability set accepted in ADR-0012:
all initial capabilities except `ExportPlaintext`. They may create, unlock,
read, reveal, copy, mutate, export KDBX, read history, change KDF settings, and
perform permanent deletion only through `vault-api` checks for their client
kind. They must not add plaintext export without a later ADR and human review.

Mobile autofill clients may use only `Unlock`, `ReadMeta`, and `CopySecret`.
They must deny reveal, mutate, create vault, change KDF, plaintext export, KDBX
export, history, and permanent delete operations even if the host app supports
those operations.

All mobile bridge calls must identify their `ClientKind` before dispatch. The
native iOS app, native Android app, iOS autofill extension, and Android Autofill
Service must be unable to bypass capability policy by calling lower-level Rust
APIs directly.

The mobile implementation must not introduce custom cryptography, mobile-only
KDBX parsing, mobile-only KDF behavior, or mobile-only vault mutation semantics.
Those remain owned by `vault-api` and `vault-core`.

## Secure Storage Expectations

### iOS

Use iOS Keychain, Secure Enclave, LocalAuthentication, app-scoped storage, and
security-scoped document access only for their appropriate platform roles.

Keychain or Secure Enclave backed storage may hold platform-bound auxiliary
material such as:

- recent vault handles or security-scoped bookmark data;
- user preferences and biometric policy state;
- non-secret metadata needed to restore local UI state;
- a future session re-entry token only if a later implementation design proves
  it does not persist the master password, vault key, or decrypted vault
  contents.

It must not store:

- the master password;
- KDF output or vault keys;
- unlocked vault contents;
- plaintext entry fields or generated passwords;
- a complete vault metadata cache that would be useful after device compromise.

iOS vault files may be opened through the document picker, security-scoped
bookmarks, app-scoped storage, or OS-presented local file providers. Arca must
not embed direct cloud provider SDKs for the first mobile MVP.

### Android

Use Android Keystore, BiometricPrompt, app-scoped storage, encrypted preferences
or DataStore, and the Storage Access Framework only for their appropriate
platform roles.

Android secure storage may hold platform-bound auxiliary material such as:

- persisted document URIs or recent vault handles;
- user preferences and biometric policy state;
- non-secret metadata needed to restore local UI state;
- a future session re-entry token only if a later implementation design proves
  it does not persist the master password, vault key, or decrypted vault
  contents.

It must not store:

- the master password;
- KDF output or vault keys;
- unlocked vault contents;
- plaintext entry fields or generated passwords;
- a complete vault metadata cache that would be useful after device compromise.

Android vault files may be opened through the Storage Access Framework,
app-scoped storage, or OS-presented local file providers. Arca must not embed
direct cloud provider SDKs for the first mobile MVP.

## Biometric Unlock Policy

Biometric authentication may be offered only as a local OS authentication gate
around auxiliary material or a short-lived session re-entry path. It is not an
Arca credential, not a replacement for the master password, and not a reason to
persist vault keys or decrypted vault contents.

The first unlock after app install, vault addition, device reboot, biometric
policy change, biometric enrollment change, secure storage reset, or session
expiry must require the master password.

Biometric fallback must fail closed. If biometric authentication is unavailable,
locked out, reset, changed, or refused by the user, the app must return to the
master password flow. It must not silently downgrade to a weaker persisted
secret, background unlock, or remote unlock.

Mobile implementations may offer a user preference for biometric session
re-entry only after the master password has unlocked the vault in the current
trusted lifecycle. The implementation must document the timeout, fallback, and
reset behavior before release.

## Lifecycle And Auto-Lock

Mobile clients must protect plaintext across lifecycle transitions:

- obscure secret-bearing UI before entering app switcher previews;
- clear or hide revealed fields when moving to the background;
- drop short-lived plaintext on device lock, session timeout, memory pressure,
  extension teardown, and explicit lock;
- pause or block secret reveal during detected screen recording where the
  platform exposes a reliable signal;
- prevent screenshots on Android secret-bearing views where platform policy
  allows;
- present a privacy cover for iOS app switcher snapshots and rely on platform
  review for screenshot limitations;
- re-check session state before returning from background to a secret-bearing
  view.

The secure default is to lock or obscure immediately when the app leaves the
foreground. User-configurable grace periods may be added later, but they must not
extend secret lifetime in autofill extensions and must be covered by tests or
manual release checks.

## Plaintext Lifetime Rules

Plaintext secrets may enter mobile UI memory only for an explicit user action
such as reveal, copy, edit, generate, or autofill confirmation. Plaintext must be
scoped to the smallest view, callback, bridge response, or clipboard operation
that can complete the requested action.

The mobile app and extensions must not put plaintext secrets in:

- logs;
- analytics events;
- crash reports;
- notifications;
- app badges;
- deep links;
- Android intents;
- iOS user activities;
- accessibility labels for hidden fields;
- debug descriptions;
- screenshots used for bug reports or store submissions.

Clipboard writes must require explicit user action and should be cleared after a
bounded interval where the platform allows. The app must not promise to erase OS
clipboard history or third-party keyboard and clipboard-manager copies that it
cannot control.

Notifications may mention non-secret workflow state only, such as a vault being
locked or an autofill request needing attention. They must not contain vault
names, entry titles, usernames, passwords, one-time codes, notes, URLs, or
custom fields unless a later privacy review explicitly permits a narrower subset.

## Autofill Boundaries

iOS Credential Provider extensions and Android Autofill Services are separate
restricted clients. They must request only the smallest candidate set needed for
an explicit autofill interaction and must not retain decrypted vault contents
beyond the platform extension or service lifecycle.

Autofill must require user confirmation before copying or filling a secret into
another app or website. Autofill UI should bind candidates to the platform
provided app or web origin where available and should avoid broad search results
that make account ownership visible outside the confirmation flow.

Autofill clients must not:

- parse KDBX directly;
- run independent KDF or unlock semantics outside `vault-api`;
- persist unlocked vault state beyond the extension or service lifecycle;
- reveal plaintext in a general-purpose detail view;
- mutate entries;
- create vaults;
- change KDF settings;
- export plaintext or KDBX files;
- read history;
- permanently delete entries.

Host-app-mediated unlock is preferred for the first implementation. A direct
extension unlock path requires a separate implementation review proving the
extension still uses restricted `ClientKind` policy and does not persist
unlocked vault contents.

## Threat Model

### Stolen Or Shared Device

An attacker with physical access to an unlocked or recently used device might
inspect app switcher snapshots, notifications, clipboard contents, document
provider recents, or cached UI state. Arca mitigates this by obscuring or
locking on background, keeping notifications non-secret, bounding clipboard
lifetime where possible, and keeping platform storage free of master passwords,
vault keys, and decrypted vault contents.

### Biometric Spoofing Or Enrollment Change

An attacker might attempt biometric spoofing, add a biometric factor, trigger
lockout fallback, or rely on a shared device biometric. Arca mitigates this by
treating biometrics as optional local gates, requiring master password fallback
after enrollment or policy changes, and failing closed when the OS biometric
state is unavailable or invalidated.

### Platform Backup And Secure Storage Extraction

An attacker might restore backups, extract preference files, inspect document
provider metadata, or compromise platform secure storage. Arca mitigates this by
storing only auxiliary material in platform storage and keeping vault files
encrypted as KDBX data owned by the user.

### Clipboard And Keyboard Observation

An attacker might use OS clipboard history, a keyboard, accessibility services,
or another app to observe copied or typed secrets. Arca mitigates this by making
clipboard writes explicit, clearing them where possible, not promising control
over third-party history, and favoring platform autofill confirmation over
manual copy when appropriate.

### Autofill Phishing And Origin Confusion

A malicious app or website might impersonate another service, request autofill
for a lookalike origin, or infer stored accounts from candidate lists. Arca
mitigates this by treating autofill as a restricted client, binding candidates to
platform-provided app or web origins where available, minimizing candidate
disclosure, and requiring user confirmation.

### FFI, Bridge, And Logging Misuse

A mobile UI, extension, or bridge bug might call a privileged Rust operation,
retain plaintext longer than intended, or leak secrets through diagnostics. Arca
mitigates this by enforcing `ClientKind` and `Capability` in `vault-api`, keeping
secret-bearing methods explicit, denying lower-level bypasses, and excluding
secrets from logs, analytics, crash reports, notifications, intents, and debug
descriptions.

### Local File Race Or Conflict

Cloud-backed document providers and external editors may change a local KDBX
file while mobile has it open. Arca mitigates this by remaining local-first,
using OS-presented files only, and allowing the first mobile implementation to
fail closed, require reopen, or use a tested single-writer handoff until a sync
ADR defines conflict resolution.

## Compliance

- [ ] Add mobile binding or adapter tests proving every call identifies
  `IosApp`, `AndroidApp`, `IosAutofillExtension`, or `AndroidAutofillService`.
- [ ] Add capability denial tests proving full mobile apps deny
  `ExportPlaintext`.
- [ ] Add capability denial tests proving autofill clients deny reveal, mutate,
  create vault, change KDF, export, history, and permanent-delete operations.
- [ ] Add iOS release checks for Keychain use, LocalAuthentication fallback,
  app switcher privacy cover, background lock, clipboard clearing, notification
  privacy, document picker access, and Credential Provider lifecycle.
- [ ] Add Android release checks for Keystore use, BiometricPrompt fallback,
  app switcher privacy, background lock, screenshot protection where supported,
  clipboard clearing, notification privacy, Storage Access Framework access, and
  Autofill Service lifecycle.
- [ ] Add no-secret diagnostics checks for mobile logs, crash reports,
  analytics, notifications, intents, deep links, accessibility labels, and
  debug descriptions.
- [ ] Add the mobile security model, platform threat model, and release checks
  to the #215 mobile MVP checklist before implementation starts.

## Revisit / Out Of Scope

- Creating `arca-mobile`.
- Implementing iOS or Android apps.
- Implementing iOS Credential Provider or Android Autofill Service code.
- Cross-device sync, accounts, shared vaults, device pairing, push unlock, or
  remote unlock.
- Plaintext export from mobile clients.
- Direct cloud provider SDK integrations.
- React Native, Flutter, or Tauri mobile as the primary strategy.
- Custom cryptography, custom KDF behavior, or mobile-only KDBX semantics.

## References

- #138
- #207
- #210
- #214
- #215
- #216
- ADR-0003: Client capability matrix
- ADR-0004: Secret lifetime
- ADR-0008: Mobile UniFFI and native UI
- ADR-0009: Local-first access constraints before sync or new clients
- ADR-0012: Mobile UniFFI repository bootstrap
- Apple Keychain Services documentation:
  https://developer.apple.com/documentation/security/keychain-services
- Apple LocalAuthentication documentation:
  https://developer.apple.com/documentation/localauthentication
- Apple AuthenticationServices credential provider extension documentation:
  https://developer.apple.com/documentation/authenticationservices/ascredentialproviderextension
- Android Keystore documentation:
  https://developer.android.com/privacy-and-security/keystore
- Android biometric authentication documentation:
  https://developer.android.com/identity/sign-in/biometric-auth
- Android Autofill framework documentation:
  https://developer.android.com/identity/autofill
- Android Storage Access Framework documentation:
  https://developer.android.com/guide/topics/providers/document-provider
