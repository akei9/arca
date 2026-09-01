---
status: proposed
date: 2026-09-01
decision-makers: ["akei9"]
related: ["#207", "#223", "#224", "#226"]
---

# ADR-0004: Secret lifetime

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.

## Context

Arca handles master passwords, entry passwords, generated passwords, revision
passwords, and copied secrets. Future clients need one rule set for how
plaintext can move through the system.

## Decision Drivers

- Plaintext exposure must be explicit and user-driven.
- Non-secret DTOs must not carry passwords.
- Secret-bearing APIs should be narrow and easy to audit.
- Locking should clear unlocked session material.

## Considered Options

- Centralize secret lifetime policy in `vault-core` and `vault-api`.
- Let each client define its own reveal and copy behavior.
- Treat secret lifetime as documentation only.

## Proposed Decision

Secret lifetime policy belongs in Rust and is surfaced through narrow API
operations. Clients may request a secret only through explicit reveal, copy, or
fill operations allowed by their capabilities.

Non-secret list, search, detail, audit, and revision DTOs must not include
plaintext passwords or revision passwords.

## Consequences

### Positive

- Reduces passive secret exposure.
- Gives every client the same baseline behavior.
- Makes secret-bearing paths easier to review.

### Negative

- Some UI flows require extra explicit calls.
- Tests can prove DTO and API behavior, but cannot fully prove OS memory or
  clipboard behavior.

### Neutral

- Desktop integration tests remain important for clipboard and lock behavior.

## Compliance

- [ ] Add tests that Debug and Display output cannot expose secrets.
- [ ] Add tests that non-secret DTOs omit current and historical passwords.
- [ ] Add lock tests for clearing unlocked session material.
- [ ] Add clipboard TTL tests where the platform boundary allows it.

## Revisit / Out Of Scope

- OS-level secure memory guarantees are platform-specific and need separate
  research before being promised.

## References

- #224
- #226
