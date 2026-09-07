use std::fs;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;
use vault_api::{
    ApiError, ApiOperation, AuditFinding, AuditFindingKind, AuditSeverity, Capability,
    ClientCapabilities, ClientKind, ContractVersions, CreateEntryRequest, EntryMutation, EntryView,
    ErrorCode, GeneratedSecret, GeneratorMode, GeneratorParams, RevealedSecret, RevisionView,
    SecretString, VaultSummary,
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
    let response = RevealedSecret::new(redacted_fixture_secret());

    assert_fixture("revealed_secret_redacted.json", &response);
}

#[test]
fn generated_secret_matches_redacted_golden_fixture() {
    let response = GeneratedSecret::new(redacted_fixture_secret(), 96.0);

    assert_fixture("generated_secret_redacted.json", &response);
}

#[test]
fn capability_fixture_round_trips() {
    let fixture: ClientCapabilities = ClientKind::BrowserExtension.granted_capabilities();

    assert_fixture("client_capabilities.json", &fixture);
    assert_round_trip("client_capabilities.json", fixture);
}

#[test]
fn desktop_app_has_the_full_initial_capability_surface() {
    assert_eq!(ClientKind::DesktopApp.capabilities(), Capability::all());

    for capability in Capability::all() {
        assert!(ClientKind::DesktopApp.allows(*capability));
        ClientKind::DesktopApp
            .require_capability(*capability)
            .expect("desktop app should receive every initial capability");
    }
}

#[test]
fn restricted_clients_deny_privileged_capabilities() {
    let restricted_clients = [
        ClientKind::BrowserExtension,
        ClientKind::IosAutofillExtension,
        ClientKind::AndroidAutofillService,
    ];
    let denied_capabilities = [
        Capability::RevealSecret,
        Capability::MutateEntry,
        Capability::CreateVault,
        Capability::ChangeKdf,
        Capability::ExportPlaintext,
        Capability::ExportKdbx,
        Capability::ReadHistory,
        Capability::DeletePermanent,
    ];

    for client_kind in restricted_clients {
        for capability in denied_capabilities {
            let error = client_kind
                .require_capability(capability)
                .expect_err("restricted clients should fail closed");

            assert_eq!(error.code, ErrorCode::CapabilityDenied);
            assert!(
                error.message.contains(client_kind.as_str())
                    && error.message.contains(capability.as_str()),
                "denial should name the client kind and denied capability"
            );
        }
    }
}

#[test]
fn restricted_clients_deny_privileged_operations() {
    let restricted_clients = [
        ClientKind::BrowserExtension,
        ClientKind::IosAutofillExtension,
        ClientKind::AndroidAutofillService,
    ];
    let denied_operations = [
        ApiOperation::RevealSecret,
        ApiOperation::CreateEntry,
        ApiOperation::UpdateEntry,
        ApiOperation::CreateVault,
        ApiOperation::ChangeKdf,
        ApiOperation::ExportPlaintext,
        ApiOperation::ExportKdbx,
        ApiOperation::ReadHistory,
        ApiOperation::DeletePermanent,
    ];

    for client_kind in restricted_clients {
        for operation in denied_operations {
            let error = client_kind
                .require_operation(operation)
                .expect_err("restricted clients should fail closed by operation");

            assert_eq!(error.code, ErrorCode::CapabilityDenied);
            assert!(error
                .message
                .contains(operation.required_capability().as_str()));
        }
    }
}

#[test]
fn secret_bearing_entry_dtos_do_not_bypass_mutation_capability() {
    let _create_request = CreateEntryRequest {
        title: "GitHub".to_string(),
        username: "arca".to_string(),
        password: SecretString::new(redacted_fixture_secret()),
        collection: None,
        url: None,
        notes: None,
        tags: Vec::new(),
    };
    let _update_request = EntryMutation {
        password: Some(SecretString::new(redacted_fixture_secret())),
        ..EntryMutation::default()
    };

    assert_eq!(
        ApiOperation::CreateEntry.required_capability(),
        Capability::MutateEntry
    );
    assert_eq!(
        ApiOperation::UpdateEntry.required_capability(),
        Capability::MutateEntry
    );

    for client_kind in [
        ClientKind::BrowserExtension,
        ClientKind::IosAutofillExtension,
        ClientKind::AndroidAutofillService,
        ClientKind::FutureSyncServer,
    ] {
        ClientKind::DesktopApp
            .require_operation(ApiOperation::CreateEntry)
            .expect("desktop app should be allowed to create entries");
        client_kind
            .require_operation(ApiOperation::CreateEntry)
            .expect_err("restricted clients should not create entries with shared DTOs");
        client_kind
            .require_operation(ApiOperation::UpdateEntry)
            .expect_err("restricted clients should not update entries with shared DTOs");
    }
}

#[test]
fn plaintext_secret_response_operations_require_reveal_secret_capability() {
    let plaintext_response_operations = [
        ApiOperation::RevealSecret,
        ApiOperation::RevealRevisionSecret,
        ApiOperation::GenerateSecret,
    ];

    for operation in plaintext_response_operations {
        assert_eq!(operation.required_capability(), Capability::RevealSecret);

        ClientKind::DesktopApp
            .require_operation(operation)
            .expect("desktop app should receive plaintext secret responses");
        ClientKind::BrowserExtension
            .require_operation(operation)
            .expect_err("browser extension should not receive plaintext secret responses");
        ClientKind::IosAutofillExtension
            .require_operation(operation)
            .expect_err("autofill extension should not receive plaintext secret responses");
        ClientKind::FutureSyncServer
            .require_operation(operation)
            .expect_err("sync server should never receive plaintext secret responses");
    }
}

#[test]
fn browser_and_autofill_surfaces_are_strict_subsets_of_desktop_app() {
    let desktop_capabilities = ClientKind::DesktopApp.capabilities();
    let restricted_clients = [
        ClientKind::BrowserExtension,
        ClientKind::IosAutofillExtension,
        ClientKind::AndroidAutofillService,
    ];

    for client_kind in restricted_clients {
        let capabilities = client_kind.capabilities();

        assert!(capabilities.len() < desktop_capabilities.len());
        assert!(
            capabilities
                .iter()
                .all(|capability| desktop_capabilities.contains(capability)),
            "{client_kind:?} should not receive any capability outside DesktopApp"
        );
    }
}

#[test]
fn mobile_apps_deny_plaintext_export_but_keep_app_management_surface() {
    let allowed_capabilities = [
        Capability::Unlock,
        Capability::ReadMeta,
        Capability::RevealSecret,
        Capability::CopySecret,
        Capability::MutateEntry,
        Capability::CreateVault,
        Capability::ChangeKdf,
        Capability::ExportKdbx,
        Capability::ReadHistory,
        Capability::DeletePermanent,
    ];

    for client_kind in [ClientKind::IosApp, ClientKind::AndroidApp] {
        for capability in allowed_capabilities {
            client_kind
                .require_capability(capability)
                .expect("mobile app should keep normal app management capability");
        }

        let error = client_kind
            .require_capability(Capability::ExportPlaintext)
            .expect_err("mobile app should not receive plaintext export by default");

        assert_eq!(error.code, ErrorCode::CapabilityDenied);
    }
}

#[test]
fn future_sync_server_remains_ciphertext_only() {
    assert!(ClientKind::FutureSyncServer.capabilities().is_empty());

    for capability in Capability::all() {
        let error = ClientKind::FutureSyncServer
            .require_capability(*capability)
            .expect_err("future sync should not receive public plaintext API capabilities");

        assert_eq!(error.code, ErrorCode::CapabilityDenied);
    }
}

#[test]
fn future_sync_server_has_no_plaintext_secret_capabilities() {
    let plaintext_capabilities: Vec<_> = ClientKind::FutureSyncServer
        .capabilities()
        .iter()
        .copied()
        .filter(|capability| capability.carries_plaintext_secret())
        .collect();

    assert!(
        plaintext_capabilities.is_empty(),
        "FutureSyncServer must stay ciphertext-only"
    );
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

fn redacted_fixture_secret() -> String {
    ['[', 'r', 'e', 'd', 'a', 'c', 't', 'e', 'd', ']']
        .into_iter()
        .collect()
}
