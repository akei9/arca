<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getEntryRevisions, revealEntryRevisionPassword, type EntryDto, type RevisionDto } from '../../ipc';
  import { COPY_CONFIRMATION_MS, writeConfiguredClipboardText } from '../../clipboard';
  import { runtimeSettings } from '../../stores/settings.svelte';
  import { Icon } from '../icons';
  import { Button, IconButton } from '../primitives';

  interface Props {
    entry: EntryDto;
    onclose: () => void;
  }

  let { entry, onclose }: Props = $props();

  type Change =
    | { kind: 'password' }
    | { kind: 'note'; key: string }
    | { kind: 'field'; key: string; from: string; to: string };

  const MASK = '●'.repeat(20);
  const REVEAL_SECONDS = 10;

  let revisions = $state<RevisionDto[]>([]);
  let loading = $state(true);
  let loadError = $state('');
  let revealedIndex = $state<number | null>(null);
  let revealedPassword = $state('');
  let copiedIndex = $state<number | null>(null);
  let busyIndex = $state<number | null>(null);
  let countdown = $state(REVEAL_SECONDS);
  let actionError = $state('');
  let countdownTimer: ReturnType<typeof setInterval> | null = null;
  let copyTimer: ReturnType<typeof setTimeout> | null = null;
  let dialog: HTMLDivElement | null = null;
  let destroyed = false;

  const items = $derived(
    revisions.map((revision, index) => {
      const newer: EntryDto | RevisionDto = index === 0 ? entry : revisions[index - 1];
      return {
        index,
        ref: 'R' + String(revisions.length - index).padStart(2, '0'),
        when: relativeTime(revision.capturedAt),
        stamp: formatStamp(revision.capturedAt),
        latest: index === 0,
        changes: computeChanges(revision, newer),
      };
    }),
  );

  const countLabel = $derived(String(revisions.length).padStart(2, '0'));
  const maxLabel = $derived(runtimeSettings.current.entryRevisionLimit ?? 25);
  const clipboardNote = $derived(clipboardClearNote(runtimeSettings.current.clipboardClearSeconds));

  onMount(() => {
    void load();

    const previouslyFocused = document.activeElement as HTMLElement | null;
    dialog?.focus();

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        onclose();
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
      previouslyFocused?.focus();
    };
  });

  onDestroy(() => {
    destroyed = true;
    clearCountdown();

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
    actionError = '';

    try {
      const password = await revealEntryRevisionPassword(entry.id, index);
      return destroyed ? '' : password;
    } catch (error) {
      actionError = messageFromError(error);
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

    if (destroyed || !password) {
      return;
    }

    revealedPassword = password;
    revealedIndex = index;
    startCountdown();
  }

  async function copyRevision(index: number) {
    const password =
      revealedIndex === index && revealedPassword ? revealedPassword : await loadPassword(index);

    if (destroyed || !password) {
      return;
    }

    if (!(await writeConfiguredClipboardText(password))) {
      return;
    }

    if (destroyed) {
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

  function startCountdown() {
    clearCountdown();
    countdown = REVEAL_SECONDS;

    countdownTimer = setInterval(() => {
      if (countdown <= 1) {
        hideRevealed();
      } else {
        countdown -= 1;
      }
    }, 1000);
  }

  function clearCountdown() {
    if (countdownTimer) {
      clearInterval(countdownTimer);
      countdownTimer = null;
    }
  }

  function hideRevealed() {
    clearCountdown();
    revealedIndex = null;
    revealedPassword = '';
    countdown = REVEAL_SECONDS;
  }

  function computeChanges(revision: RevisionDto, newer: EntryDto | RevisionDto): Change[] {
    const changes: Change[] = [];

    if (revision.passwordChanged) {
      changes.push({ kind: 'password' });
    }

    pushFieldChange(changes, 'title', revision.title, newer.title);
    pushFieldChange(changes, 'user', revision.username, newer.username);
    pushFieldChange(changes, 'url', revision.url, newer.url);
    pushFieldChange(changes, 'collection', revision.collection, newer.collection);

    if (normalize(revision.notes) !== normalize(newer.notes)) {
      changes.push({ kind: 'note', key: 'notes' });
    }
    if (!sameTags(revision.tags, newer.tags)) {
      changes.push({ kind: 'note', key: 'tags' });
    }

    return changes;
  }

  function pushFieldChange(changes: Change[], key: string, from: string | null, to: string | null) {
    if (normalize(from) === normalize(to)) {
      return;
    }

    changes.push({ kind: 'field', key, from: fieldValue(from), to: fieldValue(to) });
  }

  function normalize(value: string | null): string {
    return (value ?? '').trim();
  }

  function fieldValue(value: string | null): string {
    return value && value.trim() ? value : 'none';
  }

  function sameTags(a: string[], b: string[]): boolean {
    return a.length === b.length && a.every((tag, index) => tag === b[index]);
  }

  function relativeTime(value: string): string {
    const time = Date.parse(value);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    const minutes = Math.round((Date.now() - time) / 60_000);

    if (minutes < 1) {
      return 'just now';
    }
    if (minutes < 60) {
      return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
    }

    const hours = Math.round(minutes / 60);
    if (hours < 24) {
      return `${hours} hour${hours === 1 ? '' : 's'} ago`;
    }

    const days = Math.round(hours / 24);
    if (days === 1) {
      return 'yesterday';
    }
    if (days < 7) {
      return `${days} days ago`;
    }

    const weeks = Math.round(days / 7);
    if (weeks === 1) {
      return 'last week';
    }
    if (weeks < 5) {
      return `${weeks} weeks ago`;
    }

    const months = Math.round(days / 30);
    if (months === 1) {
      return 'last month';
    }
    if (months < 12) {
      return `${months} months ago`;
    }

    const years = Math.round(days / 365);
    return `${years} year${years === 1 ? '' : 's'} ago`;
  }

  function formatStamp(value: string): string {
    const time = Date.parse(value);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    return new Date(time).toISOString().slice(0, 16).replace('T', ' ');
  }

  function formatCountdown(value: number): string {
    return '0:' + String(value).padStart(2, '0');
  }

  function clipboardClearNote(seconds: number | null | undefined): string {
    return typeof seconds === 'number' && seconds > 0
      ? `clipboard clears in ${seconds}s`
      : 'copied to clipboard';
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to load revision history';
  }
</script>

<div class="rev-overlay">
  <button type="button" class="rev-overlay__scrim" aria-label="Close revision history" onclick={onclose}
  ></button>

  <div
    bind:this={dialog}
    tabindex="-1"
    class="rev-panel"
    role="dialog"
    aria-modal="true"
    aria-labelledby="rev-panel-title"
  >
    <div class="rev-panel__head">
      <span class="rev-panel__ico"><Icon name="history" size={17} /></span>
      <div class="rev-panel__titles">
        <div class="rev-panel__eyebrow">revision_history</div>
        <div id="rev-panel-title" class="rev-panel__title">{entry.title}</div>
      </div>
      {#if !loading && !loadError && revisions.length > 0}
        <div class="rev-panel__count"><b>{countLabel}</b> kept &middot; max {maxLabel}</div>
      {/if}
      <Button variant="bare" size="sm" class="rev-panel__close" onclick={onclose}>
        close <kbd>esc</kbd>
      </Button>
    </div>

    {#if actionError}
      <div class="rev-panel__error mono" role="alert"><b>error</b> &middot; {actionError}</div>
    {/if}

    {#if loading}
      <div class="rev-list">
        <div class="rev-loading__cap">
          <span class="rev-loading__spin"></span>loading_revisions<span class="rev-loading__dots"></span>
        </div>
        {#each [0, 1, 2] as row (row)}
          <div class="rev rev--skel">
            <span class="rev__node"></span>
            <div class="rev__head">
              <span class="skel skel--ref"></span><span class="skel skel--when"></span>
            </div>
            <span class="skel skel--pw"></span>
          </div>
        {/each}
      </div>
    {:else if loadError}
      <div class="rev-list">
        <div class="rev__note error" role="alert"><b>error</b> &middot; {loadError}</div>
      </div>
    {:else if revisions.length === 0}
      <div class="rev-empty">
        <div class="rev-empty__mark"><Icon name="history" size={27} /></div>
        <div class="rev-empty__k">no_revisions</div>
        <h3 class="rev-empty__title">no earlier versions</h3>
        <p class="rev-empty__sub">
          Arca snapshots a revision automatically whenever you change a field on this entry. Once you
          make an edit, prior versions collect here - newest first.
        </p>
      </div>
    {:else}
      <div class="rev-list">
        {#each items as item (item.index)}
          {@const isOpen = revealedIndex === item.index}
          {@const isCopied = copiedIndex === item.index}
          <div class={item.latest ? 'rev rev--latest' : 'rev'}>
            <span class="rev__node"></span>
            <div class="rev__head">
              <span class="rev__ref">{item.ref}</span>
              <span class="rev__when">{item.when}</span>
              <span class="rev__stamp">{item.stamp}</span>
              {#if item.latest}<span class="rev__tag">latest</span>{/if}
            </div>

            {#if item.changes.length > 0}
              <div class="rev__diff">
                {#each item.changes as change, changeIndex (changeIndex)}
                  {#if change.kind === 'password'}
                    <span class="rev__chg rev__chg--pw"><span class="rev__chg-k">password</span>updated</span>
                  {:else if change.kind === 'note'}
                    <span class="rev__chg"><span class="rev__chg-k">{change.key}</span>changed</span>
                  {:else}
                    <span class="rev__chg">
                      <span class="rev__chg-k">{change.key}</span>
                      <span class="rev__chg-old">{change.from}</span>
                      <span class="rev__chg-arrow">&rarr;</span>
                      <span class="rev__chg-new">{change.to}</span>
                    </span>
                  {/if}
                {/each}
              </div>
            {/if}

            <div class={isOpen ? 'rev__pw rev__pw--open' : 'rev__pw'}>
              <span class="rev__pw-k">password</span>
              <span class={isOpen ? 'rev__pw-v rev__pw-v--open' : 'rev__pw-v'}>
                {isOpen ? revealedPassword : MASK}
              </span>
              <span class="rev__pw-right">
                {#if isOpen}<span class="rev__pw-cd">{formatCountdown(countdown)}</span>{/if}
                <span class="rev__pw-actions">
                  <IconButton
                    label={isOpen ? 'Hide revision password' : 'Reveal revision password'}
                    variant={isOpen ? 'accent' : 'default'}
                    disabled={busyIndex === item.index}
                    onclick={() => toggleReveal(item.index)}
                  >
                    <Icon name={isOpen ? 'eye-off' : 'eye'} size={13} />
                  </IconButton>
                  <IconButton
                    label={isCopied ? 'Revision password copied' : 'Copy revision password'}
                    variant={isCopied ? 'vault' : 'default'}
                    disabled={busyIndex === item.index}
                    onclick={() => copyRevision(item.index)}
                  >
                    {#if isCopied}
                      <span class="rev__ok">&#10003;</span>
                    {:else}
                      <Icon name="copy" size={13} />
                    {/if}
                  </IconButton>
                </span>
              </span>
              {#if isOpen}
                <span class="rev__pw-timer" style="width: {countdown * 10}%"></span>
              {/if}
            </div>

            {#if isCopied}
              <div class="rev__note"><b>copied</b> &middot; {clipboardNote}</div>
            {:else if isOpen}
              <div class="rev__note rev__note--reveal">
                <b>revealed</b> &middot; auto-hides in {countdown}s
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
