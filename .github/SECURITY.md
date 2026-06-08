# Security Policy

Arca is a password manager. Please avoid public disclosure of suspected vulnerabilities until maintainers have had time to investigate and remediate.

## Reporting a Vulnerability

Use GitHub private vulnerability reporting for this repository if it is available. If private reporting is not enabled, contact the maintainers directly before sharing exploit details publicly.

Do not include real passwords, vault files, private keys, recovery material, or other secrets in reports.

## Scope

Security-sensitive areas include:

- `packages/vault-core`
- Tauri IPC commands and capabilities
- vault locking, unlocking, encryption, persistence, and import/export behavior
- clipboard handling
- password generation, display, reveal, copy, and masking behavior

## Maintainer Setup Checklist

These settings require GitHub admin access:

- Enable private vulnerability reporting.
- Enable secret scanning and push protection where available.
- Enable Dependabot alerts and Dependabot security updates.
- Protect `main` with required PRs, at least one human approval, required status checks, up-to-date branches, admin enforcement, and restricted direct pushes.
- Consider requiring linear history for an auditable commit log.

Create these labels if they do not exist:

- `security` - red - crypto, auth, vault, secret-handling, or vulnerability-related changes.
- `agent-assisted` - purple - PR contains AI-generated or AI-edited code.
- `needs-human-review` - orange - must receive explicit human review before merge.
- `dependabot` - blue - automated dependency update.
