---
status: proposed
date: 2026-09-01
decision-makers: ["akei9"]
related: ["#207", "#223", "#226"]
---

# ADR-0003: Client capability matrix

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.

## Context

Desktop, browser extension, mobile app, autofill extensions, and future sync
components should not all have the same powers. The policy needs to be enforced
in code, not only described in Markdown.

## Decision Drivers

- Least privilege by client kind.
- Explicit secret exposure.
- Prevent privilege expansion through reused DTOs.
- Keep future sync ciphertext-only.

## Considered Options

- Encode `ClientKind` and `Capability` in the public API layer.
- Rely on UI-level checks per client.
- Rely on documentation and review only.

## Proposed Decision

Model client privileges in Rust using `ClientKind` and `Capability`.

Initial client kinds:

- `DesktopApp`
- `BrowserExtension`
- `IosApp`
- `AndroidApp`
- `IosAutofillExtension`
- `AndroidAutofillService`
- `FutureSyncServer`

Initial capabilities:

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

## Consequences

### Positive

- Makes privilege policy reviewable and testable.
- Gives browser and autofill clients a deliberately narrow surface.

### Negative

- Adds checks that every binding adapter must respect.

### Neutral

- The exact grants may change before acceptance.

## Compliance

- [ ] Add capability denial tests.
- [ ] Add tests proving restricted clients cannot call privileged operations.
- [ ] Document the capability matrix in the public API docs.

## Revisit / Out Of Scope

- User roles, teams, and account permissions are out of scope.

## References

- #226
