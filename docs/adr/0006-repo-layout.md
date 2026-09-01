---
status: proposed
date: 2026-09-01
decision-makers: ["akei9"]
related: ["#208", "#223"]
---

# ADR-0006: Repository layout

## Status History

- 2026-09-01 - proposed by Codex for maintainer review.

## Context

Browser extension and mobile work may eventually deserve separate repositories,
but splitting too early makes contracts and security policy harder to stabilize.

## Decision Drivers

- Keep shared core and compatibility gates canonical.
- Avoid premature multi-repo coordination.
- Allow future clients to ship independently once real.

## Considered Options

- Keep monorepo during the contract phase.
- Split browser and mobile repositories immediately.
- Keep every future client in this repository permanently.

## Proposed Decision

Keep this repository as canonical core plus desktop during the contract phase.
Future repositories should be created only when implementation starts and after
the shared contract is stable enough to consume.

Expected future repositories:

- `arca-extension`
- `arca-mobile`, unless a later mobile ADR chooses separate native repositories

## Consequences

### Positive

- Keeps early contract work focused.
- Avoids duplicating policy and fixtures before they are stable.

### Negative

- Future repo bootstraps are deferred.

### Neutral

- Issue references may span repositories later.

## Compliance

- [ ] Keep `vault-core`, `vault-api`, fixtures, and security policy canonical in
  `akei9/arca`.
- [ ] Add repo bootstrap checklists before creating new repositories.

## Revisit / Out Of Scope

- Exact mobile repo layout waits for ADR-0008.

## References

- #208
