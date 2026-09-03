---
status: accepted
date: 2026-09-03
decision-makers: ["akei9"]
related: ["#209", "#223", "#224"]
---

# ADR-0002: Rust vault-api is the machine contract

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.
- 2026-09-03 - accepted by maintainer approval.

## Context

Desktop TypeScript DTOs and future mobile or extension bindings need a stable
contract. That contract should not expose `vault-core` internals such as KDBX
parsing, session storage, or zeroization implementation details.

## Decision Drivers

- Rust remains canonical for vault semantics.
- Clients stay thin.
- Generated bindings should reduce type drift.
- JSON Schema should document and verify snapshots, not become source of truth.

## Considered Options

- Rust `vault-api` crate as the source of truth.
- Thrift, Protobuf, or OpenAPI first.
- JSON Schema generated from `vault-core` types.
- Hand-maintained TypeScript interfaces.

## Proposed Decision

Introduce `packages/vault-api` as the public machine contract. It defines
client-facing DTOs, operations, capabilities, and typed errors. It delegates to
`vault-core` for implementation and does not expose `vault-core` internals.

Derived artifacts may include:

- desktop TypeScript types
- JSON Schema snapshots for public DTOs
- UniFFI Swift and Kotlin bindings
- future WASM TypeScript bindings if an ADR allows them

## Consequences

### Positive

- Gives every client one public contract to target.
- Keeps security-sensitive internals behind Rust boundaries.
- Makes generated bindings possible without exposing the whole core.

### Negative

- Adds another Rust crate and boundary to maintain.
- Requires migration away from hand-maintained desktop IPC DTOs.

### Neutral

- Some types may exist twice temporarily during migration.

## Compliance

- [ ] Add a `vault-api` crate or accepted equivalent.
- [ ] Generate or verify desktop DTOs from public API types.
- [ ] Add CI drift checks for generated artifacts.
- [ ] Add tests for public secret-bearing and non-secret DTO boundaries.

## Revisit / Out Of Scope

- A future network sync service may use OpenAPI, Protobuf, or another service
  IDL, but that is not this contract.

## References

- #209
- #224
