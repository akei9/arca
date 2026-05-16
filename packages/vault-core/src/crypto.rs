use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

use crate::error::VaultError;
use crate::types::{KdfConfig, VaultKey};

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;

/// Derives a 256-bit vault key from master password using Argon2id.
pub fn derive_key(
    password: &str,
    salt: &[u8; 32],
    config: &KdfConfig,
) -> Result<VaultKey, VaultError> {
    let params = Params::new(
        config.memory_kib,
        config.iterations,
        config.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|_| VaultError::InvalidPassword)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut output = [0u8; KEY_LEN];

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|_| VaultError::InvalidPassword)?;

    Ok(VaultKey(output))
}

/// Generates a random 32-byte salt for vault key derivation.
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypts plaintext using XChaCha20-Poly1305.
///
/// Returns nonce (24 bytes) followed by ciphertext and authentication tag.
pub fn encrypt(key: &VaultKey, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key.0));
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|error| VaultError::EncryptionError(error.to_string()))?;

    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypts ciphertext produced by `encrypt`.
pub fn decrypt(key: &VaultKey, ciphertext: &[u8]) -> Result<Vec<u8>, VaultError> {
    if ciphertext.len() < NONCE_LEN {
        return Err(VaultError::DecryptionError(
            "ciphertext shorter than nonce".to_string(),
        ));
    }

    let (nonce, encrypted_payload) = ciphertext.split_at(NONCE_LEN);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key.0));

    cipher
        .decrypt(XNonce::from_slice(nonce), encrypted_payload)
        .map_err(|error| VaultError::DecryptionError(error.to_string()))
}

/// Zeroizes key material. Called on lock.
pub fn zeroize_key(key: &mut VaultKey) {
    key.0.zeroize();
}

#[cfg(test)]
mod tests {
    use super::{decrypt, derive_key, encrypt, generate_salt, zeroize_key};
    use crate::types::KdfConfig;

    fn test_kdf_config(salt: [u8; 32]) -> KdfConfig {
        KdfConfig {
            memory_kib: 32,
            iterations: 1,
            parallelism: 1,
            salt,
        }
    }

    #[test]
    fn generate_salt_produces_random_values() {
        let first = generate_salt();
        let second = generate_salt();

        assert_ne!(first, second);
    }

    #[test]
    fn zeroize_key_clears_key_material() {
        let mut key = derive_key(
            "password",
            &[7u8; 32],
            &test_kdf_config([7u8; 32]),
        )
        .expect("key derivation should succeed");

        zeroize_key(&mut key);

        assert_eq!(key.0, [0u8; 32]);
    }

    #[test]
    fn decrypt_rejects_too_short_ciphertext() {
        let key = derive_key(
            "password",
            &[9u8; 32],
            &test_kdf_config([9u8; 32]),
        )
        .expect("key derivation should succeed");

        let result = decrypt(&key, b"short");

        assert!(result.is_err());
    }

    #[test]
    fn encrypt_rejects_tampered_payload() {
        let key = derive_key(
            "password",
            &[11u8; 32],
            &test_kdf_config([11u8; 32]),
        )
        .expect("key derivation should succeed");
        let mut ciphertext = encrypt(&key, b"sealed message").expect("encryption should succeed");
        let last_index = ciphertext.len() - 1;
        ciphertext[last_index] ^= 0x01;

        let result = decrypt(&key, &ciphertext);

        assert!(result.is_err());
    }
}
