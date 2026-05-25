<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    value,
    class: className = '',
    children,
    ...rest
  } = $props<{
    value?: string;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  }>();

  const classes = $derived(['kbd', className].filter(Boolean).join(' '));
</script>

<kbd {...rest} class={classes}>
  {#if value}
    {value}
  {:else if children}
    {@render children()}
  {/if}
</kbd>
