use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::ArcaError;

const PREFERENCES_FILE_NAME: &str = "preferences.json";
const PREFERENCES_VERSION: u8 = 2;
pub const MAX_RECENT_VAULTS: usize = 5;

#[derive(Default)]
pub struct PreferencesState {
    operation: Mutex<()>,
}

impl PreferencesState {
    pub fn operation(&self) -> Result<MutexGuard<'_, ()>, ArcaError> {
        self.operation.lock().map_err(|_| ArcaError::state_lock())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopPreferences {
    version: u8,
    recent_vaults: Vec<StoredRecentVault>,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            version: PREFERENCES_VERSION,
            recent_vaults: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyDesktopPreferences {
    #[serde(default)]
    remembered_vault_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredRecentVault {
    path: PathBuf,
    last_opened_at: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentVaultDto {
    pub path: String,
    pub display_name: String,
    pub available: bool,
    pub last_opened_at: u64,
}

pub fn preferences_file_path(app: &AppHandle) -> Result<PathBuf, ArcaError> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(PREFERENCES_FILE_NAME))
        .map_err(|_| ArcaError::preferences("Unable to locate desktop preferences"))
}

pub fn load_recent_vaults(path: &Path) -> Result<Vec<RecentVaultDto>, ArcaError> {
    let (preferences, migrated) = load_preferences(path)?;

    if migrated {
        persist_preferences(path, &preferences, "Unable to migrate desktop preferences")?;
    }

    Ok(preferences
        .recent_vaults
        .into_iter()
        .map(recent_vault_dto)
        .collect())
}

pub fn persist_recent_vault(
    preferences_path: &Path,
    vault_path: &Path,
) -> Result<RecentVaultDto, ArcaError> {
    let normalized_path = fs::canonicalize(vault_path)
        .map_err(|_| ArcaError::preferences("Unable to remember vault"))?;
    let (mut preferences, _) = load_preferences(preferences_path)?;
    preferences
        .recent_vaults
        .retain(|recent| recent.path != normalized_path);
    let stored = StoredRecentVault {
        path: normalized_path,
        last_opened_at: unix_time_millis(SystemTime::now()),
    };
    preferences.recent_vaults.insert(0, stored.clone());
    preferences.recent_vaults.truncate(MAX_RECENT_VAULTS);
    persist_preferences(preferences_path, &preferences, "Unable to remember vault")?;

    Ok(recent_vault_dto(stored))
}

pub fn forget_recent_vault(preferences_path: &Path, vault_path: &Path) -> Result<(), ArcaError> {
    let (mut preferences, migrated) = load_preferences(preferences_path)?;
    let original_len = preferences.recent_vaults.len();
    preferences
        .recent_vaults
        .retain(|recent| recent.path != vault_path);

    if migrated || preferences.recent_vaults.len() != original_len {
        persist_preferences(preferences_path, &preferences, "Unable to forget vault")?;
    }

    Ok(())
}

fn recent_vault_dto(recent: StoredRecentVault) -> RecentVaultDto {
    let display_name = recent
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Recent vault")
        .to_string();
    let available = recent.path.is_file();

    RecentVaultDto {
        path: recent.path.display().to_string(),
        display_name,
        available,
        last_opened_at: recent.last_opened_at,
    }
}

fn load_preferences(path: &Path) -> Result<(DesktopPreferences, bool), ArcaError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok((DesktopPreferences::default(), false));
        }
        Err(_) => return Err(ArcaError::preferences("Unable to load desktop preferences")),
    };

    if let Ok(mut preferences) = serde_json::from_slice::<DesktopPreferences>(&bytes) {
        if preferences.version != PREFERENCES_VERSION {
            return Err(ArcaError::preferences(
                "Unsupported desktop preferences version",
            ));
        }

        normalize_recent_collection(&mut preferences.recent_vaults);
        return Ok((preferences, false));
    }

    let legacy: LegacyDesktopPreferences = serde_json::from_slice(&bytes)
        .map_err(|_| ArcaError::preferences("Unable to load desktop preferences"))?;
    let last_opened_at = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map(unix_time_millis)
        .unwrap_or_default();
    let recent_vaults = legacy
        .remembered_vault_path
        .map(|path| {
            vec![StoredRecentVault {
                path,
                last_opened_at,
            }]
        })
        .unwrap_or_default();

    Ok((
        DesktopPreferences {
            version: PREFERENCES_VERSION,
            recent_vaults,
        },
        true,
    ))
}

fn normalize_recent_collection(recents: &mut Vec<StoredRecentVault>) {
    let mut unique_paths = Vec::<PathBuf>::new();
    recents.retain(|recent| {
        if unique_paths.contains(&recent.path) {
            false
        } else {
            unique_paths.push(recent.path.clone());
            true
        }
    });
    recents.truncate(MAX_RECENT_VAULTS);
}

fn unix_time_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or_default()
}

fn persist_preferences(
    preferences_path: &Path,
    preferences: &DesktopPreferences,
    error_message: &'static str,
) -> Result<(), ArcaError> {
    let serialized =
        serde_json::to_vec(preferences).map_err(|_| ArcaError::preferences(error_message))?;
    let parent = preferences_path
        .parent()
        .ok_or_else(|| ArcaError::preferences(error_message))?;

    create_private_directory(parent, error_message)?;
    replace_private_file(preferences_path, &serialized, error_message)
}

fn remove_file_if_present(path: &Path, error_message: &'static str) -> Result<(), ArcaError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ArcaError::preferences(error_message)),
    }
}

fn create_private_directory(path: &Path, error_message: &'static str) -> Result<(), ArcaError> {
    fs::create_dir_all(path).map_err(|_| ArcaError::preferences(error_message))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| ArcaError::preferences(error_message))?;
    }

    Ok(())
}

fn replace_private_file(
    path: &Path,
    contents: &[u8],
    error_message: &'static str,
) -> Result<(), ArcaError> {
    let staged_path = staged_preferences_path(path);
    remove_file_if_present(&staged_path, error_message)?;
    write_private_file(&staged_path, contents, error_message)?;

    if fs::rename(&staged_path, path).is_err() {
        let _ = fs::remove_file(&staged_path);
        return Err(ArcaError::preferences(error_message));
    }

    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .ok_or_else(|| ArcaError::preferences(error_message))?;
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| ArcaError::preferences(error_message))?;
    }

    Ok(())
}

fn staged_preferences_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(PREFERENCES_FILE_NAME);

    path.with_file_name(format!(".{file_name}.tmp"))
}

fn write_private_file(
    path: &Path,
    contents: &[u8],
    error_message: &'static str,
) -> Result<(), ArcaError> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }

    let mut file = options
        .open(path)
        .map_err(|_| ArcaError::preferences(error_message))?;
    file.write_all(contents)
        .and_then(|_| file.sync_all())
        .map_err(|_| ArcaError::preferences(error_message))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|_| ArcaError::preferences(error_message))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{forget_recent_vault, load_recent_vaults, persist_recent_vault, MAX_RECENT_VAULTS};

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

    fn create_vault_fixture(root: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = root.join(name);
        fs::create_dir_all(root).expect("create fixture root");
        fs::write(&path, b"synthetic encrypted vault").expect("create vault fixture");
        path
    }

    #[test]
    fn legacy_single_locator_migrates_without_losing_the_vault() {
        let root = unique_temp_dir();
        let vault_path = create_vault_fixture(&root.join("vaults"), "primary.arca");
        let preferences_path = root.join("config").join("preferences.json");
        fs::create_dir_all(preferences_path.parent().unwrap()).expect("create config directory");
        fs::write(
            &preferences_path,
            serde_json::json!({ "rememberedVaultPath": vault_path }).to_string(),
        )
        .expect("write legacy preferences");

        let recent = load_recent_vaults(&preferences_path).expect("migrate preferences");
        let json = fs::read_to_string(&preferences_path).expect("read migrated preferences");

        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].display_name, "primary.arca");
        assert!(json.contains("\"version\":2"));
        assert!(json.contains("recentVaults"));
        assert!(!json.contains("rememberedVaultPath"));

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn successful_opens_are_ordered_replaced_and_bounded() {
        let root = unique_temp_dir();
        let preferences_path = root.join("config").join("preferences.json");
        let mut vaults = Vec::new();

        for index in 0..=MAX_RECENT_VAULTS {
            let vault = create_vault_fixture(&root.join("vaults"), &format!("{index}.arca"));
            persist_recent_vault(&preferences_path, &vault).expect("remember vault");
            vaults.push(vault);
        }

        let recent = load_recent_vaults(&preferences_path).expect("load bounded recents");
        assert_eq!(recent.len(), MAX_RECENT_VAULTS);
        assert_eq!(recent[0].display_name, "5.arca");
        assert!(!recent.iter().any(|item| item.display_name == "0.arca"));

        persist_recent_vault(&preferences_path, &vaults[2]).expect("promote existing vault");
        let promoted = load_recent_vaults(&preferences_path).expect("load promoted recents");
        assert_eq!(promoted[0].display_name, "2.arca");
        assert_eq!(
            promoted
                .iter()
                .filter(|item| item.display_name == "2.arca")
                .count(),
            1
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn forgetting_one_recent_never_touches_its_vault_file() {
        let root = unique_temp_dir();
        let preferences_path = root.join("config").join("preferences.json");
        let first = create_vault_fixture(&root.join("vaults"), "first.arca");
        let second = create_vault_fixture(&root.join("vaults"), "second.arca");
        persist_recent_vault(&preferences_path, &first).expect("remember first");
        persist_recent_vault(&preferences_path, &second).expect("remember second");

        forget_recent_vault(&preferences_path, &first.canonicalize().unwrap())
            .expect("forget first locator");

        let recent = load_recent_vaults(&preferences_path).expect("load remaining recents");
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].display_name, "second.arca");
        assert!(first.is_file());
        assert!(second.is_file());

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn unavailable_recent_remains_available_for_recovery() {
        let root = unique_temp_dir();
        let preferences_path = root.join("config").join("preferences.json");
        let vault_path = create_vault_fixture(&root.join("vaults"), "primary.arca");
        persist_recent_vault(&preferences_path, &vault_path).expect("remember vault");
        fs::remove_file(&vault_path).expect("remove vault fixture");

        let recent = load_recent_vaults(&preferences_path).expect("load unavailable recent");
        assert_eq!(recent.len(), 1);
        assert!(!recent[0].available);

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn malformed_preferences_fail_with_a_stable_nonsecret_error() {
        let root = unique_temp_dir();
        let preferences_path = root.join("preferences.json");
        fs::create_dir_all(&root).expect("create fixture root");
        fs::write(&preferences_path, br#"{"recentVaults":42}"#)
            .expect("write malformed preferences");

        let error = load_recent_vaults(&preferences_path)
            .expect_err("malformed preferences should fail closed");

        assert_eq!(error.code, "preferences_unavailable");
        assert_eq!(error.message, "Unable to load desktop preferences");
        assert!(!error
            .message
            .contains(preferences_path.to_string_lossy().as_ref()));

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn display_name_preserves_a_posix_backslash() {
        let root = unique_temp_dir();
        let preferences_path = root.join("config").join("preferences.json");
        let vault_path = create_vault_fixture(&root, r"team\primary.arca");
        let recent = persist_recent_vault(&preferences_path, &vault_path)
            .expect("remembered vault should persist");

        assert_eq!(recent.display_name, r"team\primary.arca");
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(unix)]
    #[test]
    fn preference_storage_remains_private_to_the_current_user() {
        use std::os::unix::fs::PermissionsExt;

        let root = unique_temp_dir();
        let preferences_path = root.join("config").join("preferences.json");
        let vault_path = create_vault_fixture(&root, "primary.arca");
        persist_recent_vault(&preferences_path, &vault_path).expect("remember vault");

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
        fs::remove_dir_all(root).expect("remove fixture");
    }
}
