<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Lockup } from '../brand';

  let {
    path = 'vault.local',
    prefix = './',
    rightText = 'v0.2 · 2026',
    right,
    class: className = '',
    ...rest
  } = $props<{
    path?: string;
    prefix?: string;
    rightText?: string;
    right?: Snippet;
    class?: string;
    [key: string]: unknown;
  }>();

  const classes = $derived(['chrome', className].filter(Boolean).join(' '));
</script>

<div {...rest} class={classes} data-tauri-drag-region="deep">
  <span class="chrome__lockup">
    <Lockup size={18} />
  </span>
  <span class="chrome__divider"></span>
  <div class="chrome__path mono">{prefix}<b>{path}</b></div>
  <div class="chrome__right">
    {#if right}
      {@render right()}
    {:else}
      {rightText}
    {/if}
  </div>
</div>
