use core::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::Zeroizing;

/// Plaintext value that may cross a public API boundary only through explicit
/// secret-bearing operations.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(Zeroizing<String>);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> String {
        self.0.to_string()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[secret]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[secret]")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expose_secret())
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::new)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VaultSummary {
    pub name: String,
    pub path: String,
    pub entry_count: usize,
    pub modified_at: String,
}

impl VaultSummary {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        entry_count: usize,
        modified_at: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            entry_count,
            modified_at: modified_at.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntryView {
    pub id: String,
    pub title: String,
    pub username: String,
    pub collection: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision_count: usize,
}

#[derive(Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RevisionView {
    pub captured_at: String,
    pub updated_at: String,
    pub title: String,
    pub username: String,
    pub collection: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub password_changed: bool,
}

impl fmt::Debug for RevisionView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevisionView")
            .field("captured_at", &self.captured_at)
            .field("updated_at", &self.updated_at)
            .field("title", &self.title)
            .field("username", &self.username)
            .field("collection", &self.collection)
            .field("url", &self.url)
            .field("notes", &self.notes)
            .field("tags", &self.tags)
            .field("password_changed", &self.password_changed)
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CreateEntryRequest {
    pub title: String,
    pub username: String,
    pub password: SecretString,
    pub collection: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct EntryMutation {
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<SecretString>,
    pub collection: Option<Option<String>>,
    pub url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorParams {
    pub length: Option<usize>,
    pub uppercase: Option<bool>,
    pub lowercase: Option<bool>,
    pub digits: Option<bool>,
    pub symbols: Option<bool>,
    pub exclude_ambiguous: Option<bool>,
    pub mode: Option<GeneratorMode>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GeneratorMode {
    Random,
    Passphrase,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedSecret {
    pub password: SecretString,
    pub entropy_bits: f64,
}

impl GeneratedSecret {
    pub fn new(password: impl Into<String>, entropy_bits: f64) -> Self {
        Self {
            password: SecretString::new(password),
            entropy_bits,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ClientKind {
    DesktopApp,
    BrowserExtension,
    IosApp,
    AndroidApp,
    IosAutofillExtension,
    AndroidAutofillService,
    FutureSyncServer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    Unlock,
    ReadMeta,
    RevealSecret,
    CopySecret,
    MutateEntry,
    CreateVault,
    ChangeKdf,
    ExportPlaintext,
    ExportKdbx,
    ReadHistory,
    DeletePermanent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuditSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuditFindingKind {
    WeakPassword,
    ReusedPassword,
    InsecureUrl,
    DuplicateUrl,
    DuplicateUsername,
    StaleEntry,
    MissingUrl,
    MissingCollection,
    Untagged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditFinding {
    pub key: String,
    pub severity: AuditSeverity,
    pub kind: AuditFindingKind,
    pub entry_id: String,
    pub meta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{code}: {message}")]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidPassword,
    FileNotFound,
    CorruptedVault,
    EncryptionError,
    DecryptionError,
    IoError,
    SerializationError,
    VaultLocked,
    NotFound,
    InvalidInput,
    CapabilityDenied,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::InvalidPassword => "invalid_password",
            Self::FileNotFound => "file_not_found",
            Self::CorruptedVault => "corrupted_vault",
            Self::EncryptionError => "encryption_error",
            Self::DecryptionError => "decryption_error",
            Self::IoError => "io_error",
            Self::SerializationError => "serialization_error",
            Self::VaultLocked => "vault_locked",
            Self::NotFound => "not_found",
            Self::InvalidInput => "invalid_input",
            Self::CapabilityDenied => "capability_denied",
        };

        f.write_str(code)
    }
}

impl From<vault_core::VaultError> for ApiError {
    fn from(error: vault_core::VaultError) -> Self {
        let code = match error {
            vault_core::VaultError::InvalidPassword => ErrorCode::InvalidPassword,
            vault_core::VaultError::FileNotFound(_) => ErrorCode::FileNotFound,
            vault_core::VaultError::CorruptedVault => ErrorCode::CorruptedVault,
            vault_core::VaultError::EncryptionError(_) => ErrorCode::EncryptionError,
            vault_core::VaultError::DecryptionError(_) => ErrorCode::DecryptionError,
            vault_core::VaultError::IoError(_) => ErrorCode::IoError,
            vault_core::VaultError::SerializationError(_) => ErrorCode::SerializationError,
        };

        Self::new(code, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        CreateEntryRequest, EntryMutation, EntryView, GeneratedSecret, RevisionView, SecretString,
    };

    #[test]
    fn entry_view_serialization_excludes_current_and_revision_passwords() {
        let secret = test_credential("current");
        let view = EntryView {
            id: "entry-id".to_string(),
            title: "GitHub".to_string(),
            username: "arca".to_string(),
            collection: Some("work".to_string()),
            url: Some("https://github.com".to_string()),
            notes: Some("fixture notes".to_string()),
            tags: vec!["dev".to_string()],
            created_at: "2026-09-01T00:00:00+00:00".to_string(),
            updated_at: "2026-09-01T00:00:00+00:00".to_string(),
            revision_count: 1,
        };
        let json = serde_json::to_string(&view).expect("entry view should serialize");
        let debug = format!("{view:?}");

        assert!(!json.contains("\"password\""));
        assert!(!json.contains(secret.as_str()));
        assert!(!debug.contains(secret.as_str()));
        assert_eq!(view.revision_count, 1);
    }

    #[test]
    fn revision_view_serialization_and_debug_exclude_password() {
        let historical = test_credential("historical");
        let view = RevisionView {
            captured_at: "2026-09-01T00:00:00+00:00".to_string(),
            updated_at: "2026-09-01T00:00:00+00:00".to_string(),
            title: "GitHub".to_string(),
            username: "arca".to_string(),
            collection: Some("work".to_string()),
            url: Some("https://github.com".to_string()),
            notes: Some("fixture notes".to_string()),
            tags: vec!["dev".to_string()],
            password_changed: true,
        };

        let json = serde_json::to_string(&view).expect("revision view should serialize");
        let debug = format!("{view:?}");

        assert!(!json.contains("\"password\""));
        assert!(!json.contains(historical.as_str()));
        assert!(!debug.contains(historical.as_str()));
        assert!(view.password_changed);
    }

    #[test]
    fn secret_string_serializes_but_redacts_debug_and_display() {
        let secret = test_credential("secret");
        let value = SecretString::new(secret.clone());

        let json = serde_json::to_string(&value).expect("secret string should serialize");

        assert_eq!(json, format!("\"{secret}\""));
        assert_eq!(format!("{value:?}"), "[secret]");
        assert_eq!(value.to_string(), "[secret]");
    }

    #[test]
    fn secret_bearing_request_debug_output_is_redacted() {
        let secret = test_credential("request");
        let request = CreateEntryRequest {
            title: "GitHub".to_string(),
            username: "arca".to_string(),
            password: SecretString::new(secret.clone()),
            collection: None,
            url: None,
            notes: None,
            tags: Vec::new(),
        };
        let mutation = EntryMutation {
            password: Some(SecretString::new(secret.clone())),
            ..EntryMutation::default()
        };

        assert!(!format!("{request:?}").contains(secret.as_str()));
        assert!(!format!("{mutation:?}").contains(secret.as_str()));
    }

    #[test]
    fn generated_secret_debug_output_is_redacted() {
        let secret = test_credential("generated");
        let generated = GeneratedSecret::new(secret.clone(), 128.0);

        assert!(!format!("{generated:?}").contains(secret.as_str()));
    }

    #[test]
    fn entry_mutation_debug_output_is_redacted() {
        let secret = test_credential("patch");
        let mutation = EntryMutation {
            title: Some("GitHub".to_string()),
            password: Some(SecretString::new(secret.clone())),
            ..EntryMutation::default()
        };

        assert!(!format!("{mutation:?}").contains(secret.as_str()));
    }

    fn test_credential(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        format!("test-credential-{label}-{nanos}")
    }
}
