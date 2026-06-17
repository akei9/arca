<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { EntryDto } from '../../ipc';
  import { COPY_CONFIRMATION_MS, writeConfiguredClipboardText } from '../../clipboard';
  import { Icon, type IconName } from '../icons';
  import { Tag } from '../primitives';

  let {
    entry,
    selected = false,
    shortcut,
    onselect,
  } = $props<{
    entry: EntryDto;
    selected?: boolean;
    shortcut?: string;
    onselect?: (entry: EntryDto) => void;
  }>();

  const iconName = $derived(iconForEntry(entry));
  const weak = $derived(Boolean(entry.tags.find((tag: string) => normalize(tag) === 'weak')));
  const subtitle = $derived(entry.username || entry.url || entry.id);
  const copyValue = $derived(entry.username || entry.url || entry.title);
  let copied = $state(false);
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
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onselect?.(entry);
    }
  }

  async function handleCopy(event: MouseEvent) {
    event.stopPropagation();

    if (!copyValue) {
      return;
    }

    if (!(await writeConfiguredClipboardText(copyValue))) {
      return;
    }

    copied = true;

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copied = false;
      copyTimer = null;
    }, COPY_CONFIRMATION_MS);
  }
</script>

<div
  class={selected ? 'row row--selected' : 'row'}
  role="option"
  tabindex="0"
  aria-selected={selected}
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
  {#if shortcut}
    <Tag value={shortcut} />
  {/if}
  <div class="row__actions">
    <button
      class="row__action"
      type="button"
      aria-label={`Reveal ${entry.title}`}
      onclick={(event) => event.stopPropagation()}
    >
      <Icon name="eye" size={13} sw={1.5} />
    </button>
    <button
      class={copied || selected ? 'row__action row__action--ok' : 'row__action'}
      type="button"
      aria-label={copied ? `${entry.title} copied` : `Copy ${entry.title}`}
      onclick={handleCopy}
    >
      {#if copied}
        ✓
      {:else}
        <Icon name="copy" size={13} sw={1.5} />
      {/if}
    </button>
  </div>
</div>
