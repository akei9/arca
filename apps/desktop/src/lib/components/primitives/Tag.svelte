<script lang="ts">
  import type { Snippet } from 'svelte';

  type TagVariant = 'default' | 'vault' | 'slate' | 'out' | 'ink' | 'paper';

  let {
    variant = 'default',
    value,
    bracketed = true,
    class: className = '',
    children,
    ...rest
  } = $props<{
    variant?: TagVariant;
    value?: string;
    bracketed?: boolean;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  }>();

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
