<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    value?: string;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  }

  let {
    value,
    class: className = '',
    children,
    ...rest
  }: Props = $props();

  const classes = $derived(['kbd', className].filter(Boolean).join(' '));
</script>

<kbd {...rest} class={classes}>
  {#if value}
    {value}
  {:else if children}
    {@render children()}
  {/if}
</kbd>
