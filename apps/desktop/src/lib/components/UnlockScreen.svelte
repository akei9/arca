<script lang="ts">
  import { onMount } from 'svelte';
  import { createVault, listEntries, unlockVault, type EntryDto } from '../ipc';
  import { vaultState } from '../stores/vault.svelte';
  import { uiState } from '../stores/ui.svelte';
  import { Lockup } from './brand';
  import { Icon } from './icons';
  import { Button, IconButton, Kbd } from './primitives';

  type Mode = 'open' | 'create';
  type Variant = 'two-pane' | 'sealed';

  let {
    variant = 'two-pane',
  } = $props<{
    variant?: Variant;
  }>();

  let mode: Mode = $state('open');
  let path = $state(vaultState.vaultPath);
  let password = $state('');
  let vaultName = $state('personal');
  let busy = $state(false);
  let errorMessage = $state('');
  let openButton = $state<HTMLButtonElement | null>(null);
  let passwordInput = $state<HTMLInputElement | null>(null);
  let focusTimer: ReturnType<typeof setTimeout> | null = null;

  const canSubmit = $derived(
    path.trim().length > 0 &&
      password.length > 0 &&
      (mode === 'open' || vaultName.trim().length > 0) &&
      !busy,
  );
  const isSealed = $derived(variant === 'sealed');
  const sealedOpen = $derived(isSealed && uiState.sealedPromptOpen);

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
      uiState.unlockSurface = 'two-pane';
      uiState.sealedPromptOpen = false;
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
            aria-label="open vault"
          >
            <span class="sealed__cta-pill">
              <Icon name="key" size={11} sw={2} />
              press to unlock
            </span>
            <span class="sealed__cta-hint"><Kbd value="↵" /> &nbsp;or click</span>
          </button>
        </div>

        <div class="sealed__brand-meta mono">
          <span><span class="status__dot"></span> local_store · <b>ready</b></span>
          <span>zero_knowledge · <b>enabled</b></span>
        </div>
      </div>

      <div class="sealed__panel">
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

          <label>
            <div class="unlock__field-label">
              <span>master_password</span>
              <span>argon2id · aes-512</span>
            </div>
            <div class="unlock__field">
              <input
                bind:this={passwordInput}
                bind:value={password}
                autocomplete="current-password"
                class="unlock__input"
                type="password"
                aria-label="master password"
              />
              <IconButton label="Password remains hidden" variant="ghost" disabled>
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

          <div class="unlock__hints mono">
            <span><Kbd value="↵" /> <b>unlock</b></span>
            <span><Kbd value="⌘" />+<Kbd value="," /> settings</span>
            <span><Kbd value="⌘" />+<Kbd value="O" /> other vault</span>
          </div>

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

        <label>
          <div class="unlock__field-label">
            <span>vault_path</span>
            <span>{mode === 'open' ? 'existing vault' : 'new vault'}</span>
          </div>
          <div class="unlock__field unlock__field--compact">
            <input
              bind:value={path}
              autocomplete="off"
              class="unlock__input"
              placeholder="/Users/you/.arca/vaults/primary.arca"
              spellcheck="false"
            />
          </div>
        </label>

        <label>
          <div class="unlock__field-label">
            <span>master_password</span>
            <span>argon2id · aes-512</span>
          </div>
          <div class="unlock__field">
            <input
              bind:value={password}
              autocomplete="current-password"
              class="unlock__input"
              type="password"
              aria-label="master password"
            />
            <IconButton label="Password remains hidden" variant="ghost" disabled>
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
          <span><Kbd value="⌘" />+<Kbd value="," /> settings</span>
          <span><Kbd value="⌘" />+<Kbd value="O" /> other vault</span>
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
