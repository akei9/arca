use std::fs::{self, File};
use std::io::{BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};
use keepass::db::{Entry as KeepassEntry, EntryId, EntryMut};
use keepass::error::{DatabaseKeyError, DatabaseOpenError, DatabaseSaveError};
use keepass::{Database, DatabaseKey};
use uuid::Uuid;

use crate::error::VaultError;
use crate::types::{VaultEntry, VaultMeta};

const FIELD_TITLE: &str = "Title";
const FIELD_USERNAME: &str = "UserName";
const FIELD_PASSWORD: &str = "Password";
const FIELD_URL: &str = "URL";
const FIELD_NOTES: &str = "Notes";

/// Open and decrypt a KDBX file.
///
/// Maps KDBX entries to VaultEntry structs.
pub fn open_vault(path: &Path, password: &str) -> Result<(VaultMeta, Vec<VaultEntry>), VaultError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(VaultError::FileNotFound(path.display().to_string()));
        }
        Err(error) => return Err(VaultError::IoError(error)),
    };

    let key = DatabaseKey::new().with_password(password);
    let database = Database::open(&mut file, key).map_err(map_open_error)?;
    let meta = meta_from_database(&database);
    let entries = entries_from_database(&database);

    Ok((meta, entries))
}

/// Serialize entries back to KDBX and write to disk atomically.
pub fn save_vault(
    path: &Path,
    password: &str,
    meta: &VaultMeta,
    entries: &[VaultEntry],
) -> Result<(), VaultError> {
    let tmp_path = tmp_path_for(path);
    let result = write_vault_file(&tmp_path, password, meta, entries)
        .and_then(|()| fs::rename(&tmp_path, path).map_err(VaultError::IoError));

    if result.is_err() {
        let _cleanup_result = fs::remove_file(&tmp_path);
    }

    result
}

/// Create a new empty KDBX vault at path.
pub fn create_vault(path: &Path, password: &str, name: &str) -> Result<VaultMeta, VaultError> {
    let now = Utc::now().to_rfc3339();
    let meta = VaultMeta {
        name: name.to_string(),
        created_at: now.clone(),
        modified_at: now,
    };

    save_vault(path, password, &meta, &[])?;

    Ok(meta)
}

fn write_vault_file(
    path: &Path,
    password: &str,
    meta: &VaultMeta,
    entries: &[VaultEntry],
) -> Result<(), VaultError> {
    let database = database_from_entries(meta, entries)?;
    let key = DatabaseKey::new().with_password(password);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    database.save(&mut writer, key).map_err(map_save_error)?;
    writer.flush()?;

    Ok(())
}

fn database_from_entries(meta: &VaultMeta, entries: &[VaultEntry]) -> Result<Database, VaultError> {
    let mut database = Database::new();
    database.meta.generator = Some("Arca vault-core".to_string());
    database.meta.database_name = Some(meta.name.clone());
    database.meta.database_name_changed = Some(parse_rfc3339(&meta.modified_at)?);
    database.meta.settings_changed = Some(parse_rfc3339(&meta.modified_at)?);

    {
        let mut root = database.root_mut();
        root.name = meta.name.clone();
        root.times.creation = Some(parse_rfc3339(&meta.created_at)?);
        root.times.last_modification = Some(parse_rfc3339(&meta.modified_at)?);

        for entry in entries {
            let id = EntryId::from_uuid(
                Uuid::parse_str(&entry.id)
                    .map_err(|error| VaultError::SerializationError(error.to_string()))?,
            );
            let mut keepass_entry = root
                .add_entry_with_id(id)
                .map_err(|error| VaultError::SerializationError(error.to_string()))?;
            populate_keepass_entry(&mut keepass_entry, entry)?;
        }
    }

    Ok(database)
}

fn populate_keepass_entry(
    keepass_entry: &mut EntryMut<'_>,
    entry: &VaultEntry,
) -> Result<(), VaultError> {
    keepass_entry.set_unprotected(FIELD_TITLE, entry.title.clone());
    keepass_entry.set_unprotected(FIELD_USERNAME, entry.username.clone());
    keepass_entry.set_protected(FIELD_PASSWORD, entry.password.as_str());

    if let Some(url) = &entry.url {
        keepass_entry.set_unprotected(FIELD_URL, url.clone());
    }
    if let Some(notes) = &entry.notes {
        keepass_entry.set_unprotected(FIELD_NOTES, notes.clone());
    }

    keepass_entry.tags = entry.tags.clone();
    keepass_entry.times.creation = Some(parse_rfc3339(&entry.created_at)?);
    keepass_entry.times.last_modification = Some(parse_rfc3339(&entry.updated_at)?);
    keepass_entry.times.last_access = Some(parse_rfc3339(&entry.updated_at)?);
    keepass_entry.times.location_changed = Some(parse_rfc3339(&entry.created_at)?);

    Ok(())
}

fn meta_from_database(database: &Database) -> VaultMeta {
    let now = Utc::now().to_rfc3339();
    let root = database.root();
    let created_at = root
        .times
        .creation
        .as_ref()
        .map(naive_to_rfc3339)
        .unwrap_or_else(|| now.clone());
    let modified_at = root
        .times
        .last_modification
        .as_ref()
        .map(naive_to_rfc3339)
        .unwrap_or(now);

    VaultMeta {
        name: database
            .meta
            .database_name
            .clone()
            .unwrap_or_else(|| root.name.clone()),
        created_at,
        modified_at,
    }
}

fn entries_from_database(database: &Database) -> Vec<VaultEntry> {
    database
        .iter_all_entries()
        .map(|entry| vault_entry_from_keepass_entry(&entry))
        .collect()
}

fn vault_entry_from_keepass_entry(entry: &KeepassEntry) -> VaultEntry {
    let now = Utc::now().to_rfc3339();
    let created_at = entry
        .times
        .creation
        .as_ref()
        .map(naive_to_rfc3339)
        .unwrap_or_else(|| now.clone());
    let updated_at = entry
        .times
        .last_modification
        .as_ref()
        .map(naive_to_rfc3339)
        .unwrap_or(now);

    VaultEntry {
        id: entry.id().to_string(),
        title: entry.get_title().unwrap_or_default().to_string(),
        username: entry.get_username().unwrap_or_default().to_string(),
        password: entry.get_password().unwrap_or_default().to_string(),
        url: optional_string(entry.get_url()),
        notes: optional_string(entry.get(FIELD_NOTES)),
        tags: entry.tags.clone(),
        created_at,
        updated_at,
    }
}

fn optional_string(value: Option<&str>) -> Option<String> {
    value
        .filter(|text| !text.is_empty())
        .map(std::string::ToString::to_string)
}

fn parse_rfc3339(value: &str) -> Result<NaiveDateTime, VaultError> {
    DateTime::parse_from_rfc3339(value)
        .map(|datetime| datetime.naive_utc())
        .map_err(|error| VaultError::SerializationError(error.to_string()))
}

fn naive_to_rfc3339(value: &NaiveDateTime) -> String {
    DateTime::<Utc>::from_naive_utc_and_offset(*value, Utc).to_rfc3339()
}

fn tmp_path_for(path: &Path) -> PathBuf {
    let mut tmp_path = path.as_os_str().to_os_string();
    tmp_path.push(".tmp");
    PathBuf::from(tmp_path)
}

fn map_open_error(error: DatabaseOpenError) -> VaultError {
    match error {
        DatabaseOpenError::Key(DatabaseKeyError::IncorrectKey) => VaultError::InvalidPassword,
        DatabaseOpenError::Io(error) => VaultError::IoError(error),
        DatabaseOpenError::Cryptography(error) => VaultError::DecryptionError(error.to_string()),
        DatabaseOpenError::UnexpectedEof
        | DatabaseOpenError::VersionParse(_)
        | DatabaseOpenError::UnsupportedVersion
        | DatabaseOpenError::Format(_) => VaultError::CorruptedVault,
        DatabaseOpenError::Key(error) => VaultError::DecryptionError(error.to_string()),
        _ => VaultError::CorruptedVault,
    }
}

fn map_save_error(error: DatabaseSaveError) -> VaultError {
    match error {
        DatabaseSaveError::Io(error) => VaultError::IoError(error),
        DatabaseSaveError::Cryptography(error) => VaultError::EncryptionError(error.to_string()),
        DatabaseSaveError::Key(error) => VaultError::EncryptionError(error.to_string()),
        DatabaseSaveError::Random(error) => VaultError::EncryptionError(error.to_string()),
        DatabaseSaveError::UnsupportedVersion | DatabaseSaveError::Serialization(_) => {
            VaultError::SerializationError(error.to_string())
        }
        _ => VaultError::SerializationError(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{create_vault, tmp_path_for};

    #[test]
    fn tmp_path_appends_tmp_suffix() {
        assert_eq!(
            tmp_path_for(Path::new("/tmp/example.kdbx")),
            Path::new("/tmp/example.kdbx.tmp")
        );
    }

    #[test]
    fn create_vault_returns_named_meta() {
        let dir = tempfile::tempdir().expect("tempdir should be created");
        let path = dir.path().join("empty.kdbx");

        let meta =
            create_vault(&path, "master-password", "TEST_VAULT").expect("vault should be created");

        assert_eq!(meta.name, "TEST_VAULT");
        assert!(path.exists());
    }
}
