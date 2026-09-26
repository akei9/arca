<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createVault,
    forgetRememberedVault,
    listEntries,
    rememberCurrentVault,
    suggestPaths,
    unlockVault,
    type EntryDto,
    type PathSuggestion,
  } from '../ipc';
  import { isEditableTarget, primaryModifierLabel, primaryModifierPressed } from '../keyboard';
  import { clearRememberedVaultState } from '../recent-vault';
  import { vaultState } from '../stores/vault.svelte';
  import { uiState } from '../stores/ui.svelte';
  import { Lockup } from './brand';
  import { Icon } from './icons';
  import { Button, IconButton, Kbd } from './primitives';

  type Mode = 'open' | 'create';
  type Variant = 'two-pane' | 'sealed';

  interface Props {
    variant?: Variant;
  }

  let {
    variant = 'two-pane',
  }: Props = $props();

  let mode: Mode = $state('open');
  let path = $state(vaultState.vaultPath);
  let password = $state('');
  let passwordRevealed = $state(false);
  let vaultName = $state('personal');
  let busy = $state(false);
  let errorMessage = $state('');
  let openButton = $state<HTMLButtonElement | null>(null);
  let pathInput = $state<HTMLInputElement | null>(null);
  let passwordInput = $state<HTMLInputElement | null>(null);
  let focusTimer: ReturnType<typeof setTimeout> | null = null;
  let pathSuggestTimer: ReturnType<typeof setTimeout> | null = null;
  let pathFocused = $state(false);
  let pathSuggestions = $state<PathSuggestion[]>([]);
  let selectedPathIndex = $state(0);
  let suggestCounter = 0;

  const canSubmit = $derived(
    path.trim().length > 0 &&
      password.length > 0 &&
      (mode === 'open' || vaultName.trim().length > 0) &&
      !busy,
  );
  const isSealed = $derived(variant === 'sealed');
  const sealedOpen = $derived(isSealed && uiState.sealedPromptOpen);
  const showPathSuggestions = $derived(pathFocused && pathSuggestions.length > 0);
  const modLabel = $derived(primaryModifierLabel());

  onMount(() => {
    function handleKeydown(event: KeyboardEvent) {
      if (!isSealed || !vaultState.locked) {
        return;
      }

      if (event.key === 'Escape' && uiState.sealedPromptOpen) {
        event.preventDefault();
        closeSealedPrompt();
        return;
      }

      if (isEditableTarget(event.target)) {
        return;
      }

      if (primaryModifierPressed(event) && !event.shiftKey && event.key.toLowerCase() === 'o') {
        event.preventDefault();
        if (busy) {
          return;
        }
        showVaultPicker(true);
        return;
      }

      if (!uiState.sealedPromptOpen && (event.key === 'Enter' || event.key === ' ')) {
        event.preventDefault();
        openSealedPrompt();
      }
    }

    window.addEventListener('keydown', handleKeydown);

    return () => {
      window.removeEventListener('keydown', handleKeydown);

      if (focusTimer) {
        clearTimeout(focusTimer);
      }

      if (pathSuggestTimer) {
        clearTimeout(pathSuggestTimer);
      }
    };
  });

  $effect(() => {
    const query = path;

    if (pathSuggestTimer) {
      clearTimeout(pathSuggestTimer);
      pathSuggestTimer = null;
    }

    if (!pathFocused || query.trim().length === 0 || isSealed) {
      pathSuggestions = [];
      selectedPathIndex = 0;
      return;
    }

    const currentSuggest = ++suggestCounter;

    pathSuggestTimer = setTimeout(() => {
      void loadPathSuggestions(query, currentSuggest);
      pathSuggestTimer = null;
    }, 90);

    return () => {
      if (pathSuggestTimer) {
        clearTimeout(pathSuggestTimer);
        pathSuggestTimer = null;
      }
    };
  });

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
      passwordRevealed = false;
      uiState.unlockSurface = 'two-pane';
      uiState.sealedPromptOpen = false;
      uiState.view = 'list';

      try {
        const rememberedVault = await rememberCurrentVault();
        vaultState.rememberedVaultDisplayName = rememberedVault.displayName;
        vaultState.rememberedVaultAvailable = rememberedVault.available;
      } catch {
        uiState.notification = {
          kind: 'error',
          message: 'Vault opened, but Arca could not remember it',
        };
      }
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

  async function loadPathSuggestions(query: string, currentSuggest: number) {
    try {
      const suggestions = await suggestPaths(query);

      if (currentSuggest !== suggestCounter) {
        return;
      }

      pathSuggestions = suggestions;
      selectedPathIndex = suggestions.length > 0 ? Math.min(selectedPathIndex, suggestions.length - 1) : 0;
    } catch {
      if (currentSuggest === suggestCounter) {
        pathSuggestions = [];
        selectedPathIndex = 0;
      }
    }
  }

  function handlePathKeydown(event: KeyboardEvent) {
    if (!showPathSuggestions) {
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedPathIndex = (selectedPathIndex + 1) % pathSuggestions.length;
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedPathIndex = (selectedPathIndex + pathSuggestions.length - 1) % pathSuggestions.length;
      return;
    }

    if (event.key === 'Tab' || event.key === 'Enter') {
      event.preventDefault();
      applyPathSuggestion(pathSuggestions[selectedPathIndex]);
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      pathSuggestions = [];
      selectedPathIndex = 0;
    }
  }

  function applyPathSuggestion(suggestion: PathSuggestion | undefined) {
    if (!suggestion) {
      return;
    }

    path = suggestion.path;
    selectedPathIndex = 0;

    if (suggestion.kind === 'file') {
      pathSuggestions = [];
      passwordInput?.focus();
    } else {
      pathInput?.focus();
    }
  }

  function handlePathBlur() {
    window.setTimeout(() => {
      pathFocused = false;
    }, 120);
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to open vault';
  }

  function openSealedPrompt() {
    if (!isSealed || busy) {
      return;
    }

    errorMessage = '';
    uiState.sealedPromptOpen = true;
    schedulePasswordFocus();
  }

  function closeSealedPrompt() {
    if (busy) {
      return;
    }

    password = '';
    passwordRevealed = false;
    errorMessage = '';
    uiState.sealedPromptOpen = false;
    openButton?.focus();

    if (focusTimer) {
      clearTimeout(focusTimer);
      focusTimer = null;
    }
  }

  function schedulePasswordFocus() {
    if (focusTimer) {
      clearTimeout(focusTimer);
    }

    const delay =
      window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 0 : 380;

    focusTimer = setTimeout(() => {
      passwordInput?.focus();
      focusTimer = null;
    }, delay);
  }

  function togglePasswordReveal() {
    passwordRevealed = !passwordRevealed;
  }

  function showVaultPicker(clearPath: boolean) {
    if (busy) {
      return;
    }

    password = '';
    passwordRevealed = false;
    errorMessage = '';
    path = clearPath ? '' : vaultState.vaultPath;
    uiState.unlockSurface = 'two-pane';
    uiState.sealedPromptOpen = false;

    if (focusTimer) {
      clearTimeout(focusTimer);
    }
    focusTimer = setTimeout(() => {
      pathInput?.focus();
      focusTimer = null;
    }, 0);
  }

  async function forgetVault() {
    if (busy) {
      return;
    }

    busy = true;
    errorMessage = '';

    try {
      await forgetRememberedVault();
      path = '';
      password = '';
      passwordRevealed = false;
      clearRememberedVaultState();
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }
</script>

<section class="unlock-screen" aria-labelledby="unlock-title">
  {#if isSealed}
    <div class={sealedOpen ? 'sealed sealed--open' : 'sealed'}>
      <div class="sealed__brand">
        <div class="sealed__brand-meta mono">
          <span>v01 · 2026</span>
          <span>identity · <b>arca</b></span>
        </div>

        <div class="sealed__brand-center">
          <Lockup size={128} />
          <h1 id="unlock-title" class="sealed__tagline">
            the vault for what you <em>can't lose.</em><br />
            kept where only you can reach it.
          </h1>
          <button
            bind:this={openButton}
            type="button"
            class="sealed__cta"
            onclick={openSealedPrompt}
            disabled={sealedOpen}
            aria-label={vaultState.rememberedVaultAvailable ? 'open vault' : 'recover unavailable vault'}
          >
            <span class="sealed__cta-pill">
              <Icon name="key" size={11} sw={2} />
              {vaultState.rememberedVaultAvailable ? 'press to unlock' : 'vault unavailable'}
            </span>
            <span class="sealed__cta-hint"><Kbd value="↵" /> &nbsp;or click</span>
          </button>
        </div>

        <div class="sealed__brand-meta mono">
          <span>
            <span class={vaultState.rememberedVaultAvailable ? 'status__dot' : 'status__dot status__dot--warn'}></span>
            last_vault · <b>{vaultState.rememberedVaultDisplayName || 'remembered'}</b>
          </span>
          <span>zero_knowledge · <b>enabled</b></span>
        </div>
      </div>

      <div class="sealed__panel" aria-hidden={!sealedOpen} inert={!sealedOpen}>
        <form
          class="sealed__panel-inner"
          onsubmit={(event) => {
            event.preventDefault();
            submit();
          }}
        >
          <button type="button" class="sealed__panel-back" onclick={closeSealedPrompt} aria-label="cancel">
            ← cancel
          </button>

          {#if vaultState.rememberedVaultAvailable}
            <label>
              <div class="unlock__field-label">
                <span>master_password</span>
                <span>argon2id · chacha20</span>
              </div>
              <div class="unlock__field">
                <input
                  bind:this={passwordInput}
                  bind:value={password}
                  autocomplete="current-password"
                  class="unlock__input"
                  type={passwordRevealed ? 'text' : 'password'}
                  aria-label="master password"
                />
                <IconButton
                  label={passwordRevealed ? 'Hide master password' : 'Reveal master password'}
                  variant="ghost"
                  onclick={togglePasswordReveal}
                  disabled={!password}
                >
                  <Icon name="eye" size={14} />
                </IconButton>
              </div>
            </label>

            {#if errorMessage}
              <div class="unlock__error mono" role="alert">{errorMessage}</div>
            {/if}

            <Button class="unlock__cta" variant="primary" type="submit" disabled={!canSubmit}>
              <Icon name="key" size={12} sw={2} />
              {busy ? 'working' : 'unlock_vault'}
              <Kbd value="↵" />
            </Button>

            <Button
              class="unlock__cta sealed__switch-vault"
              variant="ghost"
              type="button"
              onclick={() => showVaultPicker(true)}
              aria-keyshortcuts="Meta+O Control+O"
            >
              <Icon name="vault" size={13} sw={1.6} />
              open another vault
              <span class="sealed__switch-shortcut"><Kbd value={modLabel} /> + <Kbd value="O" /></span>
            </Button>

            <div class="unlock__hints mono">
              <span><Kbd value="↵" /> <b>unlock</b></span>
              <button type="button" class="unlock__hint-action" onclick={forgetVault}>
                forget vault
              </button>
            </div>
          {:else}
            <div class="sealed__recovery" role="alert">
              <span class="sealed__recovery-kicker mono">vault_unavailable</span>
              <h2>{vaultState.rememberedVaultDisplayName || 'Last vault'} can’t be reached.</h2>
              <p>Locate it at a new path, open a different vault, or forget this saved locator.</p>
            </div>

            {#if errorMessage}
              <div class="unlock__error mono" role="alert">{errorMessage}</div>
            {/if}

            <div class="sealed__recovery-actions">
              <Button class="unlock__cta" variant="primary" type="button" onclick={() => showVaultPicker(false)}>
                locate vault
              </Button>
              <Button class="unlock__cta" variant="ghost" type="button" onclick={() => showVaultPicker(true)}>
                open another vault
              </Button>
              <Button class="unlock__cta" variant="danger" type="button" onclick={forgetVault} disabled={busy}>
                {busy ? 'working' : 'forget vault'}
              </Button>
            </div>
          {/if}

          <div class="ds-hr"></div>

          <div class="sealed__brand-meta mono sealed__panel-meta">
            <span>argon2id · m=128 · t=3 · p=4</span>
            <span>local_only · <b>ready</b></span>
          </div>
        </form>
      </div>
    </div>
  {:else}
    <div class="unlock">
      <div class="unlock__left">
        <div>
          <div class="unlock__caption mono">
            <span>v01 · 2026</span>
            <span>identity · <b>arca</b></span>
          </div>
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
          <span>identity system · <b>p. 01</b></span>
          <span class="unlock__caption-trail">bricolage grotesque · 800</span>
        </div>
      </div>

      <form
        class="unlock__right"
        onsubmit={(event) => {
          event.preventDefault();
          submit();
        }}
      >
        <div class="unlock__mode-tabs" role="tablist" aria-label="Vault mode">
          <button
            type="button"
            role="tab"
            aria-selected={mode === 'open'}
            class={mode === 'open' ? 'unlock__mode-tab unlock__mode-tab--active' : 'unlock__mode-tab'}
            onclick={() => (mode = 'open')}
          >
            open
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={mode === 'create'}
            class={mode === 'create' ? 'unlock__mode-tab unlock__mode-tab--active' : 'unlock__mode-tab'}
            onclick={() => (mode = 'create')}
          >
            create
          </button>
        </div>

        {#if mode === 'create'}
          <label>
            <div class="unlock__field-label">
              <span>vault_name</span>
              <span>local_first · encrypted</span>
            </div>
            <div class="unlock__field unlock__field--compact">
              <input bind:value={vaultName} autocomplete="off" class="unlock__input" spellcheck="false" />
            </div>
          </label>
        {/if}

        <div>
          <div id="vault-path-label" class="unlock__field-label">
            <span>vault_path</span>
            <span>{mode === 'open' ? 'existing vault' : 'new vault'}</span>
          </div>
          <div class="unlock__field unlock__field--compact unlock__field--path">
            <input
              bind:this={pathInput}
              bind:value={path}
              autocomplete="off"
              class="unlock__input"
              placeholder="/Users/you/.arca/vaults/primary.arca"
              spellcheck="false"
              role="combobox"
              aria-labelledby="vault-path-label"
              aria-expanded={showPathSuggestions}
              aria-controls="vault-path-suggestions"
              aria-autocomplete="list"
              onfocus={() => (pathFocused = true)}
              onblur={handlePathBlur}
              onkeydown={handlePathKeydown}
            />
            {#if showPathSuggestions}
              <div id="vault-path-suggestions" class="path-suggest" role="listbox" aria-label="Path suggestions">
                {#each pathSuggestions as suggestion, index}
                  <button
                    type="button"
                    class={index === selectedPathIndex ? 'path-suggest__item path-suggest__item--active' : 'path-suggest__item'}
                    role="option"
                    aria-selected={index === selectedPathIndex}
                    onmouseenter={() => (selectedPathIndex = index)}
                    onmousedown={(event) => {
                      event.preventDefault();
                      applyPathSuggestion(suggestion);
                    }}
                  >
                    <span class="path-suggest__prompt">&gt;</span>
                    <span class="path-suggest__name">{suggestion.name}</span>
                    <span class={suggestion.vaultCandidate ? 'path-suggest__kind path-suggest__kind--vault' : 'path-suggest__kind'}>
                      {suggestion.vaultCandidate ? 'vault' : suggestion.kind}
                    </span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <label>
          <div class="unlock__field-label">
            <span>master_password</span>
            <span>argon2id · chacha20</span>
          </div>
          <div class="unlock__field">
            <input
              bind:this={passwordInput}
              bind:value={password}
              autocomplete="current-password"
              class="unlock__input"
              type={passwordRevealed ? 'text' : 'password'}
              aria-label="master password"
            />
            <IconButton
              label={passwordRevealed ? 'Hide master password' : 'Reveal master password'}
              variant="ghost"
              onclick={togglePasswordReveal}
              disabled={!password}
            >
              <Icon name="eye" size={14} />
            </IconButton>
          </div>
        </label>

        {#if errorMessage}
          <div class="unlock__error mono" role="alert">{errorMessage}</div>
        {/if}

        <Button class="unlock__cta" variant="primary" type="submit" disabled={!canSubmit}>
          <Icon name="key" size={12} sw={2} />
          {busy ? 'working' : mode === 'open' ? 'unlock_vault' : 'create_vault'}
          <Kbd value="↵" />
        </Button>

        <div class="unlock__hints mono">
          <span><Kbd value="↵" /> <b>{mode === 'open' ? 'unlock' : 'create'}</b></span>
        </div>

        <div class="ds-hr"></div>

        <div class="unlock__caption mono unlock__connection">
          <span><span class="status__dot"></span> local_store · <b>ready</b></span>
          <span>zero_knowledge · <b>enabled</b></span>
        </div>
      </form>
    </div>
  {/if}
</section>
