import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getRememberedVault, type RememberedVault } from './ipc';

vi.mock('./ipc', () => ({
  getRememberedVault: vi.fn(),
}));

import {
  applyRememberedVault,
  clearRememberedVaultState,
  restoreRememberedVault,
} from './recent-vault';
import { uiState } from './stores/ui.svelte';
import { vaultState } from './stores/vault.svelte';

const rememberedVault: RememberedVault = {
  path: '/Users/example/private/primary.arca',
  displayName: 'primary.arca',
  available: true,
};

describe('remembered vault state', () => {
  beforeEach(() => {
    clearRememberedVaultState();
    vaultState.rememberedVaultLoaded = false;
    vi.mocked(getRememberedVault).mockReset();
  });

  it('loads the persisted locator before completing startup restoration', async () => {
    vi.mocked(getRememberedVault).mockResolvedValue(rememberedVault);

    await restoreRememberedVault();

    expect(getRememberedVault).toHaveBeenCalledOnce();
    expect(vaultState.vaultPath).toBe(rememberedVault.path);
    expect(vaultState.rememberedVaultLoaded).toBe(true);
    expect(uiState.unlockSurface).toBe('sealed');
  });

  it('completes startup restoration when the preference cannot be loaded', async () => {
    vi.mocked(getRememberedVault).mockRejectedValue(new Error('preferences unavailable'));

    await expect(restoreRememberedVault()).rejects.toThrow('preferences unavailable');

    expect(vaultState.rememberedVaultLoaded).toBe(true);
    expect(vaultState.vaultPath).toBe('');
  });

  it('restores the sealed unlock flow without exposing the path as display metadata', () => {
    applyRememberedVault(rememberedVault);

    expect(vaultState.vaultPath).toBe(rememberedVault.path);
    expect(vaultState.rememberedVaultDisplayName).toBe('primary.arca');
    expect(vaultState.rememberedVaultDisplayName).not.toContain('/Users/example');
    expect(uiState.unlockSurface).toBe('sealed');
    expect(uiState.sealedPromptOpen).toBe(false);
  });

  it('opens recovery when the remembered vault is unavailable without forgetting it', () => {
    applyRememberedVault({ ...rememberedVault, available: false });

    expect(vaultState.vaultPath).toBe(rememberedVault.path);
    expect(vaultState.rememberedVaultAvailable).toBe(false);
    expect(uiState.unlockSurface).toBe('sealed');
    expect(uiState.sealedPromptOpen).toBe(true);
  });

  it('clears the locator and returns to the first-run flow', () => {
    applyRememberedVault(rememberedVault);
    clearRememberedVaultState();

    expect(vaultState.vaultPath).toBe('');
    expect(vaultState.rememberedVaultDisplayName).toBe('');
    expect(uiState.unlockSurface).toBe('two-pane');
    expect(uiState.sealedPromptOpen).toBe(false);
  });
});
