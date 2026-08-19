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

/// A single credential stored in the vault, including any retained historical revisions.
///
/// Derives `Zeroize` so the plaintext password and other secret fields can be wiped from
/// memory when a copy is discarded.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Zeroize)]
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
    /// Newest-first snapshots of prior values; defaults to empty for vaults saved before
    /// revision history existed.
    #[serde(default)]
    pub revisions: Vec<EntryRevision>,
}

/// A point-in-time snapshot of an entry's values captured before a meaningful update.
///
/// Revisions carry historical plaintext secrets, so this type derives `Zeroize` and is only
/// ever persisted inside the encrypted vault, never surfaced in list/search/audit DTOs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Zeroize)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VaultMeta {
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
}

#[cfg(test)]
mod tests {
    use super::{KdfConfig, VaultKey};

    fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}

    #[test]
    fn vault_key_implements_zeroize_on_drop() {
        assert_zeroize_on_drop::<VaultKey>();
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
