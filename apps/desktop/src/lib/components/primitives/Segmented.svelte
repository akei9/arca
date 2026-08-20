<script lang="ts">
  export interface SegmentedOption {
    value: string;
    label: string;
    disabled?: boolean;
  }

  interface Props {
    options?: SegmentedOption[];
    value: string;
    ariaLabel?: string;
    onselect?: (value: string) => void;
    class?: string;
    [key: string]: unknown;
  }

  let {
    options = [],
    value,
    ariaLabel,
    onselect,
    class: className = '',
    ...rest
  }: Props = $props();

  const classes = $derived(['seg', className].filter(Boolean).join(' '));
</script>

<div {...rest} class={classes} role="group" aria-label={ariaLabel}>
  {#each options as option (option.value)}
    <button
      type="button"
      class={value === option.value ? 'seg__opt seg__opt--active' : 'seg__opt'}
      aria-pressed={value === option.value}
      disabled={option.disabled}
      onclick={() => onselect?.(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>
