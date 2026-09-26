import { getRememberedVault, type RememberedVault } from './ipc';
import { vaultState } from './stores/vault.svelte';
import { uiState } from './stores/ui.svelte';

export async function restoreRememberedVault(): Promise<RememberedVault | null> {
  try {
    const rememberedVault = await getRememberedVault();
    applyRememberedVault(rememberedVault);
    return rememberedVault;
  } finally {
    vaultState.rememberedVaultLoaded = true;
  }
}

export function applyRememberedVault(rememberedVault: RememberedVault | null) {
  if (!rememberedVault) {
    clearRememberedVaultState();
    return;
  }

  vaultState.vaultPath = rememberedVault.path;
  vaultState.rememberedVaultDisplayName = rememberedVault.displayName;
  vaultState.rememberedVaultAvailable = rememberedVault.available;
  uiState.unlockSurface = 'sealed';
  uiState.sealedPromptOpen = !rememberedVault.available;
}

export function clearRememberedVaultState() {
  vaultState.vaultPath = '';
  vaultState.vaultName = '';
  vaultState.rememberedVaultDisplayName = '';
  vaultState.rememberedVaultAvailable = true;
  uiState.unlockSurface = 'two-pane';
  uiState.sealedPromptOpen = false;
}
