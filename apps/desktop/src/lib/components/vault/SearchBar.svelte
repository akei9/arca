<script lang="ts">
  import { primaryModifierLabel } from '../../keyboard';

  const searchShortcut = primaryModifierLabel() === '⌘' ? '⌘F' : 'Ctrl+F';

  let {
    query = '',
    focused = false,
    focusToken = 0,
    onquery,
    onfocus,
    onblur,
    onclear,
    shortcut = searchShortcut,
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
    shortcut?: string;
    class?: string;
    [key: string]: unknown;
  }>();

  let inputElement = $state<HTMLInputElement | null>(null);
  const classes = $derived(['search', focused ? 'search--focus' : '', className].filter(Boolean).join(' '));

  $effect(() => {
    if (focused) {
      inputElement?.focus();
    }
  });

  $effect(() => {
    if (focusToken > 0) {
      inputElement?.focus();
    }
  });
</script>

<div
  {...rest}
  class={classes}
  onclick={(event) => {
    const target = event.target;

    if (target instanceof Element && target.closest('button')) {
      return;
    }

    inputElement?.focus();
  }}
>
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
    <button
      type="button"
      class="search__clear"
      onclick={(event) => {
        event.stopPropagation();
        onclear?.();
      }}
      aria-label="Clear query"
    >
      clear
    </button>
  {/if}
  <span class="search__hint">{shortcut}</span>
</div>
