<script lang="ts">
  let {
    query = '',
    focused = false,
    focusToken = 0,
    onquery,
    onfocus,
    onblur,
    onclear,
    class: className = '',
    ...rest
  } = $props<{
    query?: string;
    focused?: boolean;
    focusToken?: number;
    onquery?: (query: string) => void;
    onfocus?: () => void;
    onblur?: () => void;
    onclear?: () => void;
    class?: string;
    [key: string]: unknown;
  }>();

  let inputElement = $state<HTMLInputElement | null>(null);
  const classes = $derived(['search', focused ? 'search--focus' : '', className].filter(Boolean).join(' '));

  $effect(() => {
    if (focused || focusToken > 0) {
      inputElement?.focus();
    }
  });
</script>

<div {...rest} class={classes}>
  <span class="search__prompt">&gt;</span>
  <input
    bind:this={inputElement}
    class="search__input"
    aria-label="query vault"
    autocomplete="off"
    spellcheck="false"
    value={query}
    placeholder="query_vault"
    oninput={(event) => onquery?.(event.currentTarget.value)}
    onfocus={onfocus}
    onblur={onblur}
  />
  {#if query.trim()}
    <button type="button" class="search__clear" onclick={() => onclear?.()} aria-label="Clear query">clear</button>
  {/if}
  <span class="search__hint">⌘F</span>
</div>
