<script lang="ts">
  import type { Snippet } from 'svelte';

  type PillKind = 'vault' | 'ink' | 'slate' | 'ghost';

  let {
    pill = 'SEALED',
    pillKind = 'vault',
    leftText = '',
    connected = true,
    connectionLabel,
    left,
    right,
    class: className = '',
    ...rest
  } = $props<{
    pill?: string;
    pillKind?: PillKind;
    leftText?: string;
    connected?: boolean;
    connectionLabel?: string;
    left?: Snippet;
    right?: Snippet;
    class?: string;
    [key: string]: unknown;
  }>();

  const classes = $derived(['status', className].filter(Boolean).join(' '));
  const pillClasses = $derived(['status__pill', `status__pill--${pillKind}`].join(' '));
  const dotClasses = $derived(['status__dot', connected ? '' : 'status__dot--warn'].filter(Boolean).join(' '));
</script>

<div {...rest} class={classes}>
  <span class={pillClasses}>{pill}</span>
  {#if left}
    {@render left()}
  {:else if leftText}
    <span>{leftText}</span>
  {/if}
  <span class="status__spacer"></span>
  {#if right}
    {@render right()}
  {:else}
    <span class="status__pill status__pill--ghost">UTF-8</span>
    <span class="status__pill status__pill--ghost">aes-512</span>
    <span><span class={dotClasses}></span>{connectionLabel ?? (connected ? 'online' : 'offline')}</span>
  {/if}
</div>
