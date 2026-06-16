<script lang="ts">
  export interface SegmentedOption {
    value: string;
    label: string;
    disabled?: boolean;
  }

  let {
    options = [],
    value,
    ariaLabel,
    onselect,
    class: className = '',
    ...rest
  } = $props<{
    options?: SegmentedOption[];
    value: string;
    ariaLabel?: string;
    onselect?: (value: string) => void;
    class?: string;
    [key: string]: unknown;
  }>();

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
