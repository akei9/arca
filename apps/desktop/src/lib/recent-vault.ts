import { getRecentVaults, type RecentVault } from './ipc';
import { vaultState } from './stores/vault.svelte';
import { uiState } from './stores/ui.svelte';

export async function restoreRecentVaults(): Promise<RecentVault[]> {
  try {
    const recentVaults = await getRecentVaults();
    applyRecentVaults(recentVaults);
    return recentVaults;
  } finally {
    vaultState.rememberedVaultLoaded = true;
  }
}

export function applyRecentVaults(recentVaults: RecentVault[]) {
  vaultState.recentVaults = recentVaults.slice(0, 5);
  const restoredVault = recentVaults.find((vault) => vault.available) ?? recentVaults[0] ?? null;

  if (!restoredVault) {
    clearRememberedVaultState();
    return;
  }

  selectRecentVault(restoredVault);
  uiState.unlockSurface = 'sealed';
  uiState.sealedPromptOpen = !restoredVault.available;
}

export function selectRecentVault(recentVault: RecentVault) {
  vaultState.vaultPath = recentVault.path;
  vaultState.rememberedVaultDisplayName = recentVault.displayName;
  vaultState.rememberedVaultAvailable = recentVault.available;
}

export function promoteRecentVault(recentVault: RecentVault) {
  applyRecentVaults([
    recentVault,
    ...vaultState.recentVaults.filter((item) => item.path !== recentVault.path),
  ]);
}

export function removeRecentVault(path: string) {
  applyRecentVaults(vaultState.recentVaults.filter((vault) => vault.path !== path));
}

export function clearRememberedVaultState() {
  vaultState.vaultPath = '';
  vaultState.vaultName = '';
  vaultState.rememberedVaultDisplayName = '';
  vaultState.rememberedVaultAvailable = true;
  uiState.unlockSurface = 'two-pane';
  uiState.sealedPromptOpen = false;
}
