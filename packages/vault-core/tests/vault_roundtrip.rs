use vault_core::entry::create_entry;
use vault_core::error::VaultError;
use vault_core::vault::{create_vault, open_vault, save_vault};

#[test]
fn test_create_open_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("roundtrip.kdbx");
    let meta =
        create_vault(&path, "master-password", "ROUNDTRIP").expect("vault should be created");
    let mut entry = create_entry("GitHub", "arca_admin", "correct horse");
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
    assert_eq!(entries[0].url.as_deref(), Some("https://github.com"));
    assert_eq!(
        entries[0].notes.as_deref(),
        Some("primary repository account")
    );
    assert_eq!(entries[0].tags, vec!["work", "ssh"]);
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
fn test_open_nonexistent_file_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("missing.kdbx");

    let result = open_vault(&path, "master-password");

    assert!(matches!(result, Err(VaultError::FileNotFound(_))));
}
