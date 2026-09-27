import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getRecentVaults, type RecentVault } from './ipc';

vi.mock('./ipc', () => ({
  getRecentVaults: vi.fn(),
}));

import {
  applyRecentVaults,
  clearRememberedVaultState,
  removeRecentVault,
  restoreRecentVaults,
} from './recent-vault';
import { uiState } from './stores/ui.svelte';
import { vaultState } from './stores/vault.svelte';

const recentVault = (name: string, available = true, lastOpenedAt = 100): RecentVault => ({
  path: `/Users/example/private/${name}`,
  displayName: name,
  available,
  lastOpenedAt,
});

describe('recent vault state', () => {
  beforeEach(() => {
    vaultState.recentVaults = [];
    clearRememberedVaultState();
    vaultState.rememberedVaultLoaded = false;
    vi.mocked(getRecentVaults).mockReset();
  });

  it('loads the persisted collection before completing startup restoration', async () => {
    vi.mocked(getRecentVaults).mockResolvedValue([recentVault('primary.arca')]);

    await restoreRecentVaults();

    expect(getRecentVaults).toHaveBeenCalledOnce();
    expect(vaultState.vaultPath).toContain('primary.arca');
    expect(vaultState.rememberedVaultLoaded).toBe(true);
    expect(uiState.unlockSurface).toBe('sealed');
  });

  it('restores the most recent available vault without exposing its parent as display metadata', () => {
    applyRecentVaults([
      recentVault('missing.arca', false, 200),
      recentVault('primary.arca', true, 100),
    ]);

    expect(vaultState.vaultPath).toContain('/Users/example/private/primary.arca');
    expect(vaultState.rememberedVaultDisplayName).toBe('primary.arca');
    expect(vaultState.rememberedVaultDisplayName).not.toContain('/Users/example');
    expect(uiState.sealedPromptOpen).toBe(false);
  });

  it('opens unavailable recovery when no recent vault can be reached', () => {
    applyRecentVaults([recentVault('missing.arca', false)]);

    expect(vaultState.rememberedVaultAvailable).toBe(false);
    expect(uiState.unlockSurface).toBe('sealed');
    expect(uiState.sealedPromptOpen).toBe(true);
  });

  it('removes one recent locator and selects the next without clearing the collection', () => {
    const first = recentVault('first.arca');
    const second = recentVault('second.arca');
    applyRecentVaults([first, second]);

    removeRecentVault(first.path);

    expect(vaultState.recentVaults).toEqual([second]);
    expect(vaultState.rememberedVaultDisplayName).toBe('second.arca');
  });

  it('completes startup restoration when preferences cannot load', async () => {
    vi.mocked(getRecentVaults).mockRejectedValue(new Error('preferences unavailable'));

    await expect(restoreRecentVaults()).rejects.toThrow('preferences unavailable');

    expect(vaultState.rememberedVaultLoaded).toBe(true);
    expect(vaultState.vaultPath).toBe('');
  });
});
