use core::fmt;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

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
            .field("revision_count", &self.revisions.len())
            .finish_non_exhaustive()
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
        f.debug_struct("EntryRevision").finish_non_exhaustive()
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
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{EntryRevision, KdfConfig, VaultEntry, VaultKey};

    fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}

    fn test_credential(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        format!("test-credential-{label}-{nanos}")
    }

    fn entry_with_revision(password: &str, revision_password: &str) -> VaultEntry {
        VaultEntry {
            id: "11111111-2222-4333-8444-555555555555".to_string(),
            title: "sentinel-title".to_string(),
            username: "sentinel-username".to_string(),
            password: password.to_string(),
            collection: Some("sentinel-collection".to_string()),
            url: Some("sentinel-url".to_string()),
            notes: Some("sentinel-notes".to_string()),
            tags: vec!["sentinel-tag".to_string()],
            created_at: "2026-08-19T00:00:00+00:00".to_string(),
            updated_at: "2026-08-19T00:00:00+00:00".to_string(),
            revisions: vec![EntryRevision {
                captured_at: "2026-08-18T00:00:00+00:00".to_string(),
                title: "sentinel-revision-title".to_string(),
                username: "sentinel-revision-username".to_string(),
                password: revision_password.to_string(),
                collection: Some("sentinel-revision-collection".to_string()),
                url: Some("sentinel-revision-url".to_string()),
                notes: Some("sentinel-revision-notes".to_string()),
                tags: vec!["sentinel-revision-tag".to_string()],
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
    fn debug_output_excludes_decrypted_vault_contents() {
        let current = test_credential("current");
        let historical = test_credential("historical");
        let entry = entry_with_revision(&current, &historical);

        let sensitive = [
            current.as_str(),
            historical.as_str(),
            "sentinel-title",
            "sentinel-username",
            "sentinel-collection",
            "sentinel-url",
            "sentinel-notes",
            "sentinel-tag",
            "sentinel-revision-title",
            "sentinel-revision-username",
            "sentinel-revision-collection",
            "sentinel-revision-url",
            "sentinel-revision-notes",
            "sentinel-revision-tag",
        ];

        for output in [format!("{entry:?}"), format!("{entry:#?}")] {
            for value in sensitive {
                assert!(!output.contains(value), "Debug output leaked {value}");
            }
            assert!(output.contains("revision_count"));
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
