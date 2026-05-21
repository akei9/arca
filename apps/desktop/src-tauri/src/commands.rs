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
pub struct VaultInfo {
    pub name: String,
    pub path: String,
    pub entry_count: usize,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::{CreateEntryDto, EntryDto, GeneratorConfigDto};
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
}
