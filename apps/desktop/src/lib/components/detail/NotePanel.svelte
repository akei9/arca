<script lang="ts">
  import { Tag } from '../primitives';

  let {
    notes,
    tags = [],
  } = $props<{
    notes?: string | null;
    tags?: string[];
  }>();

  const body = $derived(
    notes?.trim() ||
      'No plain_text notes are stored for this entry yet.\n\nUse edit to add recovery instructions, account context, or rotation notes.',
  );
  const visibleTags = $derived(tags.filter((tag: string) => tag.trim().length > 0).slice(0, 4));
</script>

<div class="note-panel">
  <div class="note-panel__head">notes · plain_text</div>
  <div class="note-panel__body">{body}</div>
  <div class="note-panel__tags">
    {#if visibleTags.length > 0}
      {#each visibleTags as tag}
        <Tag variant="paper" value={tag} bracketed={false} />
      {/each}
    {:else}
      <Tag variant="paper" value="scope · vault" bracketed={false} />
    {/if}
  </div>
</div>
