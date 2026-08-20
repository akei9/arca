<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getEntryRevisions, revealEntryRevisionPassword, type EntryDto, type RevisionDto } from '../../ipc';
  import { COPY_CONFIRMATION_MS, writeConfiguredClipboardText } from '../../clipboard';
  import { Icon } from '../icons';
  import { Button, IconButton } from '../primitives';

  let {
    entry,
    onclose,
  } = $props<{
    entry: EntryDto;
    onclose: () => void;
  }>();

  const passwordMask = '●●●●●●●●●●●●●●●●●●●●●●●●';
  const passwordRevealMs = 10_000;

  let revisions = $state<RevisionDto[]>([]);
  let loading = $state(true);
  let loadError = $state('');
  let revealedIndex = $state<number | null>(null);
  let revealedPassword = $state('');
  let busyIndex = $state<number | null>(null);
  let copiedIndex = $state<number | null>(null);
  let revealTimer: ReturnType<typeof setTimeout> | null = null;
  let copyTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    void load();

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        onclose();
      }
    }

    window.addEventListener('keydown', handleKeydown, true);
    return () => window.removeEventListener('keydown', handleKeydown, true);
  });

  onDestroy(() => {
    clearRevealTimer();

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    revealedPassword = '';
  });

  async function load() {
    loading = true;
    loadError = '';

    try {
      revisions = await getEntryRevisions(entry.id);
    } catch (error) {
      loadError = messageFromError(error);
    } finally {
      loading = false;
    }
  }

  async function loadPassword(index: number): Promise<string> {
    if (busyIndex !== null) {
      return '';
    }

    busyIndex = index;

    try {
      return await revealEntryRevisionPassword(entry.id, index);
    } catch {
      return '';
    } finally {
      busyIndex = null;
    }
  }

  async function toggleReveal(index: number) {
    if (revealedIndex === index) {
      hideRevealed();
      return;
    }

    const password = await loadPassword(index);

    if (!password) {
      return;
    }

    revealedPassword = password;
    revealedIndex = index;
    scheduleRevealHide();
  }

  async function copyRevision(index: number) {
    const password = revealedIndex === index && revealedPassword ? revealedPassword : await loadPassword(index);

    if (!password) {
      return;
    }

    if (!(await writeConfiguredClipboardText(password))) {
      return;
    }

    copiedIndex = index;

    if (copyTimer) {
      clearTimeout(copyTimer);
    }

    copyTimer = setTimeout(() => {
      copiedIndex = null;
      copyTimer = null;
    }, COPY_CONFIRMATION_MS);
  }

  function hideRevealed() {
    revealedIndex = null;
    revealedPassword = '';
    clearRevealTimer();
  }

  function scheduleRevealHide() {
    clearRevealTimer();

    revealTimer = setTimeout(() => {
      revealedIndex = null;
      revealedPassword = '';
      revealTimer = null;
    }, passwordRevealMs);
  }

  function clearRevealTimer() {
    if (revealTimer) {
      clearTimeout(revealTimer);
      revealTimer = null;
    }
  }

  function formatCaptured(value: string): string {
    const time = Date.parse(value);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    return new Date(time).toISOString().slice(0, 16).replace('T', ' ');
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to load revision history';
  }
</script>

<div
  class="modal revision-history"
  role="dialog"
  aria-modal="true"
  aria-labelledby="revision-history-title"
>
  <div class="revision-history__head">
    <div id="revision-history-title" class="modal__q">revision_history · {entry.title}</div>
    <Button variant="ghost" size="sm" onclick={onclose} aria-keyshortcuts="Escape">close</Button>
  </div>

  {#if loading}
    <p class="revision-history__status mono">loading_revisions</p>
  {:else if loadError}
    <div class="revision-history__status mono error" role="alert">{loadError}</div>
  {:else if revisions.length === 0}
    <p class="revision-history__status mono">no_revisions</p>
  {:else}
    <ul class="revision-history__list">
      {#each revisions as revision, index (index)}
        <li class="revision-history__item">
          <div class="revision-history__meta mono">
            <span>captured · <b>{formatCaptured(revision.capturedAt)}</b></span>
            <span>title · <b>{revision.title}</b></span>
            <span>user · <b>{revision.username.trim() || 'not_set'}</b></span>
            {#if revision.url?.trim()}
              <span>url · <b>{revision.url}</b></span>
            {/if}
          </div>
          <div class={revealedIndex === index ? 'field field--focus field--secret-revealed' : 'field field--focus'}>
            <div class="field__k">password</div>
            <div class={revealedIndex === index ? 'field__v field__v--secret' : 'field__v field__v--mask'}>
              {revealedIndex === index ? revealedPassword : passwordMask}
            </div>
            <div class="field__actions">
              <IconButton
                label={revealedIndex === index ? 'Hide revision password' : 'Reveal revision password'}
                variant={revealedIndex === index ? 'accent' : 'default'}
                disabled={busyIndex === index}
                onclick={() => toggleReveal(index)}
              >
                <Icon name="eye" size={13} />
              </IconButton>
              <IconButton
                label={copiedIndex === index ? 'Revision password copied' : 'Copy revision password'}
                variant={copiedIndex === index ? 'accent' : 'default'}
                disabled={busyIndex === index}
                onclick={() => copyRevision(index)}
              >
                {#if copiedIndex === index}
                  ✓
                {:else}
                  <Icon name="copy" size={13} />
                {/if}
              </IconButton>
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .revision-history {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: min(70vh, 34rem);
  }

  .revision-history__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .revision-history__status {
    opacity: 0.7;
  }

  .revision-history__list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin: 0;
    padding: 0;
    list-style: none;
    overflow-y: auto;
  }

  .revision-history__item {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border, rgba(127, 127, 127, 0.2));
  }

  .revision-history__item:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .revision-history__meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem 1rem;
    font-size: 0.75rem;
    opacity: 0.75;
  }
</style>
