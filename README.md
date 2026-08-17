# arca

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/77654f67-5294-4d34-94d2-e79786ffc99c">
  <img alt="Arca — a vault for what you can't lose." src="https://github.com/user-attachments/assets/c44544b8-8737-41c1-8d1a-9aa8583bb7a5">
</picture>

Open-source, local-first password vault with a terminal-inspired UI.

## Current Entry Semantics

Entries require a non-empty password. Editing an entry may omit the password to
keep the existing value, or provide a non-empty replacement. Passwordless entries
and password clearing are not supported in this release; see
[#74](https://github.com/akei9/arca/issues/74).

## Release Preparation

Maintainer release and smoke-test steps live in [RELEASE.md](RELEASE.md).
