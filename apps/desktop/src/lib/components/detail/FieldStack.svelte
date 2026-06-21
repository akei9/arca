<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { EntryDto } from '../../ipc';
  import { writeConfiguredClipboardText } from '../../clipboard';
  import { Icon } from '../icons';
  import { Entropy, IconButton, Kbd } from '../primitives';

  let {
    entry,
  } = $props<{
    entry: EntryDto;
  }>();

  type CopyKind = 'url' | 'username' | 'password';

  const passwordMask = '●●●●●●●●●●●●●●●●●●●●●●●●';
  const passwordRevealMs = 10_000;

  let copied = $state<CopyKind | null>(null);
  let passwordRevealed = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;
  let revealTimer: ReturnType<typeof setTimeout> | null = null;

  const normalizedUrl = $derived(entry.url?.trim() || '');
  const url = $derived(normalizedUrl || 'not_set');
  const username = $derived(entry.username.trim() || 'not_set');
  const password = $derived(entry.password ?? '');
  const displayedPassword = $derived(passwordRevealed ? password : passwordMask);
  const entropyBits = $derived(Math.max(72, Math.min(112, entry.title.length * 6 + entry.username.length * 4 + 48)));
  const entropyFilled = $derived(Math.max(10, Math.min(16, Math.round(entropyBits / 7))));

  onDestroy(() => {
    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    clearRevealTimer();
  });

  onMount(() => {
    function handleKeydown(event: KeyboardEvent) {
      if (event.repeat || event.metaKey || event.ctrlKey || event.altKey || isEditableTarget(event.target)) {
        return;
      }

      if (event.key.toLowerCase() === 'u' && entry.username.trim()) {
        event.preventDefault();
        void copyValue('username', entry.username);
        return;
      }

      if (event.key.toLowerCase() === 'c' && password) {
        event.preventDefault();
        void copyValue('password', password);
        return;
      }

      if (event.key.toLowerCase() === 'r' && password) {
        event.preventDefault();
        togglePasswordReveal();
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  async function copyValue(kind: CopyKind, value: string) {
    if (!value) {
      return;
    }

    if (!(await writeConfiguredClipboardText(value))) {
      return;
    }

    copied = kind;

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copied = null;
      copyTimer = null;
    }, 1500);
  }

  function togglePasswordReveal() {
    if (!password) {
      return;
    }

    passwordRevealed = !passwordRevealed;

    if (passwordRevealed) {
      schedulePasswordHide();
    } else {
      clearRevealTimer();
    }
  }

  function schedulePasswordHide() {
    clearRevealTimer();

    revealTimer = setTimeout(() => {
      passwordRevealed = false;
      revealTimer = null;
    }, passwordRevealMs);
  }

  function clearRevealTimer() {
    if (revealTimer) {
      clearTimeout(revealTimer);
      revealTimer = null;
    }
  }

  function isEditableTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) {
      return false;
    }

    return (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target.isContentEditable
    );
  }

  $effect(() => {
    entry.id;
    password;
    passwordRevealed = false;
    copied = null;
    clearRevealTimer();
  });
</script>

<div class="fields">
  <div class={normalizedUrl ? 'field' : 'field field--empty'}>
    <div class="field__k">url</div>
    <div class={normalizedUrl ? 'field__v field__v--url' : 'field__v'}>{url}</div>
    <div class="field__actions">
      <IconButton
        label={copied === 'url' ? 'URL copied' : `Copy ${entry.title} URL`}
        variant={copied === 'url' ? 'accent' : 'default'}
        disabled={!normalizedUrl}
        onclick={() => normalizedUrl && copyValue('url', normalizedUrl)}
      >
        {#if copied === 'url'}
          ✓
        {:else}
          <Icon name="copy" size={13} />
        {/if}
      </IconButton>
    </div>
  </div>

  <div class={entry.username.trim() ? 'field' : 'field field--empty'}>
    <div class="field__k">user <Kbd value="U" /></div>
    <div class="field__v">{username}</div>
    <div class="field__actions">
      <IconButton
        label={copied === 'username' ? 'Username copied' : `Copy ${entry.title} username`}
        variant={copied === 'username' ? 'accent' : 'default'}
        disabled={!entry.username.trim()}
        onclick={() => copyValue('username', entry.username)}
      >
        {#if copied === 'username'}
          ✓
        {:else}
          <Icon name="copy" size={13} />
        {/if}
      </IconButton>
    </div>
  </div>

  <div class={passwordRevealed ? 'field field--focus field--secret-revealed' : 'field field--focus'}>
    <div class="field__k">password <Kbd value="C" /><Kbd value="R" /></div>
    <div class={passwordRevealed ? 'field__v field__v--secret' : 'field__v field__v--mask'}>{displayedPassword}</div>
    <div class="field__actions">
      <IconButton
        label={passwordRevealed ? `Hide ${entry.title} password` : `Reveal ${entry.title} password`}
        variant={passwordRevealed ? 'accent' : 'default'}
        disabled={!password}
        onclick={togglePasswordReveal}
      >
        <Icon name="eye" size={13} />
      </IconButton>
      <IconButton
        variant={copied === 'password' ? 'accent' : 'default'}
        label={copied === 'password' ? 'Password copied' : `Copy ${entry.title} password`}
        disabled={!password}
        onclick={() => copyValue('password', password)}
      >
        {#if copied === 'password'}
          ✓
        {:else}
          <Icon name="copy" size={13} />
        {/if}
      </IconButton>
    </div>
  </div>

  <div class="field">
    <div class="field__k">entropy</div>
    <div>
      <Entropy filled={entropyFilled} bits={entropyBits} />
    </div>
    <div></div>
  </div>
</div>
