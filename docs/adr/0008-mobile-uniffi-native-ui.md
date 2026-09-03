---
status: accepted
date: 2026-09-03
decision-makers: ["akei9"]
related: ["#214", "#216", "#223"]
---

# ADR-0008: Mobile UniFFI and native UI

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.
- 2026-09-03 - accepted by maintainer approval.

## Context

Mobile password managers are primarily judged by secure unlock, search, copy,
and autofill. Autofill is platform-native and has stricter lifecycle constraints
than the main app.

## Decision Drivers

- Reuse Rust vault semantics without rewriting crypto.
- Keep plaintext out of unnecessary JS or Dart heaps.
- Treat mobile autofill extensions as restricted clients.
- Ship the smallest useful mobile path first.

## Considered Options

- `vault-api` plus UniFFI, native iOS first, Android next.
- Flutter plus Rust bridge.
- React Native plus Rust bridge.
- Tauri mobile.

## Proposed Decision

Prefer `vault-api` plus UniFFI, with native iOS SwiftUI first and Android Jetpack
Compose after that. Autofill extensions are separate restricted client kinds.

React Native, Flutter, or Tauri mobile require a separate ADR if chosen as the
primary strategy.

## Consequences

### Positive

- Keeps the security-critical contract in Rust.
- Fits native autofill constraints.
- Gives iOS a natural first target after macOS.

### Negative

- Requires native UI work instead of reusing desktop Svelte UI.
- Android follows after iOS unless the roadmap changes.

### Neutral

- Flutter may remain a fallback if one maintainer must ship both platforms
  quickly, but native autofill still needs native code.

## Compliance

- [ ] Add a UniFFI feasibility spike before mobile implementation.
- [ ] Add mobile capability denial tests.
- [ ] Add secure storage and autofill lifecycle requirements to the mobile MVP
  checklist.

## Revisit / Out Of Scope

- Cross-device sync and accounts are out of scope.

## References

- #214
- #216
