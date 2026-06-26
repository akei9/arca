# arca
Open-source, local-first password vault with a terminal-inspired UI.

## Current Entry Semantics

Entries require a non-empty password. Editing an entry may omit the password to
keep the existing value, or provide a non-empty replacement. Passwordless entries
and password clearing are not supported in this release; see
[#74](https://github.com/akei9/arca/issues/74).

## Release Preparation

Maintainer release and smoke-test steps live in [RELEASE.md](RELEASE.md).
