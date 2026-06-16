<script lang="ts">
  import { Tag } from '../primitives';

  export interface FilterItem {
    key: string;
    label: string;
    count: number;
  }

  export interface TagItem {
    value: string;
    count: number;
  }

  let {
    filters = [],
    tags = [],
    active = 'all',
    activeTag = null,
    onselect,
    ontagselect,
    onclear,
  } = $props<{
    filters?: FilterItem[];
    tags?: TagItem[];
    active?: string;
    activeTag?: string | null;
    onselect?: (key: string) => void;
    ontagselect?: (tag: string) => void;
    onclear?: () => void;
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
      {#each tags as tag (tag.value)}
        <button
          type="button"
          class={activeTag === tag.value ? 'tag-filter tag-filter--active' : 'tag-filter'}
          aria-pressed={activeTag === tag.value}
          onclick={() => ontagselect?.(tag.value)}
        >
          <Tag variant={activeTag === tag.value ? 'default' : 'slate'} value={tag.value} bracketed={false} />
          <span class="tag-filter__count mono">{tag.count}</span>
        </button>
      {/each}
    {:else}
      <Tag variant="slate" value="none" bracketed={false} />
    {/if}
  </div>

  {#if active !== 'all' || activeTag}
    <button type="button" class="sidepanel__clear mono" onclick={() => onclear?.()}>
      clear_filters
    </button>
  {/if}
</aside>
