import type { RecentVault } from './ipc';

export interface PassphraseStrength {
  bits: number;
  level: 0 | 1 | 2 | 3 | 4;
  label: 'empty' | 'weak' | 'fair' | 'strong' | 'excellent';
}

export interface LockSecretState {
  password: string;
  confirmation: string;
  passwordRevealed: boolean;
  confirmationRevealed: boolean;
}

export function emptyLockSecretState(): LockSecretState {
  return {
    password: '',
    confirmation: '',
    passwordRevealed: false,
    confirmationRevealed: false,
  };
}

export function nextSelectionIndex(index: number, length: number, direction: 1 | -1): number {
  return length > 0 ? (index + direction + length) % length : 0;
}

export function firstDialogPath(selection: string | string[] | null): string | null {
  return Array.isArray(selection) ? selection[0] ?? null : selection;
}

export function restoreFocusAfterDialogCancel(
  selection: string | string[] | null,
  trigger: { focus(): void } | null,
): boolean {
  if (firstDialogPath(selection)) return false;
  trigger?.focus();
  return true;
}

export function validateNewPassphrase(password: string, confirmation: string): string | null {
  if (password.length < 8) {
    return 'use at least 8 characters';
  }

  if (password !== confirmation) {
    return "passphrases don't match";
  }

  return null;
}

export function estimatePassphraseStrength(password: string): PassphraseStrength {
  if (!password) {
    return { bits: 0, level: 0, label: 'empty' };
  }

  let pool = 0;
  if (/[a-z]/.test(password)) pool += 26;
  if (/[A-Z]/.test(password)) pool += 26;
  if (/\d/.test(password)) pool += 10;
  if (/[^A-Za-z0-9]/.test(password)) pool += 32;

  const bits = Math.round(password.length * Math.log2(Math.max(pool, 1)));
  if (bits < 40) return { bits, level: 1, label: 'weak' };
  if (bits < 60) return { bits, level: 2, label: 'fair' };
  if (bits < 80) return { bits, level: 3, label: 'strong' };
  return { bits, level: 4, label: 'excellent' };
}

export function filterRecentVaults(recents: RecentVault[], query: string): RecentVault[] {
  const normalized = query.trim().toLowerCase().replace(/^~[\\/]/, '');
  if (!normalized) {
    return recents;
  }

  return recents.filter((vault) => vault.path.toLowerCase().includes(normalized));
}

export function vaultDisplayName(path: string): string {
  const pieces = path.split(/[\\/]/).filter(Boolean);
  return pieces.at(-1) || 'vault.arca';
}

export function vaultNameWithoutExtension(name: string): string {
  return name.replace(/\.(arca|kdbx)$/i, '');
}

export function parentPath(path: string): string {
  const separator = path.includes('\\') && !path.includes('/') ? '\\' : '/';
  const pieces = path.split(/[\\/]/);
  pieces.pop();
  const parent = pieces.join(separator) || separator;
  return parent.endsWith(separator) ? parent : `${parent}${separator}`;
}

export function relativeRecency(lastOpenedAt: number, now = Date.now()): string {
  if (!lastOpenedAt) return 'previous session';
  const elapsed = Math.max(0, now - lastOpenedAt);
  const minutes = Math.floor(elapsed / 60_000);
  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}
