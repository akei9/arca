use vault_core::crypto::{decrypt, derive_key, encrypt};
use vault_core::types::KdfConfig;

fn test_kdf_config(salt: [u8; 32]) -> KdfConfig {
    KdfConfig {
        memory_kib: 32,
        iterations: 1,
        parallelism: 1,
        salt,
    }
}

#[test]
fn test_derive_key_deterministic() {
    let salt = [42u8; 32];
    let config = test_kdf_config(salt);

    let first = derive_key("correct horse battery staple", &salt, &config)
        .expect("first key derivation should succeed");
    let second = derive_key("correct horse battery staple", &salt, &config)
        .expect("second key derivation should succeed");

    assert_eq!(first.0, second.0);
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let salt = [13u8; 32];
    let config = test_kdf_config(salt);
    let key = derive_key("master password", &salt, &config).expect("key derivation should succeed");
    let plaintext = b"vault entry payload";

    let ciphertext = encrypt(&key, plaintext).expect("encryption should succeed");
    let decrypted = decrypt(&key, &ciphertext).expect("decryption should succeed");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_wrong_password_fails() {
    let salt = [99u8; 32];
    let config = test_kdf_config(salt);
    let correct_key = derive_key("right password", &salt, &config)
        .expect("correct key derivation should succeed");
    let wrong_key = derive_key("wrong password", &salt, &config)
        .expect("wrong key derivation should succeed");
    let ciphertext = encrypt(&correct_key, b"top secret").expect("encryption should succeed");

    let result = decrypt(&wrong_key, &ciphertext);

    assert!(result.is_err());
}

#[test]
fn test_nonce_is_random() {
    let salt = [5u8; 32];
    let config = test_kdf_config(salt);
    let key = derive_key("master password", &salt, &config).expect("key derivation should succeed");
    let plaintext = b"same plaintext";

    let first = encrypt(&key, plaintext).expect("first encryption should succeed");
    let second = encrypt(&key, plaintext).expect("second encryption should succeed");

    assert_ne!(first, second);
}
