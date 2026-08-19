use core::fmt;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

const REDACTED: &str = "[redacted]";

/// Vault encryption key — zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct VaultKey(pub [u8; 32]);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KdfConfig {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub salt: [u8; 32],
}

impl Default for KdfConfig {
    fn default() -> Self {
        Self {
            memory_kib: 131_072,
            iterations: 3,
            parallelism: 4,
            salt: [0u8; 32],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct VaultEntry {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub collection: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub revisions: Vec<EntryRevision>,
}

impl fmt::Debug for VaultEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VaultEntry")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &REDACTED)
            .field("collection", &self.collection)
            .field("url", &self.url)
            .field("notes", &self.notes)
            .field("tags", &self.tags)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field("revisions", &self.revisions)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct EntryRevision {
    pub captured_at: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub collection: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub updated_at: String,
}

impl fmt::Debug for EntryRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryRevision")
            .field("captured_at", &self.captured_at)
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &REDACTED)
            .field("collection", &self.collection)
            .field("url", &self.url)
            .field("notes", &self.notes)
            .field("tags", &self.tags)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VaultMeta {
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
}

#[cfg(test)]
mod tests {
    use super::{EntryRevision, KdfConfig, VaultEntry, VaultKey};

    fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}

    fn entry_with_revision(password: &str, revision_password: &str) -> VaultEntry {
        VaultEntry {
            id: "11111111-2222-4333-8444-555555555555".to_string(),
            title: "GitHub".to_string(),
            username: "arca".to_string(),
            password: password.to_string(),
            collection: None,
            url: None,
            notes: None,
            tags: Vec::new(),
            created_at: "2026-08-19T00:00:00+00:00".to_string(),
            updated_at: "2026-08-19T00:00:00+00:00".to_string(),
            revisions: vec![EntryRevision {
                captured_at: "2026-08-18T00:00:00+00:00".to_string(),
                title: "GitHub".to_string(),
                username: "arca".to_string(),
                password: revision_password.to_string(),
                collection: None,
                url: None,
                notes: None,
                tags: Vec::new(),
                updated_at: "2026-08-18T00:00:00+00:00".to_string(),
            }],
        }
    }

    #[test]
    fn vault_key_implements_zeroize_on_drop() {
        assert_zeroize_on_drop::<VaultKey>();
    }

    #[test]
    fn entry_secrets_implement_zeroize_on_drop() {
        assert_zeroize_on_drop::<VaultEntry>();
        assert_zeroize_on_drop::<EntryRevision>();
    }

    #[test]
    fn debug_output_redacts_current_and_historical_passwords() {
        let entry = entry_with_revision("current-secret-abc", "historical-secret-xyz");

        let rendered = format!("{entry:?}");
        let pretty = format!("{entry:#?}");

        for output in [&rendered, &pretty] {
            assert!(!output.contains("current-secret-abc"));
            assert!(!output.contains("historical-secret-xyz"));
            assert!(output.contains("[redacted]"));
        }
    }

    #[test]
    fn default_kdf_config_matches_spec() {
        let config = KdfConfig::default();

        assert_eq!(config.memory_kib, 131_072);
        assert_eq!(config.iterations, 3);
        assert_eq!(config.parallelism, 4);
        assert_eq!(config.salt, [0u8; 32]);
    }
}
