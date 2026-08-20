<script lang="ts">
  interface Props {
    checked?: boolean;
    variant?: 'default' | 'vault';
    label: string;
    ontoggle?: (checked: boolean) => void;
    class?: string;
    [key: string]: unknown;
  }

  let {
    checked = false,
    variant = 'default',
    label,
    ontoggle,
    class: className = '',
    ...rest
  }: Props = $props();

  const classes = $derived(
    ['switch', checked ? 'switch--on' : '', variant === 'vault' ? 'switch--vault' : '', className]
      .filter(Boolean)
      .join(' '),
  );
</script>

<button
  {...rest}
  type="button"
  class={classes}
  role="switch"
  aria-label={label}
  aria-checked={checked}
  onclick={() => ontoggle?.(!checked)}
></button>
