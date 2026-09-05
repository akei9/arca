use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;
use vault_api::{
    CreateEntryRequest as CreateEntryDto, EntryMutation as UpdateEntryDto, EntryView as EntryDto,
    GeneratedSecret as GeneratedPassword, GeneratorMode as GeneratorModeDto,
    GeneratorParams as GeneratorConfigDto, RevisionView as RevisionDto, SecretString,
    VaultSummary as VaultInfo,
};
use vault_core::entry as core_entry;
use vault_core::generator as core_generator;
use vault_core::types::EntryRevision;
use vault_core::vault as core_vault;
use vault_core::{GeneratorConfig, GeneratorMode, VaultEntry, VaultMeta};
use zeroize::Zeroizing;

use crate::error::ArcaError;
use crate::state::{AppState, Settings};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PathSuggestionDto {
    pub name: String,
    pub path: String,
    pub kind: PathSuggestionKind,
    pub vault_candidate: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PathSuggestionKind {
    Directory,
    File,
}

#[tauri::command]
/// Opens an existing vault and stores the unlocked session in memory.
pub fn unlock_vault(
    path: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<VaultInfo, ArcaError> {
    let vault_path = PathBuf::from(&path);
    let (meta, entries) = core_vault::open_vault(&vault_path, &password)?;
    let info = vault_info(&vault_path, &meta, entries.len());
    let mut session = state.session()?;
    session.unlock(vault_path, Zeroizing::new(password), meta, entries);

    Ok(info)
}

#[tauri::command]
/// Clears the in-memory vault session and any stored master password.
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), ArcaError> {
    state.session()?.lock();
    Ok(())
}

#[tauri::command]
/// Creates a new empty vault and opens it as the active session.
pub fn create_vault(
    path: String,
    password: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), ArcaError> {
    let vault_path = PathBuf::from(path);
    let meta = core_vault::create_vault(&vault_path, &password, &name)?;
    let mut session = state.session()?;
    session.unlock(vault_path, Zeroizing::new(password), meta, Vec::new());

    Ok(())
}

#[tauri::command]
/// Returns metadata-only entry views for the unlocked vault.
pub fn list_entries(state: State<'_, AppState>) -> Result<Vec<EntryDto>, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    Ok(session.entries.iter().map(entry_metadata_dto).collect())
}

#[tauri::command]
/// Returns one metadata-only entry view by id.
pub fn get_entry(id: String, state: State<'_, AppState>) -> Result<EntryDto, ArcaError> {
    get_entry_in_state(&id, state.inner())
}

/// Looks up a metadata-only entry view in application state.
fn get_entry_in_state(id: &str, state: &AppState) -> Result<EntryDto, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    session
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .map(entry_metadata_dto)
        .ok_or_else(|| ArcaError::not_found("Entry"))
}

#[tauri::command]
/// Reveals the current password for one entry through an explicit secret-bearing call.
pub fn reveal_entry_password(
    id: String,
    state: State<'_, AppState>,
) -> Result<SecretString, ArcaError> {
    reveal_entry_password_in_state(&id, state.inner())
}

/// Loads an entry password from the unlocked session without logging it.
fn reveal_entry_password_in_state(id: &str, state: &AppState) -> Result<SecretString, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    session
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| SecretString::new(entry.password.clone()))
        .ok_or_else(|| ArcaError::not_found("Entry"))
}

#[tauri::command]
/// Returns metadata-only revision history for one entry.
pub fn get_entry_revisions(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<RevisionDto>, ArcaError> {
    get_entry_revisions_in_state(&id, state.inner())
}

/// Builds revision views and marks whether each revision changed the password.
fn get_entry_revisions_in_state(id: &str, state: &AppState) -> Result<Vec<RevisionDto>, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    session
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| {
            entry
                .revisions
                .iter()
                .enumerate()
                .map(|(index, revision)| {
                    let newer_password = if index == 0 {
                        entry.password.as_str()
                    } else {
                        entry.revisions[index - 1].password.as_str()
                    };
                    revision_dto_from_revision(
                        revision,
                        revision.password.as_str() != newer_password,
                    )
                })
                .collect()
        })
        .ok_or_else(|| ArcaError::not_found("Entry"))
}

#[tauri::command]
/// Reveals a historical revision password through an explicit secret-bearing call.
pub fn reveal_entry_revision_password(
    id: String,
    index: usize,
    state: State<'_, AppState>,
) -> Result<SecretString, ArcaError> {
    reveal_entry_revision_password_in_state(&id, index, state.inner())
}

/// Loads one revision password from the unlocked session without logging it.
fn reveal_entry_revision_password_in_state(
    id: &str,
    index: usize,
    state: &AppState,
) -> Result<SecretString, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    let entry = session
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| ArcaError::not_found("Entry"))?;

    entry
        .revisions
        .get(index)
        .map(|revision| SecretString::new(revision.password.clone()))
        .ok_or_else(|| ArcaError::not_found("Revision"))
}

#[tauri::command]
/// Creates a vault entry from the public request contract and persists it.
pub fn create_entry(
    data: CreateEntryDto,
    state: State<'_, AppState>,
) -> Result<EntryDto, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    validate_entry_password(data.password.expose_secret())?;

    let mut entry =
        core_entry::create_entry(&data.title, &data.username, data.password.expose_secret());
    entry.collection = data.collection;
    entry.url = data.url;
    entry.notes = data.notes;
    entry.tags = data.tags;
    session.entries.push(entry.clone());
    persist_session(&session)?;
    session.touch();

    Ok(entry_metadata_dto(&entry))
}

#[tauri::command]
/// Applies an entry mutation from the public contract and persists the vault.
pub fn update_entry(
    id: String,
    data: UpdateEntryDto,
    state: State<'_, AppState>,
) -> Result<EntryDto, ArcaError> {
    let revision_limit = state.settings()?.entry_revision_limit;
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    validate_optional_entry_password(data.password.as_ref().map(SecretString::expose_secret))?;

    let dto = {
        let entry = session
            .entries
            .iter_mut()
            .find(|entry| entry.id == id)
            .ok_or_else(|| ArcaError::not_found("Entry"))?;
        core_entry::update_entry_with_revision_limit(
            entry,
            entry_patch_from_dto(data),
            revision_limit,
        );
        entry_metadata_dto(entry)
    };
    persist_session(&session)?;
    session.touch();

    Ok(dto)
}

#[tauri::command]
/// Deletes one entry from the active vault and persists the change.
pub fn delete_entry(id: String, state: State<'_, AppState>) -> Result<(), ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;

    let original_len = session.entries.len();
    session.entries.retain(|entry| entry.id != id);

    if session.entries.len() == original_len {
        return Err(ArcaError::not_found("Entry"));
    }

    persist_session(&session)?;
    session.touch();

    Ok(())
}

#[tauri::command]
/// Searches unlocked entries and returns metadata-only views.
pub fn search_entries(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryDto>, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    Ok(core_entry::search_entries(&session.entries, &query)
        .into_iter()
        .map(entry_metadata_dto)
        .collect())
}

#[tauri::command]
/// Generates a password from public generator parameters.
pub fn generate_password(config: GeneratorConfigDto) -> Result<GeneratedPassword, ArcaError> {
    let config = generator_config_from_dto(config);
    let password = core_generator::generate_password(&config);
    let entropy_bits = core_generator::calculate_entropy(&password, &config);

    Ok(GeneratedPassword::new(password, entropy_bits))
}

#[tauri::command]
/// Suggests filesystem paths for the unlock/create vault picker.
pub fn suggest_paths(partial: String) -> Result<Vec<PathSuggestionDto>, ArcaError> {
    if partial.len() > 4096 {
        return Err(ArcaError::invalid_input("Path query is too long"));
    }

    Ok(suggest_paths_for(&partial))
}

#[tauri::command]
/// Returns persisted desktop settings.
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, ArcaError> {
    Ok(state.settings()?.clone())
}

#[tauri::command]
/// Updates desktop settings and trims persisted revision history when needed.
pub fn update_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), ArcaError> {
    update_settings_in_state(settings, state.inner())
}

/// Applies settings to state and persists any revision-retention changes atomically.
fn update_settings_in_state(mut settings: Settings, state: &AppState) -> Result<(), ArcaError> {
    settings.entry_revision_limit = settings
        .entry_revision_limit
        .min(core_entry::MAX_ENTRY_REVISION_LIMIT);

    {
        let mut session = state.session()?;
        if ensure_unlocked(&session).is_ok() {
            // Atomic persistence needs a staged copy; keep it wrapped so failed saves clear copied secrets.
            let mut staged_entries = Zeroizing::new(session.entries.clone());
            let mut trimmed = false;

            for entry in staged_entries.iter_mut() {
                trimmed |= core_entry::trim_entry_revisions(entry, settings.entry_revision_limit);
            }

            if trimmed {
                persist_entries(&session, staged_entries.as_slice())?;
                session.entries = std::mem::take(staged_entries.as_mut());
                session.touch();
            }
        }
    }

    *state.settings()? = settings;
    Ok(())
}

/// Converts a core entry into the metadata-only public entry contract.
fn entry_metadata_dto(entry: &VaultEntry) -> EntryDto {
    EntryDto {
        id: entry.id.clone(),
        title: entry.title.clone(),
        username: entry.username.clone(),
        collection: entry.collection.clone(),
        url: entry.url.clone(),
        notes: entry.notes.clone(),
        tags: entry.tags.clone(),
        created_at: entry.created_at.clone(),
        updated_at: entry.updated_at.clone(),
        revision_count: entry.revisions.len(),
    }
}

/// Converts a core revision into the metadata-only public revision contract.
fn revision_dto_from_revision(revision: &EntryRevision, password_changed: bool) -> RevisionDto {
    RevisionDto {
        captured_at: revision.captured_at.clone(),
        updated_at: revision.updated_at.clone(),
        title: revision.title.clone(),
        username: revision.username.clone(),
        collection: revision.collection.clone(),
        url: revision.url.clone(),
        notes: revision.notes.clone(),
        tags: revision.tags.clone(),
        password_changed,
    }
}

/// Converts a public entry mutation into the core patch model.
fn entry_patch_from_dto(value: UpdateEntryDto) -> core_entry::EntryPatch {
    core_entry::EntryPatch {
        title: value.title,
        username: value.username,
        password: value.password.map(SecretString::into_inner),
        collection: value.collection,
        url: value.url,
        notes: value.notes,
        tags: value.tags,
    }
}

/// Applies core defaults to partial public generator parameters.
fn generator_config_from_dto(value: GeneratorConfigDto) -> GeneratorConfig {
    let default = GeneratorConfig::default();

    GeneratorConfig {
        length: value.length.unwrap_or(default.length),
        uppercase: value.uppercase.unwrap_or(default.uppercase),
        lowercase: value.lowercase.unwrap_or(default.lowercase),
        digits: value.digits.unwrap_or(default.digits),
        symbols: value.symbols.unwrap_or(default.symbols),
        exclude_ambiguous: value.exclude_ambiguous.unwrap_or(default.exclude_ambiguous),
        mode: value
            .mode
            .map(generator_mode_from_dto)
            .unwrap_or(default.mode),
    }
}

/// Converts the public generator mode into the core generator mode.
fn generator_mode_from_dto(value: GeneratorModeDto) -> GeneratorMode {
    match value {
        GeneratorModeDto::Random => GeneratorMode::Random,
        GeneratorModeDto::Passphrase => GeneratorMode::Passphrase,
    }
}

/// Ensures a command is operating on an unlocked session.
fn ensure_unlocked(session: &crate::state::SessionState) -> Result<(), ArcaError> {
    if session.meta.is_some() && session.vault_path.is_some() && session.master_password.is_some() {
        Ok(())
    } else {
        Err(ArcaError::locked())
    }
}

/// Validates an optional replacement entry password.
fn validate_optional_entry_password(password: Option<&str>) -> Result<(), ArcaError> {
    if let Some(password) = password {
        validate_entry_password(password)?;
    }

    Ok(())
}

/// Rejects empty entry passwords at the desktop command boundary.
fn validate_entry_password(password: &str) -> Result<(), ArcaError> {
    if password.is_empty() {
        Err(ArcaError::invalid_input(
            "Entry passwords cannot be empty; passwordless entries are unsupported",
        ))
    } else {
        Ok(())
    }
}

/// Persists the current in-memory entries for the active session.
fn persist_session(session: &crate::state::SessionState) -> Result<(), ArcaError> {
    persist_entries(session, &session.entries)
}

/// Writes the provided entry set using the active vault path and master password.
fn persist_entries(
    session: &crate::state::SessionState,
    entries: &[VaultEntry],
) -> Result<(), ArcaError> {
    let path = session
        .vault_path
        .as_deref()
        .ok_or_else(ArcaError::locked)?;
    let meta = session.meta.as_ref().ok_or_else(ArcaError::locked)?;
    let password = session
        .master_password
        .as_ref()
        .ok_or_else(ArcaError::locked)?;

    core_vault::save_vault(path, password.as_str(), meta, entries)?;

    Ok(())
}

/// Builds the public vault summary returned to the frontend.
fn vault_info(path: &Path, meta: &VaultMeta, entry_count: usize) -> VaultInfo {
    VaultInfo::new(
        meta.name.clone(),
        path.display().to_string(),
        entry_count,
        meta.modified_at.clone(),
    )
}

/// Expands and ranks path suggestions from a partial user input.
fn suggest_paths_for(partial: &str) -> Vec<PathSuggestionDto> {
    let trimmed = partial.trim_start();
    let expanded = expand_path(trimmed);
    let trailing_separator = trimmed.ends_with('/') || trimmed.ends_with('\\');
    let home = home_dir();

    let (search_dir, prefix) = if trimmed.is_empty() {
        (home.unwrap_or_else(|| PathBuf::from("/")), String::new())
    } else if trimmed == "~" || trailing_separator {
        (expanded, String::new())
    } else {
        let parent = expanded
            .parent()
            .map(Path::to_path_buf)
            .filter(|path| !path.as_os_str().is_empty())
            .or_else(|| {
                if expanded.is_absolute() {
                    Some(PathBuf::from("/"))
                } else {
                    home
                }
            })
            .unwrap_or_else(|| PathBuf::from("."));
        let prefix = expanded
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_lowercase();

        (parent, prefix)
    };

    let mut suggestions = match fs::read_dir(search_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|entry| path_suggestion(entry.path(), &prefix))
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    suggestions.sort_by(|left, right| {
        let left_rank = suggestion_rank(left);
        let right_rank = suggestion_rank(right);

        left_rank
            .cmp(&right_rank)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    suggestions.truncate(8);

    suggestions
}

/// Converts one filesystem path into a path suggestion when possible.
fn path_suggestion(path: PathBuf, prefix: &str) -> Option<PathSuggestionDto> {
    let metadata = fs::metadata(&path).ok()?;

    if !metadata.is_dir() && !metadata.is_file() {
        return None;
    }

    let raw_name = path.file_name()?.to_str()?;

    if !prefix.is_empty() && !raw_name.to_lowercase().starts_with(prefix) {
        return None;
    }

    let is_dir = metadata.is_dir();
    let name = if is_dir {
        format!("{raw_name}/")
    } else {
        raw_name.to_string()
    };
    let mut display_path = path.display().to_string();

    if is_dir && !display_path.ends_with(std::path::MAIN_SEPARATOR) {
        display_path.push(std::path::MAIN_SEPARATOR);
    }

    Some(PathSuggestionDto {
        vault_candidate: !is_dir && is_vault_candidate(raw_name),
        kind: if is_dir {
            PathSuggestionKind::Directory
        } else {
            PathSuggestionKind::File
        },
        name,
        path: display_path,
    })
}

/// Orders likely vault files before directories and other files.
fn suggestion_rank(suggestion: &PathSuggestionDto) -> u8 {
    match (&suggestion.kind, suggestion.vault_candidate) {
        (PathSuggestionKind::File, true) => 0,
        (PathSuggestionKind::Directory, _) => 1,
        (PathSuggestionKind::File, false) => 2,
    }
}

/// Checks whether a path basename looks like a supported vault file.
fn is_vault_candidate(name: &str) -> bool {
    let lower = name.to_lowercase();

    lower.ends_with(".arca") || lower.ends_with(".kdbx")
}

/// Expands `~` prefixes in user-provided paths.
fn expand_path(value: &str) -> PathBuf {
    if value == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from(value));
    }

    if let Some(rest) = value.strip_prefix("~/") {
        return home_dir()
            .map(|home| home.join(rest))
            .unwrap_or_else(|| PathBuf::from(value));
    }

    let path = PathBuf::from(value);

    if path.is_absolute() {
        path
    } else {
        home_dir()
            .map(|home| home.join(path))
            .unwrap_or_else(|| PathBuf::from(value))
    }
}

/// Returns the current user's home directory when the process can resolve it.
fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{
        entry_metadata_dto, entry_patch_from_dto, generator_config_from_dto, get_entry_in_state,
        get_entry_revisions_in_state, reveal_entry_password_in_state,
        reveal_entry_revision_password_in_state, revision_dto_from_revision, suggest_paths_for,
        update_settings_in_state, validate_entry_password, validate_optional_entry_password,
        CreateEntryDto, GeneratorConfigDto, UpdateEntryDto,
    };
    use crate::error::ArcaError;
    use crate::state::{AppState, Settings};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use vault_api::SecretString;
    use vault_core::entry::{create_entry, update_entry};
    use vault_core::entry::{EntryPatch, DEFAULT_ENTRY_REVISION_LIMIT};
    use vault_core::types::VaultMeta;
    use vault_core::vault as core_vault;
    use zeroize::Zeroizing;

    #[test]
    fn entry_dto_masks_password_for_list_views() {
        let credential = test_credential("current");
        let previous_credential = test_credential("previous");
        let mut entry = create_entry("GitHub", "arca", &credential);
        entry.revisions.push(vault_core::types::EntryRevision {
            captured_at: "2026-06-11T00:00:00+00:00".to_string(),
            title: "GitHub".to_string(),
            username: "arca".to_string(),
            password: previous_credential,
            collection: None,
            url: None,
            notes: None,
            tags: Vec::new(),
            updated_at: "2026-06-11T00:00:00+00:00".to_string(),
        });

        let masked = entry_metadata_dto(&entry);
        let json = serde_json::to_string(&masked).expect("entry dto should serialize");

        assert!(!json.contains("\"password\""));
        assert!(!json.contains(credential.as_str()));
        assert_eq!(masked.revision_count, 1);
    }

    #[test]
    fn entry_metadata_dto_never_serializes_password() {
        let credential = test_credential("current");
        let entry = create_entry("GitHub", "arca", &credential);

        let dto = entry_metadata_dto(&entry);
        let json = serde_json::to_string(&dto).expect("entry dto should serialize");

        assert!(!json.contains("\"password\""));
        assert!(!json.contains(credential.as_str()));
    }

    #[test]
    fn get_entry_returns_metadata_without_password() {
        let state = AppState::default();
        let credential = test_credential("current");
        let entry = create_entry("GitHub", "arca", &credential);
        let entry_id = entry.id.clone();
        unlock_with_entry(&state, entry);

        let dto = get_entry_in_state(&entry_id, &state)
            .expect("entry metadata should be returned for an unlocked entry");

        let json = serde_json::to_string(&dto).expect("entry dto should serialize");
        assert!(!json.contains("\"password\""));
        assert!(!json.contains(credential.as_str()));
    }

    #[test]
    fn reveal_entry_password_returns_plaintext_for_unlocked_entry() {
        let state = AppState::default();
        let credential = test_credential("current");
        let entry = create_entry("GitHub", "arca", &credential);
        let entry_id = entry.id.clone();
        unlock_with_entry(&state, entry);

        let password = reveal_entry_password_in_state(&entry_id, &state)
            .expect("entry password should be revealed for an unlocked entry");

        assert_eq!(password.expose_secret(), credential);
    }

    #[test]
    fn reveal_entry_password_rejects_missing_entry() {
        let state = AppState::default();
        unlock_with_entry(
            &state,
            create_entry("GitHub", "arca", &test_credential("current")),
        );

        let error = expect_error(
            reveal_entry_password_in_state("missing", &state),
            "a missing entry should not reveal a password",
        );

        assert_eq!(error.code, "not_found");
    }

    #[test]
    fn get_entry_revisions_returns_metadata_for_unlocked_entry() {
        let state = AppState::default();
        let entry = create_entry_with_revisions(3);
        let entry_id = entry.id.clone();
        let expected_titles: Vec<String> = entry
            .revisions
            .iter()
            .map(|revision| revision.title.clone())
            .collect();
        unlock_with_entry(&state, entry);

        let revisions = get_entry_revisions_in_state(&entry_id, &state)
            .expect("revisions should be returned for an unlocked entry");

        assert_eq!(revisions.len(), 3);
        let titles: Vec<String> = revisions
            .iter()
            .map(|revision| revision.title.clone())
            .collect();
        assert_eq!(titles, expected_titles);
    }

    #[test]
    fn get_entry_revisions_flags_password_changes() {
        let state = AppState::default();
        let mut entry = create_entry("GitHub", "arca", &test_credential("current"));
        update_entry(
            &mut entry,
            EntryPatch {
                password: Some(test_credential("rotated")),
                ..EntryPatch::default()
            },
        );
        update_entry(
            &mut entry,
            EntryPatch {
                title: Some("GitHub Enterprise".to_string()),
                ..EntryPatch::default()
            },
        );
        let entry_id = entry.id.clone();
        unlock_with_entry(&state, entry);

        let revisions = get_entry_revisions_in_state(&entry_id, &state)
            .expect("revisions should be returned for an unlocked entry");

        assert_eq!(revisions.len(), 2);
        assert!(!revisions[0].password_changed);
        assert!(revisions[1].password_changed);
    }

    #[test]
    fn revision_dto_serialization_excludes_password() {
        let entry = create_entry_with_revisions(1);
        let revision = &entry.revisions[0];
        let dto = revision_dto_from_revision(revision, true);

        let json = serde_json::to_string(&dto).expect("revision dto should serialize");

        assert!(!json.contains("\"password\""));
        assert!(!json.contains(revision.password.as_str()));
    }

    #[test]
    fn reveal_entry_revision_password_returns_plaintext_for_unlocked_entry() {
        let state = AppState::default();
        let entry = create_entry_with_revisions(3);
        let entry_id = entry.id.clone();
        let expected = entry.revisions[0].password.clone();
        unlock_with_entry(&state, entry);

        let password = reveal_entry_revision_password_in_state(&entry_id, 0, &state)
            .expect("revision password should be revealed for an unlocked entry");

        assert_eq!(password.expose_secret(), expected);
    }

    #[test]
    fn reveal_entry_revision_password_rejects_out_of_range_index() {
        let state = AppState::default();
        let entry = create_entry_with_revisions(2);
        let entry_id = entry.id.clone();
        unlock_with_entry(&state, entry);

        let error = expect_error(
            reveal_entry_revision_password_in_state(&entry_id, 99, &state),
            "an out-of-range revision index should error",
        );

        assert_eq!(error.code, "not_found");
    }

    #[test]
    fn revision_commands_require_unlocked_vault() {
        let state = AppState::default();

        let current_reveal_error = expect_error(
            reveal_entry_password_in_state("missing", &state),
            "a locked vault should not reveal an entry password",
        );
        let list_error = expect_error(
            get_entry_revisions_in_state("missing", &state),
            "a locked vault should not return revisions",
        );
        let reveal_error = expect_error(
            reveal_entry_revision_password_in_state("missing", 0, &state),
            "a locked vault should not reveal a revision password",
        );

        assert_eq!(current_reveal_error.code, "vault_locked");
        assert_eq!(list_error.code, "vault_locked");
        assert_eq!(reveal_error.code, "vault_locked");
    }

    #[test]
    fn generator_config_dto_uses_secure_defaults() {
        let config = generator_config_from_dto(GeneratorConfigDto::default());

        assert_eq!(config.length, 24);
        assert!(config.uppercase);
        assert!(config.lowercase);
        assert!(config.digits);
        assert!(config.symbols);
    }

    #[test]
    fn create_entry_dto_accepts_missing_tags() {
        let json = serde_json::json!({
            "title": "GitHub",
            "username": "arca",
            "password": test_credential("current"),
        })
        .to_string();
        let dto: CreateEntryDto =
            serde_json::from_str(&json).expect("create entry dto should deserialize");

        assert!(dto.collection.is_none());
        assert!(dto.tags.is_empty());
    }

    #[test]
    fn entry_password_policy_rejects_empty_create_passwords() {
        let error = validate_entry_password("").expect_err("empty passwords should be rejected");

        assert_eq!(error.code, "invalid_input");
        assert!(validate_entry_password(&test_credential("valid")).is_ok());
    }

    #[test]
    fn update_entry_dto_omitted_password_means_unchanged() {
        let dto: UpdateEntryDto =
            serde_json::from_str(r#"{"title":"GitHub"}"#).expect("update dto should deserialize");
        let patch = entry_patch_from_dto(dto);

        assert!(patch.password.is_none());
        assert!(validate_optional_entry_password(patch.password.as_deref()).is_ok());
    }

    #[test]
    fn entry_password_policy_rejects_empty_update_passwords() {
        let dto = UpdateEntryDto {
            password: Some(SecretString::new(String::new())),
            ..UpdateEntryDto::default()
        };
        let error = validate_optional_entry_password(
            dto.password.as_ref().map(SecretString::expose_secret),
        )
        .expect_err("empty update passwords should be rejected");

        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn update_settings_persistence_failure_leaves_session_and_settings_unchanged() {
        let state = AppState::default();
        let password = test_credential("vault");
        let entry = create_entry_with_revisions(3);
        let entry_id = entry.id.clone();
        let root = unique_temp_dir();
        let missing_parent = root.join("missing-parent").join("vault.arca");
        let original_settings = Settings {
            entry_revision_limit: DEFAULT_ENTRY_REVISION_LIMIT,
            ..Settings::default()
        };

        *state.settings().expect("settings lock should be available") = original_settings.clone();
        state
            .session()
            .expect("session lock should be available")
            .unlock(
                missing_parent,
                Zeroizing::new(password),
                test_meta(),
                vec![entry.clone()],
            );

        let error = update_settings_in_state(
            Settings {
                entry_revision_limit: 1,
                ..Settings::default()
            },
            &state,
        )
        .expect_err("persisting into a missing parent directory should fail");

        assert_eq!(error.code, "io_error");
        let session = state.session().expect("session lock should be available");
        let stored_entry = session
            .entries
            .iter()
            .find(|stored| stored.id == entry_id)
            .expect("entry should remain in memory");
        assert_eq!(stored_entry.revisions.len(), entry.revisions.len());
        drop(session);
        assert_eq!(
            *state.settings().expect("settings lock should be available"),
            original_settings
        );

        fs::remove_dir_all(root).expect("remove failed persistence fixture");
    }

    #[test]
    fn update_settings_lowering_revision_limit_persists_trimmed_revisions() {
        let state = AppState::default();
        let root = unique_temp_dir();
        let vault_path = root.join("revision-retention.arca");
        let password = test_credential("vault");
        let meta = core_vault::create_vault(&vault_path, &password, "Revision Retention")
            .expect("create vault");
        let entry = create_entry_with_revisions(4);

        state
            .session()
            .expect("session lock should be available")
            .unlock(
                vault_path.clone(),
                Zeroizing::new(password.clone()),
                meta,
                vec![entry],
            );

        update_settings_in_state(
            Settings {
                entry_revision_limit: 2,
                ..Settings::default()
            },
            &state,
        )
        .expect("settings update should persist trimmed revisions");

        assert_eq!(
            state
                .session()
                .expect("session lock should be available")
                .entries[0]
                .revisions
                .len(),
            2
        );
        assert_eq!(
            state
                .settings()
                .expect("settings lock should be available")
                .entry_revision_limit,
            2
        );

        let (_meta, persisted_entries) =
            core_vault::open_vault(&vault_path, &password).expect("open persisted vault");
        assert_eq!(persisted_entries[0].revisions.len(), 2);

        fs::remove_dir_all(root).expect("remove revision retention fixture");
    }

    fn test_credential(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        format!("test-credential-{label}-{nanos}")
    }

    fn create_entry_with_revisions(count: usize) -> vault_core::VaultEntry {
        let mut entry = create_entry("GitHub", "arca", &test_credential("current"));

        for index in 0..count {
            update_entry(
                &mut entry,
                EntryPatch {
                    title: Some(format!("GitHub {index}")),
                    password: Some(test_credential(&format!("revision-{index}"))),
                    ..EntryPatch::default()
                },
            );
        }

        entry
    }

    fn test_meta() -> VaultMeta {
        VaultMeta {
            name: "Test Vault".to_string(),
            created_at: "2026-08-19T00:00:00+00:00".to_string(),
            modified_at: "2026-08-19T00:00:00+00:00".to_string(),
        }
    }

    fn unlock_with_entry(state: &AppState, entry: vault_core::VaultEntry) {
        state
            .session()
            .expect("session lock should be available")
            .unlock(
                PathBuf::from("in-memory.arca"),
                Zeroizing::new(test_credential("vault")),
                test_meta(),
                vec![entry],
            );
    }

    fn expect_error<T>(result: Result<T, ArcaError>, message: &str) -> ArcaError {
        match result {
            Ok(_) => panic!("{message}"),
            Err(error) => error,
        }
    }

    #[test]
    fn path_suggestions_prioritize_vault_files_then_directories() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("apps")).expect("create directory fixture");
        fs::write(root.join("alpha.arca"), "").expect("create vault fixture");
        fs::write(root.join("alpha.txt"), "").expect("create file fixture");

        let partial = root.join("a").display().to_string();
        let suggestions = suggest_paths_for(&partial);
        let names = suggestions
            .iter()
            .map(|suggestion| suggestion.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["alpha.arca", "apps/", "alpha.txt"]);
        assert!(suggestions[0].vault_candidate);

        fs::remove_dir_all(root).expect("remove suggestion fixture");
    }

    #[test]
    fn path_suggestions_complete_inside_trailing_directory() {
        let root = unique_temp_dir();
        let vaults = root.join("vaults");
        fs::create_dir_all(&vaults).expect("create directory fixture");
        fs::write(vaults.join("primary.kdbx"), "").expect("create vault fixture");

        let partial = format!("{}{}", vaults.display(), std::path::MAIN_SEPARATOR);
        let suggestions = suggest_paths_for(&partial);

        assert_eq!(suggestions[0].name, "primary.kdbx");
        assert!(suggestions[0].vault_candidate);

        fs::remove_dir_all(root).expect("remove suggestion fixture");
    }

    fn unique_temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("arca-path-suggest-{nanos}"));

        fs::create_dir_all(&dir).expect("create temp suggestion dir");
        dir
    }
}
