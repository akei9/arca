<script lang="ts">
  import type { EntryDto } from '../../ipc';

  let {
    entry,
  } = $props<{
    entry: EntryDto;
  }>();

  const created = $derived(formatDate(entry.createdAt));
  const updated = $derived(formatRelative(entry.updatedAt));

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
</script>

<div class="strip">
  <div class="strip__cell">
    <div class="strip__k">created</div>
    <div class="strip__v">{created}</div>
  </div>
  <div class="strip__cell">
    <div class="strip__k">last_used</div>
    <div class="strip__v">{updated}</div>
  </div>
  <div class="strip__cell">
    <div class="strip__k">revisions</div>
    <div class="strip__v strip__v--accent">01</div>
  </div>
</div>
