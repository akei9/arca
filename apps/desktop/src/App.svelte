<script lang="ts">
  import UnlockScreen from './lib/components/UnlockScreen.svelte';
  import VaultShell from './lib/components/VaultShell.svelte';
  import { vaultState } from './lib/stores/vault.svelte';
  import { uiState } from './lib/stores/ui.svelte';

  function setTheme(theme: 'terminal' | 'amber') {
    uiState.theme = theme;
  }
</script>

<main class="app-shell" data-theme={uiState.theme}>
  <div class="theme-switch" aria-label="Theme selector">
    <button
      type="button"
      class:active={uiState.theme === 'terminal'}
      aria-pressed={uiState.theme === 'terminal'}
      onclick={() => setTheme('terminal')}
    >
      TERMINAL
    </button>
    <button
      type="button"
      class:active={uiState.theme === 'amber'}
      aria-pressed={uiState.theme === 'amber'}
      onclick={() => setTheme('amber')}
    >
      AMBER
    </button>
  </div>

  {#if vaultState.locked}
    <UnlockScreen />
  {:else}
    <VaultShell />
  {/if}
</main>
