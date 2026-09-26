use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::ArcaError;

const PREFERENCES_FILE_NAME: &str = "preferences.json";

/// Coordinates preference operations that may arrive on different Tauri threads.
#[derive(Default)]
pub struct PreferencesState {
    operation: Mutex<()>,
}

impl PreferencesState {
    /// Serializes reads, writes, and forget operations into one observable order.
    pub fn operation(&self) -> Result<MutexGuard<'_, ()>, ArcaError> {
        self.operation.lock().map_err(|_| ArcaError::state_lock())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopPreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    remembered_vault_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// Describes one remembered locator without opening or decrypting its vault.
pub struct RememberedVaultDto {
    pub path: String,
    pub display_name: String,
    pub available: bool,
}

/// Resolves the application-owned preference file for the current platform.
pub fn preferences_file_path(app: &AppHandle) -> Result<PathBuf, ArcaError> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(PREFERENCES_FILE_NAME))
        .map_err(|_| ArcaError::preferences("Unable to locate desktop preferences"))
}

/// Loads the remembered locator and reports whether its vault file is available.
pub fn load_remembered_vault(path: &Path) -> Result<Option<RememberedVaultDto>, ArcaError> {
    let preferences = load_preferences(path)?;

    Ok(preferences.remembered_vault_path.map(remembered_vault_dto))
}

/// Atomically persists a normalized locator and returns its native display metadata.
pub fn persist_remembered_vault(
    preferences_path: &Path,
    vault_path: &Path,
) -> Result<RememberedVaultDto, ArcaError> {
    let normalized_path = fs::canonicalize(vault_path)
        .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    let preferences = DesktopPreferences {
        remembered_vault_path: Some(normalized_path.clone()),
    };
    let serialized = serde_json::to_vec(&preferences)
        .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    let parent = preferences_path
        .parent()
        .ok_or_else(|| ArcaError::preferences("Unable to remember vault"))?;

    create_private_directory(parent)?;
    replace_private_file(preferences_path, &serialized)?;

    Ok(remembered_vault_dto(normalized_path))
}

/// Removes the remembered locator and any interrupted staged replacement.
pub fn forget_remembered_vault(preferences_path: &Path) -> Result<(), ArcaError> {
    remove_file_if_present(
        &staged_preferences_path(preferences_path),
        "Unable to forget vault",
    )?;
    remove_file_if_present(preferences_path, "Unable to forget vault")
}

/// Converts one native path into display metadata using this platform's separator rules.
fn remembered_vault_dto(vault_path: PathBuf) -> RememberedVaultDto {
    let display_name = vault_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Last vault")
        .to_string();
    let available = vault_path.is_file();

    RememberedVaultDto {
        path: vault_path.display().to_string(),
        display_name,
        available,
    }
}

/// Removes a preference artifact while keeping forget idempotent.
fn remove_file_if_present(path: &Path, error_message: &'static str) -> Result<(), ArcaError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ArcaError::preferences(error_message)),
    }
}

/// Loads and validates the complete desktop preference document.
fn load_preferences(path: &Path) -> Result<DesktopPreferences, ArcaError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(DesktopPreferences::default());
        }
        Err(_) => return Err(ArcaError::preferences("Unable to load desktop preferences")),
    };

    serde_json::from_slice(&bytes)
        .map_err(|_| ArcaError::preferences("Unable to load desktop preferences"))
}

/// Creates the application preference directory with owner-only permissions.
fn create_private_directory(path: &Path) -> Result<(), ArcaError> {
    fs::create_dir_all(path).map_err(|_| ArcaError::preferences("Unable to remember vault"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    }

    Ok(())
}

/// Replaces preferences only after a private staged file is fully synchronized.
fn replace_private_file(path: &Path, contents: &[u8]) -> Result<(), ArcaError> {
    let staged_path = staged_preferences_path(path);
    remove_file_if_present(&staged_path, "Unable to remember vault")?;
    write_private_file(&staged_path, contents)?;

    if fs::rename(&staged_path, path).is_err() {
        let _ = fs::remove_file(&staged_path);
        return Err(ArcaError::preferences("Unable to remember vault"));
    }

    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .ok_or_else(|| ArcaError::preferences("Unable to remember vault"))?;
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    }

    Ok(())
}

/// Returns the private sibling used to stage an atomic preference replacement.
fn staged_preferences_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(PREFERENCES_FILE_NAME);

    path.with_file_name(format!(".{file_name}.tmp"))
}

/// Writes and synchronizes one private preference file.
fn write_private_file(path: &Path, contents: &[u8]) -> Result<(), ArcaError> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }

    let mut file = options
        .open(path)
        .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    file.write_all(contents)
        .and_then(|_| file.sync_all())
        .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{forget_remembered_vault, load_remembered_vault, persist_remembered_vault};

    fn unique_temp_dir() -> std::path::PathBuf {
        static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "arca-preferences-{}-{nonce}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn remembered_vault_round_trips_only_normalized_path_metadata() {
        let root = unique_temp_dir();
        let vault_directory = root.join("vaults");
        let vault_path = vault_directory.join("primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(&vault_directory).expect("create vault directory");
        fs::write(&vault_path, b"synthetic encrypted vault").expect("create vault fixture");

        persist_remembered_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");

        let remembered = load_remembered_vault(&preferences_path)
            .expect("remembered vault should load")
            .expect("remembered vault should exist");
        let json = fs::read_to_string(&preferences_path).expect("read preferences fixture");

        assert_eq!(
            remembered.path,
            vault_path.canonicalize().unwrap().display().to_string()
        );
        assert_eq!(remembered.display_name, "primary.arca");
        assert!(remembered.available);
        assert!(json.contains("rememberedVaultPath"));
        assert!(!json.contains("password"));
        assert!(!json.contains("entries"));
        assert!(!preferences_path
            .with_file_name(".preferences.json.tmp")
            .exists());

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[test]
    fn missing_remembered_vault_stays_available_for_recovery() {
        let root = unique_temp_dir();
        let vault_path = root.join("primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&vault_path, b"synthetic encrypted vault").expect("create vault fixture");
        persist_remembered_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");
        fs::remove_file(&vault_path).expect("remove remembered vault fixture");

        let remembered = load_remembered_vault(&preferences_path)
            .expect("remembered vault should load")
            .expect("remembered vault should remain stored");

        assert!(!remembered.available);
        assert_eq!(remembered.display_name, "primary.arca");

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[test]
    fn remembering_another_vault_replaces_the_previous_locator() {
        let root = unique_temp_dir();
        let first_vault = root.join("first.arca");
        let second_vault = root.join("second.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&first_vault, b"first synthetic encrypted vault")
            .expect("create first vault fixture");
        fs::write(&second_vault, b"second synthetic encrypted vault")
            .expect("create second vault fixture");

        persist_remembered_vault(&preferences_path, &first_vault)
            .expect("first vault should persist");
        persist_remembered_vault(&preferences_path, &second_vault)
            .expect("second vault should replace the first");

        let remembered = load_remembered_vault(&preferences_path)
            .expect("remembered vault should load")
            .expect("remembered vault should exist");

        assert_eq!(remembered.display_name, "second.arca");
        assert_eq!(
            remembered.path,
            second_vault.canonicalize().unwrap().display().to_string()
        );

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[test]
    fn failed_staging_keeps_the_previous_locator_readable() {
        let root = unique_temp_dir();
        let first_vault = root.join("first.arca");
        let second_vault = root.join("second.arca");
        let preferences_path = root.join("config").join("preferences.json");
        let staged_path = preferences_path.with_file_name(".preferences.json.tmp");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&first_vault, b"first synthetic encrypted vault")
            .expect("create first vault fixture");
        fs::write(&second_vault, b"second synthetic encrypted vault")
            .expect("create second vault fixture");
        persist_remembered_vault(&preferences_path, &first_vault)
            .expect("first vault should persist");
        fs::create_dir(&staged_path).expect("block the staged file path");

        let error = persist_remembered_vault(&preferences_path, &second_vault)
            .expect_err("a blocked staged path should fail before replacement");
        let remembered = load_remembered_vault(&preferences_path)
            .expect("previous preferences should remain readable")
            .expect("previous locator should remain present");

        assert_eq!(error.code, "preferences_unavailable");
        assert_eq!(remembered.display_name, "first.arca");

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[test]
    fn forgetting_removes_the_locator_and_is_idempotent() {
        let root = unique_temp_dir();
        let vault_path = root.join("primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&vault_path, b"synthetic encrypted vault").expect("create vault fixture");
        persist_remembered_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");

        forget_remembered_vault(&preferences_path).expect("remembered vault should be forgotten");
        forget_remembered_vault(&preferences_path).expect("forget should be idempotent");

        assert!(load_remembered_vault(&preferences_path)
            .expect("empty preferences should load")
            .is_none());

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[test]
    fn failed_staged_cleanup_preserves_the_remembered_locator() {
        let root = unique_temp_dir();
        let vault_path = root.join("primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        let staged_path = preferences_path.with_file_name(".preferences.json.tmp");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&vault_path, b"synthetic encrypted vault").expect("create vault fixture");
        persist_remembered_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");
        fs::create_dir(&staged_path).expect("block staged cleanup");

        let error = forget_remembered_vault(&preferences_path)
            .expect_err("blocked staged cleanup should fail before removing the locator");
        let remembered = load_remembered_vault(&preferences_path)
            .expect("remembered vault should remain readable")
            .expect("remembered locator should remain present");

        assert_eq!(error.code, "preferences_unavailable");
        assert_eq!(remembered.display_name, "primary.arca");

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[test]
    fn malformed_preferences_fail_with_a_stable_nonsecret_error() {
        let root = unique_temp_dir();
        let preferences_path = root.join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&preferences_path, br#"{"rememberedVaultPath":42}"#)
            .expect("write malformed preferences");

        let error = load_remembered_vault(&preferences_path)
            .expect_err("malformed preferences should fail closed");

        assert_eq!(error.code, "preferences_unavailable");
        assert_eq!(error.message, "Unable to load desktop preferences");
        assert!(!error
            .message
            .contains(preferences_path.to_string_lossy().as_ref()));

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[cfg(unix)]
    #[test]
    fn display_name_preserves_a_posix_backslash() {
        let root = unique_temp_dir();
        let vault_path = root.join(r"team\primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&vault_path, b"synthetic encrypted vault").expect("create vault fixture");

        let remembered = persist_remembered_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");

        assert_eq!(remembered.display_name, r"team\primary.arca");

        fs::remove_dir_all(root).expect("remove preference fixture");
    }

    #[cfg(unix)]
    #[test]
    fn preference_storage_is_private_to_the_current_user() {
        use std::os::unix::fs::PermissionsExt;

        let root = unique_temp_dir();
        let vault_path = root.join("primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&vault_path, b"synthetic encrypted vault").expect("create vault fixture");

        persist_remembered_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");

        let directory_mode = fs::metadata(preferences_path.parent().unwrap())
            .expect("read directory metadata")
            .permissions()
            .mode()
            & 0o777;
        let file_mode = fs::metadata(&preferences_path)
            .expect("read preferences metadata")
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(directory_mode, 0o700);
        assert_eq!(file_mode, 0o600);

        fs::remove_dir_all(root).expect("remove preference fixture");
    }
}
