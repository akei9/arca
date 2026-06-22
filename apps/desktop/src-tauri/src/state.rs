use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use vault_core::{VaultEntry, VaultMeta};
use zeroize::Zeroizing;

use crate::error::ArcaError;

pub struct AppState {
    session: Mutex<SessionState>,
    settings: Mutex<Settings>,
}

impl AppState {
    pub fn session(&self) -> Result<MutexGuard<'_, SessionState>, ArcaError> {
        self.session.lock().map_err(|_| ArcaError::state_lock())
    }

    pub fn settings(&self) -> Result<MutexGuard<'_, Settings>, ArcaError> {
        self.settings.lock().map_err(|_| ArcaError::state_lock())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session: Mutex::new(SessionState::default()),
            settings: Mutex::new(Settings::default()),
        }
    }
}

#[derive(Default)]
pub struct SessionState {
    pub master_password: Option<Zeroizing<String>>,
    pub vault_path: Option<PathBuf>,
    pub entries: Vec<VaultEntry>,
    pub meta: Option<VaultMeta>,
    pub last_active: Option<Instant>,
}

impl SessionState {
    pub fn unlock(
        &mut self,
        vault_path: PathBuf,
        master_password: Zeroizing<String>,
        meta: VaultMeta,
        entries: Vec<VaultEntry>,
    ) {
        self.master_password = Some(master_password);
        self.vault_path = Some(vault_path);
        self.entries = entries;
        self.meta = Some(meta);
        self.touch();
    }

    pub fn lock(&mut self) {
        self.master_password = None;
        self.vault_path = None;
        self.entries.clear();
        self.meta = None;
        self.last_active = None;
    }

    pub fn touch(&mut self) {
        self.last_active = Some(Instant::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_lock_timeout_minutes: Option<u64>,
    pub clipboard_clear_seconds: Option<u64>,
    pub theme: Theme,
    pub font_size: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_lock_timeout_minutes: Some(15),
            clipboard_clear_seconds: Some(30),
            theme: Theme::Paper,
            font_size: 13,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[serde(alias = "terminal")]
    Paper,
    #[serde(alias = "amber")]
    Ink,
}

#[cfg(test)]
mod tests {
    use super::{Settings, Theme};

    #[test]
    fn settings_serialize_v2_theme_names() {
        let settings = Settings {
            theme: Theme::Ink,
            ..Settings::default()
        };

        let json = serde_json::to_value(settings).expect("settings should serialize");

        assert_eq!(json["theme"], "ink");
    }

    #[test]
    fn settings_deserialize_legacy_theme_names() {
        let terminal: Settings = serde_json::from_str(
            r#"{
                "autoLockTimeoutMinutes": 15,
                "clipboardClearSeconds": 30,
                "theme": "terminal",
                "fontSize": 13
            }"#,
        )
        .expect("legacy terminal theme should deserialize");
        let amber: Settings = serde_json::from_str(
            r#"{
                "autoLockTimeoutMinutes": 15,
                "clipboardClearSeconds": 30,
                "theme": "amber",
                "fontSize": 13
            }"#,
        )
        .expect("legacy amber theme should deserialize");

        assert_eq!(terminal.theme, Theme::Paper);
        assert_eq!(amber.theme, Theme::Ink);
    }
}
