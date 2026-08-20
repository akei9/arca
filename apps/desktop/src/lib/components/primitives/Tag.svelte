<script lang="ts">
  import type { Snippet } from 'svelte';

  type TagVariant = 'default' | 'vault' | 'slate' | 'out' | 'ink' | 'paper';

  interface Props {
    variant?: TagVariant;
    value?: string;
    bracketed?: boolean;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  }

  let {
    variant = 'default',
    value,
    bracketed = true,
    class: className = '',
    children,
    ...rest
  }: Props = $props();

  const classes = $derived(
    ['tag', variant !== 'default' ? `tag--${variant}` : '', className].filter(Boolean).join(' '),
  );
</script>

<span {...rest} class={classes}>
  {#if value}
    {bracketed ? `[${value}]` : value}
  {:else if children}
    {@render children()}
  {/if}
</span>
