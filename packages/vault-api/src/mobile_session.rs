//! Canonical contract for the full-app iOS and Android vault session.
//!
//! This module defines the boundary that issue [#250] will implement and that
//! the native adapters in [`arca-mobile` #12] and [`arca-mobile` #13] will
//! consume through UniFFI. It intentionally contains no `vault-core` types.
//!
//! Native code owns document-picker permissions and I/O. An iOS adapter keeps
//! its security-scoped URL/bookmark and an Android adapter keeps its persisted
//! Storage Access Framework URI permission. Neither raw locator crosses this
//! boundary. The adapter maps it to a process-local [`MobileDocumentHandle`],
//! reads encrypted KDBX bytes while its permission is active, and supplies an
//! opaque [`DocumentRevision`] derived from platform metadata. Rust owns the
//! decrypted session, validates the revision before save preparation, and
//! returns encrypted bytes for a coordinated native write.
//!
//! A future implementation must keep the [`MobileSessionClient`] guard inside
//! each session object and call [`MobileSessionClient::authorize`] before every
//! operation. Swift and Kotlin bind only the session facade in `vault-api`; they
//! never bind `vault-core`.
//!
//! [#250]: https://github.com/akei9/arca/issues/250
//! [`arca-mobile` #12]: https://github.com/akei9/arca-mobile/issues/12
//! [`arca-mobile` #13]: https://github.com/akei9/arca-mobile/issues/13

use core::fmt;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{ApiError, ApiOperation, ClientKind, ErrorCode, SecretString};

/// Operations exposed by the first full-app mobile vault session.
pub const MOBILE_VAULT_SESSION_OPERATIONS: &[ApiOperation] = &[
    ApiOperation::CreateVault,
    ApiOperation::OpenVault,
    ApiOperation::UnlockVault,
    ApiOperation::LockVault,
    ApiOperation::ReadVaultSummary,
    ApiOperation::ListEntries,
    ApiOperation::GetEntry,
    ApiOperation::SearchEntries,
    ApiOperation::RevealSecret,
    ApiOperation::CopySecret,
    ApiOperation::GenerateSecret,
    ApiOperation::CreateEntry,
    ApiOperation::UpdateEntry,
    ApiOperation::DeleteEntry,
    ApiOperation::SaveVault,
];

/// Immutable capability guard owned by one mobile session facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobileSessionClient {
    client_kind: ClientKind,
}

impl MobileSessionClient {
    /// Accepts only the two full mobile application client kinds.
    pub fn new(client_kind: ClientKind) -> Result<Self, ApiError> {
        if matches!(client_kind, ClientKind::IosApp | ClientKind::AndroidApp) {
            Ok(Self { client_kind })
        } else {
            Err(ApiError::stable(ErrorCode::CapabilityDenied))
        }
    }

    /// Returns the immutable client kind selected when the session was created.
    pub fn client_kind(self) -> ClientKind {
        self.client_kind
    }

    /// Performs the mandatory capability check for one public operation.
    pub fn authorize(self, operation: ApiOperation) -> Result<(), ApiError> {
        self.client_kind
            .require_operation(operation)
            .map_err(|error| ApiError::stable(error.code))
    }

    /// Rejects a document handle created for the other mobile platform.
    pub fn authorize_document(self, document: &MobileDocument) -> Result<(), ApiError> {
        let allowed = matches!(
            (self.client_kind, document.kind),
            (ClientKind::IosApp, MobileDocumentKind::IosSecurityScoped)
                | (ClientKind::IosApp, MobileDocumentKind::IosAppScoped)
                | (
                    ClientKind::AndroidApp,
                    MobileDocumentKind::AndroidStorageAccessFramework
                )
                | (ClientKind::AndroidApp, MobileDocumentKind::AndroidAppScoped)
        );

        if allowed {
            Ok(())
        } else {
            Err(ApiError::stable(ErrorCode::CapabilityDenied))
        }
    }

    /// Authorizes a mutating operation and requires a writable native grant.
    pub fn authorize_write(
        self,
        operation: ApiOperation,
        document: &MobileDocument,
    ) -> Result<(), ApiError> {
        self.authorize(operation)?;
        self.authorize_document(document)?;

        if document.access == MobileDocumentAccess::ReadWrite {
            Ok(())
        } else {
            Err(ApiError::stable(ErrorCode::DocumentReadOnly))
        }
    }
}

/// Platform-owned document surface represented by an opaque native handle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileDocumentKind {
    IosSecurityScoped,
    IosAppScoped,
    AndroidStorageAccessFramework,
    AndroidAppScoped,
}

/// Access granted by the native document provider for the current handle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileDocumentAccess {
    ReadOnly,
    ReadWrite,
}

/// Opaque process-local identifier for a native-owned URL or URI permission.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct MobileDocumentHandle(String);

impl MobileDocumentHandle {
    /// Creates a handle while rejecting raw paths, URLs, and document URIs.
    pub fn new(value: impl Into<String>) -> Result<Self, ApiError> {
        let value = value.into();
        let is_opaque_handle = value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));

        if value.is_empty() || value.len() > 256 || !is_opaque_handle {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        Ok(Self(value))
    }

    /// Returns the opaque value for native handle-table lookup only.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MobileDocumentHandle {
    type Error = ApiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<MobileDocumentHandle> for String {
    fn from(value: MobileDocumentHandle) -> Self {
        value.0
    }
}

impl fmt::Debug for MobileDocumentHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[document handle]")
    }
}

/// Opaque revision token captured by the native adapter after a coordinated read.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct DocumentRevision(String);

impl DocumentRevision {
    /// Creates a non-empty, bounded revision token.
    pub fn new(value: impl Into<String>) -> Result<Self, ApiError> {
        let value = value.into();

        if value.is_empty() || value.len() > 512 {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        Ok(Self(value))
    }

    /// Returns the token for equality checks, never for diagnostics.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for DocumentRevision {
    type Error = ApiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<DocumentRevision> for String {
    fn from(value: DocumentRevision) -> Self {
        value.0
    }
}

impl fmt::Debug for DocumentRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[document revision]")
    }
}

/// Password-free document descriptor stored by the Rust session.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileDocument {
    pub handle: MobileDocumentHandle,
    pub kind: MobileDocumentKind,
    pub access: MobileDocumentAccess,
    pub revision: DocumentRevision,
}

impl fmt::Debug for MobileDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MobileDocument")
            .field("handle", &self.handle)
            .field("kind", &self.kind)
            .field("access", &self.access)
            .field("revision", &self.revision)
            .finish()
    }
}

/// Encrypted KDBX bytes crossing between native document I/O and Rust.
///
/// The buffer is cleared on drop as defense in depth and deliberately cannot be
/// serialized or displayed.
#[derive(Clone, PartialEq, Eq)]
pub struct EncryptedVaultBytes(Zeroizing<Vec<u8>>);

impl zeroize::ZeroizeOnDrop for EncryptedVaultBytes {}

impl EncryptedVaultBytes {
    /// Wraps encrypted KDBX bytes in storage that clears itself on drop.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    /// Borrows the encrypted payload without creating another buffer.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl fmt::Debug for EncryptedVaultBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[encrypted vault bytes]")
    }
}

/// Input for registering encrypted document contents with a locked session.
#[derive(Clone, PartialEq, Eq)]
pub struct OpenMobileVaultRequest {
    pub document: MobileDocument,
    pub encrypted_vault: EncryptedVaultBytes,
}

impl fmt::Debug for OpenMobileVaultRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenMobileVaultRequest")
            .field("document", &self.document)
            .field("encrypted_vault", &self.encrypted_vault)
            .finish()
    }
}

/// Secret-bearing input for unlocking the registered encrypted document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockMobileVaultRequest {
    pub password: SecretString,
}

impl zeroize::ZeroizeOnDrop for UnlockMobileVaultRequest {}

/// Secret-bearing input for creating a new vault at a native-owned document.
#[derive(Clone, PartialEq, Eq)]
pub struct CreateMobileVaultRequest {
    pub document: MobileDocument,
    pub name: String,
    pub password: SecretString,
}

impl zeroize::ZeroizeOnDrop for CreateMobileVaultRequest {}

impl fmt::Debug for CreateMobileVaultRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateMobileVaultRequest")
            .field("document", &self.document)
            .field("name", &"[vault name]")
            .field("password", &self.password)
            .finish()
    }
}

/// Password-free summary returned only after successful unlock or create.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileVaultSummary {
    pub name: String,
    pub entry_count: usize,
    pub modified_at: String,
}

/// Observable session state. Decrypted contents and credentials are never fields.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileSessionPhase {
    Empty,
    Locked,
    UnlockedClean,
    UnlockedDirty,
}

/// Password-free state returned to Swift and Kotlin coordinators.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileSessionStatus {
    pub phase: MobileSessionPhase,
    pub writable: bool,
    pub summary: Option<MobileVaultSummary>,
}

/// Explicit secret-bearing result intended only for an immediate clipboard write.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretForCopy {
    secret: SecretString,
}

impl zeroize::ZeroizeOnDrop for SecretForCopy {}

impl SecretForCopy {
    /// Wraps one plaintext value for an immediate native clipboard write.
    pub fn new(secret: SecretString) -> Self {
        Self { secret }
    }

    /// Borrows the plaintext for the immediate native clipboard callback.
    pub fn expose_secret(&self) -> &str {
        self.secret.expose_secret()
    }
}

impl fmt::Debug for SecretForCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretForCopy")
            .field("secret", &"[secret]")
            .finish()
    }
}

impl Serialize for SecretForCopy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SecretForCopy", 1)?;
        state.serialize_field("secret", self.secret.expose_secret())?;
        state.end()
    }
}

/// Revision supplied immediately before Rust prepares an encrypted save payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrepareMobileVaultSaveRequest {
    pub observed_revision: DocumentRevision,
}

/// Encrypted bytes and compare-before-write token returned by save preparation.
#[derive(Clone, PartialEq, Eq)]
pub struct PreparedMobileVaultWrite {
    pub document_handle: MobileDocumentHandle,
    pub expected_revision: DocumentRevision,
    pub encrypted_vault: EncryptedVaultBytes,
}

impl zeroize::ZeroizeOnDrop for PreparedMobileVaultWrite {}

/// Native acknowledgement after a coordinated write and revision refresh.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommitMobileVaultWriteRequest {
    pub committed_revision: DocumentRevision,
}

impl fmt::Debug for PreparedMobileVaultWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedMobileVaultWrite")
            .field("document_handle", &self.document_handle)
            .field("expected_revision", &self.expected_revision)
            .field("encrypted_vault", &self.encrypted_vault)
            .finish()
    }
}

/// Stable state transition required after a public mobile session error.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileFailureAction {
    KeepState,
    StayLocked,
    LockAndDiscard,
    KeepUnlockedDirty,
}

/// Returns the required fail-closed state transition for stable mobile errors.
pub fn failure_action(code: ErrorCode) -> MobileFailureAction {
    match code {
        ErrorCode::InvalidPassword => MobileFailureAction::StayLocked,
        ErrorCode::CorruptedVault
        | ErrorCode::FileNotFound
        | ErrorCode::DocumentPermissionLost
        | ErrorCode::ExternalFileChanged => MobileFailureAction::LockAndDiscard,
        ErrorCode::SaveFailed => MobileFailureAction::KeepUnlockedDirty,
        _ => MobileFailureAction::KeepState,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::{GeneratedSecret, RevealedSecret};

    #[test]
    fn every_session_operation_is_authorized_for_each_full_mobile_app() {
        for client_kind in [ClientKind::IosApp, ClientKind::AndroidApp] {
            let client = MobileSessionClient::new(client_kind).expect("full app should be valid");

            for operation in MOBILE_VAULT_SESSION_OPERATIONS {
                client
                    .authorize(*operation)
                    .expect("full mobile app operation should pass capability policy");
            }
        }
    }

    #[test]
    fn non_app_clients_cannot_own_a_full_mobile_session() {
        for client_kind in [
            ClientKind::DesktopApp,
            ClientKind::BrowserExtension,
            ClientKind::IosAutofillExtension,
            ClientKind::AndroidAutofillService,
            ClientKind::FutureSyncServer,
        ] {
            let error = MobileSessionClient::new(client_kind)
                .expect_err("non-app clients must fail closed");

            assert_eq!(error.code, ErrorCode::CapabilityDenied);
        }
    }

    #[test]
    fn denied_mobile_operations_return_only_stable_error_text() {
        let client = MobileSessionClient::new(ClientKind::IosApp).expect("iOS should be valid");
        let error = client
            .authorize(ApiOperation::ExportPlaintext)
            .expect_err("mobile plaintext export must remain denied");

        assert_eq!(error.code, ErrorCode::CapabilityDenied);
        assert_eq!(error.message, ErrorCode::CapabilityDenied.safe_message());
        assert!(!error.message.contains(ClientKind::IosApp.as_str()));
        assert!(!error
            .message
            .contains(crate::Capability::ExportPlaintext.as_str()));
    }

    #[test]
    fn document_handles_reject_raw_paths_urls_and_uris() {
        for locator in [
            "/private/mobile/vault.kdbx",
            "Documents/vault.kdbx",
            r"C:\vault.kdbx",
            "../vault.kdbx",
            "file:///private/mobile/vault.kdbx",
            "content://provider/document/vault",
            "https://provider.example/vault.kdbx",
        ] {
            let error = MobileDocumentHandle::new(locator)
                .expect_err("raw platform locators must stay native-owned");

            assert_eq!(error.code, ErrorCode::InvalidInput);

            let encoded = serde_json::to_string(locator).expect("locator should encode");
            serde_json::from_str::<MobileDocumentHandle>(&encoded)
                .expect_err("deserialization must not bypass locator validation");
        }

        for handle in ["document-7", "DOCUMENT_8", "9"] {
            MobileDocumentHandle::new(handle)
                .expect("opaque process-local handles should be accepted");
        }
    }

    #[test]
    fn document_handles_are_bound_to_the_session_platform() {
        let ios = MobileSessionClient::new(ClientKind::IosApp).expect("iOS should be valid");
        let android =
            MobileSessionClient::new(ClientKind::AndroidApp).expect("Android should be valid");
        let ios_document = document(MobileDocumentKind::IosSecurityScoped);
        let android_document = document(MobileDocumentKind::AndroidStorageAccessFramework);

        ios.authorize_document(&ios_document)
            .expect("iOS session should accept iOS document handles");
        android
            .authorize_document(&android_document)
            .expect("Android session should accept Android document handles");
        assert_eq!(
            ios.authorize_document(&android_document)
                .expect_err("iOS must reject Android document handles")
                .code,
            ErrorCode::CapabilityDenied
        );
        assert_eq!(
            android
                .authorize_document(&ios_document)
                .expect_err("Android must reject iOS document handles")
                .code,
            ErrorCode::CapabilityDenied
        );
    }

    #[test]
    fn mutations_require_a_writable_platform_grant() {
        let client = MobileSessionClient::new(ClientKind::IosApp).expect("iOS should be valid");
        let mut read_only = document(MobileDocumentKind::IosSecurityScoped);
        read_only.access = MobileDocumentAccess::ReadOnly;

        let error = client
            .authorize_write(ApiOperation::UpdateEntry, &read_only)
            .expect_err("read-only documents must reject mutation before state changes");

        assert_eq!(error.code, ErrorCode::DocumentReadOnly);
    }

    #[test]
    fn secret_bearing_surfaces_redact_debug_and_zeroize_on_drop() {
        fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}

        assert_zeroize_on_drop::<SecretString>();
        assert_zeroize_on_drop::<UnlockMobileVaultRequest>();
        assert_zeroize_on_drop::<CreateMobileVaultRequest>();
        assert_zeroize_on_drop::<RevealedSecret>();
        assert_zeroize_on_drop::<SecretForCopy>();
        assert_zeroize_on_drop::<GeneratedSecret>();

        let secret = unique_test_secret();
        let unlock = UnlockMobileVaultRequest {
            password: SecretString::new(secret.clone()),
        };
        let create = CreateMobileVaultRequest {
            document: document(MobileDocumentKind::IosAppScoped),
            name: "private vault name".to_string(),
            password: SecretString::new(secret.clone()),
        };
        let reveal = RevealedSecret::new(secret.clone());
        let copy = SecretForCopy::new(SecretString::new(secret.clone()));
        let generated = GeneratedSecret::new(secret.clone(), 96.0);

        for debug in [
            format!("{unlock:?}"),
            format!("{create:?}"),
            format!("{reveal:?}"),
            format!("{copy:?}"),
            format!("{generated:?}"),
        ] {
            assert!(!debug.contains(&secret));
            assert!(!debug.contains("private vault name"));
        }
    }

    #[test]
    fn password_free_dtos_do_not_grow_secret_fields() {
        let status = MobileSessionStatus {
            phase: MobileSessionPhase::UnlockedClean,
            writable: true,
            summary: Some(MobileVaultSummary {
                name: "Synthetic vault".to_string(),
                entry_count: 2,
                modified_at: "2026-09-15T00:00:00Z".to_string(),
            }),
        };
        let json = serde_json::to_value(status).expect("status should serialize");
        let object = json.as_object().expect("status should be an object");

        assert!(!object.contains_key("password"));
        assert!(!object.contains_key("secret"));
        assert!(!json.to_string().contains("documentHandle"));
    }

    #[test]
    fn explicit_secret_responses_are_the_only_serialized_plaintext_surfaces() {
        let secret = unique_test_secret();
        let revealed = serde_json::to_string(&RevealedSecret::new(secret.clone()))
            .expect("reveal response should serialize explicitly");
        let copied = serde_json::to_string(&SecretForCopy::new(SecretString::new(secret.clone())))
            .expect("copy response should serialize explicitly");
        let generated = serde_json::to_string(&GeneratedSecret::new(secret.clone(), 96.0))
            .expect("generated response should serialize explicitly");

        assert!(revealed.contains(&secret));
        assert!(copied.contains(&secret));
        assert!(generated.contains(&secret));
    }

    #[test]
    fn mobile_error_messages_and_state_actions_are_stable() {
        let cases = [
            (
                ErrorCode::InvalidPassword,
                "invalid_password",
                MobileFailureAction::StayLocked,
            ),
            (
                ErrorCode::CorruptedVault,
                "corrupted_vault",
                MobileFailureAction::LockAndDiscard,
            ),
            (
                ErrorCode::FileNotFound,
                "file_not_found",
                MobileFailureAction::LockAndDiscard,
            ),
            (
                ErrorCode::DocumentPermissionLost,
                "document_permission_lost",
                MobileFailureAction::LockAndDiscard,
            ),
            (
                ErrorCode::DocumentReadOnly,
                "document_read_only",
                MobileFailureAction::KeepState,
            ),
            (
                ErrorCode::ExternalFileChanged,
                "external_file_changed",
                MobileFailureAction::LockAndDiscard,
            ),
            (
                ErrorCode::SaveFailed,
                "save_failed",
                MobileFailureAction::KeepUnlockedDirty,
            ),
        ];

        for (code, wire_code, action) in cases {
            let error = ApiError::stable(code);

            assert_eq!(code.to_string(), wire_code);
            assert_eq!(error.message, code.safe_message());
            assert_eq!(failure_action(code), action);
        }
    }

    #[test]
    fn stable_errors_cannot_embed_secret_or_platform_locator_details() {
        let sentinel = unique_test_secret();

        for code in [
            ErrorCode::InvalidPassword,
            ErrorCode::CorruptedVault,
            ErrorCode::DocumentPermissionLost,
            ErrorCode::ExternalFileChanged,
            ErrorCode::SaveFailed,
        ] {
            let error = ApiError::stable(code);
            let outputs = [
                format!("{error:?}"),
                error.to_string(),
                serde_json::to_string(&error).expect("error should serialize"),
            ];

            for output in outputs {
                assert!(!output.contains(&sentinel));
                assert!(!output.contains("content://"));
                assert!(!output.contains("file://"));
            }
        }
    }

    fn document(kind: MobileDocumentKind) -> MobileDocument {
        MobileDocument {
            handle: MobileDocumentHandle::new("document-7").expect("handle should be valid"),
            kind,
            access: MobileDocumentAccess::ReadWrite,
            revision: DocumentRevision::new("revision-3").expect("revision should be valid"),
        }
    }

    fn unique_test_secret() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        format!("test-secret-{nanos}")
    }
}
