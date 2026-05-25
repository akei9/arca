<script lang="ts">
  export interface TabItem {
    key: string;
    label: string;
    count?: number | null;
    disabled?: boolean;
  }

  let {
    items = [],
    active,
    onselect,
    label = 'Workspace sections',
    class: className = '',
    ...rest
  } = $props<{
    items?: TabItem[];
    active?: string;
    onselect?: (key: string) => void;
    label?: string;
    class?: string;
    [key: string]: unknown;
  }>();

  const classes = $derived(['tabs', className].filter(Boolean).join(' '));
</script>

<div {...rest} class={classes} role="tablist" aria-label={label}>
  {#each items as item (item.key)}
    <button
      type="button"
      role="tab"
      class={active === item.key ? 'tab tab--active' : 'tab'}
      aria-selected={active === item.key}
      disabled={item.disabled}
      onclick={() => onselect?.(item.key)}
    >
      {item.label}
      {#if item.count !== undefined && item.count !== null}
        <span class="tab__count">[{item.count}]</span>
      {/if}
    </button>
  {/each}
</div>
