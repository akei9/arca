<script lang="ts">
  import UnlockScreen from './lib/components/UnlockScreen.svelte';
  import VaultShell from './lib/components/VaultShell.svelte';
  import { vaultState } from './lib/stores/vault.svelte';
  import { uiState } from './lib/stores/ui.svelte';
</script>

<main class="arca arca--desktop" data-theme={uiState.theme}>
  {#if vaultState.locked}
    <UnlockScreen />
  {:else}
    <VaultShell />
  {/if}
</main>
