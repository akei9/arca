<script lang="ts">
  import { onMount } from 'svelte';
  import { cancelClipboardClear } from './lib/clipboard';
  import UnlockScreen from './lib/components/UnlockScreen.svelte';
  import VaultShell from './lib/components/VaultShell.svelte';
  import { StatusBar, Tabs, WindowChrome, type TabItem } from './lib/components/chrome';
  import { lockCurrentVault } from './lib/session';
  import { loadRuntimeSettings, runtimeSettings } from './lib/stores/settings.svelte';
  import { vaultState } from './lib/stores/vault.svelte';
  import { loadThemePreference, uiState, type ViewName } from './lib/stores/ui.svelte';

  const BASE_FONT_SIZE = 13;
  const ACTIVITY_EVENTS = ['pointerdown', 'keydown', 'wheel', 'touchstart'] as const;

  let lastActivityAt = $state(Date.now());
  let autoLockTimer: ReturnType<typeof setTimeout> | null = null;

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

  const unlockSurface = $derived(
    uiState.unlockSurface === 'sealed' && vaultState.vaultPath ? 'sealed' : 'two-pane',
  );
  const unlockedStatusPill = $derived(activeTab === 'vault' ? 'VIEWING' : 'AUTH');
  const lockedStatusPill = $derived(unlockSurface === 'sealed' && uiState.sealedPromptOpen ? 'AUTH' : 'SEALED');
  const statusKind = $derived(
    vaultState.locked
      ? unlockSurface === 'sealed' && uiState.sealedPromptOpen
        ? 'slate'
        : 'vault'
      : activeTab === 'vault'
        ? 'slate'
        : 'ink',
  );
  const statusText = $derived(
    vaultState.locked
      ? unlockSurface === 'sealed' && !uiState.sealedPromptOpen
        ? 'tap, click, or press ↵ to unlock · argon2id'
        : 'awaiting_credentials · argon2id'
      : `vault.local · ${vaultState.entries.length} entries`,
  );
  const fontSize = $derived(runtimeSettings.current.fontSize);
  const appStyle = $derived(
    `--arca-font-size: ${fontSize}px; --arca-font-scale: ${fontSize / BASE_FONT_SIZE};`,
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

  function recordActivity() {
    if (!vaultState.locked) {
      lastActivityAt = Date.now();
    }
  }

  async function autoLockVault() {
    if (vaultState.locked) {
      return;
    }

    try {
      await lockCurrentVault();
    } catch {
      uiState.notification = {
        kind: 'error',
        message: 'Unable to auto-lock vault',
      };
    }
  }

  function clearAutoLockTimer() {
    if (autoLockTimer) {
      clearTimeout(autoLockTimer);
      autoLockTimer = null;
    }
  }

  onMount(() => {
    loadThemePreference();

    void loadRuntimeSettings().catch(() => {
      // Browser previews keep the default runtime settings when Tauri IPC is unavailable.
    });

    for (const eventName of ACTIVITY_EVENTS) {
      window.addEventListener(eventName, recordActivity, { passive: true });
    }

    return () => {
      clearAutoLockTimer();

      for (const eventName of ACTIVITY_EVENTS) {
        window.removeEventListener(eventName, recordActivity);
      }
    };
  });

  $effect(() => {
    const timeoutMinutes = runtimeSettings.current.autoLockTimeoutMinutes;

    clearAutoLockTimer();

    if (vaultState.locked || timeoutMinutes === null || timeoutMinutes === undefined) {
      return;
    }

    const timeoutMs = timeoutMinutes * 60 * 1000;
    const elapsedMs = Date.now() - lastActivityAt;
    const delayMs = Math.max(0, timeoutMs - elapsedMs);

    autoLockTimer = setTimeout(() => {
      void autoLockVault();
    }, delayMs);

    return clearAutoLockTimer;
  });

  $effect(() => {
    if (!vaultState.locked) {
      lastActivityAt = Date.now();
    }
  });

  $effect(() => {
    if (runtimeSettings.current.clipboardClearSeconds === null) {
      cancelClipboardClear();
    }
  });
</script>

<main class="arca arca--desktop" data-theme={uiState.theme} style={appStyle}>
  <WindowChrome path={chromePath} />

  {#if !vaultState.locked}
    <Tabs items={tabItems} active={activeTab} onselect={selectTab} />
  {/if}

  <div class="app-content app-content--full">
    {#if vaultState.locked}
      <UnlockScreen variant={unlockSurface} />
    {:else}
      <VaultShell />
    {/if}
  </div>

  <StatusBar
    pill={vaultState.locked ? lockedStatusPill : unlockedStatusPill}
    pillKind={statusKind}
    leftText={statusText}
  />
</main>
