use serde::Serialize;
use vault_api::ErrorCode;
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
        let code = match error {
            VaultError::InvalidPassword => ErrorCode::InvalidPassword,
            VaultError::FileNotFound(_) => ErrorCode::FileNotFound,
            VaultError::CorruptedVault => ErrorCode::CorruptedVault,
            VaultError::EncryptionError(_) => ErrorCode::EncryptionError,
            VaultError::DecryptionError(_) => ErrorCode::DecryptionError,
            VaultError::IoError(_) => ErrorCode::IoError,
            VaultError::SerializationError(_) => ErrorCode::SerializationError,
        };

        Self::new(code.to_string(), error.to_string())
    }
}
