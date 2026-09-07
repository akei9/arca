---
status: proposed
date: 2026-09-07
decision-makers: ["akei9"]
related: ["#138", "#208", "#211", "#216"]
---

# ADR-0010: Client repository governance

## Status History

- 2026-09-07 - proposed by Codex for maintainer review.

## Context

ADR-0006 keeps `akei9/arca` as the canonical core plus desktop repository
during the contract phase. The browser extension and mobile app should become
separate repositories once their implementations begin, but those repositories
must still consume the same Rust contract, compatibility fixtures, and security
policy.

This ADR defines when to create future client repositories and how governance,
issues, releases, and security advisories should flow across them.

## Decision Drivers

- Keep shared vault semantics and compatibility gates canonical.
- Avoid splitting before the shared contract is stable enough to consume.
- Let real extension and mobile clients ship independently once implementation
  starts.
- Keep security-sensitive policy changes reviewed in one canonical place.
- Make cross-repository issue, release, and advisory ownership predictable.

## Considered Options

- Keep browser extension and mobile clients in `akei9/arca` permanently.
- Split extension and mobile repositories immediately.
- Create one separate repository per future client once implementation starts.
- Create separate iOS and Android repositories instead of one mobile repository.

## Proposed Decision

Keep `akei9/arca` as the canonical repository for core vault behavior, desktop,
and shared client contracts during the platform contract phase.

Create `arca-extension` when browser extension implementation starts. That
repository should contain the browser extension app, store packaging, extension
permission documentation, native messaging client code, browser-specific tests,
and extension release workflow.

Create `arca-mobile` when mobile implementation starts, unless a later accepted
mobile ADR chooses separate native repositories. The default mobile repository
contains shared mobile planning plus the native iOS and Android implementations
needed for the MVP. Separate `arca-ios` and `arca-android` repositories require
a later ADR that explains why one mobile repository is no longer sufficient.

The following stay canonical in `akei9/arca`:

- `packages/vault-core`
- `packages/vault-api`
- public API contracts and generated compatibility fixtures
- ADRs and security policy that constrain all clients
- KDBX compatibility fixtures
- cargo-deny, RustSec, CodeQL, fuzzing, and other shared security gates
- cross-client capability policy

Future client repositories consume `vault-api` and `vault-core` artifacts, but
must not fork the contract, duplicate KDBX parsing, change cryptographic
parameters, or silently diverge from shared fixtures. Any required contract
change starts in `akei9/arca`, lands with compatibility tests, and is then
consumed by downstream client repositories.

## Cross-Repository Governance

Issues that change shared vault behavior, public contracts, compatibility,
security policy, or client capabilities belong in `akei9/arca`. Issues that only
affect browser UI, browser packaging, extension store submission, or native
messaging UX belong in `arca-extension`. Issues that only affect native mobile
UI, app store submission, mobile platform build tooling, or autofill extension
packaging belong in `arca-mobile`.

Cross-client features should start with a tracking issue in `akei9/arca` and
link downstream implementation issues. The canonical issue should define the
contract, security constraints, and release order. Downstream issues should
describe client-specific implementation and test coverage.

Release notes in downstream client repositories must name the consumed
`vault-api` or `vault-core` version, commit, or tag. Breaking contract changes
must land in `akei9/arca` first, with an ADR when required by the client
constitution.

Security advisories that affect `vault-core`, `vault-api`, KDBX handling,
secret lifetime, capability policy, native messaging trust, sync boundaries, or
multiple clients belong in `akei9/arca`. Client-specific packaging or UI
vulnerabilities may be handled in the affected client repository, but should
link back to a canonical security tracking issue when shared policy or contract
changes are needed.

## Bootstrap Checklist

Before creating `arca-extension`:

- [ ] ADR-0007 and ADR-0009 are accepted.
- [ ] `vault-api` exports the client contract needed by the first extension
  surface.
- [ ] Native messaging protocol scope is documented.
- [ ] Extension permissions and store-review rationale are documented.
- [ ] The repository has CI for linting, tests, dependency scanning, secret
  scanning, and contract fixture drift.
- [ ] The repository documents how it consumes `vault-api` and shared fixtures.
- [ ] Security issue routing points shared contract or vault issues back to
  `akei9/arca`.

Before creating `arca-mobile`:

- [ ] ADR-0008 and ADR-0009 are accepted.
- [ ] A mobile ADR confirms whether one mobile repository is still sufficient.
- [ ] UniFFI or the selected binding strategy has a feasibility spike.
- [ ] OS file access, secure storage, biometric unlock, and autofill lifecycle
  requirements are documented.
- [ ] The repository has CI for platform builds, tests, dependency scanning,
  secret scanning, and contract fixture drift.
- [ ] The repository documents how it consumes `vault-api`, `vault-core`, and
  shared fixtures.
- [ ] Security issue routing points shared contract or vault issues back to
  `akei9/arca`.

## Consequences

### Positive

- Keeps the security-critical contract stable before downstream clients appear.
- Gives browser and mobile work independent release paths once implementation is
  real.
- Makes shared-policy ownership explicit.
- Reduces the chance of client repos drifting from vault compatibility rules.

### Negative

- Future changes spanning clients will require cross-repository coordination.
- Client repositories cannot move faster than shared contract review for
  contract-affecting work.
- Bootstrap work must include CI, security routing, and fixture consumption
  before feature development accelerates.

### Neutral

- The exact repository names may change before creation, but the governance
  rules still apply to whatever repositories replace them.

## Compliance

- [ ] Create `arca-extension` only when browser extension implementation starts.
- [ ] Create `arca-mobile` only when mobile implementation starts, unless a later
  accepted ADR chooses separate native repositories.
- [ ] Keep shared contracts, fixtures, security policy, and capability policy
  canonical in `akei9/arca`.
- [ ] Link cross-client implementation issues back to canonical tracking issues
  in `akei9/arca`.
- [ ] Require downstream client repositories to run contract fixture drift checks
  before release.

## Revisit / Out Of Scope

- Creating the repositories.
- Choosing browser extension framework or store packaging details.
- Choosing native mobile project structure beyond the default single
  `arca-mobile` repository.
- Account-backed sync, sharing, or remote services.

## References

- #138
- #208
- #211
- #216
