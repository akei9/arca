<script lang="ts">
  import type { EntryDto } from '../../ipc';

  interface Props {
    entry: EntryDto;
    onopenhistory?: () => void;
  }

  let { entry, onopenhistory }: Props = $props();

  const created = $derived(formatDate(entry.createdAt));
  const updated = $derived(formatRelative(entry.updatedAt));
  const revisions = $derived(formatRevisionCount(entry.revisionCount));

  function formatDate(value: string): string {
    const time = Date.parse(value);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    return new Date(time).toISOString().slice(0, 10);
  }

  function formatRelative(value: string): string {
    const time = Date.parse(value);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    const elapsedMs = Date.now() - time;
    const elapsedHours = Math.max(0, Math.round(elapsedMs / 3_600_000));

    if (elapsedHours < 24) {
      return `${elapsedHours}h ago`;
    }

    return `${Math.round(elapsedHours / 24)}d ago`;
  }

  function formatRevisionCount(value: number): string {
    return value.toString().padStart(2, '0');
  }
</script>

<div class="strip">
  <div class="strip__cell">
    <div class="strip__k">created</div>
    <div class="strip__v">{created}</div>
  </div>
  <div class="strip__cell">
    <div class="strip__k">last_modified</div>
    <div class="strip__v">{updated}</div>
  </div>
  {#if onopenhistory}
    <button
      type="button"
      class="strip__cell strip__cell--btn"
      onclick={onopenhistory}
      aria-label={`View revision history (${entry.revisionCount})`}
    >
      <div class="strip__k">revisions</div>
      <div class="strip__v strip__v--accent strip__v--link">{revisions}</div>
    </button>
  {:else}
    <div class="strip__cell">
      <div class="strip__k">revisions</div>
      <div class="strip__v strip__v--accent">{revisions}</div>
    </div>
  {/if}
</div>
