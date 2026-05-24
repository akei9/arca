<script lang="ts">
  import { lockVault } from '../ipc';
  import { vaultState } from '../stores/vault.svelte';
  import { uiState } from '../stores/ui.svelte';

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
</script>

<section class="vault-shell" aria-labelledby="vault-title">
  <header class="top-bar">
    <div>
      <h1 id="vault-title">{vaultState.vaultName || 'VAULT'}</h1>
      <p>{vaultState.vaultPath}</p>
    </div>
    <div class="top-bar-actions">
      <span>{vaultState.entries.length} entries</span>
      <button type="button" onclick={lock} disabled={busy}>LOCK</button>
    </div>
  </header>

  {#if errorMessage}
    <div class="error" role="alert">{errorMessage}</div>
  {/if}

  <div class="empty-state">
    <span>&gt;_</span>
    <p>{vaultState.entries.length === 0 ? 'NO_ENTRIES' : 'ENTRY_LIST_READY'}</p>
  </div>
</section>
