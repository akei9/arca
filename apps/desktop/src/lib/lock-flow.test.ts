import { describe, expect, it } from 'vitest';
import type { RecentVault } from './ipc';
import {
  estimatePassphraseStrength,
  emptyLockSecretState,
  filterRecentVaults,
  firstDialogPath,
  nextSelectionIndex,
  restoreFocusAfterDialogCancel,
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

  it('restores focus when a native dialog is cancelled', () => {
    let focused = false;
    const trigger = { focus: () => (focused = true) };

    expect(restoreFocusAfterDialogCancel(null, trigger)).toBe(true);
    expect(focused).toBe(true);
    expect(firstDialogPath(['/tmp/selected.arca'])).toBe('/tmp/selected.arca');
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
