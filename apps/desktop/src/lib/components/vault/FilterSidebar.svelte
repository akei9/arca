<script lang="ts">
  import { Tag } from '../primitives';

  export interface FilterItem {
    key: string;
    label: string;
    count: number;
  }

  let {
    filters = [],
    tags = [],
    active = 'all',
    onselect,
  } = $props<{
    filters?: FilterItem[];
    tags?: string[];
    active?: string;
    onselect?: (key: string) => void;
  }>();
</script>

<aside class="sidepanel" aria-label="Vault filters">
  <div class="sidepanel__title">
    categories
    <span class="sidepanel__title-rule"></span>
  </div>

  {#each filters as filter (filter.key)}
    <button
      type="button"
      class={active === filter.key ? 'filter filter--active' : 'filter'}
      aria-pressed={active === filter.key}
      onclick={() => onselect?.(filter.key)}
    >
      <span>{filter.label}</span>
      <span class="filter__count">{filter.count}</span>
    </button>
  {/each}

  <div class="sidepanel__title">
    tags
    <span class="sidepanel__title-rule"></span>
  </div>

  <div class="tag-cloud">
    {#if tags.length > 0}
      {#each tags as tag}
        <Tag variant="slate" value={tag} bracketed={false} />
      {/each}
    {:else}
      <Tag variant="slate" value="none" bracketed={false} />
    {/if}
  </div>
</aside>
