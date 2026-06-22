use std::fs::{self, File};
use std::path::PathBuf;

use keepass::{Database, DatabaseKey};
use vault_core::entry::{create_entry, update_entry, EntryPatch};
use vault_core::error::VaultError;
use vault_core::types::VaultMeta;
use vault_core::vault::{create_vault, open_vault, save_vault};

const FIXTURE_PASSWORD: &str = "fixture-master-password";

#[test]
fn test_create_open_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("roundtrip.kdbx");
    let meta =
        create_vault(&path, "master-password", "ROUNDTRIP").expect("vault should be created");
    let mut entry = create_entry("GitHub", "arca_admin", "correct horse");
    entry.collection = Some("work".to_string());
    entry.url = Some("https://github.com".to_string());
    entry.notes = Some("primary repository account".to_string());
    entry.tags = vec!["work".to_string(), "ssh".to_string()];

    save_vault(&path, "master-password", &meta, &[entry.clone()]).expect("vault should be saved");
    let (opened_meta, entries) =
        open_vault(&path, "master-password").expect("vault should open with the correct password");

    assert_eq!(opened_meta.name, "ROUNDTRIP");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, entry.id);
    assert_eq!(entries[0].title, "GitHub");
    assert_eq!(entries[0].username, "arca_admin");
    assert_eq!(entries[0].password, "correct horse");
    assert_eq!(entries[0].collection.as_deref(), Some("work"));
    assert_eq!(entries[0].url.as_deref(), Some("https://github.com"));
    assert_eq!(
        entries[0].notes.as_deref(),
        Some("primary repository account")
    );
    assert_eq!(entries[0].tags, vec!["work", "ssh"]);
}

#[test]
fn test_update_and_delete_persist_after_reopen() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("update-delete.kdbx");
    let meta = create_vault(&path, "master-password", "PERSIST").expect("vault should be created");
    let mut primary = create_entry("Primary", "primary_user", "primary-secret");
    let secondary = create_entry("Secondary", "secondary_user", "secondary-secret");
    let secondary_id = secondary.id.clone();

    save_vault(
        &path,
        "master-password",
        &meta,
        &[primary.clone(), secondary],
    )
    .expect("initial vault should be saved");

    update_entry(
        &mut primary,
        EntryPatch {
            title: Some("Primary Updated".to_string()),
            username: Some("primary_updated".to_string()),
            password: Some("primary-updated-secret".to_string()),
            collection: Some(Some("work".to_string())),
            url: Some(Some("https://example.test/updated".to_string())),
            notes: Some(Some("updated metadata".to_string())),
            tags: Some(vec!["work".to_string(), "updated".to_string()]),
        },
    );

    let (_, opened_entries) =
        open_vault(&path, "master-password").expect("vault should reopen before update");
    let retained = opened_entries
        .into_iter()
        .filter(|entry| entry.id == secondary_id)
        .collect::<Vec<_>>();
    let updated_entries = [vec![primary.clone()], retained].concat();
    save_vault(&path, "master-password", &meta, &updated_entries)
        .expect("updated vault should save");

    let (_, reopened_entries) =
        open_vault(&path, "master-password").expect("updated vault should reopen");
    let reopened_primary = reopened_entries
        .iter()
        .find(|entry| entry.id == primary.id)
        .expect("updated entry should remain");

    assert_eq!(reopened_entries.len(), 2);
    assert_eq!(reopened_primary.title, "Primary Updated");
    assert_eq!(reopened_primary.username, "primary_updated");
    assert_eq!(reopened_primary.password, "primary-updated-secret");
    assert_eq!(reopened_primary.collection.as_deref(), Some("work"));
    assert_eq!(
        reopened_primary.url.as_deref(),
        Some("https://example.test/updated")
    );
    assert_eq!(reopened_primary.notes.as_deref(), Some("updated metadata"));
    assert_eq!(reopened_primary.tags, vec!["work", "updated"]);

    let after_delete = reopened_entries
        .into_iter()
        .filter(|entry| entry.id != secondary_id)
        .collect::<Vec<_>>();
    save_vault(&path, "master-password", &meta, &after_delete).expect("deleted vault should save");

    let (_, final_entries) =
        open_vault(&path, "master-password").expect("deleted vault should reopen");

    assert_eq!(final_entries.len(), 1);
    assert_eq!(final_entries[0].id, primary.id);
}

#[test]
fn test_open_keepass_0_6_fixture() {
    let (meta, entries) = open_vault(&fixture_path("keepass-0-6-vault.kdbx"), FIXTURE_PASSWORD)
        .expect("legacy keepass 0.6 fixture should open");

    assert_eq!(meta.name, "KEEPASS_0_6_COMPAT");
    assert_eq!(meta.created_at, "2026-06-11T00:00:00+00:00");
    assert_eq!(meta.modified_at, "2026-06-11T00:00:00+00:00");
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.id, "11111111-2222-4333-8444-555555555555");
    assert_eq!(entry.title, "Legacy KeePass");
    assert_eq!(entry.username, "fixture_user");
    assert_eq!(entry.password, "fixture-password");
    assert_eq!(entry.url.as_deref(), Some("https://example.test/legacy"));
    assert_eq!(
        entry.notes.as_deref(),
        Some("Non-secret fixture generated with keepass 0.6.x")
    );
    assert_eq!(entry.tags, vec!["compat", "keepass-0.6"]);
}

#[test]
fn test_current_save_opens_with_keepass_and_protects_password() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("current-save.kdbx");
    let meta = VaultMeta {
        name: "CURRENT_KEEPASS".to_string(),
        created_at: "2026-06-11T00:00:00+00:00".to_string(),
        modified_at: "2026-06-11T00:00:00+00:00".to_string(),
    };
    let entry_password = test_password(&["fixture", "password"]);
    let mut entry = create_entry("Current KeePass", "fixture_user", &entry_password);

    entry.id = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee".to_string();
    entry.collection = Some("infrastructure".to_string());
    entry.url = Some("https://example.test/current".to_string());
    entry.notes = Some("Non-secret fixture saved by current vault-core".to_string());
    entry.tags = vec!["compat".to_string(), "keepass-current".to_string()];
    entry.created_at = "2026-06-11T00:00:00+00:00".to_string();
    entry.updated_at = "2026-06-11T00:00:00+00:00".to_string();

    save_vault(&path, FIXTURE_PASSWORD, &meta, &[entry]).expect("current vault should save");

    let mut file = File::open(&path).expect("saved vault should exist");
    let key = DatabaseKey::new().with_password(FIXTURE_PASSWORD);
    let database = Database::open(&mut file, key).expect("keepass crate should open saved vault");
    let keepass_entry = database
        .iter_all_entries()
        .next()
        .expect("saved vault should contain one entry");
    let password_value = keepass_entry
        .fields
        .get("Password")
        .expect("saved entry should contain password field");

    assert_eq!(database.root().name, "CURRENT_KEEPASS");
    assert_eq!(
        keepass_entry.id().to_string(),
        "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"
    );
    assert_eq!(keepass_entry.get_title(), Some("Current KeePass"));
    assert_eq!(keepass_entry.get_username(), Some("fixture_user"));
    assert_eq!(keepass_entry.get_password(), Some(entry_password.as_str()));
    assert!(password_value.is_protected());
}

#[test]
fn test_open_invalid_password_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("invalid-password.kdbx");
    create_vault(&path, "right-password", "LOCKED").expect("vault should be created");

    let result = open_vault(&path, "wrong-password");

    assert!(result.is_err());
}

#[test]
fn test_open_corrupt_file_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("corrupt.kdbx");

    fs::write(&path, b"not a kdbx file").expect("corrupt fixture should be written");

    let password = test_password(&["master", "password"]);
    let result = open_vault(&path, &password);

    assert!(matches!(result, Err(VaultError::CorruptedVault)));
}

#[test]
fn test_save_invalid_entry_id_returns_serialization_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("invalid-entry-id.kdbx");
    let meta = VaultMeta {
        name: "INVALID_ENTRY_ID".to_string(),
        created_at: "2026-06-11T00:00:00+00:00".to_string(),
        modified_at: "2026-06-11T00:00:00+00:00".to_string(),
    };
    let entry_password = test_password(&["fixture", "password"]);
    let mut entry = create_entry("Invalid", "fixture_user", &entry_password);

    entry.id = "not-a-uuid".to_string();

    let password = test_password(&["master", "password"]);
    let result = save_vault(&path, &password, &meta, &[entry]);

    assert!(matches!(result, Err(VaultError::SerializationError(_))));
}

#[test]
fn test_open_nonexistent_file_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("missing.kdbx");

    let result = open_vault(&path, "master-password");

    assert!(matches!(result, Err(VaultError::FileNotFound(_))));
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn test_password(parts: &[&str]) -> String {
    parts.join("-")
}
