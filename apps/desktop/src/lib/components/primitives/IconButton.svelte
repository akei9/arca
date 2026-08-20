<script lang="ts">
  import type { Snippet } from 'svelte';

  type IconButtonVariant = 'default' | 'accent' | 'vault' | 'ghost';
  type ButtonType = 'button' | 'submit' | 'reset';

  interface Props {
    variant?: IconButtonVariant;
    label: string;
    title?: string;
    type?: ButtonType;
    disabled?: boolean;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  }

  let {
    variant = 'default',
    label,
    title,
    type = 'button',
    disabled = false,
    class: className = '',
    children,
    ...rest
  }: Props = $props();

  const classes = $derived(
    ['iconbtn', variant !== 'default' ? `iconbtn--${variant}` : '', className]
      .filter(Boolean)
      .join(' '),
  );
</script>

<button {...rest} class={classes} aria-label={label} title={title ?? label} {type} {disabled}>
  {#if children}
    {@render children()}
  {/if}
</button>
