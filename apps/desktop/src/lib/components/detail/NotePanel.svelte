<script lang="ts">
  import { Tag } from '../primitives';

  let {
    notes,
    tags = [],
    scope = 'vault',
  } = $props<{
    notes?: string | null;
    tags?: string[];
    scope?: string | null;
  }>();

  const body = $derived(
    notes?.trim() ||
      'No plain_text notes are stored for this entry yet.\n\nUse edit to add recovery instructions, account context, or rotation notes.',
  );
  const scopeLabel = $derived(`scope · ${normalizeScope(scope)}`);
  const visibleTags = $derived(
    tags
      .map((tag: string) => formatTag(tag))
      .filter((tag: string) => tag.length > 0)
      .slice(0, 4),
  );

  function normalizeScope(value: string | null | undefined): string {
    return value?.trim().toLowerCase() || 'vault';
  }

  function formatTag(value: string): string {
    const tag = value.trim().replace(/^#+/, '');
    return tag ? `#${tag.toUpperCase()}` : '';
  }
</script>

<div class="note-panel">
  <div class="note-panel__head">notes · plain_text</div>
  <div class="note-panel__body">{body}</div>
  <div class="note-panel__tags">
    <Tag variant="paper" value={scopeLabel} bracketed={false} />
    {#each visibleTags as tag}
      <Tag variant="paper" value={tag} bracketed={false} />
    {/each}
  </div>
</div>
