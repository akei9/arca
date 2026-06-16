import { lockVault } from './ipc';
import { uiState } from './stores/ui.svelte';
import { vaultState } from './stores/vault.svelte';

export async function lockCurrentVault(): Promise<void> {
  if (vaultState.locked) {
    return;
  }

  await lockVault();
  applyLockedVaultState();
}

export function applyLockedVaultState() {
  vaultState.locked = true;
  vaultState.entries = [];
  vaultState.selectedEntry = null;
  vaultState.searchQuery = '';
  uiState.unlockSurface = vaultState.vaultPath ? 'sealed' : 'two-pane';
  uiState.sealedPromptOpen = false;
  uiState.view = 'unlock';
}
