# vault-api contract fixtures

These JSON files are deterministic public-contract fixtures. They are used by
`cargo test --workspace` to detect accidental DTO drift.

Mobile adapters should consume the mobile operation, document, session-status,
stable-error, and explicit copy-secret fixtures when generated Swift and Kotlin
binding checks are added.

Rules:

- Do not store real vault data, real credentials, recovery material, or copied
  secrets in fixtures.
- Non-secret DTO fixtures must not contain secret-bearing fields.
- Secret-bearing response fixtures must use explicit redacted placeholder values
  only, never realistic passwords or copied secrets.
