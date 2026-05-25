<script lang="ts">
  import { lockVault } from '../ipc';
  import { vaultState } from '../stores/vault.svelte';
  import { uiState } from '../stores/ui.svelte';
  import { EntryDetail } from './detail';
  import { EntryList } from './vault';

  let busy = $state(false);
  let errorMessage = $state('');

  async function lock() {
    busy = true;
    errorMessage = '';

    try {
      await lockVault();
      vaultState.locked = true;
      vaultState.entries = [];
      vaultState.selectedEntry = null;
      vaultState.searchQuery = '';
      vaultState.vaultName = '';
      vaultState.vaultPath = '';
      vaultState.lastSaved = null;
      uiState.view = 'unlock';
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to lock vault';
  }
  const placeholderTitle = $derived(
    uiState.view === 'generator'
      ? 'generate'
      : uiState.view === 'audit' || uiState.view === 'shared' || uiState.view === 'settings'
        ? uiState.view
        : '',
  );
</script>

{#if uiState.view === 'detail'}
  <EntryDetail />
{:else if uiState.view === 'list'}
  <EntryList />
{:else}
  <section class="vault-placeholder" aria-labelledby="vault-placeholder-title">
    <p class="vault-placeholder__eyebrow">placeholder</p>
    <h1 id="vault-placeholder-title">{placeholderTitle}</h1>
    <p>screen_port_pending</p>
    <button class="btn btn--ghost" type="button" onclick={lock} disabled={busy}>lock_vault</button>
    {#if errorMessage}
      <div class="error" role="alert">{errorMessage}</div>
    {/if}
  </section>
{/if}
