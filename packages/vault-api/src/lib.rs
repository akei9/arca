use core::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use zeroize::Zeroizing;

/// Plaintext value that may cross a public API boundary only through explicit
/// secret-bearing operations.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(Zeroizing<String>);

impl SecretString {
    /// Wraps a plaintext value for explicit secret-bearing API calls.
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    /// Returns the plaintext value for the narrow adapter boundary that needs it.
    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes the wrapper and returns plaintext for core mutation calls.
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
    /// Builds vault metadata returned after creating or unlocking a vault.
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
    #[serde(default, deserialize_with = "deserialize_present_nullable")]
    pub collection: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_present_nullable")]
    pub url: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_present_nullable")]
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

/// Preserves the difference between an omitted field and explicit JSON `null`.
fn deserialize_present_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Option::<T>::deserialize(deserializer).map(Some)
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

#[derive(Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RevealedSecret {
    pub secret: String,
}

impl RevealedSecret {
    /// Builds an explicit response for intentional secret reveal operations.
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

impl From<SecretString> for RevealedSecret {
    fn from(secret: SecretString) -> Self {
        Self::new(secret.into_inner())
    }
}

impl fmt::Debug for RevealedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevealedSecret")
            .field("secret", &"[secret]")
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GeneratorMode {
    Random,
    Passphrase,
}

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedSecret {
    pub password: String,
    pub entropy_bits: f64,
}

impl GeneratedSecret {
    /// Builds a generated secret response with its entropy estimate.
    pub fn new(password: impl Into<String>, entropy_bits: f64) -> Self {
        Self {
            password: password.into(),
            entropy_bits,
        }
    }
}

impl fmt::Debug for GeneratedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedSecret")
            .field("password", &"[secret]")
            .field("entropy_bits", &self.entropy_bits)
            .finish()
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
    /// Builds a portable API error with a stable machine-readable code.
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        CreateEntryRequest, EntryMutation, EntryView, GeneratedSecret, RevealedSecret,
        RevisionView, SecretString,
    };

    #[test]
    fn entry_view_serialization_excludes_current_and_revision_passwords() {
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
        assert!(!debug.contains("password"));
        assert_eq!(view.revision_count, 1);
    }

    #[test]
    fn revision_view_serialization_and_debug_exclude_password() {
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
        assert!(!debug.contains("\"password\""));
        assert!(view.password_changed);
    }

    #[test]
    fn secret_string_redacts_debug_and_display() {
        let secret = unique_test_secret();
        let value = SecretString::new(secret.clone());

        assert_eq!(format!("{value:?}"), "[secret]");
        assert_eq!(value.to_string(), "[secret]");
    }

    #[test]
    fn revealed_secret_serializes_only_through_explicit_response_type() {
        let secret = unique_test_secret();
        let response = RevealedSecret::new(secret.clone());

        let json = serde_json::to_string(&response).expect("revealed secret should serialize");

        assert_eq!(json, format!("{{\"secret\":\"{secret}\"}}"));
        assert!(!format!("{response:?}").contains(secret.as_str()));
    }

    #[test]
    fn generated_secret_serializes_only_through_explicit_response_type() {
        let secret = unique_test_secret();
        let generated = GeneratedSecret::new(secret.clone(), 128.0);

        let json = serde_json::to_string(&generated).expect("generated secret should serialize");

        assert_eq!(
            json,
            format!("{{\"password\":\"{secret}\",\"entropyBits\":128.0}}")
        );
        assert!(!format!("{generated:?}").contains(secret.as_str()));
    }

    #[test]
    fn secret_bearing_request_debug_output_is_redacted() {
        let secret = unique_test_secret();
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
        let secret = unique_test_secret();
        let generated = GeneratedSecret::new(secret.clone(), 128.0);

        assert!(!format!("{generated:?}").contains(secret.as_str()));
    }

    #[test]
    fn entry_mutation_debug_output_is_redacted() {
        let secret = unique_test_secret();
        let mutation = EntryMutation {
            title: Some("GitHub".to_string()),
            password: Some(SecretString::new(secret.clone())),
            ..EntryMutation::default()
        };

        assert!(!format!("{mutation:?}").contains(secret.as_str()));
    }

    #[test]
    fn entry_mutation_omitted_metadata_fields_leave_values_unchanged() {
        let mutation: EntryMutation =
            serde_json::from_str("{}").expect("empty mutation should deserialize");

        assert_eq!(mutation.collection, None);
        assert_eq!(mutation.url, None);
        assert_eq!(mutation.notes, None);
    }

    #[test]
    fn entry_mutation_explicit_null_metadata_fields_clear_values() {
        let mutation: EntryMutation =
            serde_json::from_str(r#"{"collection":null,"url":null,"notes":null}"#)
                .expect("null metadata mutation should deserialize");

        assert_eq!(mutation.collection, Some(None));
        assert_eq!(mutation.url, Some(None));
        assert_eq!(mutation.notes, Some(None));
    }

    #[test]
    fn entry_mutation_present_metadata_fields_replace_values() {
        let mutation: EntryMutation = serde_json::from_str(
            r#"{"collection":"work","url":"https://example.test","notes":"notes"}"#,
        )
        .expect("present metadata mutation should deserialize");

        assert_eq!(mutation.collection, Some(Some("work".to_string())));
        assert_eq!(mutation.url, Some(Some("https://example.test".to_string())));
        assert_eq!(mutation.notes, Some(Some("notes".to_string())));
    }

    fn unique_test_secret() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        nanos.to_string()
    }
}
