#![no_main]

use std::collections::hash_map::DefaultHasher;
use std::fmt::Write as _;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 1_048_576;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_BYTES {
        return;
    }

    if let Some(path) = write_temp_vault(data) {
        let credential = fuzz_credential(data);
        let _ = vault_core::vault::open_vault(&path, &credential);
        let _ = fs::remove_file(path);
    }
});

fn write_temp_vault(data: &[u8]) -> Option<PathBuf> {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "arca-vault-core-fuzz-{}-{:016x}.kdbx",
        std::process::id(),
        input_hash(data)
    ));

    fs::write(&path, data).ok()?;

    Some(path)
}

fn input_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

fn fuzz_credential(data: &[u8]) -> String {
    let mut credential = String::from("arca-fuzz-credential");

    for byte in data.iter().take(8) {
        let _ = write!(&mut credential, "{byte:02x}");
    }

    credential
}
