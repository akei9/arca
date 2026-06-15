import { invoke } from '@tauri-apps/api/core';

export type Theme = 'terminal' | 'amber';
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
  password: string | null;
  url: string | null;
  notes: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

export interface CreateEntryDto {
  title: string;
  username: string;
  password: string;
  url?: string | null;
  notes?: string | null;
  tags?: string[];
}

export interface UpdateEntryDto {
  title?: string;
  username?: string;
  password?: string;
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

export interface Settings {
  autoLockTimeoutMinutes?: number | null;
  clipboardClearSeconds?: number | null;
  theme: Theme;
  fontSize: number;
}

export function unlockVault(path: string, password: string): Promise<VaultInfo> {
  return invoke('unlock_vault', { path, password });
}

export function lockVault(): Promise<void> {
  return invoke('lock_vault');
}

export function createVault(path: string, password: string, name: string): Promise<void> {
  return invoke('create_vault', { path, password, name });
}

export function listEntries(): Promise<EntryDto[]> {
  return invoke('list_entries');
}

export function getEntry(id: string): Promise<EntryDto> {
  return invoke('get_entry', { id });
}

export function createEntry(data: CreateEntryDto): Promise<EntryDto> {
  return invoke('create_entry', { data });
}

export function updateEntry(id: string, data: UpdateEntryDto): Promise<EntryDto> {
  return invoke('update_entry', { id, data });
}

export function deleteEntry(id: string): Promise<void> {
  return invoke('delete_entry', { id });
}

export function searchEntries(query: string): Promise<EntryDto[]> {
  return invoke('search_entries', { query });
}

export function generatePassword(config: GeneratorConfigDto): Promise<GeneratedPassword> {
  return invoke('generate_password', { config });
}

export function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export function updateSettings(settings: Settings): Promise<void> {
  return invoke('update_settings', { settings });
}
