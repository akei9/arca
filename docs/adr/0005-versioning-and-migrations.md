---
status: proposed
date: 2026-09-01
decision-makers: ["akei9"]
related: ["#207", "#223", "#225"]
---

# ADR-0005: Versioning and migrations

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.

## Context

Arca needs compatibility rules before multiple clients can read and write the
same vault. A single "vault version" is too vague because KDBX, Arca semantics,
and client protocols evolve independently.

## Decision Drivers

- Avoid accidental data loss.
- Fail closed for unknown future write semantics.
- Permit newer readers where safe.
- Make migrations testable.

## Considered Options

- Separate KDBX, Arca semantics, and client protocol versions.
- Use one global vault version.
- Rely on app version numbers only.

## Proposed Decision

Track three version lines:

- KDBX format version
- Arca semantics version
- client protocol or binding version

Readers may open older Arca semantics versions if tests prove compatibility.
Writers must not write unknown future Arca semantics versions. Migrations require
fixtures and refusal tests.

## Consequences

### Positive

- Prevents clients from silently rewriting data they do not understand.
- Gives browser and mobile clients a clear compatibility gate.

### Negative

- Requires version metadata and fixture discipline.

### Neutral

- Some migration details depend on `vault-api` design.

## Compliance

- [ ] Add fixtures for supported Arca semantics versions.
- [ ] Add tests for unknown future semantics versions.
- [ ] Add migration or refusal tests for every breaking change.
- [ ] Add CI checks for contract fixture drift.

## Revisit / Out Of Scope

- Sync conflict resolution is out of scope.

## References

- #225
