use serde::Serialize;
use vault_core::VaultError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ArcaError {
    pub code: String,
    pub message: String,
}

impl ArcaError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn locked() -> Self {
        Self::new("vault_locked", "Vault is locked")
    }

    pub fn not_found(entity: &str) -> Self {
        Self::new("not_found", format!("{entity} not found"))
    }

    pub fn state_lock() -> Self {
        Self::new("state_lock", "Application state is unavailable")
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }
}

impl From<VaultError> for ArcaError {
    fn from(error: VaultError) -> Self {
        match error {
            VaultError::InvalidPassword => Self::new("invalid_password", error.to_string()),
            VaultError::FileNotFound(_) => Self::new("file_not_found", error.to_string()),
            VaultError::CorruptedVault => Self::new("corrupted_vault", error.to_string()),
            VaultError::EncryptionError(_) => Self::new("encryption_error", error.to_string()),
            VaultError::DecryptionError(_) => Self::new("decryption_error", error.to_string()),
            VaultError::IoError(_) => Self::new("io_error", error.to_string()),
            VaultError::SerializationError(_) => {
                Self::new("serialization_error", error.to_string())
            }
        }
    }
}
