<script lang="ts">
  import { createVault, listEntries, unlockVault, type EntryDto } from '../ipc';
  import { vaultState } from '../stores/vault.svelte';
  import { uiState } from '../stores/ui.svelte';

  type Mode = 'open' | 'create';

  let mode: Mode = $state('open');
  let path = $state('');
  let password = $state('');
  let vaultName = $state('SOVEREIGN_CONSOLE');
  let busy = $state(false);
  let errorMessage = $state('');

  const canSubmit = $derived(
    path.trim().length > 0 &&
      password.length > 0 &&
      (mode === 'open' || vaultName.trim().length > 0) &&
      !busy,
  );

  async function submit() {
    if (!canSubmit) {
      return;
    }

    busy = true;
    errorMessage = '';

    try {
      if (mode === 'open') {
        const info = await unlockVault(path.trim(), password);
        const entries = await listEntries();
        applyUnlockedState(info.name, info.path, entries, info.modifiedAt);
      } else {
        await createVault(path.trim(), password, vaultName.trim());
        applyUnlockedState(vaultName.trim(), path.trim(), [], new Date().toISOString());
      }

      password = '';
      uiState.view = 'list';
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }

  function applyUnlockedState(name: string, vaultPath: string, entries: EntryDto[], modifiedAt: string) {
    vaultState.locked = false;
    vaultState.entries = entries;
    vaultState.selectedEntry = null;
    vaultState.searchQuery = '';
    vaultState.vaultName = name;
    vaultState.vaultPath = vaultPath;
    vaultState.lastSaved = new Date(modifiedAt);
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to open vault';
  }
</script>

<section class="unlock-screen" aria-labelledby="unlock-title">
  <div class="unlock-header">
    <h1 id="unlock-title">ARCA</h1>
    <div class="divider"></div>
    <p>{mode === 'open' ? 'OPEN_VAULT' : 'CREATE_VAULT'}</p>
  </div>

  <div class="mode-tabs" role="tablist" aria-label="Vault mode">
    <button
      type="button"
      role="tab"
      aria-selected={mode === 'open'}
      class:active={mode === 'open'}
      onclick={() => (mode = 'open')}
    >
      OPEN
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={mode === 'create'}
      class:active={mode === 'create'}
      onclick={() => (mode = 'create')}
    >
      CREATE
    </button>
  </div>

  <form
    class="unlock-form"
    onsubmit={(event) => {
      event.preventDefault();
      submit();
    }}
  >
    {#if mode === 'create'}
      <label>
        <span>VAULT_NAME</span>
        <input bind:value={vaultName} autocomplete="off" spellcheck="false" />
      </label>
    {/if}

    <label>
      <span>VAULT_PATH</span>
      <input
        bind:value={path}
        autocomplete="off"
        placeholder="/Users/you/.arca/vaults/primary.kdbx"
        spellcheck="false"
      />
    </label>

    <label>
      <span>MASTER_PASSWORD</span>
      <input bind:value={password} autocomplete="current-password" type="password" />
    </label>

    {#if errorMessage}
      <div class="error" role="alert">{errorMessage}</div>
    {/if}

    <button class="primary-action" type="submit" disabled={!canSubmit}>
      {busy ? 'WORKING' : mode === 'open' ? 'UNLOCK' : 'CREATE'}
    </button>
  </form>
</section>
