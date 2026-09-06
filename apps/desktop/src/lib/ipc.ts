import { invoke } from '@tauri-apps/api/core';

export type Theme = 'paper' | 'ink';
export type LegacyTheme = 'terminal' | 'amber';
export type SettingsTheme = Theme | LegacyTheme;
export type GeneratorMode = 'random' | 'passphrase';
export type ClientKind =
  | 'desktopApp'
  | 'browserExtension'
  | 'iosApp'
  | 'androidApp'
  | 'iosAutofillExtension'
  | 'androidAutofillService'
  | 'futureSyncServer';
export type Capability =
  | 'unlock'
  | 'readMeta'
  | 'revealSecret'
  | 'copySecret'
  | 'mutateEntry'
  | 'createVault'
  | 'changeKdf'
  | 'exportPlaintext'
  | 'exportKdbx'
  | 'readHistory'
  | 'deletePermanent';
export type AuditSeverity = 'high' | 'medium' | 'low';
export type AuditFindingKind =
  | 'weakPassword'
  | 'reusedPassword'
  | 'insecureUrl'
  | 'duplicateUrl'
  | 'duplicateUsername'
  | 'staleEntry'
  | 'missingUrl'
  | 'missingCollection'
  | 'untagged';
export type ErrorCode =
  | 'invalid_password'
  | 'file_not_found'
  | 'corrupted_vault'
  | 'encryption_error'
  | 'decryption_error'
  | 'io_error'
  | 'serialization_error'
  | 'vault_locked'
  | 'not_found'
  | 'invalid_input'
  | 'capability_denied';

export interface ContractVersions {
  kdbxFormatVersion: number;
  arcaSemanticsVersion: number;
  clientProtocolVersion: number;
}

export interface VaultInfo {
  name: string;
  path: string;
  entryCount: number;
  modifiedAt: string;
}

export interface EntryDto {
  id: string;
  title: string;
  username: string;
  collection: string | null;
  url: string | null;
  notes: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  revisionCount: number;
}

export interface RevisionDto {
  capturedAt: string;
  updatedAt: string;
  title: string;
  username: string;
  collection: string | null;
  url: string | null;
  notes: string | null;
  tags: string[];
  passwordChanged: boolean;
}

export interface CreateEntryDto {
  title: string;
  username: string;
  password: string;
  collection?: string | null;
  url?: string | null;
  notes?: string | null;
  tags?: string[];
}

export interface UpdateEntryDto {
  title?: string;
  username?: string;
  /**
   * Omit to keep the existing password. Passwordless entries are unsupported in
   * this release, so callers must send a non-empty replacement when changing it.
   */
  password?: string;
  collection?: string | null;
  url?: string | null;
  notes?: string | null;
  tags?: string[];
}

export interface GeneratorConfigDto {
  length?: number;
  uppercase?: boolean;
  lowercase?: boolean;
  digits?: boolean;
  symbols?: boolean;
  excludeAmbiguous?: boolean;
  mode?: GeneratorMode;
}

export interface GeneratedPassword {
  password: string;
  entropyBits: number;
}

export interface RevealedSecret {
  secret: string;
}

export interface AuditFindingDto {
  key: string;
  severity: AuditSeverity;
  kind: AuditFindingKind;
  entryId: string;
  meta: string;
}

export interface ClientCapabilitiesDto {
  clientKind: ClientKind;
  capabilities: Capability[];
}

export interface ApiErrorDto {
  code: ErrorCode;
  message: string;
}

export interface PathSuggestion {
  name: string;
  path: string;
  kind: 'directory' | 'file';
  vaultCandidate: boolean;
}

export interface Settings {
  autoLockTimeoutMinutes?: number | null;
  clipboardClearSeconds?: number | null;
  entryRevisionLimit?: number;
  theme: SettingsTheme;
  fontSize: number;
}

interface IpcCommandMap {
  unlock_vault: {
    args: { path: string; password: string };
    result: VaultInfo;
  };
  lock_vault: {
    args: undefined;
    result: void;
  };
  create_vault: {
    args: { path: string; password: string; name: string };
    result: void;
  };
  list_entries: {
    args: undefined;
    result: EntryDto[];
  };
  get_entry: {
    args: { id: string };
    result: EntryDto;
  };
  reveal_entry_password: {
    args: { id: string };
    result: RevealedSecret;
  };
  get_entry_revisions: {
    args: { id: string };
    result: RevisionDto[];
  };
  reveal_entry_revision_password: {
    args: { id: string; index: number };
    result: RevealedSecret;
  };
  create_entry: {
    args: { data: CreateEntryDto };
    result: EntryDto;
  };
  update_entry: {
    args: { id: string; data: UpdateEntryDto };
    result: EntryDto;
  };
  delete_entry: {
    args: { id: string };
    result: void;
  };
  search_entries: {
    args: { query: string };
    result: EntryDto[];
  };
  suggest_paths: {
    args: { partial: string };
    result: PathSuggestion[];
  };
  generate_password: {
    args: { config: GeneratorConfigDto };
    result: GeneratedPassword;
  };
  get_settings: {
    args: undefined;
    result: Settings;
  };
  update_settings: {
    args: { settings: Settings };
    result: void;
  };
}

type IpcCommand = keyof IpcCommandMap;
type IpcCommandArgs<Command extends IpcCommand> = IpcCommandMap[Command]['args'];
type IpcCommandResult<Command extends IpcCommand> = IpcCommandMap[Command]['result'];

function invokeCommand<Command extends IpcCommand>(
  command: Command,
  ...args: IpcCommandArgs<Command> extends undefined ? [] : [IpcCommandArgs<Command>]
): Promise<IpcCommandResult<Command>> {
  return args.length === 0
    ? invoke<IpcCommandResult<Command>>(command)
    : invoke<IpcCommandResult<Command>>(command, args[0]);
}

/** Opens an existing vault and returns its summary metadata. */
export function unlockVault(path: string, password: string): Promise<VaultInfo> {
  return invokeCommand('unlock_vault', { path, password });
}

/** Locks the active vault session in the desktop backend. */
export function lockVault(): Promise<void> {
  return invokeCommand('lock_vault');
}

/** Creates a new vault file and opens it as the active session. */
export function createVault(path: string, password: string, name: string): Promise<void> {
  return invokeCommand('create_vault', { path, password, name });
}

/** Lists metadata-only entry views for the active vault. */
export function listEntries(): Promise<EntryDto[]> {
  return invokeCommand('list_entries');
}

/** Loads a metadata-only entry view by id. */
export function getEntry(id: string): Promise<EntryDto> {
  return invokeCommand('get_entry', { id });
}

/** Reveals an entry password through an explicit secret-bearing command. */
export async function revealEntryPassword(id: string): Promise<string> {
  const response = await invokeCommand('reveal_entry_password', { id });
  return response.secret;
}

/** Lists metadata-only revision views for an entry. */
export function getEntryRevisions(id: string): Promise<RevisionDto[]> {
  return invokeCommand('get_entry_revisions', { id });
}

/** Reveals a historical revision password through an explicit command. */
export async function revealEntryRevisionPassword(id: string, index: number): Promise<string> {
  const response = await invokeCommand('reveal_entry_revision_password', { id, index });
  return response.secret;
}

/** Creates an entry from a secret-bearing request payload. */
export function createEntry(data: CreateEntryDto): Promise<EntryDto> {
  return invokeCommand('create_entry', { data });
}

/** Updates an entry and omits password unless replacing it. */
export function updateEntry(id: string, data: UpdateEntryDto): Promise<EntryDto> {
  return invokeCommand('update_entry', { id, data });
}

/** Deletes an entry from the active vault. */
export function deleteEntry(id: string): Promise<void> {
  return invokeCommand('delete_entry', { id });
}

/** Searches entries and returns metadata-only entry views. */
export function searchEntries(query: string): Promise<EntryDto[]> {
  return invokeCommand('search_entries', { query });
}

/** Suggests local filesystem paths for vault selection. */
export function suggestPaths(partial: string): Promise<PathSuggestion[]> {
  return invokeCommand('suggest_paths', { partial });
}

/** Generates a password from the configured generator options. */
export function generatePassword(config: GeneratorConfigDto): Promise<GeneratedPassword> {
  return invokeCommand('generate_password', { config });
}

/** Loads persisted desktop settings. */
export function getSettings(): Promise<Settings> {
  return invokeCommand('get_settings');
}

/** Persists desktop settings. */
export function updateSettings(settings: Settings): Promise<void> {
  return invokeCommand('update_settings', { settings });
}
