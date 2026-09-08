---
status: accepted
date: 2026-09-08
decision-makers: ["akei9"]
related: ["#138", "#207", "#208", "#209", "#210", "#214", "#215", "#216"]
---

# ADR-0012: Mobile UniFFI repository bootstrap

## Status History

- 2026-09-08 - proposed by Codex for maintainer review.
- 2026-09-08 - accepted by maintainer approval.

## Context

ADR-0008 accepts `vault-api` plus UniFFI, native iOS SwiftUI first, and Android
Jetpack Compose after that. ADR-0009 keeps the first mobile clients local-first
through OS file surfaces, app-scoped storage, secure storage, and restricted
autofill extensions. ADR-0010 defers creating `arca-mobile` until mobile
implementation starts, unless a later accepted ADR chooses separate native
repositories.

This ADR turns that accepted direction into the mobile implementation and
repository bootstrap plan needed before creating `arca-mobile`.

## Decision Drivers

- Reuse Rust vault semantics without rewriting crypto, KDBX parsing, password
  generation, audit rules, or secret lifetime behavior.
- Keep full mobile apps more capable than autofill extensions, but still
  narrower than desktop for plaintext export.
- Fit platform-native secure storage, biometrics, app backgrounding,
  screenshots, clipboard, and autofill lifecycles.
- Confirm whether one `arca-mobile` repository is sufficient before bootstrap.
- Make mobile binding and fixture compatibility testable from the first commit.

## Considered Options

- Bootstrap one `arca-mobile` repository for iOS and Android.
- Split immediately into `arca-ios` and `arca-android`.
- Keep mobile implementation in `akei9/arca`.
- Use React Native, Flutter, or Tauri mobile as the primary UI layer.
- Use native SwiftUI and Jetpack Compose with Rust `vault-api` bindings.

## Proposed Decision

Keep all #216 planning work in `akei9/arca`. Create one `arca-mobile`
repository when mobile implementation starts. That repository should contain the
native iOS app, native Android app, mobile binding integration, mobile-specific
fixtures, platform CI, app-store documentation, and mobile release workflow.

Do not split into `arca-ios` and `arca-android` for the first mobile MVP. A
separate native-repo split requires a later accepted ADR showing that one
`arca-mobile` repository blocks release, security review, ownership, CI, or
store operations.

The implementation order is:

1. UniFFI feasibility spike against `vault-api`.
2. iOS SwiftUI app with local vault open, unlock, search, view, copy, edit, and
   KDBX export support, but no plaintext export.
3. iOS autofill extension as a restricted `IosAutofillExtension` client.
4. Android Jetpack Compose app with equivalent full-app behavior.
5. Android Autofill Service as a restricted `AndroidAutofillService` client.

React Native, Flutter, or Tauri mobile remain out of scope for the primary
strategy unless a later ADR explains the security, memory, binding, autofill,
and store-review tradeoffs.

## Binding Architecture

`vault-api` remains the public machine contract. Mobile bindings should expose a
small FFI facade around `vault-api` operations and DTOs rather than exposing
`vault-core` internals directly.

The UniFFI layer should:

- identify every call as `IosApp`, `AndroidApp`, `IosAutofillExtension`, or
  `AndroidAutofillService`;
- enforce `ClientKind` and `Capability` before dispatching operations;
- preserve typed errors and contract versions across Swift and Kotlin;
- keep secret-bearing methods explicit and easy to audit;
- avoid implementing mobile-only vault semantics outside Rust;
- regenerate bindings from checked-in source definitions or generated artifacts
  whose drift is verified by CI.

The full iOS and Android apps use the mobile app capability set: all initial
capabilities except `ExportPlaintext`. Mobile autofill extensions use only
`Unlock`, `ReadMeta`, and `CopySecret`.

## Platform Boundaries

Mobile apps may use OS document pickers, security-scoped or app-scoped local
file access, and cloud file providers only as OS-presented local files. Direct
cloud-provider SDKs, account-backed Arca services, remote sync, pairing, push
unlock, and conflict resolution are out of scope for the first MVP.

Secure storage is for platform-bound auxiliary material such as recent local
vault handles, user preferences, biometric policy state, and non-secret
metadata. It must not become a replacement vault store and must not persist the
master password or unlocked vault contents.

Biometric unlock can be used only as a platform gate around locally protected
auxiliary material or session re-entry. The master password flow, key material
lifetime, fallback behavior, and biometric reset behavior need tests before any
biometric unlock feature ships.

The app must protect plaintext during mobile lifecycle transitions. On
backgrounding, screen recording, screenshot surfaces, app switcher previews,
lock events, and OS memory pressure, the UI should obscure secret views and drop
short-lived plaintext where platform APIs allow it.

Clipboard writes must be explicit, bounded, and clearable where the platform
allows. Clipboard behavior belongs to the mobile security model and must be
covered by tests or manual release checks before release.

## Autofill Constraints

Mobile autofill extensions are separate restricted clients. They must request
only the smallest candidate set needed for an explicit autofill interaction and
must not retain decrypted vault contents beyond the platform extension
lifecycle.

Autofill extensions should prefer host-app-mediated unlock or a tested
platform-approved extension unlock path. They must not add independent KDBX
parsing, KDF execution, vault persistence, plaintext export, entry mutation, or
history access.

The mobile app may maintain user preferences that influence autofill behavior,
but the extension must still pass `vault-api` capability checks as its own
client kind.

## Repository Bootstrap Checklist

Create `arca-mobile` when implementation starts and all of the following are
true:

- [ ] ADR-0008, ADR-0009, ADR-0010, and this ADR are accepted.
- [ ] A UniFFI feasibility spike proves Swift and Kotlin can call the selected
  `vault-api` facade and receive typed DTOs/errors.
- [ ] Binding generation and fixture drift checks are defined in `akei9/arca`.
- [ ] Capability denial tests prove mobile apps deny plaintext export and
  autofill clients deny reveal, mutate, export, history, KDF, create-vault, and
  permanent-delete operations.
- [ ] iOS local file access, secure storage, biometric, clipboard,
  screenshot/backgrounding, and autofill requirements are documented.
- [ ] Android local file access, secure storage, biometric, clipboard,
  screenshot/backgrounding, and autofill requirements are documented.
- [ ] The first repository layout is documented, including ownership of shared
  mobile bindings, `ios/`, `android/`, test fixtures, and release notes.
- [ ] CI covers iOS build/test, Android build/test, Rust binding tests,
  dependency scanning, secret scanning, and contract fixture drift.
- [ ] Security issue routing points shared contract, vault, binding,
  secret-lifetime, and capability issues back to `akei9/arca`.

## Consequences

### Positive

- Keeps mobile vault behavior aligned with the canonical Rust contract.
- Avoids committing to a repo split before there is implementation pressure.
- Gives iOS and Android a shared mobile release and security-review home.
- Treats autofill extensions as restricted clients from the beginning.

### Negative

- Native UI work cannot reuse the desktop Svelte implementation directly.
- Android follows iOS unless the roadmap changes.
- UniFFI setup and mobile CI must be solved before feature work accelerates.

### Neutral

- A later ADR may still split iOS and Android repositories if one mobile repo
  becomes operationally harmful.

## Compliance

- [ ] Complete the UniFFI feasibility spike before creating `arca-mobile`.
- [ ] Add mobile generated binding drift tests.
- [ ] Add capability denial tests for iOS, Android, and autofill clients.
- [ ] Add secure storage, biometric, clipboard, screenshot/backgrounding, and
  autofill lifecycle requirements to the mobile MVP checklist.
- [ ] Create `arca-mobile` only when implementation starts.

## Revisit / Out Of Scope

- Creating `arca-mobile` in this PR.
- Splitting into `arca-ios` and `arca-android`.
- React Native, Flutter, or Tauri mobile as the primary strategy.
- Sync, accounts, pairing, push unlock, or shared vaults.
- Plaintext export from mobile apps.

## References

- #138
- #207
- #208
- #209
- #210
- #214
- #215
- #216
- UniFFI user guide:
  https://mozilla.github.io/uniffi-rs/latest/
- Apple AuthenticationServices credential provider extension documentation:
  https://developer.apple.com/documentation/authenticationservices/ascredentialproviderextension
- Apple LocalAuthentication documentation:
  https://developer.apple.com/documentation/localauthentication
- Android Autofill framework documentation:
  https://developer.android.com/identity/autofill
- Android biometric authentication documentation:
  https://developer.android.com/identity/sign-in/biometric-auth
