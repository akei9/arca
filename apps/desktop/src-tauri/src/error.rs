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

        Self::new(code.to_string(), safe_vault_error_message(&code))
    }
}

fn safe_vault_error_message(code: &ErrorCode) -> &'static str {
    match code {
        ErrorCode::InvalidPassword => "Invalid password",
        ErrorCode::FileNotFound => "Vault file not found",
        ErrorCode::CorruptedVault => "Vault file is corrupted",
        ErrorCode::EncryptionError => "Unable to encrypt vault data",
        ErrorCode::DecryptionError => "Unable to decrypt vault data",
        ErrorCode::IoError => "Unable to read or write vault data",
        ErrorCode::SerializationError => "Unable to process vault data",
        ErrorCode::VaultLocked => "Vault is locked",
        ErrorCode::NotFound => "Item not found",
        ErrorCode::InvalidInput => "Invalid input",
        ErrorCode::CapabilityDenied => "Capability denied",
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::ArcaError;
    use vault_core::VaultError;

    #[test]
    fn vault_error_messages_do_not_expose_core_payloads() {
        let path = "/Users/example/private/vault.arca";
        let errors = [
            VaultError::FileNotFound(path.to_string()),
            VaultError::EncryptionError("backend nonce detail".to_string()),
            VaultError::DecryptionError("backend decrypt detail".to_string()),
            VaultError::IoError(io::Error::new(io::ErrorKind::PermissionDenied, path)),
            VaultError::SerializationError("parser detail".to_string()),
        ];

        for error in errors {
            let arca_error = ArcaError::from(error);

            assert!(!arca_error.message.contains(path));
            assert!(!arca_error.message.contains("backend"));
            assert!(!arca_error.message.contains("parser"));
        }
    }
}
