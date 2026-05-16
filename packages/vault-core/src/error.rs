use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Invalid master password")]
    InvalidPassword,

    #[error("Vault file not found: {0}")]
    FileNotFound(String),

    #[error("Vault file is corrupted")]
    CorruptedVault,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::VaultError;

    #[test]
    fn invalid_password_message_is_stable() {
        assert_eq!(VaultError::InvalidPassword.to_string(), "Invalid master password");
    }
}
