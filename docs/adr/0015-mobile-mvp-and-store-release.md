---
status: accepted
date: 2026-09-09
decision-makers: ["akei9"]
related: ["#138", "#207", "#210", "#214", "#215", "#216"]
---

# ADR-0015: Mobile MVP scope and store release checklist

## Status History

- 2026-09-09 - proposed by Codex for maintainer review.
- 2026-09-10 - accepted by maintainer approval.

## Context

ADR-0008 accepts native iOS and Android apps using `vault-api` through UniFFI.
ADR-0009 keeps early mobile clients local-first and offline-first. ADR-0012
defers creating `arca-mobile` until implementation starts and defines the
repository bootstrap plan. ADR-0014 defines the mobile security model for
storage, biometrics, lifecycle handling, plaintext lifetime, and autofill.

This ADR defines the smallest useful mobile release and the release-readiness
checklist for App Store and Google Play submission. It does not create
`arca-mobile` and does not implement mobile app code.

## Decision Drivers

- Ship a useful local-first password manager before adding broader mobile
  platform surface area.
- Keep the first release small enough to test on real devices.
- Avoid App Store and Google Play metadata promises for features that are not in
  the shipped build.
- Keep autofill, biometrics, import/export, and backup behavior aligned with the
  accepted mobile security model.
- Make future implementation issues clear before creating `arca-mobile`.

## Considered Options

- Ship the first mobile MVP as a full app only, with autofill deferred.
- Include iOS Credential Provider and Android Autofill Service in the first
  mobile MVP.
- Ship iOS first, then Android after the shared binding path is proven.
- Require simultaneous iOS and Android store releases.
- Include plaintext export, cloud sync, accounts, and direct cloud-provider SDKs
  in the first mobile MVP.

## Proposed Decision

The first mobile release is a full-app MVP. It should ship iOS first, then
Android with equivalent app behavior after the shared Rust binding path is
stable. Store submission may happen per platform when that platform satisfies
the checklist in this ADR; simultaneous iOS and Android release is not required.

The first mobile MVP excludes mobile autofill. iOS Credential Provider and
Android Autofill Service work should start after the full-app MVP proves local
vault access, unlock, search, view, copy, generate, edit, lifecycle protection,
store metadata, and release operations. Autofill remains a high-priority follow
up and must use the restricted client model from ADR-0014.

The first mobile MVP must not include plaintext export, Arca-hosted sync,
accounts, device pairing, push unlock, shared vaults, direct cloud-provider SDKs,
or custom mobile vault semantics.

## MVP User Flows

### Vault Setup

The first mobile MVP must support:

- create a new local KDBX vault;
- open an existing local KDBX vault through OS file access;
- reopen a recent vault handle where the platform allows;
- save changes back to the selected local vault path or document URI;
- show a clear error when a vault cannot be accessed, parsed, unlocked, or
  saved.

The MVP may support app-scoped copies of user-selected vault files if the
platform implementation needs that for reliable writes, but it must describe the
copy/overwrite behavior in release notes and avoid presenting it as cloud sync.

### Unlock And Lock

The first mobile MVP must support:

- unlock with the master password;
- explicit lock;
- auto-lock or privacy-cover behavior on app backgrounding;
- session re-check after returning from background;
- fail-closed behavior for invalid password, corrupted vault, permission loss,
  and external file changes.

Biometric session re-entry is optional for the first mobile MVP. If included, it
must satisfy ADR-0014 before release. If not included, store metadata and
screenshots must not imply biometric unlock.

### Search And Browse

The first mobile MVP must support:

- list non-secret entry metadata after unlock;
- search by non-secret metadata supported by `vault-api`;
- view entry metadata without revealing secrets by default;
- preserve secret redaction in logs, debug output, screenshots used for review,
  and fixture output.

### Secret Actions

The first mobile MVP must support:

- reveal a password only after explicit user action;
- copy a password only after explicit user action;
- generate a password through the Rust password generator;
- clear or overwrite copied secrets after a bounded interval where the platform
  allows;
- hide or clear revealed plaintext on lock, background, timeout, and explicit
  navigation away from the secret view.

The first mobile MVP may defer one-time password display, custom fields,
attachments, passkeys, and advanced audit features.

### Entry Mutation

The first mobile MVP must support basic entry creation and editing:

- title;
- username;
- password;
- URL;
- notes;
- tags, if already supported cleanly by the shared API;
- delete-to-trash or equivalent non-permanent deletion if supported by the
  shared API.

Permanent delete may ship only if the implementation provides an explicit
confirmation flow, capability checks, and tests. It may be deferred without
blocking the MVP.

### Backup, Import, And Export

The first mobile MVP must support KDBX save behavior for local-first use. It
should support explicit KDBX export or duplicate-save to an OS-selected
destination if the platform file flow can do so without weakening ADR-0014.

The first mobile MVP must not support plaintext export. Import from other
password managers, CSV import, direct cloud-provider import, and automated
backup are out of scope.

## Explicitly Out Of Scope For First Release

- iOS Credential Provider extension.
- Android Autofill Service.
- Plaintext export.
- CSV import or import from other password managers.
- Arca-hosted sync, accounts, shared vaults, device pairing, or push unlock.
- Direct Dropbox, iCloud Drive, Google Drive, OneDrive, or other cloud-provider
  SDK integrations.
- Browser extension handoff.
- Passkeys.
- Attachments.
- Team, organization, or role management.
- Remote crash-log or analytics collection that can include vault metadata or
  plaintext.
- Watch, tablet-only, desktop-class, or browser-extension companion releases.

## Implementation Issue Breakdown

Create implementation issues from this ADR before creating `arca-mobile` or as
the first issues in `arca-mobile` when implementation starts:

- Mobile binding spike: prove Swift and Kotlin can call the selected UniFFI
  facade, receive typed DTOs/errors, and run fixture drift checks.
- Mobile repository bootstrap: create `arca-mobile` with shared Rust binding
  integration, `ios/`, `android/`, fixtures, CI, security issue routing, and
  release documentation.
- iOS local vault files: document picker, security-scoped bookmarks where
  needed, app-scoped fallback, save failure handling, and external-change
  behavior.
- iOS full-app MVP: create/open/unlock/search/view/reveal/copy/generate/edit
  flows with ADR-0014 lifecycle protections.
- iOS release readiness: TestFlight build, device smoke matrix, app privacy
  answers, screenshots with synthetic data, review notes, support URL, privacy
  policy URL, and release notes.
- Android local vault files: Storage Access Framework, persisted document
  permissions where needed, app-scoped fallback, save failure handling, and
  external-change behavior.
- Android full-app MVP: create/open/unlock/search/view/reveal/copy/generate/edit
  flows with ADR-0014 lifecycle protections.
- Android release readiness: internal testing build, device smoke matrix, Data
  safety answers, screenshots with synthetic data, reviewer access notes,
  privacy policy URL, and release notes.
- Mobile biometric session re-entry: optional post-MVP issue unless pulled into
  the first release with ADR-0014 tests.
- Mobile autofill milestone: separate iOS Credential Provider and Android
  Autofill Service issues using the restricted client model.

## Smoke Test Matrix

Every store-bound mobile release candidate must pass these smoke tests on real
devices or representative hosted devices before submission:

- create a vault, close the app, reopen, unlock, and verify the created entry is
  still present;
- open an existing KDBX fixture, unlock it, search for an entry, view metadata,
  reveal once, copy once, lock, and verify secrets are hidden;
- generate a password and save it into a new entry;
- edit title, username, URL, notes, and password for an existing entry;
- handle invalid password without exposing internal errors or secret material;
- handle missing file permission, moved file, corrupted file, and save failure;
- background the app from secret-bearing views and verify the app switcher does
  not expose plaintext;
- lock the device from a secret-bearing view and verify return-to-app behavior;
- trigger memory pressure or process recreation where tooling allows and verify
  locked or fail-closed recovery;
- verify clipboard clear behavior where the platform allows and document any OS
  limitation;
- verify logs, crash reports, screenshots, and test artifacts do not contain
  plaintext secrets or real user data;
- verify store screenshots and review fixtures use synthetic data only.

Minimum device coverage before first public release:

- one current iPhone model on the current stable iOS version;
- one older supported iPhone model on the oldest supported iOS version;
- one current iPad or iPad simulator if iPad is listed as supported;
- one current Android Pixel or equivalent reference device on the current stable
  Android version;
- one older supported Android version;
- one Android device with an OEM skin or manufacturer behavior that commonly
  differs from Pixel devices;
- one small-screen phone and one large-screen phone across the matrix.

## Store Release Checklist

### Shared Store Requirements

- [ ] App name, subtitle, short description, long description, keywords, and
  category accurately describe only shipped MVP behavior.
- [ ] Store copy says local-first and does not promise sync, accounts, passkeys,
  autofill, plaintext export, cloud backup, or cloud provider integrations.
- [ ] Screenshots and videos show the app in use with synthetic vault data only.
- [ ] No screenshot, preview, or review fixture contains real vault names,
  usernames, URLs, notes, passwords, one-time codes, or personal data.
- [ ] Privacy policy is publicly reachable and names Arca, the developer, data
  handling, retention, deletion, support contact, and local-first storage
  behavior.
- [ ] Support URL or contact path is publicly reachable.
- [ ] Security contact and vulnerability reporting route are documented.
- [ ] Release notes explain backup responsibility, local file behavior, and any
  platform clipboard limitations.
- [ ] Reviewer notes include a synthetic test vault, master password, and exact
  steps for create/open/unlock/search/view/copy/generate/edit flows.
- [ ] Export/import wording is accurate: KDBX local save or duplicate-save only,
  no plaintext export.
- [ ] Dependency, secret scanning, and license review have run for mobile code
  and generated bindings.
- [ ] ADR-0014 lifecycle, storage, clipboard, diagnostics, and no-secret checks
  pass for the submitted build.

### App Store Checklist

- [ ] Build is submitted through App Store Connect with complete metadata and no
  placeholder text.
- [ ] App Review notes describe all security-sensitive behavior and any feature
  that requires a supplied test vault.
- [ ] TestFlight has been used for beta testing before App Store submission.
- [ ] App privacy answers are accurate for the shipped app and third-party SDKs.
- [ ] Privacy Nutrition Label reflects whether any data leaves the device; local
  on-device vault processing is not treated as collected data unless transmitted
  off device.
- [ ] Screenshots show real app flows, not only splash screens or marketing art.
- [ ] Screenshots, previews, icons, and metadata avoid third-party trademarks
  unless rights are documented.
- [ ] Age rating answers are accurate for a password manager with user-provided
  vault content.
- [ ] iPad support is either tested and represented accurately or excluded where
  allowed by the chosen target.
- [ ] No in-app purchase, subscription, account, or external purchase messaging
  appears unless implemented and reviewed separately.

### Google Play Checklist

- [ ] Play Console app content declarations are complete before review.
- [ ] Data safety form is accurate for the shipped app and every third-party SDK.
- [ ] Privacy policy is linked in Play Console and inside the app.
- [ ] Store listing text and graphics accurately describe only shipped MVP
  behavior.
- [ ] Reviewer instructions provide any needed test vault and steps for
  restricted flows.
- [ ] Target audience, content rating, ads declaration, app access, and
  sensitive permission declarations are complete.
- [ ] Android App Bundle signing and release tracks are configured.
- [ ] Internal testing or closed testing has exercised the smoke matrix before
  production submission.
- [ ] Any permission or sensitive API that requires prominent disclosure has an
  in-app disclosure before the system prompt.
- [ ] Data safety and privacy policy remain consistent after every dependency,
  SDK, analytics, crash reporting, or networking change.

## Consequences

### Positive

- Gives mobile implementation a clear first release target.
- Keeps store metadata honest and limited to shipped behavior.
- Defers autofill until the full app and security lifecycle are proven.
- Turns mobile release readiness into checkable work instead of tribal memory.

### Negative

- The first mobile release will not provide native autofill.
- Users must rely on manual reveal/copy flows until the autofill milestone.
- iOS and Android release timing may diverge.

### Neutral

- Biometric session re-entry can be included in the first MVP only if it
  satisfies ADR-0014; otherwise it remains a follow-up.
- Android may reuse lessons from iOS rather than launching at the same time.

## Compliance

- [ ] Create the implementation issues listed in this ADR before mobile
  implementation starts or as the first issues in `arca-mobile`.
- [ ] Add the shared, App Store, and Google Play checklists to mobile release
  documentation.
- [ ] Verify ADR-0014 checks before any store-bound mobile release candidate.
- [ ] Keep store copy, screenshots, privacy labels, Data safety, and reviewer
  notes aligned with shipped behavior on every release.

## Revisit / Out Of Scope

- Creating `arca-mobile`.
- Implementing mobile app code.
- Accepting autofill into the first mobile MVP.
- Accepting plaintext export on mobile.
- Designing sync, accounts, sharing, or cloud backup.
- Legal review of privacy policy text.
- Final store metadata copy and screenshots.

## References

- #138
- #207
- #210
- #214
- #215
- #216
- ADR-0008: Mobile UniFFI and native UI
- ADR-0009: Local-first access constraints before sync or new clients
- ADR-0012: Mobile UniFFI repository bootstrap
- ADR-0014: Mobile security model for storage, biometrics, and autofill
- Apple App Review Guidelines:
  https://developer.apple.com/app-store/review/guidelines/
- Apple App privacy details:
  https://developer.apple.com/app-store/app-privacy-details/
- Google Play User Data policy:
  https://support.google.com/googleplay/android-developer/answer/10144311
- Google Play Data safety section guidance:
  https://support.google.com/googleplay/android-developer/answer/10787469
- Google Play app review preparation:
  https://support.google.com/googleplay/android-developer/answer/9859455
- Google Play prominent disclosure and consent guidance:
  https://support.google.com/googleplay/android-developer/answer/11150561
