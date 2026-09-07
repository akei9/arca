use std::fs;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use vault_api::{
    ApiError, AuditFinding, AuditFindingKind, AuditSeverity, Capability, ClientKind,
    ContractVersions, EntryMutation, EntryView, ErrorCode, GeneratedSecret, GeneratorMode,
    GeneratorParams, RevealedSecret, RevisionView, VaultSummary,
};

#[test]
fn contract_versions_match_golden_fixture() {
    assert_fixture("contract_versions.json", &ContractVersions::current());
}

#[test]
fn vault_summary_matches_golden_fixture() {
    let summary = VaultSummary::new(
        "vault.local",
        "/tmp/arca/vault.local.kdbx",
        3,
        "2026-09-02T00:00:00+00:00",
    );

    assert_fixture("vault_summary.json", &summary);
    assert_round_trip("vault_summary.json", summary);
}

#[test]
fn entry_view_matches_golden_fixture_without_secret_fields() {
    let view = EntryView {
        id: "entry-0001".to_string(),
        title: "GitHub".to_string(),
        username: "arca-admin".to_string(),
        collection: Some("work".to_string()),
        url: Some("https://github.com".to_string()),
        notes: Some("Non-secret fixture notes".to_string()),
        tags: vec!["work".to_string(), "ssh".to_string()],
        created_at: "2026-09-01T00:00:00+00:00".to_string(),
        updated_at: "2026-09-02T00:00:00+00:00".to_string(),
        revision_count: 2,
    };

    assert_fixture("entry_view.json", &view);
    assert_round_trip("entry_view.json", view);
    assert_fixture_has_no_secret_fields("entry_view.json");
}

#[test]
fn archived_entry_view_matches_golden_fixture_without_secret_fields() {
    let view = EntryView {
        id: "entry-archived-0001".to_string(),
        title: "Archived Entry".to_string(),
        username: "archived-user".to_string(),
        collection: Some("archive".to_string()),
        url: None,
        notes: None,
        tags: Vec::new(),
        created_at: "2026-09-01T00:00:00+00:00".to_string(),
        updated_at: "2026-09-02T00:00:00+00:00".to_string(),
        revision_count: 0,
    };

    assert_fixture("archived_entry_view.json", &view);
    assert_round_trip("archived_entry_view.json", view);
    assert_fixture_has_no_secret_fields("archived_entry_view.json");
}

#[test]
fn revision_view_matches_golden_fixture_without_secret_fields() {
    let view = RevisionView {
        captured_at: "2026-09-03T00:00:00+00:00".to_string(),
        updated_at: "2026-09-02T00:00:00+00:00".to_string(),
        title: "GitHub".to_string(),
        username: "arca-admin".to_string(),
        collection: Some("work".to_string()),
        url: Some("https://github.com".to_string()),
        notes: Some("Previous non-secret fixture notes".to_string()),
        tags: vec!["work".to_string(), "ssh".to_string()],
        password_changed: true,
    };

    assert_fixture("revision_view.json", &view);
    assert_round_trip("revision_view.json", view);
    assert_fixture_has_no_secret_fields("revision_view.json");
}

#[test]
fn generator_params_match_golden_fixture() {
    let params = GeneratorParams {
        length: Some(24),
        uppercase: Some(true),
        lowercase: Some(true),
        digits: Some(true),
        symbols: Some(true),
        exclude_ambiguous: Some(false),
        mode: Some(GeneratorMode::Random),
    };

    assert_fixture("generator_params.json", &params);
    assert_round_trip("generator_params.json", params);
}

#[test]
fn audit_finding_matches_golden_fixture() {
    let finding = AuditFinding {
        key: "duplicate-url:entry-0001".to_string(),
        severity: AuditSeverity::Medium,
        kind: AuditFindingKind::DuplicateUrl,
        entry_id: "entry-0001".to_string(),
        meta: "https://example.test".to_string(),
    };

    assert_fixture("audit_finding.json", &finding);
    assert_round_trip("audit_finding.json", finding);
}

#[test]
fn api_error_matches_golden_fixture() {
    let error = ApiError::new(ErrorCode::CapabilityDenied, "client cannot reveal secrets");

    assert_fixture("api_error.json", &error);
    assert_round_trip("api_error.json", error);
}

#[test]
fn revealed_secret_matches_redacted_golden_fixture() {
    let response = RevealedSecret::new("[redacted]");

    assert_fixture("revealed_secret_redacted.json", &response);
}

#[test]
fn generated_secret_matches_redacted_golden_fixture() {
    let response = GeneratedSecret::new("[redacted]", 96.0);

    assert_fixture("generated_secret_redacted.json", &response);
}

#[test]
fn capability_fixture_round_trips() {
    let fixture = ClientCapabilityFixture {
        client_kind: ClientKind::BrowserExtension,
        capabilities: vec![
            Capability::Unlock,
            Capability::ReadMeta,
            Capability::CopySecret,
        ],
    };

    assert_fixture("client_capabilities.json", &fixture);
    assert_round_trip("client_capabilities.json", fixture);
}

#[test]
fn entry_mutation_fixture_preserves_omitted_nullable_fields() {
    let mutation: EntryMutation = read_fixture_json("entry_mutation_omitted_nullable.json");

    assert_eq!(mutation.title.as_deref(), Some("GitHub"));
    assert_eq!(mutation.username.as_deref(), Some("arca-admin"));
    assert_eq!(mutation.collection, None);
    assert_eq!(mutation.url, None);
    assert_eq!(mutation.notes, None);
    assert_eq!(
        mutation.tags,
        Some(vec!["work".to_string(), "ssh".to_string()])
    );
}

#[test]
fn entry_mutation_fixture_preserves_explicit_nullable_clears() {
    let mutation: EntryMutation = read_fixture_json("entry_mutation_clear_nullable.json");

    assert_eq!(mutation.collection, Some(None));
    assert_eq!(mutation.url, Some(None));
    assert_eq!(mutation.notes, Some(None));
    assert_eq!(mutation.tags, Some(Vec::new()));
}

#[test]
fn non_secret_response_dtos_ignore_unknown_fields() {
    let json = r#"{
      "id": "entry-0001",
      "title": "GitHub",
      "username": "arca-admin",
      "collection": "work",
      "url": "https://github.com",
      "notes": "Non-secret fixture notes",
      "tags": ["work", "ssh"],
      "createdAt": "2026-09-01T00:00:00+00:00",
      "updatedAt": "2026-09-02T00:00:00+00:00",
      "revisionCount": 2,
      "futureClientField": "ignored"
    }"#;

    let view = serde_json::from_str::<EntryView>(json)
        .expect("entry response should ignore unknown future fields");

    assert_eq!(view.title, "GitHub");
    assert_eq!(view.revision_count, 2);
}

#[test]
fn unknown_future_enum_values_fail_closed() {
    let error = serde_json::from_str::<ClientKind>(r#""watchApp""#)
        .expect_err("unknown client kind should not deserialize silently");

    assert!(error.to_string().contains("unknown variant"));
}

#[test]
fn newer_arca_semantics_versions_fail_closed_for_writers() {
    let versions: ContractVersions = read_fixture_json("unsupported_future_versions.json");
    let error = versions
        .ensure_writer_supported()
        .expect_err("future Arca semantics version should fail closed for writer paths");

    assert_eq!(error.code, ErrorCode::InvalidInput);
    assert!(error.message.contains("unsupported future Arca semantics"));
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ClientCapabilityFixture {
    client_kind: ClientKind,
    capabilities: Vec<Capability>,
}

fn assert_fixture<T>(name: &str, value: &T)
where
    T: Serialize,
{
    let expected = read_fixture(name);
    let actual =
        serde_json::to_string_pretty(value).expect("contract fixture should serialize") + "\n";

    assert_eq!(
        actual, expected,
        "{name} drifted from the checked-in fixture"
    );
}

fn assert_round_trip<T>(name: &str, value: T)
where
    T: DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let decoded: T = read_fixture_json(name);

    assert_eq!(decoded, value, "{name} did not round-trip");
}

fn assert_fixture_has_no_secret_fields(name: &str) {
    let value: serde_json::Value = read_fixture_json(name);
    let object = value.as_object().expect("fixture should be a JSON object");

    assert!(
        !object.contains_key("password") && !object.contains_key("secret"),
        "{name} must not expose secret-bearing fields"
    );
}

fn read_fixture_json<T>(name: &str) -> T
where
    T: DeserializeOwned,
{
    let json = read_fixture(name);

    serde_json::from_str(&json).expect("fixture should deserialize")
}

fn read_fixture(name: &str) -> String {
    let contents = fs::read_to_string(fixture_path(name)).expect("fixture should be readable");

    contents.trim_end().to_string() + "\n"
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}
