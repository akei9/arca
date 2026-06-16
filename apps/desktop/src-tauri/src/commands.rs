use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;
use vault_core::entry as core_entry;
use vault_core::generator as core_generator;
use vault_core::vault as core_vault;
use vault_core::{GeneratorConfig, GeneratorMode, VaultEntry, VaultMeta};
use zeroize::Zeroizing;

use crate::error::ArcaError;
use crate::state::{AppState, Settings};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VaultInfo {
    pub name: String,
    pub path: String,
    pub entry_count: usize,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntryDto {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CreateEntryDto {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct UpdateEntryDto {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorConfigDto {
    pub length: Option<usize>,
    pub uppercase: Option<bool>,
    pub lowercase: Option<bool>,
    pub digits: Option<bool>,
    pub symbols: Option<bool>,
    pub exclude_ambiguous: Option<bool>,
    pub mode: Option<GeneratorModeDto>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GeneratorModeDto {
    Random,
    Passphrase,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedPassword {
    pub password: String,
    pub entropy_bits: f64,
}

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
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), ArcaError> {
    state.session()?.lock();
    Ok(())
}

#[tauri::command]
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
pub fn list_entries(state: State<'_, AppState>) -> Result<Vec<EntryDto>, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    Ok(session
        .entries
        .iter()
        .map(|entry| EntryDto::from_entry(entry, false))
        .collect())
}

#[tauri::command]
pub fn get_entry(id: String, state: State<'_, AppState>) -> Result<EntryDto, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    session
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| EntryDto::from_entry(entry, true))
        .ok_or_else(|| ArcaError::not_found("Entry"))
}

#[tauri::command]
pub fn create_entry(
    data: CreateEntryDto,
    state: State<'_, AppState>,
) -> Result<EntryDto, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;

    let mut entry = core_entry::create_entry(&data.title, &data.username, &data.password);
    entry.url = data.url;
    entry.notes = data.notes;
    entry.tags = data.tags;
    session.entries.push(entry.clone());
    persist_session(&session)?;
    session.touch();

    Ok(EntryDto::from_entry(&entry, true))
}

#[tauri::command]
pub fn update_entry(
    id: String,
    data: UpdateEntryDto,
    state: State<'_, AppState>,
) -> Result<EntryDto, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;

    let entry = session
        .entries
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| ArcaError::not_found("Entry"))?;
    core_entry::update_entry(entry, data.into());
    let dto = EntryDto::from_entry(entry, true);
    persist_session(&session)?;
    session.touch();

    Ok(dto)
}

#[tauri::command]
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
pub fn search_entries(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryDto>, ArcaError> {
    let mut session = state.session()?;
    ensure_unlocked(&session)?;
    session.touch();

    Ok(core_entry::search_entries(&session.entries, &query)
        .into_iter()
        .map(|entry| EntryDto::from_entry(entry, false))
        .collect())
}

#[tauri::command]
pub fn generate_password(config: GeneratorConfigDto) -> Result<GeneratedPassword, ArcaError> {
    let config = config.into_generator_config();
    let password = core_generator::generate_password(&config);
    let entropy_bits = core_generator::calculate_entropy(&password, &config);

    Ok(GeneratedPassword {
        password,
        entropy_bits,
    })
}

#[tauri::command]
pub fn suggest_paths(partial: String) -> Result<Vec<PathSuggestionDto>, ArcaError> {
    if partial.len() > 4096 {
        return Err(ArcaError::invalid_input("Path query is too long"));
    }

    Ok(suggest_paths_for(&partial))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, ArcaError> {
    Ok(state.settings()?.clone())
}

#[tauri::command]
pub fn update_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), ArcaError> {
    *state.settings()? = settings;
    Ok(())
}

impl EntryDto {
    fn from_entry(entry: &VaultEntry, include_password: bool) -> Self {
        Self {
            id: entry.id.clone(),
            title: entry.title.clone(),
            username: entry.username.clone(),
            password: include_password.then(|| entry.password.clone()),
            url: entry.url.clone(),
            notes: entry.notes.clone(),
            tags: entry.tags.clone(),
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
        }
    }
}

impl From<UpdateEntryDto> for core_entry::EntryPatch {
    fn from(value: UpdateEntryDto) -> Self {
        Self {
            title: value.title,
            username: value.username,
            password: value.password,
            url: value.url,
            notes: value.notes,
            tags: value.tags,
        }
    }
}

impl GeneratorConfigDto {
    fn into_generator_config(self) -> GeneratorConfig {
        let default = GeneratorConfig::default();

        GeneratorConfig {
            length: self.length.unwrap_or(default.length),
            uppercase: self.uppercase.unwrap_or(default.uppercase),
            lowercase: self.lowercase.unwrap_or(default.lowercase),
            digits: self.digits.unwrap_or(default.digits),
            symbols: self.symbols.unwrap_or(default.symbols),
            exclude_ambiguous: self.exclude_ambiguous.unwrap_or(default.exclude_ambiguous),
            mode: self
                .mode
                .map(GeneratorMode::from)
                .unwrap_or(GeneratorMode::Random),
        }
    }
}

impl From<GeneratorModeDto> for GeneratorMode {
    fn from(value: GeneratorModeDto) -> Self {
        match value {
            GeneratorModeDto::Random => Self::Random,
            GeneratorModeDto::Passphrase => Self::Passphrase,
        }
    }
}

fn ensure_unlocked(session: &crate::state::SessionState) -> Result<(), ArcaError> {
    if session.meta.is_some() && session.vault_path.is_some() && session.master_password.is_some() {
        Ok(())
    } else {
        Err(ArcaError::locked())
    }
}

fn persist_session(session: &crate::state::SessionState) -> Result<(), ArcaError> {
    let path = session
        .vault_path
        .as_deref()
        .ok_or_else(ArcaError::locked)?;
    let meta = session.meta.as_ref().ok_or_else(ArcaError::locked)?;
    let password = session
        .master_password
        .as_ref()
        .ok_or_else(ArcaError::locked)?;

    core_vault::save_vault(path, password.as_str(), meta, &session.entries)?;

    Ok(())
}

fn vault_info(path: &Path, meta: &VaultMeta, entry_count: usize) -> VaultInfo {
    VaultInfo {
        name: meta.name.clone(),
        path: path.display().to_string(),
        entry_count,
        modified_at: meta.modified_at.clone(),
    }
}

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

fn suggestion_rank(suggestion: &PathSuggestionDto) -> u8 {
    match (&suggestion.kind, suggestion.vault_candidate) {
        (PathSuggestionKind::File, true) => 0,
        (PathSuggestionKind::Directory, _) => 1,
        (PathSuggestionKind::File, false) => 2,
    }
}

fn is_vault_candidate(name: &str) -> bool {
    let lower = name.to_lowercase();

    lower.ends_with(".arca") || lower.ends_with(".kdbx")
}

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

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{suggest_paths_for, CreateEntryDto, EntryDto, GeneratorConfigDto};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use vault_core::entry::create_entry;

    #[test]
    fn entry_dto_masks_password_for_list_views() {
        let entry = create_entry("GitHub", "arca", "secret");

        let masked = EntryDto::from_entry(&entry, false);
        let revealed = EntryDto::from_entry(&entry, true);

        assert!(masked.password.is_none());
        assert_eq!(revealed.password.as_deref(), Some("secret"));
    }

    #[test]
    fn generator_config_dto_uses_secure_defaults() {
        let config = GeneratorConfigDto::default().into_generator_config();

        assert_eq!(config.length, 24);
        assert!(config.uppercase);
        assert!(config.lowercase);
        assert!(config.digits);
        assert!(config.symbols);
    }

    #[test]
    fn create_entry_dto_accepts_missing_tags() {
        let json = r#"{"title":"GitHub","username":"arca","password":"secret"}"#;
        let dto: CreateEntryDto =
            serde_json::from_str(json).expect("create entry dto should deserialize");

        assert!(dto.tags.is_empty());
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
