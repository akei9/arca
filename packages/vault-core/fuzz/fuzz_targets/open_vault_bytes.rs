#![no_main]

use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

use libfuzzer_sys::fuzz_target;
use tempfile::NamedTempFile;

const MAX_INPUT_BYTES: usize = 1_048_576;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_BYTES {
        return;
    }

    if let Some(temp_vault) = write_temp_vault(data) {
        let credential = fuzz_credential(data);
        let _ = vault_core::vault::open_vault(temp_vault.path(), &credential);
    }
});

/// Creates an exclusive private temporary vault file for the generated input.
fn write_temp_vault(data: &[u8]) -> Option<NamedTempFile> {
    let mut temp_vault = NamedTempFile::with_suffix(".kdbx").ok()?;
    temp_vault.write_all(data).ok()?;
    temp_vault.flush().ok()?;
    Some(temp_vault)
}

/// Builds a deterministic, non-secret credential from fuzz input bytes.
fn fuzz_credential(data: &[u8]) -> String {
    let mut credential = String::from("arca-fuzz-credential");

    for byte in data.iter().take(8) {
        let _ = write!(&mut credential, "{byte:02x}");
    }

    credential
}
