<script lang="ts">
  export type IconName =
    | 'at'
    | 'bank'
    | 'box'
    | 'cloud'
    | 'code'
    | 'copy'
    | 'eye'
    | 'key'
    | 'plus'
    | 'refresh'
    | 'server'
    | 'vault';

  let {
    name,
    size = 16,
    sw = 1.5,
    class: className = '',
    ...rest
  } = $props<{
    name: IconName;
    size?: number;
    sw?: number;
    class?: string;
    [key: string]: unknown;
  }>();
</script>

<svg
  {...rest}
  class={className}
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width={sw}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'copy'}
    <rect x="5" y="5" width="8" height="9" rx="1" />
    <path d="M3 11V3a1 1 0 0 1 1-1h7" />
  {:else if name === 'eye'}
    <path d="M1.5 8s2.5-5 6.5-5 6.5 5 6.5 5-2.5 5-6.5 5S1.5 8 1.5 8Z" />
    <circle cx="8" cy="8" r="2" />
  {:else if name === 'plus'}
    <path d="M8 3v10M3 8h10" />
  {:else if name === 'refresh'}
    <path d="M13 4v3h-3M3 12V9h3" />
    <path d="M12.5 7A5 5 0 0 0 3.5 6M3.5 9a5 5 0 0 0 9 1" />
  {:else if name === 'code'}
    <path d="m5 5-3 3 3 3M11 5l3 3-3 3M9 3 7 13" />
  {:else if name === 'cloud'}
    <path d="M4.5 13a3 3 0 0 1-.4-6 4 4 0 0 1 7.6.5 2.5 2.5 0 0 1-.7 4.9 2.5 2.5 0 0 1-.5.05Z" />
  {:else if name === 'bank'}
    <path d="M2 6 8 3l6 3M3 6v.5M13 6v.5M2.5 14h11M4 7v6M7 7v6M9 7v6M12 7v6" />
  {:else if name === 'at'}
    <circle cx="8" cy="8" r="2.5" />
    <path d="M10.5 8v1.5a1.5 1.5 0 0 0 3 0V8a5.5 5.5 0 1 0-2 4.3" />
  {:else if name === 'server'}
    <rect x="2.5" y="3" width="11" height="4" rx="1" />
    <rect x="2.5" y="9" width="11" height="4" rx="1" />
    <path d="M5 5h.01M5 11h.01M8 5h.01M8 11h.01" />
  {:else if name === 'box'}
    <path d="M2 5.5 8 3l6 2.5M2 5.5v6L8 14M2 5.5 8 8m0 0v6m0-6 6-2.5m0 0v6L8 14" />
  {:else if name === 'key'}
    <circle cx="5" cy="11" r="2.5" />
    <path d="M7 9.5 13 3.5M11 5l1.5 1.5M10 6.5l1.5 1.5" />
  {:else}
    <rect x="2.5" y="3" width="11" height="10" rx="1" />
    <circle cx="8.5" cy="8" r="2" />
    <path d="M8.5 6V5M8.5 11v-1M10.5 8h1M6.5 8h-1" />
  {/if}
</svg>
