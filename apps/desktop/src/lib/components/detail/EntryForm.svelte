<script lang="ts">
  import { onMount } from 'svelte';
  import { createEntry, generatePassword, updateEntry, type EntryDto } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, Entropy, IconButton, Kbd } from '../primitives';

  const editingEntry = $derived(vaultState.selectedEntry);
  const mode = $derived(editingEntry ? 'edit' : 'create');
  const titleText = $derived(mode === 'edit' ? 'edit_entry' : 'new_entry');
  const submitText = $derived(mode === 'edit' ? 'save_changes' : 'save_entry');
  const baseCollections = ['work', 'personal', 'infrastructure', 'shared', 'archive'] as const;

  let title = $state('');
  let username = $state('');
  let password = $state('');
  let collection = $state('work');
  let url = $state('');
  let notes = $state('');
  let tags = $state('');
  let tagInput = $state('');
  let busy = $state(false);
  let errorMessage = $state('');
  let loadedEntryId = $state<string | null>(null);
  let loadedPassword = $state('');
  let revealPassword = $state(false);
  let newCollectionOpen = $state(false);
  let newCollectionValue = $state('');
  let generatingPassword = $state(false);

  const passwordRequired = $derived(mode === 'create');
  const canSubmit = $derived(title.trim().length > 0 && (!passwordRequired || password.length > 0) && !busy);
  const collectionOptions = $derived(deriveCollectionOptions(vaultState.entries, collection));
  const normalizedTags = $derived(parseTags(tags));
  const suggestedTags = $derived(deriveSuggestedTags(vaultState.entries, normalizedTags));
  const entropyBits = $derived(estimateEntropyBits(password));
  const entropyFilled = $derived(password ? Math.max(1, Math.min(16, Math.round(entropyBits / 8))) : 0);
  const entropyStrength = $derived(entropyBits >= 90 ? 'strong' : entropyBits >= 60 ? 'medium' : 'weak');

  $effect(() => {
    const entry = editingEntry;
    const entryId = entry?.id ?? null;

    if (loadedEntryId === entryId) {
      return;
    }

    loadedEntryId = entryId;
    title = entry?.title ?? '';
    username = entry?.username ?? '';
    password = entry?.password ?? '';
    loadedPassword = password;
    collection = entry?.collection ?? 'work';
    url = entry?.url ?? '';
    notes = entry?.notes ?? '';
    tags = entry?.tags.join(', ') ?? '';
    tagInput = '';
    revealPassword = false;
    newCollectionOpen = false;
    newCollectionValue = '';
    errorMessage = '';
  });

  onMount(() => {
    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        cancel();
        return;
      }

      if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
        event.preventDefault();
        void submit();
      }
    }

    window.addEventListener('keydown', handleKeydown);

    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });

  async function submit() {
    if (!canSubmit) {
      return;
    }

    busy = true;
    errorMessage = '';

    try {
      const saved =
        mode === 'edit' && editingEntry
          ? await updateEntry(editingEntry.id, updatePayload())
          : await createEntry({
              title: title.trim(),
              username: username.trim(),
              password,
              collection: optionalText(collection),
              url: optionalText(url),
              notes: optionalText(notes),
              tags: normalizedTags,
            });

      applySavedEntry(saved);
      vaultState.lastSaved = new Date(saved.updatedAt);
      uiState.view = 'detail';
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      busy = false;
    }
  }

  function applySavedEntry(entry: EntryDto) {
    const existingIndex = vaultState.entries.findIndex((item) => item.id === entry.id);

    if (existingIndex === -1) {
      vaultState.entries = [entry, ...vaultState.entries];
    } else {
      vaultState.entries = vaultState.entries.map((item) => (item.id === entry.id ? entry : item));
    }

    vaultState.selectedEntry = entry;
  }

  function cancel() {
    uiState.view = editingEntry ? 'detail' : 'list';
  }

  function optionalText(value: string): string | null {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
  }

  function updatePayload() {
    return {
      title: title.trim(),
      username: username.trim(),
      collection: optionalText(collection),
      url: optionalText(url),
      notes: optionalText(notes),
      tags: normalizedTags,
      ...(password.length > 0 && password !== loadedPassword ? { password } : {}),
    };
  }

  async function generateEntryPassword() {
    generatingPassword = true;
    errorMessage = '';

    try {
      const generated = await generatePassword({
        length: 20,
        uppercase: true,
        lowercase: true,
        digits: true,
        symbols: true,
        excludeAmbiguous: true,
      });

      password = generated.password;
      revealPassword = true;
    } catch (error) {
      errorMessage = messageFromError(error);
    } finally {
      generatingPassword = false;
    }
  }

  function togglePasswordReveal() {
    if (!password) {
      return;
    }

    revealPassword = !revealPassword;
  }

  function addCollection() {
    const nextCollection = normalizeCollection(newCollectionValue);

    if (!nextCollection) {
      newCollectionOpen = false;
      newCollectionValue = '';
      return;
    }

    collection = nextCollection;
    newCollectionOpen = false;
    newCollectionValue = '';
  }

  function addTag(raw = tagInput) {
    const nextTag = normalizeTag(raw);

    if (!nextTag) {
      tagInput = '';
      return;
    }

    const nextTags = normalizedTags.includes(nextTag) ? normalizedTags : [...normalizedTags, nextTag];
    tags = nextTags.join(', ');
    tagInput = '';
  }

  function removeTag(tag: string) {
    tags = normalizedTags.filter((existingTag) => existingTag !== tag).join(', ');
  }

  function parseTags(value: string): string[] {
    const seen = new Set<string>();

    return value
      .split(',')
      .map(normalizeTag)
      .filter((tag) => {
        if (!tag || seen.has(tag)) {
          return false;
        }

        seen.add(tag);
        return true;
      });
  }

  function deriveSuggestedTags(entries: EntryDto[], selectedTags: string[]): string[] {
    const counts = new Map<string, number>();

    for (const entry of entries) {
      for (const tag of entry.tags) {
        const normalizedTag = normalizeTag(tag);

        if (!normalizedTag || selectedTags.includes(normalizedTag)) {
          continue;
        }

        counts.set(normalizedTag, (counts.get(normalizedTag) ?? 0) + 1);
      }
    }

    return [...counts.entries()]
      .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
      .slice(0, 8)
      .map(([tag]) => tag);
  }

  function deriveCollectionOptions(entries: EntryDto[], currentCollection: string): string[] {
    const seen = new Set<string>();

    for (const option of baseCollections) {
      seen.add(option);
    }

    for (const entry of entries) {
      const normalizedCollection = normalizeCollection(entry.collection ?? '');

      if (normalizedCollection) {
        seen.add(normalizedCollection);
      }
    }

    const normalizedCurrent = normalizeCollection(currentCollection);

    if (normalizedCurrent) {
      seen.add(normalizedCurrent);
    }

    return [...seen];
  }

  function estimateEntropyBits(value: string): number {
    if (!value) {
      return 0;
    }

    let pool = 0;

    if (/[a-z]/.test(value)) {
      pool += 26;
    }

    if (/[A-Z]/.test(value)) {
      pool += 26;
    }

    if (/[0-9]/.test(value)) {
      pool += 10;
    }

    if (/[^a-zA-Z0-9]/.test(value)) {
      pool += 24;
    }

    return Math.round(value.length * Math.log2(pool || 26));
  }

  function normalizeCollection(value: string): string {
    return value.trim().toLowerCase().replace(/\s+/g, '-');
  }

  function normalizeTag(value: string): string {
    return value.trim().toLowerCase().replace(/\s+/g, '-');
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to save entry';
  }
</script>

<section class="entry-compose" aria-labelledby="entry-form-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="entry-form-title" class="page__title">{titleText}<em>.</em></h1>
    <div class="page__meta mono">
      <span>vault · <b>{vaultState.vaultName || 'vault.local'}</b></span>
      <span>{mode === 'edit' ? 'editing' : 'encrypted'} · <b>{mode === 'edit' ? editingEntry?.id ?? 'selected' : 'on save'}</b></span>
    </div>
  </div>

  <form
    class="form"
    onsubmit={(event) => {
      event.preventDefault();
      void submit();
    }}
  >
    <div class="form__panel">
      <label class="form-row">
        <span class="form-row__k">name</span>
        <input class="finput mono" bind:value={title} autocomplete="off" maxlength="120" placeholder="e.g. cloudflare" required />
      </label>

      <div class="form-row">
        <div class="form-row__k">category</div>
        <div class="cat-select">
          {#each collectionOptions as option}
            <button
              type="button"
              class={collection === option ? 'cat-opt cat-opt--on mono' : 'cat-opt mono'}
              onclick={() => {
                collection = option;
                newCollectionOpen = false;
                newCollectionValue = '';
              }}
            >
              {option}
            </button>
          {/each}

          {#if !newCollectionOpen}
            <button type="button" class="cat-opt cat-opt--new mono" onclick={() => (newCollectionOpen = true)}>+ new</button>
          {:else}
            <span class="cat-newwrap">
              <input
                class="finput mono"
                bind:value={newCollectionValue}
                placeholder="collection name"
                onkeydown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    addCollection();
                  }

                  if (event.key === 'Escape') {
                    event.preventDefault();
                    newCollectionOpen = false;
                    newCollectionValue = '';
                  }
                }}
              />
              <Button variant="ghost" size="xs" onclick={addCollection}>add</Button>
            </span>
          {/if}
        </div>
      </div>

      <label class="form-row">
        <span class="form-row__k">url</span>
        <input class="finput mono" bind:value={url} autocomplete="url" inputmode="url" maxlength="400" placeholder="https://" />
      </label>

      <label class="form-row">
        <span class="form-row__k">username</span>
        <input class="finput mono" bind:value={username} autocomplete="username" maxlength="160" placeholder="user or email" />
      </label>

      <div class="form-row">
        <div class="form-row__k">password</div>
        <div>
          <div class="finput-row">
            <input
              class="finput mono"
              bind:value={password}
              autocomplete="new-password"
              required={passwordRequired}
              type={revealPassword ? 'text' : 'password'}
              placeholder="type or generate"
            />
            <IconButton
              label={revealPassword ? 'Hide entry password' : 'Reveal entry password'}
              onclick={togglePasswordReveal}
              disabled={!password}
            >
              <Icon name="eye" size={13} />
            </IconButton>
            <Button variant="ghost" size="sm" onclick={generateEntryPassword} disabled={generatingPassword}>
              <Icon name="refresh" size={12} />
              {generatingPassword ? 'generating' : 'generate'}
            </Button>
          </div>

          {#if password}
            <div class="form-entropy">
              <Entropy filled={entropyFilled} bits={entropyBits} strength={entropyStrength} />
            </div>
          {/if}
        </div>
      </div>
    </div>

    <div class="form__panel">
      <div class="form-row">
        <div class="form-row__k">tags</div>
        <div>
          <div class="tagedit">
            {#each normalizedTags as tag (tag)}
              <span class="tag-chip mono">
                #{tag}
                <button type="button" class="tag-chip__x" aria-label={`Remove ${tag}`} onclick={() => removeTag(tag)}>✕</button>
              </span>
            {/each}
            <input
              class="tag-add mono"
              bind:value={tagInput}
              placeholder={normalizedTags.length > 0 ? 'add another…' : 'add a tag, press enter'}
              onkeydown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  addTag();
                } else if (event.key === 'Backspace' && tagInput.length === 0 && normalizedTags.length > 0) {
                  event.preventDefault();
                  removeTag(normalizedTags[normalizedTags.length - 1]);
                }
              }}
            />
          </div>

          {#if suggestedTags.length > 0}
            <div class="tag-sugg">
              <span class="tag-sugg__k mono">suggested</span>
              {#each suggestedTags as tag (tag)}
                <button type="button" class="tag-sugg__item mono" onclick={() => addTag(tag)}>#{tag}</button>
              {/each}
            </div>
          {/if}

          <div class="form-hint mono">tags slice across categories — once saved, filter the vault by any tag from the sidebar.</div>
        </div>
      </div>

      <label class="form-row">
        <span class="form-row__k">notes</span>
        <textarea
          class="ftextarea mono"
          bind:value={notes}
          rows="8"
          placeholder="optional · plain text, encrypted with the entry"
        ></textarea>
      </label>
    </div>

    {#if errorMessage}
      <div class="entry-form__error mono" role="alert">{errorMessage}</div>
    {/if}

    <div class="form__actions">
      <Button variant="bare" onclick={cancel} disabled={busy}>
        cancel
        <Kbd value="esc" />
      </Button>
      <span class="form__spacer"></span>
      <Button variant="primary" type="submit" disabled={!canSubmit}>
        <Icon name={mode === 'edit' ? 'edit' : 'plus'} size={12} />
        {busy ? 'saving' : submitText}
        <Kbd value="⌘↵" />
      </Button>
    </div>
  </form>
</section>
