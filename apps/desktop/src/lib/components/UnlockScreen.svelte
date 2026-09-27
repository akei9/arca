<script lang="ts">
  import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { onMount, tick } from 'svelte';
  import {
    createVault,
    forgetRecentVault,
    inspectVaultPath,
    listEntries,
    rememberCurrentVault,
    suggestPaths,
    unlockVault,
    type EntryDto,
    type PathInspection,
    type PathSuggestion,
    type RecentVault,
  } from '../ipc';
  import { primaryModifierPressed } from '../keyboard';
  import {
    createDestinationError,
    estimatePassphraseStrength,
    emptyLockSecretState,
    filterRecentVaults,
    firstDialogPath,
    nextSelectionIndex,
    parentPath,
    relativeRecency,
    validateNewPassphrase,
    vaultDisplayName,
    vaultNameWithoutExtension,
  } from '../lock-flow';
  import { promoteRecentVault, removeRecentVault, selectRecentVault } from '../recent-vault';
  import { vaultState } from '../stores/vault.svelte';
  import { uiState } from '../stores/ui.svelte';
  import { Lockup, Lettermark } from './brand';
  import { Icon } from './icons';
  import { Button, IconButton, Kbd } from './primitives';

  interface Props {
    variant?: 'two-pane' | 'sealed';
  }

  type MenuItem =
    | { id: string; kind: 'create'; path: string; label: string }
    | { id: string; kind: 'recent'; vault: RecentVault; label: string }
    | { id: string; kind: 'suggestion'; suggestion: PathSuggestion; label: string };

  let { variant: _variant = 'two-pane' }: Props = $props();
  const initialCurrent = vaultState.recentVaults.find((vault) => vault.path === vaultState.vaultPath) ?? null;
  let current = $state<RecentVault | null>(initialCurrent);
  let previousCurrent = $state<RecentVault | null>(initialCurrent);
  let query = $state<string | null>(uiState.pendingLockAction === 'path' ? '~/' : null);
  let creating = $state(false);
  let password = $state('');
  let passwordConfirmation = $state('');
  let passwordRevealed = $state(false);
  let confirmationRevealed = $state(false);
  let errorMessage = $state('');
  let capsLockOn = $state(false);
  let busy = $state(false);
  let dialogOpen = $state(false);
  let pathSuggestions = $state<PathSuggestion[]>([]);
  let pathInspection = $state<PathInspection | null>(null);
  let selectedIndex = $state(0);
  let confirmForgetPath = $state<string | null>(null);
  let pathInput = $state<HTMLInputElement | null>(null);
  let passwordInput = $state<HTMLInputElement | null>(null);
  let switchButton = $state<HTMLButtonElement | null>(null);
  let firstPathButton = $state<HTMLButtonElement | null>(null);
  let queryTimer: ReturnType<typeof setTimeout> | null = null;
  let requestCounter = 0;

  const recentMatches = $derived(filterRecentVaults(vaultState.recentVaults, query ?? ''));
  const strength = $derived(estimatePassphraseStrength(password));
  const mode = $derived(
    dialogOpen
      ? 'dialog'
      : query !== null
      ? 'path'
      : creating
        ? 'create'
        : current
          ? current.available
            ? 'card'
            : 'unavailable'
          : 'first',
  );
  const menuItems = $derived.by<MenuItem[]>(() => {
    const items: MenuItem[] = [];
    if (pathInspection?.canCreate) {
      items.push({
        id: `create:${pathInspection.path}`,
        kind: 'create',
        path: pathInspection.path,
        label: pathInspection.displayName,
      });
    }
    for (const vault of recentMatches) {
      items.push({ id: `recent:${vault.path}`, kind: 'recent', vault, label: vault.displayName });
    }
    for (const suggestion of pathSuggestions) {
      if (suggestion.kind === 'directory' || suggestion.vaultCandidate) {
        items.push({
          id: `suggestion:${suggestion.path}`,
          kind: 'suggestion',
          suggestion,
          label: suggestion.name,
        });
      }
    }
    return items;
  });
  const selectedItem = $derived(menuItems[selectedIndex]);
  const canUnlock = $derived(Boolean(current?.available && password && !busy));
  const canCreate = $derived(Boolean(current && password && passwordConfirmation && !busy));

  $effect(() => {
    const pathQuery = query;
    if (queryTimer) clearTimeout(queryTimer);
    if (pathQuery === null) {
      pathSuggestions = [];
      pathInspection = null;
      return;
    }

    const request = ++requestCounter;
    queryTimer = setTimeout(() => {
      void Promise.all([suggestPaths(pathQuery), inspectVaultPath(pathQuery)])
        .then(([suggestions, inspection]) => {
          if (request !== requestCounter) return;
          pathSuggestions = suggestions;
          pathInspection = inspection;
          selectedIndex = 0;
        })
        .catch(() => {
          if (request !== requestCounter) return;
          pathSuggestions = [];
          pathInspection = null;
          selectedIndex = 0;
        });
    }, 80);

    return () => {
      if (queryTimer) clearTimeout(queryTimer);
    };
  });

  $effect(() => {
    uiState.unlockSurface = current ? 'sealed' : 'two-pane';
    uiState.sealedPromptOpen = mode !== 'first' && mode !== 'card';
    uiState.lockFlowMode = confirmForgetPath ? 'forget' : mode;
  });

  onMount(() => {
    applyDebugPreview();
    const pendingAction = uiState.pendingLockAction;
    uiState.pendingLockAction = null;
    if (pendingAction === 'open') void openNativeVault();
    if (pendingAction === 'create') void saveNativeVault();
    if (pendingAction === 'path') void tick().then(() => pathInput?.focus());

    function handleGlobalKeydown(event: KeyboardEvent) {
      if (!vaultState.locked || busy || event.repeat || event.altKey) return;
      const key = event.key.toLowerCase();
      const mod = primaryModifierPressed(event);

      if (mod && !event.shiftKey && key === 'o') {
        event.preventDefault();
        void openNativeVault();
        return;
      }
      if (mod && !event.shiftKey && key === 'n') {
        event.preventDefault();
        void saveNativeVault();
        return;
      }
      if (mod && !event.shiftKey && key === 'l') {
        event.preventDefault();
        openPathPrompt();
        return;
      }

      if (query === null) {
        if (creating && key === 'escape') {
          event.preventDefault();
          cancelCreation();
        }
        return;
      }

      if (confirmForgetPath) {
        if (key === 'escape') {
          event.preventDefault();
          confirmForgetPath = null;
        } else if (key === 'enter' && event.target === pathInput) {
          event.preventDefault();
          void confirmForget(confirmForgetPath);
        }
        return;
      }

      if (key === 'escape') {
        event.preventDefault();
        closePathPrompt();
        return;
      }
      if (event.target !== pathInput) return;
      if (mod && key === 'backspace' && selectedItem?.kind === 'recent') {
        event.preventDefault();
        confirmForgetPath = selectedItem.vault.path;
        return;
      }
      if (key === 'arrowdown') {
        event.preventDefault();
        selectedIndex = nextSelectionIndex(selectedIndex, menuItems.length, 1);
        return;
      }
      if (key === 'arrowup') {
        event.preventDefault();
        selectedIndex = nextSelectionIndex(selectedIndex, menuItems.length, -1);
        return;
      }
      if (key === 'tab' && selectedItem) {
        event.preventDefault();
        completeItem(selectedItem);
        return;
      }
      if (key === 'enter' && selectedItem) {
        event.preventDefault();
        activateItem(selectedItem);
      }
    }

    window.addEventListener('keydown', handleGlobalKeydown);
    return () => {
      window.removeEventListener('keydown', handleGlobalKeydown);
      clearSecrets();
      if (queryTimer) clearTimeout(queryTimer);
    };
  });

  function applyDebugPreview() {
    if (!import.meta.env.DEV || typeof window === 'undefined' || '__TAURI_INTERNALS__' in window) return;
    const preview = new URL(window.location.href).searchParams.get('lock-preview');
    if (!preview || preview === 'first') return;

    const synthetic: RecentVault = {
      path: '/Users/preview/vaults/personal.arca',
      displayName: 'personal.arca',
      available: preview !== 'unavailable',
      lastOpenedAt: Date.now() - 2 * 24 * 60 * 60 * 1000,
    };
    vaultState.recentVaults = [synthetic];
    current = synthetic;
    selectRecentVault(synthetic);

    if (preview === 'path') query = '~/';
    if (preview === 'forget') {
      query = '/Users/preview/vaults/';
      confirmForgetPath = synthetic.path;
    }
    if (preview === 'create' || preview === 'mismatch') {
      beginCreation('/Users/preview/vaults/new_test.arca', 'new_test.arca');
      if (preview === 'mismatch') {
        password = 'correct horse battery';
        passwordConfirmation = 'different passphrase';
        errorMessage = "passphrases don't match";
      }
    }
    if (preview === 'wrong') errorMessage = 'incorrect passphrase';
  }

  function clearSecrets() {
    const cleared = emptyLockSecretState();
    password = cleared.password;
    passwordConfirmation = cleared.confirmation;
    passwordRevealed = cleared.passwordRevealed;
    confirmationRevealed = cleared.confirmationRevealed;
    capsLockOn = false;
  }

  function clearErrorOnInput() {
    errorMessage = '';
  }

  function updateCapsLock(event: KeyboardEvent) {
    capsLockOn = event.getModifierState('CapsLock');
  }

  function openPathPrompt(initialPath = '~/') {
    if (busy) return;
    clearSecrets();
    errorMessage = '';
    if (creating) {
      current = previousCurrent;
      if (current) selectRecentVault(current);
      else {
        vaultState.vaultPath = '';
        vaultState.rememberedVaultDisplayName = '';
        vaultState.rememberedVaultAvailable = true;
      }
    }
    creating = false;
    confirmForgetPath = null;
    query = initialPath;
    void tick().then(() => pathInput?.focus());
  }

  function forgetCurrentVault() {
    if (!current) return;
    openPathPrompt(current.path);
    confirmForgetPath = current.path;
  }

  function closePathPrompt() {
    clearSecrets();
    errorMessage = '';
    confirmForgetPath = null;
    query = null;
    void tick().then(() => (current ? switchButton : firstPathButton)?.focus());
  }

  function completeItem(item: MenuItem) {
    if (item.kind === 'create') query = item.path;
    if (item.kind === 'recent') query = item.vault.path;
    if (item.kind === 'suggestion') query = item.suggestion.path;
    void tick().then(() => pathInput?.focus());
  }

  function activateItem(item: MenuItem) {
    if (item.kind === 'create') {
      beginCreation(item.path, item.label);
      return;
    }
    if (item.kind === 'recent') {
      chooseVault(item.vault);
      return;
    }
    if (item.suggestion.kind === 'directory') {
      query = item.suggestion.path;
      void tick().then(() => pathInput?.focus());
      return;
    }
    chooseVault({
      path: item.suggestion.path,
      displayName: item.suggestion.name,
      available: true,
      lastOpenedAt: 0,
    });
  }

  function chooseVault(vault: RecentVault) {
    clearSecrets();
    errorMessage = '';
    creating = false;
    query = null;
    current = vault;
    previousCurrent = vault;
    selectRecentVault(vault);
    void tick().then(() => (vault.available ? passwordInput : switchButton)?.focus());
  }

  function beginCreation(path: string, displayName = vaultDisplayName(path)) {
    previousCurrent = creating ? previousCurrent : current;
    clearSecrets();
    errorMessage = '';
    query = null;
    creating = true;
    current = { path, displayName, available: false, lastOpenedAt: 0 };
    vaultState.vaultPath = path;
    vaultState.rememberedVaultDisplayName = displayName;
    vaultState.rememberedVaultAvailable = false;
    void tick().then(() => passwordInput?.focus());
  }

  function cancelCreation() {
    if (!creating || busy) return;
    clearSecrets();
    errorMessage = '';
    creating = false;
    current = previousCurrent;
    if (current) selectRecentVault(current);
    else {
      vaultState.vaultPath = '';
      vaultState.rememberedVaultDisplayName = '';
      vaultState.rememberedVaultAvailable = true;
    }
    void tick().then(() => (current ? switchButton : firstPathButton)?.focus());
  }

  async function restoreDialogTriggerFocus(trigger: HTMLElement | null) {
    dialogOpen = false;
    await tick();
    trigger?.focus();
  }

  async function openNativeVault() {
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    clearSecrets();
    errorMessage = '';
    dialogOpen = true;
    try {
      const picked = await openDialog({
        multiple: false,
        directory: false,
        filters: [{ name: 'Vault files', extensions: ['arca', 'kdbx'] }],
      });
      const path = firstDialogPath(picked);
      if (!path) {
        await restoreDialogTriggerFocus(trigger);
        return;
      }
      const inspection = await inspectVaultPath(path);
      chooseVault({
        path: inspection.path,
        displayName: inspection.displayName,
        available: inspection.supportedVault,
        lastOpenedAt: 0,
      });
    } catch {
      errorMessage = 'Unable to open the system file panel';
      await restoreDialogTriggerFocus(trigger);
    } finally {
      dialogOpen = false;
    }
  }

  async function saveNativeVault() {
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    clearSecrets();
    errorMessage = '';
    dialogOpen = true;
    try {
      const preferred = await inspectVaultPath('~/vaults/');
      const defaultPath = preferred.kind === 'directory'
        ? `${preferred.path.replace(/[\\/]$/, '')}/untitled.arca`
        : 'untitled.arca';
      const picked = await saveDialog({
        defaultPath,
        filters: [{ name: 'Arca vault', extensions: ['arca'] }],
      });
      const pickedPath = firstDialogPath(picked);
      if (!pickedPath) {
        await restoreDialogTriggerFocus(trigger);
        return;
      }
      const path = pickedPath.toLowerCase().endsWith('.arca') ? pickedPath : `${pickedPath}.arca`;
      const inspection = await inspectVaultPath(path);
      const destinationError = createDestinationError(inspection);
      if (destinationError) {
        errorMessage = destinationError;
        await restoreDialogTriggerFocus(trigger);
        return;
      }
      beginCreation(inspection.path, inspection.displayName);
    } catch {
      errorMessage = 'Unable to open the system save panel';
      await restoreDialogTriggerFocus(trigger);
    } finally {
      dialogOpen = false;
    }
  }

  async function submitUnlock() {
    if (!current?.available || !canUnlock) return;
    busy = true;
    errorMessage = '';
    try {
      const info = await unlockVault(current.path, password);
      const entries = await listEntries();
      applyUnlockedState(info.name, info.path, entries, info.modifiedAt);
      clearSecrets();
      await rememberOpenedVault('Vault unlocked, but it could not be remembered');
    } catch (error) {
      const code = errorCode(error);
      clearSecrets();
      if (code === 'invalid_password') {
        errorMessage = 'incorrect passphrase';
        await tick();
        passwordInput?.focus();
      } else if (code === 'file_not_found') {
        current = { ...current, available: false };
        errorMessage = 'Vault unavailable. Locate it or choose another vault.';
      } else {
        errorMessage = safeErrorMessage(code, 'Unable to unlock vault');
      }
    } finally {
      busy = false;
    }
  }

  async function submitCreate() {
    if (!current || !canCreate) return;
    const validation = validateNewPassphrase(password, passwordConfirmation);
    if (validation) {
      errorMessage = validation;
      await tick();
      (validation.includes('match') ? document.querySelector<HTMLInputElement>('#new-password-confirmation') : passwordInput)?.focus();
      return;
    }

    busy = true;
    errorMessage = '';
    try {
      const name = vaultNameWithoutExtension(current.displayName);
      await createVault(current.path, password, name);
      applyUnlockedState(name, current.path, [], new Date().toISOString());
      clearSecrets();
      await rememberOpenedVault('Vault created, but it could not be remembered');
    } catch (error) {
      clearSecrets();
      errorMessage = safeErrorMessage(errorCode(error), 'Unable to create vault');
    } finally {
      busy = false;
    }
  }

  function applyUnlockedState(name: string, path: string, entries: EntryDto[], modifiedAt: string) {
    vaultState.locked = false;
    vaultState.entries = entries;
    vaultState.selectedEntry = null;
    vaultState.searchQuery = '';
    vaultState.vaultName = name;
    vaultState.vaultPath = path;
    vaultState.lastSaved = new Date(modifiedAt);
    uiState.view = 'list';
  }

  async function rememberOpenedVault(failureMessage: string) {
    try {
      promoteRecentVault(await rememberCurrentVault());
    } catch {
      uiState.notification = { kind: 'warning', message: failureMessage };
    }
  }

  async function confirmForget(path: string) {
    busy = true;
    errorMessage = '';
    try {
      await forgetRecentVault(path);
      removeRecentVault(path);
      const next = vaultState.recentVaults[0] ?? null;
      current = next;
      if (next) selectRecentVault(next);
      confirmForgetPath = null;
      if (!next) query = null;
      await tick();
      pathInput?.focus();
    } catch {
      errorMessage = 'Unable to forget vault';
    } finally {
      busy = false;
    }
  }

  function requestForget(path: string) {
    confirmForgetPath = path;
  }

  function errorCode(error: unknown): string {
    return typeof error === 'object' && error !== null && 'code' in error ? String(error.code) : '';
  }

  function safeErrorMessage(code: string, fallback: string): string {
    const messages: Record<string, string> = {
      corrupted_vault: 'Vault file is corrupted',
      decryption_error: 'Unable to decrypt vault data',
      io_error: 'Unable to read or write vault data',
      serialization_error: 'Unable to process vault data',
    };
    return messages[code] ?? fallback;
  }
</script>

<section class="unlock-screen" aria-labelledby="unlock-title">
  <div class="unlock lk">
    <div class="unlock__left">
      <div class="unlock__caption mono">
        <span>v01 · 2026</span>
        <span>identity · <b>arca</b></span>
      </div>
      <div>
        <Lockup size={112} />
        <div class="unlock__brand-gap"></div>
        <h1 id="unlock-title" class="unlock__lede">
          the vault for what you <em>can't lose.</em><br />
          kept where only you can reach it.
        </h1>
      </div>
      <div class="unlock__caption mono">
        <span>local-first · <b>zero-cloud</b></span>
        <span class="unlock__caption-trail">zero_knowledge · <b>enabled</b></span>
      </div>
    </div>

    <div class:lk-right--path={mode === 'path'} class="lk-right">
      <div class="lk-spacer"></div>
      <div class="lk-flow">
        {#if mode === 'first'}
          <div class="lk-label"><span>get_started</span><span>first run · no vaults on this mac</span></div>
          <p class="lk-first__lede">No vault here yet. Make a new one, or bring one you already have.</p>
          <button type="button" class="lk-choice lk-choice--pri" onclick={() => void saveNativeVault()}>
            <span class="lk-choice__tile"><Icon name="plus" size={18} sw={1.8} /></span>
            <span><b>create new vault</b><small>a fresh .arca file, saved where you choose</small></span>
            <span aria-hidden="true">›</span>
          </button>
          <button type="button" class="lk-choice" onclick={() => void openNativeVault()}>
            <span class="lk-choice__tile"><Icon name="vault" size={18} /></span>
            <span><b>open existing vault</b><small>.arca from the filesystem or a usb drive · .kdbx import</small></span>
            <span aria-hidden="true">›</span>
          </button>
          <button bind:this={firstPathButton} type="button" class="lk-first__path" onclick={() => openPathPrompt()}>
            <b>&gt;</b><span>or type a path</span><i aria-hidden="true"></i>
          </button>
        {:else if mode === 'path'}
          <div class="lk-label"><span>vault_path</span><span>open · or type a new name to create</span></div>
          <div class="lk-path-wrap">
            <div class="lk-pathfield">
              <b>&gt;</b>
              <input
                bind:this={pathInput}
                bind:value={query}
                aria-label="vault path"
                aria-controls="vault-path-menu"
                aria-expanded="true"
                aria-activedescendant={selectedItem?.id}
                autocomplete="off"
                spellcheck="false"
              />
              <Kbd value="esc" />
            </div>
            <div id="vault-path-menu" class="lk-menu" role="listbox" aria-label="Vault path results">
              <div class="lk-plist">
                {#if pathInspection?.canCreate}
                  <button
                    id={`create:${pathInspection.path}`}
                    type="button"
                    role="option"
                    aria-selected={selectedItem?.id === `create:${pathInspection.path}`}
                    class:selected={selectedItem?.id === `create:${pathInspection.path}`}
                    class="lk-prow"
                    onmouseenter={() => (selectedIndex = menuItems.findIndex((item) => item.id === `create:${pathInspection?.path}`))}
                    onmousedown={(event) => event.preventDefault()}
                    onclick={() => beginCreation(pathInspection!.path, pathInspection!.displayName)}
                  >
                    <b>+</b><span>create <strong>{pathInspection.displayName}</strong> here</span><small>new vault</small>
                  </button>
                {/if}

                {#if recentMatches.length}
                  <div class="lk-menu__h"><span>recent</span><span>{recentMatches.length}/5</span></div>
                  {#each recentMatches as vault}
                    {@const itemId = `recent:${vault.path}`}
                    {@const recentIndex = menuItems.findIndex((item) => item.id === itemId)}
                    {#if confirmForgetPath === vault.path}
                      <div class="lk-pconfirm" role="alert">
                        <span>forget <b>{vault.displayName}</b>?<small>removed from recents only · vault file stays on disk</small></span>
                        <Button size="xs" variant="bare" onclick={() => (confirmForgetPath = null)}>keep</Button>
                        <Button size="xs" variant="danger" onclick={() => void confirmForget(vault.path)}>forget</Button>
                      </div>
                    {:else}
                      <div
                        id={itemId}
                        role="option"
                        tabindex="-1"
                        aria-selected={selectedItem?.id === itemId}
                        class:selected={selectedItem?.id === itemId}
                        class="lk-prow lk-prow--recent"
                        onmouseenter={() => (selectedIndex = recentIndex)}
                        onmousedown={(event) => { event.preventDefault(); chooseVault(vault); }}
                        onkeydown={(event) => { if (event.key === 'Enter') chooseVault(vault); }}
                      >
                        <b class="lk-vault-dot">●</b>
                        <span>{vault.displayName}<small>{parentPath(vault.path)}</small></span>
                        <span class="lk-prow__tail"><em>{vault.available ? relativeRecency(vault.lastOpenedAt) : 'unavailable'}</em><button type="button" onmousedown={(event) => event.stopPropagation()} onclick={(event) => { event.stopPropagation(); requestForget(vault.path); }}>forget</button></span>
                      </div>
                    {/if}
                  {/each}
                {/if}

                <div class="lk-menu__h"><span>{query || '~/'} </span></div>
                {#each pathSuggestions as suggestion}
                  {@const itemId = `suggestion:${suggestion.path}`}
                  {@const suggestionIndex = menuItems.findIndex((item) => item.id === itemId)}
                  <button
                    id={itemId}
                    type="button"
                    role="option"
                    aria-selected={selectedItem?.id === itemId}
                    aria-disabled={suggestion.kind === 'file' && !suggestion.vaultCandidate}
                    disabled={suggestion.kind === 'file' && !suggestion.vaultCandidate}
                    class:selected={selectedItem?.id === itemId}
                    class="lk-prow"
                    onmouseenter={() => { if (suggestionIndex >= 0) selectedIndex = suggestionIndex; }}
                    onmousedown={(event) => event.preventDefault()}
                    onclick={() => {
                      if (suggestion.kind === 'directory') query = suggestion.path;
                      else if (suggestion.vaultCandidate) activateItem({ id: itemId, kind: 'suggestion', suggestion, label: suggestion.name });
                    }}
                  >
                    <b>&gt;</b><span>{suggestion.name}</span>
                    <small class:vault={suggestion.vaultCandidate}>{suggestion.kind === 'directory' ? 'directory' : suggestion.vaultCandidate ? suggestion.name.toLowerCase().endsWith('.kdbx') ? 'keepass · import' : 'arca vault' : 'not a vault'}</small>
                  </button>
                {:else}
                  <div class="lk-menu__empty">no match · end with .arca to create</div>
                {/each}
              </div>
              <div class="lk-pgui">
                <button type="button" class="lk-mi" onmousedown={(event) => event.preventDefault()} onclick={() => void openNativeVault()}><Icon name="vault" size={13} /> open vault file…</button>
                <button type="button" class="lk-mi" onmousedown={(event) => event.preventDefault()} onclick={() => void saveNativeVault()}><Icon name="plus" size={13} /> create new vault</button>
              </div>
            </div>
          </div>
        {:else if current}
          <form onsubmit={(event) => { event.preventDefault(); void (creating ? submitCreate() : submitUnlock()); }}>
            <div class="lk-label">
              <span>vault</span>
              <span class={mode === 'unavailable' ? 'lk-state lk-state--warn' : creating ? 'lk-state lk-state--accent' : 'lk-state'}>
                ● {mode === 'unavailable' ? 'unavailable' : creating ? 'new vault · set a passphrase' : 'restored from last session'}
              </span>
            </div>
            <div class="lk-card">
              <span class:creating class="lk-card__tile"><Lettermark size={20} /></span>
              <span class="lk-card__body"><b>{current.displayName}</b><small>{creating ? 'will be created' : relativeRecency(current.lastOpenedAt)}</small></span>
              <button bind:this={switchButton} type="button" class="lk-switch" onclick={() => openPathPrompt()}>
                <span>switch</span>
                <Icon name="chevron-down" size={12} sw={1.8} />
              </button>
            </div>

            {#if mode === 'unavailable'}
              <div class="lk-unavailable" role="status">
                <b>vault unavailable</b>
                <p>Locate this vault, open another one, or forget only this recent locator.</p>
                <div>
                  <Button variant="primary" onclick={() => openPathPrompt(current!.path)}>locate vault</Button>
                  <Button variant="ghost" onclick={() => void openNativeVault()}>open another</Button>
                  <Button variant="danger" onclick={forgetCurrentVault}>forget</Button>
                </div>
              </div>
            {:else}
              <div class="lk-pw">
                <label>
                  <div class="lk-label"><span>{creating ? 'new_master_password' : 'master_password'}</span><span>argon2id · chacha20</span></div>
                  <div class:error={Boolean(errorMessage)} class="unlock__field">
                    <input
                      bind:this={passwordInput}
                      bind:value={password}
                      autocomplete={creating ? 'new-password' : 'current-password'}
                      class="unlock__input"
                      type={passwordRevealed ? 'text' : 'password'}
                      placeholder={creating ? 'choose a passphrase' : 'master passphrase'}
                      aria-invalid={Boolean(errorMessage)}
                      oninput={clearErrorOnInput}
                      onkeydown={updateCapsLock}
                      onkeyup={updateCapsLock}
                    />
                    <IconButton label={passwordRevealed ? 'Hide master passphrase' : 'Reveal master passphrase'} variant="ghost" onclick={() => (passwordRevealed = !passwordRevealed)} disabled={!password}>
                      <Icon name={passwordRevealed ? 'eye-off' : 'eye'} size={14} />
                    </IconButton>
                  </div>
                </label>

                {#if creating}
                  <div class="lk-meter" aria-label={password ? `Passphrase strength ${strength.label}, approximately ${strength.bits} bits` : 'Passphrase must be at least 8 characters'}>
                    <span class="lk-meter__segments" data-level={strength.level}>{#each [1, 2, 3, 4] as segment}<i class:filled={segment <= strength.level}></i>{/each}</span>
                    <span>{password ? `${strength.label} · ~${strength.bits} bits` : 'at least 8 characters · longer beats clever'}</span>
                  </div>
                  <label class="lk-confirm-field">
                    <span class="sr-only">Repeat passphrase</span>
                    <div class:error={errorMessage.includes('match')} class="unlock__field">
                      <input
                        id="new-password-confirmation"
                        bind:value={passwordConfirmation}
                        autocomplete="new-password"
                        class="unlock__input"
                        type={confirmationRevealed ? 'text' : 'password'}
                        placeholder="repeat passphrase"
                        oninput={clearErrorOnInput}
                      />
                      <IconButton label={confirmationRevealed ? 'Hide repeated passphrase' : 'Reveal repeated passphrase'} variant="ghost" onclick={() => (confirmationRevealed = !confirmationRevealed)} disabled={!passwordConfirmation}>
                        <Icon name={confirmationRevealed ? 'eye-off' : 'eye'} size={14} />
                      </IconButton>
                    </div>
                  </label>
                  <p class="lk-note">no recovery · if you lose this passphrase, the vault can't be opened.</p>
                {/if}

                {#if errorMessage}
                  <div class="lk-err" role="alert"><span>{errorMessage}</span><span>{creating ? "can't be recovered · write it down" : capsLockOn ? 'caps lock on' : 'caps lock off'}</span></div>
                {/if}

                <Button class="unlock__cta" variant="primary" type="submit" disabled={creating ? !canCreate : !canUnlock}>
                  <Icon name={creating ? 'plus' : 'key'} size={12} sw={2} />
                  {busy ? 'working' : creating ? `create ${vaultNameWithoutExtension(current.displayName)}` : `unlock ${vaultNameWithoutExtension(current.displayName)}`}
                </Button>
              </div>
            {/if}
          </form>
        {/if}

        {#if errorMessage && mode !== 'card' && mode !== 'create'}
          <div class="lk-err lk-err--standalone" role="alert">{errorMessage}</div>
        {/if}
      </div>
      <div class="lk-spacer"></div>
      <div class="lk-foot"><span class="status__dot"></span> local_store · ready</div>
    </div>
  </div>
</section>
