import { describe, expect, it } from 'vitest';
import type { PathInspection, RecentVault } from './ipc';
import {
  createDestinationError,
  estimatePassphraseStrength,
  emptyLockSecretState,
  filterRecentVaults,
  firstDialogPath,
  nextSelectionIndex,
  validateNewPassphrase,
  vaultDisplayName,
} from './lock-flow';

describe('lock flow helpers', () => {
  it('validates new passphrases without persisting or transforming them', () => {
    expect(validateNewPassphrase('short', 'short')).toBe('use at least 8 characters');
    expect(validateNewPassphrase('long enough', 'different')).toBe("passphrases don't match");
    expect(validateNewPassphrase('long enough', 'long enough')).toBeNull();
  });

  it('presents increasing strength without claiming recovery', () => {
    expect(estimatePassphraseStrength('').level).toBe(0);
    expect(estimatePassphraseStrength('aaaaaaaa').level).toBe(1);
    expect(estimatePassphraseStrength('Correct-Horse-47!').level).toBe(4);
  });

  it('filters recents by a deliberate path query', () => {
    const recents: RecentVault[] = [
      { path: '/Users/me/vaults/personal.arca', displayName: 'personal.arca', available: true, lastOpenedAt: 2 },
      { path: '/Volumes/usb/work.arca', displayName: 'work.arca', available: true, lastOpenedAt: 1 },
    ];

    expect(filterRecentVaults(recents, 'vaults/pe')).toEqual([recents[0]]);
    expect(filterRecentVaults(recents, '~/vaults/pe')).toEqual([recents[0]]);
  });

  it('keeps passive display metadata to the basename', () => {
    const fullPath = '/Users/me/private/personal.arca';
    const name = vaultDisplayName(fullPath);

    expect(name).toBe('personal.arca');
    expect(name).not.toContain('/Users/me');
  });

  it('normalizes native dialog selections', () => {
    expect(firstDialogPath(null)).toBeNull();
    expect(firstDialogPath([])).toBeNull();
    expect(firstDialogPath(['/tmp/selected.arca'])).toBe('/tmp/selected.arca');
  });

  it('rejects native save destinations that could overwrite an existing file', () => {
    const inspection = (overrides: Partial<PathInspection>): PathInspection => ({
      path: '/tmp/new.arca',
      displayName: 'new.arca',
      kind: 'missing',
      supportedVault: false,
      canCreate: false,
      ...overrides,
    });

    expect(createDestinationError(inspection({ canCreate: true }))).toBeNull();
    expect(createDestinationError(inspection({ kind: 'file', supportedVault: true })))
      .toBe('A vault already exists there · open it instead');
    expect(createDestinationError(inspection({ kind: 'file' })))
      .toBe('A file already exists there · choose another path');
    expect(createDestinationError(inspection({})))
      .toBe('Choose a new .arca file in an existing folder');
  });

  it('wraps keyboard navigation and keeps empty menus stable', () => {
    expect(nextSelectionIndex(0, 3, -1)).toBe(2);
    expect(nextSelectionIndex(2, 3, 1)).toBe(0);
    expect(nextSelectionIndex(0, 0, 1)).toBe(0);
  });

  it('provides a fully cleared secret state for every context transition', () => {
    expect(emptyLockSecretState()).toEqual({
      password: '',
      confirmation: '',
      passwordRevealed: false,
      confirmationRevealed: false,
    });
  });
});
