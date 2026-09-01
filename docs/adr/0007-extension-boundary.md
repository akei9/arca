---
status: proposed
date: 2026-09-01
decision-makers: ["akei9"]
related: ["#211", "#213", "#223"]
---

# ADR-0007: Browser extension boundary

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.

## Context

A browser extension can make Arca much more useful, but it runs in a hostile
environment: pages, frames, scripts, origins, and browser storage all change the
threat model.

## Decision Drivers

- Do not run the full unlock path in a content script or service worker.
- Keep browser permissions narrow and explainable.
- Prefer explicit user confirmation for secret injection.
- Avoid extension storage of unlocked vault material.

## Considered Options

- Native messaging to the desktop app or a small native host.
- WASM build of the core as the primary unlock path.
- Browser extension as copy-only UI.

## Proposed Decision

Prefer native messaging to the desktop app or a small native host for the first
extension boundary. The extension is a UI and autofill client with narrow
capabilities. WASM may be considered later only for constrained already-unlocked
working-set operations.

## Consequences

### Positive

- Keeps KDF, KDBX parsing, and long-lived unlocked material out of the browser.
- Aligns the extension with least privilege.

### Negative

- Requires native host installation or desktop app presence.
- Store review needs a clear permissions explanation.

### Neutral

- Browser support depends on native messaging compatibility.

## Compliance

- [ ] Add extension capability denial tests.
- [ ] Add a native messaging protocol contract before implementation.
- [ ] Add permission justification to the extension release checklist.

## Revisit / Out Of Scope

- A full WASM unlock path is out of scope unless a later ADR accepts the memory,
  lifecycle, and storage risks.

## References

- #211
- #213
