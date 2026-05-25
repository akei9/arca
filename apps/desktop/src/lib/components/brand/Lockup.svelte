<script lang="ts">
  import Lettermark from './Lettermark.svelte';

  let {
    size = 22,
    color,
    accent = false,
    gap,
    ariaLabel = 'arca',
    class: className = '',
    ...rest
  } = $props<{
    size?: number;
    color?: string;
    accent?: boolean;
    gap?: number;
    ariaLabel?: string;
    class?: string;
    [key: string]: unknown;
  }>();

  const wordColor = $derived(color ?? (accent ? 'var(--accent)' : 'var(--text)'));
  const lockupGap = $derived(gap ?? Math.max(4, size * 0.22));
  const classes = $derived(['lockup', className].filter(Boolean).join(' '));
</script>

<span
  {...rest}
  class={classes}
  aria-label={ariaLabel}
  style:gap={`${lockupGap}px`}
  style:color={wordColor}
>
  <Lettermark size={size * 0.82} color="currentColor" />
  <span class="lockup__word" style:font-size={`${size}px`}>arca</span>
</span>
