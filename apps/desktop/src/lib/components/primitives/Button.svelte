<script lang="ts">
  import type { Snippet } from 'svelte';

  type ButtonVariant = 'primary' | 'vault' | 'ghost' | 'bare' | 'danger';
  type ButtonSize = 'default' | 'sm' | 'xs';
  type ButtonType = 'button' | 'submit' | 'reset';

  let {
    variant = 'ghost',
    size = 'default',
    type = 'button',
    disabled = false,
    class: className = '',
    children,
    ...rest
  } = $props<{
    variant?: ButtonVariant;
    size?: ButtonSize;
    type?: ButtonType;
    disabled?: boolean;
    class?: string;
    children?: Snippet;
    [key: string]: unknown;
  }>();

  const classes = $derived(
    ['btn', `btn--${variant}`, size !== 'default' ? `btn--${size}` : '', className]
      .filter(Boolean)
      .join(' '),
  );
</script>

<button {...rest} class={classes} {type} {disabled}>
  {#if children}
    {@render children()}
  {/if}
</button>
