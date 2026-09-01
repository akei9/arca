---
status: proposed
date: 2026-09-01
decision-makers: ["akei9"]
related: ["#207", "#223"]
---

# ADR-0001: KDBX 4 canonical on-disk format

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.

## Context

Arca currently writes local-first encrypted vault files through `vault-core`.
Future clients need to understand which parts of the persisted vault are
standard KDBX behavior and which parts are Arca-owned semantics.

## Decision Drivers

- Preserve local-first vault ownership.
- Avoid custom cryptography.
- Keep compatibility with established KDBX tooling where feasible.
- Make Arca-specific fields explicit and testable.

## Considered Options

- KDBX 4 as the canonical on-disk container.
- Arca-native encrypted file format.
- Dual-format support from the start.

## Proposed Decision

Use KDBX 4 as the canonical on-disk container. `vault-core` owns all reads and
writes. Arca-specific semantics are encoded as explicit custom fields and
covered by fixtures.

The current write path pins:

- outer cipher: ChaCha20
- inner cipher: ChaCha20
- KDF: Argon2id
- Argon2id memory: 128 MiB
- Argon2id iterations: 3
- Argon2id parallelism: 4

Arca custom fields currently include:

- `ArcaCollection`
- `ArcaRevisions`

## Consequences

### Positive

- Keeps Arca aligned with a mature password-vault container.
- Lets compatibility be tested with real KDBX fixtures.
- Avoids inventing a new encrypted file format.

### Negative

- Arca semantics must be carefully layered over KDBX.
- Some future behavior may not map cleanly to external KDBX clients.

### Neutral

- KeePassXC portability vs Arca-native semantics remains an explicit tradeoff.

## Compliance

- [ ] Add golden KDBX fixtures for supported Arca semantics versions.
- [ ] Add tests that verify current write-path cipher and KDF parameters.
- [ ] Add tests for malformed Arca custom fields.

## Revisit / Out Of Scope

- Full sync semantics are out of scope.
- New cryptographic algorithms require a separate ADR and human approval.

## References

- #207
- #223
- `packages/vault-core/src/vault.rs`
