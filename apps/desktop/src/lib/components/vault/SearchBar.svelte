<script lang="ts">
  let {
    query = '',
    focused = false,
    onquery,
    onfocus,
    onblur,
    class: className = '',
    ...rest
  } = $props<{
    query?: string;
    focused?: boolean;
    onquery?: (query: string) => void;
    onfocus?: () => void;
    onblur?: () => void;
    class?: string;
    [key: string]: unknown;
  }>();

  const classes = $derived(['search', focused ? 'search--focus' : '', className].filter(Boolean).join(' '));
</script>

<label {...rest} class={classes}>
  <span class="search__prompt">&gt;</span>
  <input
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
  {#if focused}
    <span class="search__caret" aria-hidden="true"></span>
  {/if}
  <span class="search__hint">⌘F</span>
</label>
