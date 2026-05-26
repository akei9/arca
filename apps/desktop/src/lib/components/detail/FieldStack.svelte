<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { EntryDto } from '../../ipc';
  import { writeClipboardText } from '../../clipboard';
  import { Icon } from '../icons';
  import { Entropy, IconButton, Kbd } from '../primitives';

  let {
    entry,
  } = $props<{
    entry: EntryDto;
  }>();

  const passwordMask = '●●●●●●●●●●●●●●●●●●●●●●●●';
  let copied = $state<'username' | 'password' | null>(null);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;

  const url = $derived(entry.url?.trim() || 'not_set');
  const username = $derived(entry.username.trim() || 'not_set');
  const password = $derived(entry.password?.trim() || '');
  const entropyBits = $derived(Math.max(72, Math.min(112, entry.title.length * 6 + entry.username.length * 4 + 48)));
  const entropyFilled = $derived(Math.max(10, Math.min(16, Math.round(entropyBits / 7))));

  onDestroy(() => {
    if (copyTimer) {
      clearTimeout(copyTimer);
    }
  });

  async function copyValue(kind: 'username' | 'password', value: string) {
    if (!value) {
      return;
    }

    await writeClipboardText(value);
    copied = kind;

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copied = null;
      copyTimer = null;
    }, 1500);
  }
</script>

<div class="fields">
  <div class="field">
    <div class="field__k">url</div>
    <div class={entry.url ? 'field__v field__v--url' : 'field__v'}>{url}</div>
    <div class="field__actions">
      <IconButton label={`Open ${entry.title} URL`} disabled={!entry.url}>
        <Icon name="external" size={13} />
      </IconButton>
    </div>
  </div>

  <div class="field">
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

  <div class="field field--focus">
    <div class="field__k">password <Kbd value="C" /></div>
    <div class="field__v field__v--mask">{passwordMask}</div>
    <div class="field__actions">
      <IconButton label={`Reveal ${entry.title} password`}>
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
