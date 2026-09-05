import { invoke } from '@tauri-apps/api/core';

export type Theme = 'paper' | 'ink';
export type LegacyTheme = 'terminal' | 'amber';
export type SettingsTheme = Theme | LegacyTheme;
export type GeneratorMode = 'random' | 'passphrase';

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

/** Opens an existing vault and returns its summary metadata. */
export function unlockVault(path: string, password: string): Promise<VaultInfo> {
  return invoke('unlock_vault', { path, password });
}

/** Locks the active vault session in the desktop backend. */
export function lockVault(): Promise<void> {
  return invoke('lock_vault');
}

/** Creates a new vault file and opens it as the active session. */
export function createVault(path: string, password: string, name: string): Promise<void> {
  return invoke('create_vault', { path, password, name });
}

/** Lists metadata-only entry views for the active vault. */
export function listEntries(): Promise<EntryDto[]> {
  return invoke('list_entries');
}

/** Loads a metadata-only entry view by id. */
export function getEntry(id: string): Promise<EntryDto> {
  return invoke('get_entry', { id });
}

/** Reveals an entry password through an explicit secret-bearing command. */
export async function revealEntryPassword(id: string): Promise<string> {
  const response = await invoke<RevealedSecret>('reveal_entry_password', { id });
  return response.secret;
}

/** Lists metadata-only revision views for an entry. */
export function getEntryRevisions(id: string): Promise<RevisionDto[]> {
  return invoke('get_entry_revisions', { id });
}

/** Reveals a historical revision password through an explicit command. */
export async function revealEntryRevisionPassword(id: string, index: number): Promise<string> {
  const response = await invoke<RevealedSecret>('reveal_entry_revision_password', { id, index });
  return response.secret;
}

/** Creates an entry from a secret-bearing request payload. */
export function createEntry(data: CreateEntryDto): Promise<EntryDto> {
  return invoke('create_entry', { data });
}

/** Updates an entry and omits password unless replacing it. */
export function updateEntry(id: string, data: UpdateEntryDto): Promise<EntryDto> {
  return invoke('update_entry', { id, data });
}

/** Deletes an entry from the active vault. */
export function deleteEntry(id: string): Promise<void> {
  return invoke('delete_entry', { id });
}

/** Searches entries and returns metadata-only entry views. */
export function searchEntries(query: string): Promise<EntryDto[]> {
  return invoke('search_entries', { query });
}

/** Suggests local filesystem paths for vault selection. */
export function suggestPaths(partial: string): Promise<PathSuggestion[]> {
  return invoke('suggest_paths', { partial });
}

/** Generates a password from the configured generator options. */
export function generatePassword(config: GeneratorConfigDto): Promise<GeneratedPassword> {
  return invoke('generate_password', { config });
}

/** Loads persisted desktop settings. */
export function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

/** Persists desktop settings. */
export function updateSettings(settings: Settings): Promise<void> {
  return invoke('update_settings', { settings });
}
