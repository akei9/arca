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

  let tablistElement = $state<HTMLDivElement | null>(null);

  function handleKeydown(event: KeyboardEvent) {
    const enabledItems: TabItem[] = items.filter((item: TabItem) => !item.disabled);

    if (enabledItems.length === 0) {
      return;
    }

    const currentIndex = Math.max(0, enabledItems.findIndex((item) => item.key === active));
    let nextIndex = currentIndex;

    if (event.key === 'ArrowRight') {
      nextIndex = (currentIndex + 1) % enabledItems.length;
    } else if (event.key === 'ArrowLeft') {
      nextIndex = (currentIndex + enabledItems.length - 1) % enabledItems.length;
    } else if (event.key === 'Home') {
      nextIndex = 0;
    } else if (event.key === 'End') {
      nextIndex = enabledItems.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = enabledItems[nextIndex];
    onselect?.(next.key);
    focusTab(next.key);
  }

  function focusTab(key: string) {
    window.requestAnimationFrame(() => {
      tablistElement?.querySelector<HTMLButtonElement>(`button[data-tab-key="${key}"]`)?.focus();
    });
  }
</script>

<div bind:this={tablistElement} {...rest} class={classes} role="tablist" aria-label={label} onkeydown={handleKeydown}>
  {#each items as item (item.key)}
    <button
      type="button"
      role="tab"
      data-tab-key={item.key}
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
