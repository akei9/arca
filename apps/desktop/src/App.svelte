<script lang="ts">
  import UnlockScreen from './lib/components/UnlockScreen.svelte';
  import VaultShell from './lib/components/VaultShell.svelte';
  import { StatusBar, Tabs, WindowChrome, type TabItem } from './lib/components/chrome';
  import { vaultState } from './lib/stores/vault.svelte';
  import { uiState, type ViewName } from './lib/stores/ui.svelte';

  const tabItems = $derived<TabItem[]>([
    { key: 'vault', label: 'vault', count: vaultState.entries.length },
    { key: 'generate', label: 'generate' },
    { key: 'audit', label: 'audit' },
    { key: 'shared', label: 'shared' },
    { key: 'settings', label: 'settings' },
  ]);

  const activeTab = $derived(
    uiState.view === 'generator'
      ? 'generate'
      : uiState.view === 'settings' || uiState.view === 'audit' || uiState.view === 'shared'
        ? uiState.view
        : 'vault',
  );

  const chromePath = $derived(
    vaultState.locked
      ? 'vault.local · sealed'
      : vaultState.selectedEntry
        ? `vault.local / ${vaultState.selectedEntry.title}`
        : 'vault.local',
  );

  const statusPill = $derived(vaultState.locked ? 'SEALED' : activeTab === 'vault' ? 'VIEWING' : 'AUTH');
  const statusKind = $derived(vaultState.locked ? 'vault' : activeTab === 'vault' ? 'slate' : 'ink');
  const statusText = $derived(
    vaultState.locked ? 'awaiting_credentials · argon2id' : `vault.local · ${vaultState.entries.length} entries`,
  );

  function selectTab(key: string) {
    const viewByTab: Record<string, ViewName> = {
      vault: 'list',
      generate: 'generator',
      audit: 'audit',
      shared: 'shared',
      settings: 'settings',
    };

    uiState.view = viewByTab[key] ?? 'list';
  }
</script>

<main class="arca arca--desktop" data-theme={uiState.theme}>
  <WindowChrome path={chromePath} />

  {#if !vaultState.locked}
    <Tabs items={tabItems} active={activeTab} onselect={selectTab} />
  {/if}

  <div class="app-content app-content--full">
    {#if vaultState.locked}
      <UnlockScreen />
    {:else}
      <VaultShell />
    {/if}
  </div>

  <StatusBar pill={statusPill} pillKind={statusKind} leftText={statusText} />
</main>
