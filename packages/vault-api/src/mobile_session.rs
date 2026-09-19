//! Canonical contract for the full-app iOS and Android vault session.
//!
//! This module defines the boundary that future native iOS and Android adapters
//! will consume through UniFFI. It intentionally contains no `vault-core`
//! types.
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
//! The implementation must keep the [`MobileSessionClient`] guard inside
//! each session object and call [`MobileSessionClient::authorize`] before every
//! operation. Swift and Kotlin bind only the session facade in `vault-api`; they
//! never bind `vault-core`.

use core::fmt;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use vault_core::entry::{self as core_entry, EntryPatch, DEFAULT_ENTRY_REVISION_LIMIT};
use vault_core::generator::{
    self as core_generator, GeneratorConfig as CoreGeneratorConfig,
    GeneratorMode as CoreGeneratorMode,
};
use vault_core::types::{EntryRevision, VaultEntry, VaultMeta};
use vault_core::vault as core_vault;
use vault_core::VaultError;

use crate::{
    ApiError, ApiOperation, ClientKind, CreateEntryRequest, EntryMutation, EntryView, ErrorCode,
    GeneratedSecret, GeneratorMode, GeneratorParams, RevealedSecret, RevisionView, SecretString,
};

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
    ApiOperation::RevealRevisionSecret,
    ApiOperation::CopySecret,
    ApiOperation::ReadHistory,
    ApiOperation::GenerateSecret,
    ApiOperation::CreateEntry,
    ApiOperation::UpdateEntry,
    ApiOperation::DeleteEntry,
    ApiOperation::SaveVault,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobileSessionClient {
    client_kind: ClientKind,
}

impl MobileSessionClient {
    pub fn new(client_kind: ClientKind) -> Result<Self, ApiError> {
        if matches!(client_kind, ClientKind::IosApp | ClientKind::AndroidApp) {
            Ok(Self { client_kind })
        } else {
            Err(ApiError::stable(ErrorCode::CapabilityDenied))
        }
    }

    pub fn client_kind(self) -> ClientKind {
        self.client_kind
    }

    pub fn authorize(self, operation: ApiOperation) -> Result<(), ApiError> {
        self.client_kind
            .require_operation(operation)
            .map_err(|error| ApiError::stable(error.code))
    }

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileDocumentKind {
    IosSecurityScoped,
    IosAppScoped,
    AndroidStorageAccessFramework,
    AndroidAppScoped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileDocumentAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct MobileDocumentHandle(String);

impl MobileDocumentHandle {
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct DocumentRevision(String);

impl DocumentRevision {
    pub fn new(value: impl Into<String>) -> Result<Self, ApiError> {
        let value = value.into();

        if value.is_empty() || value.len() > 512 {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        Ok(Self(value))
    }

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

#[derive(Clone, PartialEq, Eq)]
pub struct EncryptedVaultBytes(Zeroizing<Vec<u8>>);

impl zeroize::ZeroizeOnDrop for EncryptedVaultBytes {}

impl EncryptedVaultBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl fmt::Debug for EncryptedVaultBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[encrypted vault bytes]")
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockMobileVaultRequest {
    pub password: SecretString,
}

impl zeroize::ZeroizeOnDrop for UnlockMobileVaultRequest {}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileVaultSummary {
    pub name: String,
    pub entry_count: usize,
    pub modified_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileSessionPhase {
    Empty,
    Locked,
    UnlockedClean,
    UnlockedDirty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileSessionStatus {
    pub phase: MobileSessionPhase,
    pub writable: bool,
    pub summary: Option<MobileVaultSummary>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretForCopy {
    secret: SecretString,
}

impl zeroize::ZeroizeOnDrop for SecretForCopy {}

impl SecretForCopy {
    pub fn new(secret: SecretString) -> Self {
        Self { secret }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrepareMobileVaultSaveRequest {
    pub observed_revision: DocumentRevision,
}

#[derive(Clone, PartialEq, Eq)]
pub struct PreparedMobileVaultWrite {
    pub document_handle: MobileDocumentHandle,
    pub expected_revision: DocumentRevision,
    pub encrypted_vault: EncryptedVaultBytes,
}

impl zeroize::ZeroizeOnDrop for PreparedMobileVaultWrite {}

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

pub struct MobileVaultSession {
    client: MobileSessionClient,
    state: MobileVaultSessionState,
}

enum MobileVaultSessionState {
    Empty,
    Locked(LockedMobileVault),
    Unlocked(UnlockedMobileVault),
}

struct LockedMobileVault {
    document: MobileDocument,
    encrypted_vault: EncryptedVaultBytes,
}

struct UnlockedMobileVault {
    document: MobileDocument,
    master_password: Zeroizing<String>,
    meta: VaultMeta,
    entries: Vec<VaultEntry>,
    dirty: bool,
    write_pending: bool,
}

impl MobileVaultSession {
    pub fn new(client_kind: ClientKind) -> Result<Self, ApiError> {
        Ok(Self {
            client: MobileSessionClient::new(client_kind)?,
            state: MobileVaultSessionState::Empty,
        })
    }

    pub fn status(&self) -> Result<MobileSessionStatus, ApiError> {
        self.client.authorize(ApiOperation::ReadVaultSummary)?;
        Ok(self.status_unchecked())
    }

    pub fn create(
        &mut self,
        request: CreateMobileVaultRequest,
    ) -> Result<PreparedMobileVaultWrite, ApiError> {
        self.client
            .authorize_write(ApiOperation::CreateVault, &request.document)?;
        self.ensure_empty()?;

        if request.name.trim().is_empty() {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        let (meta, encrypted_vault) =
            core_vault::create_vault_bytes(request.password.expose_secret(), request.name.as_str())
                .map_err(|_| ApiError::stable(ErrorCode::SaveFailed))?;
        let prepared_write = PreparedMobileVaultWrite {
            document_handle: request.document.handle.clone(),
            expected_revision: request.document.revision.clone(),
            encrypted_vault: EncryptedVaultBytes::new(encrypted_vault),
        };

        self.state = MobileVaultSessionState::Unlocked(UnlockedMobileVault {
            document: request.document,
            master_password: Zeroizing::new(request.password.into_inner()),
            meta,
            entries: Vec::new(),
            dirty: true,
            write_pending: true,
        });

        Ok(prepared_write)
    }

    pub fn open(
        &mut self,
        request: OpenMobileVaultRequest,
    ) -> Result<MobileSessionStatus, ApiError> {
        self.client.authorize(ApiOperation::OpenVault)?;
        self.client.authorize_document(&request.document)?;
        self.ensure_empty()?;

        self.state = MobileVaultSessionState::Locked(LockedMobileVault {
            document: request.document,
            encrypted_vault: request.encrypted_vault,
        });

        Ok(self.status_unchecked())
    }

    pub fn unlock(
        &mut self,
        request: UnlockMobileVaultRequest,
    ) -> Result<MobileVaultSummary, ApiError> {
        self.client.authorize(ApiOperation::UnlockVault)?;

        let MobileVaultSessionState::Locked(locked) = &self.state else {
            return Err(ApiError::stable(ErrorCode::VaultLocked));
        };

        let opened = core_vault::open_vault_bytes(
            locked.encrypted_vault.as_bytes(),
            request.password.expose_secret(),
        );

        let (meta, entries) = match opened {
            Ok(opened) => opened,
            Err(VaultError::InvalidPassword) => {
                return Err(ApiError::stable(ErrorCode::InvalidPassword));
            }
            Err(_) => {
                self.clear();
                return Err(ApiError::stable(ErrorCode::CorruptedVault));
            }
        };

        let MobileVaultSessionState::Locked(locked) =
            core::mem::replace(&mut self.state, MobileVaultSessionState::Empty)
        else {
            unreachable!("locked state was checked before decrypting")
        };
        let summary = mobile_summary(&meta, entries.len());

        self.state = MobileVaultSessionState::Unlocked(UnlockedMobileVault {
            document: locked.document,
            master_password: Zeroizing::new(request.password.into_inner()),
            meta,
            entries,
            dirty: false,
            write_pending: false,
        });

        Ok(summary)
    }

    pub fn lock(&mut self) -> Result<MobileSessionStatus, ApiError> {
        self.client.authorize(ApiOperation::LockVault)?;
        self.clear();
        Ok(self.status_unchecked())
    }

    pub fn summary(&self) -> Result<MobileVaultSummary, ApiError> {
        self.client.authorize(ApiOperation::ReadVaultSummary)?;
        let unlocked = self.unlocked()?;
        Ok(mobile_summary(&unlocked.meta, unlocked.entries.len()))
    }

    pub fn list_entries(&self) -> Result<Vec<EntryView>, ApiError> {
        self.client.authorize(ApiOperation::ListEntries)?;
        let unlocked = self.unlocked()?;
        Ok(unlocked.entries.iter().map(entry_view).collect())
    }

    pub fn get_entry(&self, entry_id: &str) -> Result<EntryView, ApiError> {
        self.client.authorize(ApiOperation::GetEntry)?;
        self.unlocked()?
            .entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .map(entry_view)
            .ok_or_else(|| ApiError::stable(ErrorCode::NotFound))
    }

    pub fn search_entries(&self, query: &str) -> Result<Vec<EntryView>, ApiError> {
        self.client.authorize(ApiOperation::SearchEntries)?;
        let unlocked = self.unlocked()?;

        Ok(core_entry::search_entries(&unlocked.entries, query)
            .into_iter()
            .map(entry_view)
            .collect())
    }

    pub fn reveal_secret(&self, entry_id: &str) -> Result<RevealedSecret, ApiError> {
        self.client.authorize(ApiOperation::RevealSecret)?;
        let entry = self.find_entry(entry_id)?;
        Ok(RevealedSecret::new(entry.password.clone()))
    }

    pub fn copy_secret(&self, entry_id: &str) -> Result<SecretForCopy, ApiError> {
        self.client.authorize(ApiOperation::CopySecret)?;
        let entry = self.find_entry(entry_id)?;
        Ok(SecretForCopy::new(SecretString::new(
            entry.password.clone(),
        )))
    }

    pub fn entry_history(&self, entry_id: &str) -> Result<Vec<RevisionView>, ApiError> {
        self.client.authorize(ApiOperation::ReadHistory)?;
        let entry = self.find_entry(entry_id)?;

        Ok(entry
            .revisions
            .iter()
            .take(DEFAULT_ENTRY_REVISION_LIMIT)
            .enumerate()
            .map(|(index, revision)| {
                let newer_password = if index == 0 {
                    entry.password.as_str()
                } else {
                    entry.revisions[index - 1].password.as_str()
                };
                revision_view(revision, revision.password.as_str() != newer_password)
            })
            .collect())
    }

    pub fn reveal_revision_secret(
        &self,
        entry_id: &str,
        revision_index: usize,
    ) -> Result<RevealedSecret, ApiError> {
        self.client.authorize(ApiOperation::RevealRevisionSecret)?;
        let revision = self.find_revision(entry_id, revision_index)?;
        Ok(RevealedSecret::new(revision.password.clone()))
    }

    pub fn copy_revision_secret(
        &self,
        entry_id: &str,
        revision_index: usize,
    ) -> Result<SecretForCopy, ApiError> {
        self.client.authorize(ApiOperation::CopySecret)?;
        let revision = self.find_revision(entry_id, revision_index)?;
        Ok(SecretForCopy::new(SecretString::new(
            revision.password.clone(),
        )))
    }

    pub fn generate_secret(&self, params: GeneratorParams) -> Result<GeneratedSecret, ApiError> {
        self.client.authorize(ApiOperation::GenerateSecret)?;
        let config = generator_config(params);
        let password = core_generator::generate_password(&config);

        if password.is_empty() {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        let entropy_bits = core_generator::calculate_entropy(&password, &config);
        Ok(GeneratedSecret::new(password, entropy_bits))
    }

    pub fn create_entry(&mut self, request: CreateEntryRequest) -> Result<EntryView, ApiError> {
        self.authorize_existing_write(ApiOperation::CreateEntry)?;
        self.ensure_no_pending_write()?;

        if request.password.expose_secret().is_empty() {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        let mut entry = core_entry::create_entry(
            &request.title,
            &request.username,
            request.password.expose_secret(),
        );
        entry.collection = request.collection;
        entry.url = request.url;
        entry.notes = request.notes;
        entry.tags = request.tags;
        let view = entry_view(&entry);
        let unlocked = self.unlocked_mut()?;
        unlocked.entries.push(entry);
        unlocked.dirty = true;

        Ok(view)
    }

    pub fn update_entry(
        &mut self,
        entry_id: &str,
        mutation: EntryMutation,
    ) -> Result<EntryView, ApiError> {
        self.authorize_existing_write(ApiOperation::UpdateEntry)?;
        self.ensure_no_pending_write()?;

        if mutation
            .password
            .as_ref()
            .is_some_and(|password| password.expose_secret().is_empty())
        {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        let unlocked = self.unlocked_mut()?;
        let entry = unlocked
            .entries
            .iter_mut()
            .find(|entry| entry.id == entry_id)
            .ok_or_else(|| ApiError::stable(ErrorCode::NotFound))?;
        core_entry::update_entry_with_revision_limit(
            entry,
            entry_patch(mutation),
            DEFAULT_ENTRY_REVISION_LIMIT,
        );
        let view = entry_view(entry);
        unlocked.dirty = true;

        Ok(view)
    }

    pub fn delete_entry(&mut self, entry_id: &str) -> Result<(), ApiError> {
        self.authorize_existing_write(ApiOperation::DeleteEntry)?;
        self.ensure_no_pending_write()?;

        let unlocked = self.unlocked_mut()?;
        let index = unlocked
            .entries
            .iter()
            .position(|entry| entry.id == entry_id)
            .ok_or_else(|| ApiError::stable(ErrorCode::NotFound))?;
        unlocked.entries.remove(index);
        unlocked.dirty = true;

        Ok(())
    }

    pub fn prepare_save(
        &mut self,
        request: PrepareMobileVaultSaveRequest,
    ) -> Result<PreparedMobileVaultWrite, ApiError> {
        self.authorize_existing_write(ApiOperation::SaveVault)?;
        self.ensure_no_pending_write()?;

        if self.unlocked()?.document.revision != request.observed_revision {
            self.clear();
            return Err(ApiError::stable(ErrorCode::ExternalFileChanged));
        }

        let unlocked = self.unlocked_mut()?;
        unlocked.meta.mark_modified_now();
        unlocked.dirty = true;
        let unlocked = self.unlocked()?;
        let encrypted_vault = match core_vault::save_vault_bytes(
            unlocked.master_password.as_str(),
            &unlocked.meta,
            &unlocked.entries,
        ) {
            Ok(encrypted_vault) => encrypted_vault,
            Err(_) => {
                self.unlocked_mut()?.dirty = true;
                return Err(ApiError::stable(ErrorCode::SaveFailed));
            }
        };
        let prepared_write = PreparedMobileVaultWrite {
            document_handle: unlocked.document.handle.clone(),
            expected_revision: unlocked.document.revision.clone(),
            encrypted_vault: EncryptedVaultBytes::new(encrypted_vault),
        };

        self.unlocked_mut()?.write_pending = true;
        Ok(prepared_write)
    }

    pub fn commit_write(
        &mut self,
        request: CommitMobileVaultWriteRequest,
    ) -> Result<MobileSessionStatus, ApiError> {
        self.authorize_existing_write(ApiOperation::SaveVault)?;
        let unlocked = self.unlocked_mut()?;

        if !unlocked.write_pending {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        unlocked.document.revision = request.committed_revision;
        unlocked.write_pending = false;
        unlocked.dirty = false;

        Ok(self.status_unchecked())
    }

    pub fn report_save_failure(&mut self) -> Result<(), ApiError> {
        self.authorize_existing_write(ApiOperation::SaveVault)?;
        let unlocked = self.unlocked_mut()?;

        if !unlocked.write_pending {
            return Err(ApiError::stable(ErrorCode::InvalidInput));
        }

        unlocked.write_pending = false;
        unlocked.dirty = true;
        Err(ApiError::stable(ErrorCode::SaveFailed))
    }

    pub fn report_document_permission_lost(&mut self) -> Result<(), ApiError> {
        self.client.authorize(ApiOperation::LockVault)?;
        self.clear();
        Err(ApiError::stable(ErrorCode::DocumentPermissionLost))
    }

    pub fn report_document_not_found(&mut self) -> Result<(), ApiError> {
        self.client.authorize(ApiOperation::LockVault)?;
        self.clear();
        Err(ApiError::stable(ErrorCode::FileNotFound))
    }

    fn authorize_existing_write(&self, operation: ApiOperation) -> Result<(), ApiError> {
        self.client.authorize(operation)?;
        let unlocked = self.unlocked()?;
        self.client.authorize_write(operation, &unlocked.document)
    }

    fn ensure_empty(&self) -> Result<(), ApiError> {
        if matches!(&self.state, MobileVaultSessionState::Empty) {
            Ok(())
        } else {
            Err(ApiError::stable(ErrorCode::InvalidInput))
        }
    }

    fn ensure_no_pending_write(&self) -> Result<(), ApiError> {
        if self.unlocked()?.write_pending {
            Err(ApiError::stable(ErrorCode::InvalidInput))
        } else {
            Ok(())
        }
    }

    fn status_unchecked(&self) -> MobileSessionStatus {
        match &self.state {
            MobileVaultSessionState::Empty => MobileSessionStatus {
                phase: MobileSessionPhase::Empty,
                writable: false,
                summary: None,
            },
            MobileVaultSessionState::Locked(locked) => MobileSessionStatus {
                phase: MobileSessionPhase::Locked,
                writable: locked.document.access == MobileDocumentAccess::ReadWrite,
                summary: None,
            },
            MobileVaultSessionState::Unlocked(unlocked) => MobileSessionStatus {
                phase: if unlocked.dirty {
                    MobileSessionPhase::UnlockedDirty
                } else {
                    MobileSessionPhase::UnlockedClean
                },
                writable: unlocked.document.access == MobileDocumentAccess::ReadWrite,
                summary: Some(mobile_summary(&unlocked.meta, unlocked.entries.len())),
            },
        }
    }

    fn unlocked(&self) -> Result<&UnlockedMobileVault, ApiError> {
        match &self.state {
            MobileVaultSessionState::Unlocked(unlocked) => Ok(unlocked),
            MobileVaultSessionState::Empty | MobileVaultSessionState::Locked(_) => {
                Err(ApiError::stable(ErrorCode::VaultLocked))
            }
        }
    }

    fn unlocked_mut(&mut self) -> Result<&mut UnlockedMobileVault, ApiError> {
        match &mut self.state {
            MobileVaultSessionState::Unlocked(unlocked) => Ok(unlocked),
            MobileVaultSessionState::Empty | MobileVaultSessionState::Locked(_) => {
                Err(ApiError::stable(ErrorCode::VaultLocked))
            }
        }
    }

    fn find_entry(&self, entry_id: &str) -> Result<&VaultEntry, ApiError> {
        self.unlocked()?
            .entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .ok_or_else(|| ApiError::stable(ErrorCode::NotFound))
    }

    fn find_revision(
        &self,
        entry_id: &str,
        revision_index: usize,
    ) -> Result<&EntryRevision, ApiError> {
        self.find_entry(entry_id)?
            .revisions
            .get(revision_index)
            .ok_or_else(|| ApiError::stable(ErrorCode::NotFound))
    }

    fn clear(&mut self) {
        self.state = MobileVaultSessionState::Empty;
    }
}

fn mobile_summary(meta: &VaultMeta, entry_count: usize) -> MobileVaultSummary {
    MobileVaultSummary {
        name: meta.name.clone(),
        entry_count,
        modified_at: meta.modified_at.clone(),
    }
}

fn entry_view(entry: &VaultEntry) -> EntryView {
    EntryView {
        id: entry.id.clone(),
        title: entry.title.clone(),
        username: entry.username.clone(),
        collection: entry.collection.clone(),
        url: entry.url.clone(),
        notes: entry.notes.clone(),
        tags: entry.tags.clone(),
        created_at: entry.created_at.clone(),
        updated_at: entry.updated_at.clone(),
        revision_count: entry.revisions.len(),
    }
}

fn revision_view(revision: &EntryRevision, password_changed: bool) -> RevisionView {
    RevisionView {
        captured_at: revision.captured_at.clone(),
        updated_at: revision.updated_at.clone(),
        title: revision.title.clone(),
        username: revision.username.clone(),
        collection: revision.collection.clone(),
        url: revision.url.clone(),
        notes: revision.notes.clone(),
        tags: revision.tags.clone(),
        password_changed,
    }
}

fn entry_patch(mutation: EntryMutation) -> EntryPatch {
    EntryPatch {
        title: mutation.title,
        username: mutation.username,
        password: mutation.password.map(SecretString::into_inner),
        collection: mutation.collection,
        url: mutation.url,
        notes: mutation.notes,
        tags: mutation.tags,
    }
}

fn generator_config(params: GeneratorParams) -> CoreGeneratorConfig {
    let default = CoreGeneratorConfig::default();

    CoreGeneratorConfig {
        length: params.length.unwrap_or(default.length),
        uppercase: params.uppercase.unwrap_or(default.uppercase),
        lowercase: params.lowercase.unwrap_or(default.lowercase),
        digits: params.digits.unwrap_or(default.digits),
        symbols: params.symbols.unwrap_or(default.symbols),
        exclude_ambiguous: params
            .exclude_ambiguous
            .unwrap_or(default.exclude_ambiguous),
        mode: match params.mode {
            Some(GeneratorMode::Random) | None => CoreGeneratorMode::Random,
            Some(GeneratorMode::Passphrase) => CoreGeneratorMode::Passphrase,
        },
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MobileFailureAction {
    KeepState,
    StayLocked,
    LockAndDiscard,
    KeepUnlockedDirty,
}

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
    use std::sync::OnceLock;
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

    #[test]
    fn session_create_mutate_and_two_phase_save_round_trip() {
        let mut session =
            MobileVaultSession::new(ClientKind::IosApp).expect("session should be created");
        let password = unique_test_secret();
        let prepared_create = session
            .create(CreateMobileVaultRequest {
                document: document(MobileDocumentKind::IosAppScoped),
                name: "Synthetic vault".to_string(),
                password: SecretString::new(password.clone()),
            })
            .expect("vault creation should prepare a write");

        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::UnlockedDirty
        );
        assert!(
            session
                .create_entry(entry_request())
                .expect_err("a pending write should serialize mutations")
                .code
                == ErrorCode::InvalidInput
        );

        session
            .commit_write(CommitMobileVaultWriteRequest {
                committed_revision: revision("revision-created"),
            })
            .expect("create write should commit");
        session
            .unlocked_mut()
            .expect("created vault should be unlocked")
            .meta
            .modified_at = "2000-01-01T00:00:00+00:00".to_string();

        let entry_secret = unique_test_secret();
        let created = session
            .create_entry(CreateEntryRequest {
                title: "GitHub".to_string(),
                username: "arca".to_string(),
                password: SecretString::new(entry_secret.clone()),
                collection: Some("work".to_string()),
                url: Some("https://github.com".to_string()),
                notes: None,
                tags: vec!["dev".to_string()],
            })
            .expect("entry should be created in memory");

        assert_eq!(
            session.list_entries().expect("entries should list").len(),
            1
        );
        assert_eq!(
            session
                .search_entries("#dev")
                .expect("entries should search")
                .len(),
            1
        );
        assert!(
            session
                .reveal_secret(&created.id)
                .expect("secret should reveal")
                .expose_secret()
                == entry_secret,
            "revealed secret should match the entry"
        );
        assert!(
            session
                .copy_secret(&created.id)
                .expect("secret should copy")
                .expose_secret()
                == entry_secret,
            "copied secret should match the entry"
        );
        assert_eq!(
            session
                .get_entry(&created.id)
                .expect("entry should be readable"),
            created
        );

        let generated = session
            .generate_secret(GeneratorParams::default())
            .expect("secret should generate");
        assert!(!generated.expose_secret().is_empty());

        let deleted = session
            .create_entry(entry_request())
            .expect("second entry should be created");
        session
            .delete_entry(&deleted.id)
            .expect("second entry should be deleted");

        let updated = session
            .update_entry(
                &created.id,
                EntryMutation {
                    title: Some("GitHub Enterprise".to_string()),
                    ..EntryMutation::default()
                },
            )
            .expect("entry should update in memory");
        assert_eq!(updated.title, "GitHub Enterprise");
        assert_eq!(updated.revision_count, 1);

        let replacement_secret = unique_test_secret();
        let updated = session
            .update_entry(
                &created.id,
                EntryMutation {
                    password: Some(SecretString::new(replacement_secret)),
                    ..EntryMutation::default()
                },
            )
            .expect("entry password should update in memory");
        assert_eq!(updated.revision_count, 2);

        let history = session
            .entry_history(&created.id)
            .expect("history metadata should be readable");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].title, "GitHub Enterprise");
        assert!(history[0].password_changed);
        assert!(!history[1].password_changed);
        assert_eq!(history[1].title, "GitHub");
        assert!(
            session
                .reveal_revision_secret(&created.id, 0)
                .expect("one historical secret should reveal")
                .expose_secret()
                == entry_secret
        );
        assert!(
            session
                .copy_revision_secret(&created.id, 0)
                .expect("one historical secret should copy")
                .expose_secret()
                == entry_secret
        );
        assert_eq!(
            session
                .reveal_revision_secret(&created.id, 99)
                .expect_err("an invalid revision index should fail")
                .code,
            ErrorCode::NotFound
        );

        let prepared_save = session
            .prepare_save(PrepareMobileVaultSaveRequest {
                observed_revision: revision("revision-created"),
            })
            .expect("dirty vault should prepare a save");
        let saved_bytes = prepared_save.encrypted_vault.clone();
        assert_ne!(
            session
                .summary()
                .expect("summary should reflect save preparation")
                .modified_at,
            "2000-01-01T00:00:00+00:00"
        );

        session
            .commit_write(CommitMobileVaultWriteRequest {
                committed_revision: revision("revision-saved"),
            })
            .expect("save should commit");
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::UnlockedClean
        );

        let (saved_meta, saved_entries) =
            core_vault::open_vault_bytes(saved_bytes.as_bytes(), &password)
                .expect("prepared bytes should be a valid KDBX vault");
        assert_eq!(saved_meta.name, "Synthetic vault");
        assert_ne!(saved_meta.modified_at, "2000-01-01T00:00:00+00:00");
        assert_eq!(saved_entries.len(), 1);
        assert_eq!(saved_entries[0].title, "GitHub Enterprise");

        assert!(!format!("{prepared_create:?}").contains(&password));
        assert!(!format!("{prepared_save:?}").contains(&entry_secret));

        assert_eq!(
            session.lock().expect("lock should succeed").phase,
            MobileSessionPhase::Empty
        );
        assert_eq!(
            session
                .reveal_secret(&created.id)
                .expect_err("lock should discard decrypted entries")
                .code,
            ErrorCode::VaultLocked
        );
        assert_eq!(
            session
                .entry_history(&created.id)
                .expect_err("lock should discard revision metadata")
                .code,
            ErrorCode::VaultLocked
        );
        assert_eq!(
            session
                .reveal_revision_secret(&created.id, 0)
                .expect_err("lock should discard historical secrets")
                .code,
            ErrorCode::VaultLocked
        );
    }

    #[test]
    fn invalid_password_keeps_locked_bytes_for_retry_and_corruption_clears_them() {
        let (password, encrypted_vault) = encrypted_fixture();
        let mut session =
            MobileVaultSession::new(ClientKind::AndroidApp).expect("session should be created");
        session
            .open(open_request(encrypted_vault.clone()))
            .expect("encrypted vault should stage");

        let error = session
            .unlock(UnlockMobileVaultRequest {
                password: SecretString::new(unique_test_secret()),
            })
            .expect_err("invalid password should fail");
        assert_eq!(error.code, ErrorCode::InvalidPassword);
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::Locked
        );

        session
            .unlock(UnlockMobileVaultRequest {
                password: SecretString::new(password),
            })
            .expect("correct password should retry the staged bytes");
        session
            .lock()
            .expect("current vault should be locked first");

        session
            .open(open_request(EncryptedVaultBytes::new(vec![1, 2, 3, 4])))
            .expect("corrupted bytes can be staged before decrypting");
        let error = session
            .unlock(UnlockMobileVaultRequest {
                password: SecretString::new(unique_test_secret()),
            })
            .expect_err("corrupted bytes should fail closed");
        assert_eq!(error.code, ErrorCode::CorruptedVault);
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::Empty
        );
    }

    #[test]
    fn document_failures_and_external_changes_lock_and_discard_state() {
        let (password, encrypted_vault) = encrypted_fixture();
        let mut session = unlocked_session(password.clone(), encrypted_vault.clone());
        let error = session
            .prepare_save(PrepareMobileVaultSaveRequest {
                observed_revision: revision("revision-external"),
            })
            .expect_err("revision mismatch should fail closed");
        assert_eq!(error.code, ErrorCode::ExternalFileChanged);
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::Empty
        );

        let mut session = unlocked_session(password, encrypted_vault.clone());
        let error = session
            .report_document_permission_lost()
            .expect_err("permission loss should fail closed");
        assert_eq!(error.code, ErrorCode::DocumentPermissionLost);
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::Empty
        );

        let mut session =
            MobileVaultSession::new(ClientKind::AndroidApp).expect("session should be created");
        session
            .open(open_request(encrypted_vault))
            .expect("vault should stage");
        let error = session
            .report_document_not_found()
            .expect_err("missing document should fail closed");
        assert_eq!(error.code, ErrorCode::FileNotFound);
    }

    #[test]
    fn save_failure_keeps_dirty_state_for_retry() {
        let (password, encrypted_vault) = encrypted_fixture();
        let mut session = unlocked_session(password, encrypted_vault);
        session
            .create_entry(entry_request())
            .expect("entry should be created in memory");
        session
            .prepare_save(PrepareMobileVaultSaveRequest {
                observed_revision: revision("revision-3"),
            })
            .expect("dirty vault should prepare a save");

        let error = session
            .report_save_failure()
            .expect_err("native save failure should be stable");
        assert_eq!(error.code, ErrorCode::SaveFailed);
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::UnlockedDirty
        );

        session
            .prepare_save(PrepareMobileVaultSaveRequest {
                observed_revision: revision("revision-3"),
            })
            .expect("save should be retryable");
    }

    #[test]
    fn active_sessions_cannot_be_replaced_without_an_explicit_lock() {
        let (password, encrypted_vault) = encrypted_fixture();
        let mut session =
            MobileVaultSession::new(ClientKind::AndroidApp).expect("session should be created");
        session
            .open(open_request(encrypted_vault.clone()))
            .expect("vault should stage");

        assert_eq!(
            session
                .open(open_request(encrypted_vault.clone()))
                .expect_err("a locked session should retain its staged vault")
                .code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            session
                .create(CreateMobileVaultRequest {
                    document: document(MobileDocumentKind::AndroidAppScoped),
                    name: "Replacement vault".to_string(),
                    password: SecretString::new(unique_test_secret()),
                })
                .expect_err("create should not replace a locked session")
                .code,
            ErrorCode::InvalidInput
        );

        session
            .unlock(UnlockMobileVaultRequest {
                password: SecretString::new(password),
            })
            .expect("original staged vault should still unlock");
        let entry = session
            .create_entry(entry_request())
            .expect("entry should be created in memory");

        assert_eq!(
            session
                .open(open_request(encrypted_vault.clone()))
                .expect_err("open should not discard unsaved changes")
                .code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            session
                .create(CreateMobileVaultRequest {
                    document: document(MobileDocumentKind::AndroidAppScoped),
                    name: "Replacement vault".to_string(),
                    password: SecretString::new(unique_test_secret()),
                })
                .expect_err("create should not discard unsaved changes")
                .code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::UnlockedDirty
        );
        session
            .get_entry(&entry.id)
            .expect("original entry should remain available");

        session
            .prepare_save(PrepareMobileVaultSaveRequest {
                observed_revision: revision("revision-3"),
            })
            .expect("dirty vault should prepare a save");
        assert_eq!(
            session
                .open(open_request(encrypted_vault))
                .expect_err("open should not orphan a pending write")
                .code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            session
                .create(CreateMobileVaultRequest {
                    document: document(MobileDocumentKind::AndroidAppScoped),
                    name: "Replacement vault".to_string(),
                    password: SecretString::new(unique_test_secret()),
                })
                .expect_err("create should not orphan a pending write")
                .code,
            ErrorCode::InvalidInput
        );
        assert_eq!(
            session
                .commit_write(CommitMobileVaultWriteRequest {
                    committed_revision: revision("revision-committed"),
                })
                .expect("original pending write should still commit")
                .phase,
            MobileSessionPhase::UnlockedClean
        );
        assert_eq!(
            session
                .open(open_request(EncryptedVaultBytes::new(vec![1])))
                .expect_err("a clean session still requires explicit lock")
                .code,
            ErrorCode::InvalidInput
        );
        session
            .prepare_save(PrepareMobileVaultSaveRequest {
                observed_revision: revision("revision-committed"),
            })
            .expect("a clean session can prepare a write");
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::UnlockedDirty
        );
        session
            .lock()
            .expect("explicit lock should clear the session");
        assert_eq!(
            session
                .open(open_request(EncryptedVaultBytes::new(vec![1])))
                .expect("a new document can open after locking")
                .phase,
            MobileSessionPhase::Locked
        );
    }

    #[test]
    fn read_only_documents_reject_mutation_without_changing_state() {
        let (password, encrypted_vault) = encrypted_fixture();
        let mut request = open_request(encrypted_vault);
        request.document.access = MobileDocumentAccess::ReadOnly;
        let mut session =
            MobileVaultSession::new(ClientKind::AndroidApp).expect("session should be created");
        session.open(request).expect("vault should stage");
        assert_eq!(
            session
                .create_entry(entry_request())
                .expect_err("locked state should take precedence over document access")
                .code,
            ErrorCode::VaultLocked
        );
        assert_eq!(
            session
                .prepare_save(PrepareMobileVaultSaveRequest {
                    observed_revision: revision("revision-3"),
                })
                .expect_err("locked state should take precedence over document access")
                .code,
            ErrorCode::VaultLocked
        );
        session
            .unlock(UnlockMobileVaultRequest {
                password: SecretString::new(password),
            })
            .expect("vault should unlock");

        let error = session
            .create_entry(entry_request())
            .expect_err("read-only document should reject mutation");
        assert_eq!(error.code, ErrorCode::DocumentReadOnly);
        assert_eq!(
            session.status().expect("status should be available").phase,
            MobileSessionPhase::UnlockedClean
        );
    }

    #[test]
    fn capability_checks_precede_every_session_dispatch() {
        let denied_client = MobileSessionClient {
            client_kind: ClientKind::FutureSyncServer,
        };
        let mut session = MobileVaultSession {
            client: denied_client,
            state: MobileVaultSessionState::Empty,
        };

        assert_denied(session.status());
        assert_denied(session.create(CreateMobileVaultRequest {
            document: document(MobileDocumentKind::AndroidAppScoped),
            name: "Synthetic vault".to_string(),
            password: SecretString::new(unique_test_secret()),
        }));
        assert_denied(session.open(open_request(EncryptedVaultBytes::new(vec![1]))));
        assert_denied(session.unlock(UnlockMobileVaultRequest {
            password: SecretString::new(unique_test_secret()),
        }));
        assert_denied(session.lock());
        assert_denied(session.summary());
        assert_denied(session.list_entries());
        assert_denied(session.get_entry("entry-id"));
        assert_denied(session.search_entries("query"));
        assert_denied(session.reveal_secret("entry-id"));
        assert_denied(session.copy_secret("entry-id"));
        assert_denied(session.entry_history("entry-id"));
        assert_denied(session.reveal_revision_secret("entry-id", 0));
        assert_denied(session.copy_revision_secret("entry-id", 0));
        assert_denied(session.generate_secret(GeneratorParams::default()));
        assert_denied(session.create_entry(entry_request()));
        assert_denied(session.update_entry("entry-id", EntryMutation::default()));
        assert_denied(session.delete_entry("entry-id"));
        assert_denied(session.prepare_save(PrepareMobileVaultSaveRequest {
            observed_revision: revision("revision-3"),
        }));
        assert_denied(session.commit_write(CommitMobileVaultWriteRequest {
            committed_revision: revision("revision-4"),
        }));
        assert_denied(session.report_save_failure());
        assert_denied(session.report_document_permission_lost());
        assert_denied(session.report_document_not_found());
    }

    fn assert_denied<T>(result: Result<T, ApiError>) {
        match result {
            Ok(_) => panic!("operation should be denied"),
            Err(error) => assert_eq!(error.code, ErrorCode::CapabilityDenied),
        }
    }

    fn encrypted_fixture() -> (String, EncryptedVaultBytes) {
        static FIXTURE: OnceLock<(String, Vec<u8>)> = OnceLock::new();
        let (password, bytes) = FIXTURE.get_or_init(|| {
            let password = unique_test_secret();
            let (_, bytes) = core_vault::create_vault_bytes(&password, "Synthetic vault")
                .expect("fixture vault should be created");
            (password, bytes)
        });

        (password.clone(), EncryptedVaultBytes::new(bytes.clone()))
    }

    fn unlocked_session(
        password: String,
        encrypted_vault: EncryptedVaultBytes,
    ) -> MobileVaultSession {
        let mut session =
            MobileVaultSession::new(ClientKind::AndroidApp).expect("session should be created");
        session
            .open(open_request(encrypted_vault))
            .expect("vault should stage");
        session
            .unlock(UnlockMobileVaultRequest {
                password: SecretString::new(password),
            })
            .expect("vault should unlock");
        session
    }

    fn open_request(encrypted_vault: EncryptedVaultBytes) -> OpenMobileVaultRequest {
        OpenMobileVaultRequest {
            document: document(MobileDocumentKind::AndroidAppScoped),
            encrypted_vault,
        }
    }

    fn entry_request() -> CreateEntryRequest {
        CreateEntryRequest {
            title: "GitHub".to_string(),
            username: "arca".to_string(),
            password: SecretString::new(unique_test_secret()),
            collection: None,
            url: None,
            notes: None,
            tags: Vec::new(),
        }
    }

    fn revision(value: &str) -> DocumentRevision {
        DocumentRevision::new(value).expect("revision should be valid")
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
