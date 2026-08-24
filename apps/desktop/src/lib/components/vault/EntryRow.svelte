<script lang="ts">
  import { onDestroy } from 'svelte';
  import { revealEntryPassword, type EntryDto } from '../../ipc';
  import { COPY_CONFIRMATION_MS, writeConfiguredClipboardText } from '../../clipboard';
  import { Icon, type IconName } from '../icons';
  import { Tag } from '../primitives';

  interface Props {
    entry: EntryDto;
    selected?: boolean;
    rowKey?: string;
    onselect?: (entry: EntryDto) => void;
  }

  let {
    entry,
    selected = false,
    rowKey,
    onselect,
  }: Props = $props();

  const iconName = $derived(iconForEntry(entry));
  const weak = $derived(Boolean(entry.tags.find((tag: string) => normalize(tag) === 'weak')));
  const subtitle = $derived(entry.username || entry.url || entry.id);
  type CopyKind = 'username' | 'password';

  let copied = $state<CopyKind | null>(null);
  let copyBusy = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;

  onDestroy(() => {
    if (copyTimer) {
      clearTimeout(copyTimer);
    }
  });

  function iconForEntry(candidate: EntryDto): IconName {
    const haystack = `${candidate.title} ${candidate.url ?? ''} ${candidate.tags.join(' ')}`.toLowerCase();

    if (haystack.includes('github') || haystack.includes('code')) {
      return 'code';
    }

    if (haystack.includes('aws') || haystack.includes('cloud')) {
      return 'cloud';
    }

    if (haystack.includes('bank')) {
      return 'bank';
    }

    if (haystack.includes('mail') || haystack.includes('email')) {
      return 'at';
    }

    if (haystack.includes('server') || haystack.includes('infra') || haystack.includes('router')) {
      return 'server';
    }

    if (haystack.includes('key') || haystack.includes('ssh')) {
      return 'key';
    }

    if (haystack.includes('linear') || haystack.includes('box')) {
      return 'box';
    }

    return 'vault';
  }

  function normalize(value: string): string {
    return value.trim().toLowerCase();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!selected) {
      return;
    }

    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onselect?.(entry);
    }
  }

  async function handleCopy(event: MouseEvent, kind: CopyKind, value: string) {
    event.stopPropagation();

    if (!value) {
      return;
    }

    if (!(await writeConfiguredClipboardText(value))) {
      return;
    }

    scheduleCopied(kind);
  }

  async function handlePasswordCopy(event: MouseEvent) {
    event.stopPropagation();

    if (copyBusy) {
      return;
    }

    copyBusy = true;

    try {
      const password = entry.password ?? (await revealEntryPassword(entry.id));

      if (!password || !(await writeConfiguredClipboardText(password))) {
        return;
      }

      scheduleCopied('password');
    } catch {
      return;
    } finally {
      copyBusy = false;
    }
  }

  function scheduleCopied(kind: CopyKind) {
    copied = kind;

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copied = null;
      copyTimer = null;
    }, COPY_CONFIRMATION_MS);
  }

  function openEntry(event: MouseEvent) {
    event.stopPropagation();
    onselect?.(entry);
  }
</script>

<div
  class={selected ? 'row row--selected' : 'row'}
  role="option"
  tabindex={selected ? 0 : -1}
  aria-selected={selected}
  data-entry-row-key={rowKey}
  onclick={() => onselect?.(entry)}
  onkeydown={handleKeydown}
>
  <div class="row__bullet">
    <Icon name={iconName} size={15} sw={1.5} />
  </div>
  <div class="row__main">
    <div class="row__title">
      {entry.title}
      {#if weak}
        <Tag variant="out" value="weak" />
      {/if}
    </div>
    <div class="row__sub">{subtitle}</div>
  </div>
  <div class="row__actions">
    <button
      class="row__action"
      type="button"
      aria-label={`Open ${entry.title} details`}
      onclick={openEntry}
    >
      <Icon name="eye" size={13} sw={1.5} />
    </button>
    <button
      class={copied === 'username' ? 'row__action row__action--ok' : 'row__action'}
      type="button"
      aria-label={copied === 'username' ? `${entry.title} username copied` : `Copy ${entry.title} username`}
      disabled={!entry.username.trim()}
      onclick={(event) => handleCopy(event, 'username', entry.username)}
    >
      {#if copied === 'username'}
        ✓
      {:else}
        <Icon name="at" size={13} sw={1.5} />
      {/if}
    </button>
    <button
      class={copied === 'password' ? 'row__action row__action--ok' : 'row__action'}
      type="button"
      aria-label={copied === 'password' ? `${entry.title} password copied` : `Copy ${entry.title} password`}
      disabled={copyBusy}
      onclick={handlePasswordCopy}
    >
      {#if copied === 'password'}
        ✓
      {:else}
        <Icon name="key" size={13} sw={1.5} />
      {/if}
    </button>
  </div>
</div>
