# vault-api contract fixtures

These JSON files are deterministic public-contract fixtures. They are used by
`cargo test --workspace` to detect accidental DTO drift.

Rules:

- Do not store real vault data, real credentials, recovery material, or copied
  secrets in fixtures.
- Non-secret DTO fixtures must not contain secret-bearing fields.
- Secret-bearing request and response types are tested separately with runtime
  values so static analysis does not confuse fixture text with production
  credentials.

