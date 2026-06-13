<script lang="ts">
  import { createEntry, updateEntry, type EntryDto } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, Tag } from '../primitives';

  const editingEntry = $derived(vaultState.selectedEntry);
  const mode = $derived(editingEntry ? 'edit' : 'create');
  const titleText = $derived(mode === 'edit' ? 'edit_entry' : 'new_entry');
  const submitText = $derived(mode === 'edit' ? 'save_changes' : 'create_entry');

  let title = $state('');
  let username = $state('');
  let password = $state('');
  let url = $state('');
  let notes = $state('');
  let tags = $state('');
  let busy = $state(false);
  let errorMessage = $state('');
  let loadedEntryId = $state<string | null>(null);

  const canSubmit = $derived(title.trim().length > 0 && password.length > 0 && !busy);

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
    url = entry?.url ?? '';
    notes = entry?.notes ?? '';
    tags = entry?.tags.join(', ') ?? '';
    errorMessage = '';
  });

  async function submit() {
    if (!canSubmit) {
      return;
    }

    busy = true;
    errorMessage = '';

    try {
      const payload = {
        title: title.trim(),
        username: username.trim(),
        password,
        url: optionalText(url),
        notes: optionalText(notes),
        tags: parseTags(tags),
      };
      const saved =
        mode === 'edit' && editingEntry
          ? await updateEntry(editingEntry.id, payload)
          : await createEntry(payload);

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

  function parseTags(value: string): string[] {
    const seen = new Set<string>();

    return value
      .split(',')
      .map((tag) => tag.trim())
      .filter((tag) => {
        const key = tag.toLowerCase();

        if (!key || seen.has(key)) {
          return false;
        }

        seen.add(key);
        return true;
      });
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to save entry';
  }
</script>

<section class="entry-form" aria-labelledby="entry-form-title">
  <div class="detail-head">
    <div class="detail__title-wrap">
      <div class="detail__crumbs mono">
        <button type="button" class="detail__crumb-link" onclick={cancel}>vault</button>
        &nbsp;/&nbsp; <b>{titleText}</b>
      </div>
      <h1 id="entry-form-title" class="detail__title">{titleText}<em>.</em></h1>
      <div class="detail__meta mono">
        <Tag value={mode} />
        <span>vault · <b>{vaultState.vaultName || 'local'}</b></span>
        <span>fields · <b>password</b></span>
      </div>
    </div>
    <div class="detail__actions">
      <Button variant="ghost" size="sm" onclick={cancel} disabled={busy}>cancel</Button>
    </div>
  </div>

  <form
    class="entry-form__body"
    onsubmit={(event) => {
      event.preventDefault();
      submit();
    }}
  >
    <div class="entry-form__grid">
      <label class="entry-form__field entry-form__field--wide">
        <span>title</span>
        <input bind:value={title} autocomplete="off" maxlength="120" required />
      </label>

      <label class="entry-form__field">
        <span>username</span>
        <input bind:value={username} autocomplete="username" maxlength="160" />
      </label>

      <label class="entry-form__field">
        <span>password</span>
        <input bind:value={password} autocomplete="new-password" required type="password" />
      </label>

      <label class="entry-form__field entry-form__field--wide">
        <span>url</span>
        <input bind:value={url} autocomplete="url" inputmode="url" maxlength="400" />
      </label>

      <label class="entry-form__field entry-form__field--wide">
        <span>tags</span>
        <input bind:value={tags} autocomplete="off" maxlength="240" />
      </label>

      <label class="entry-form__field entry-form__field--wide">
        <span>notes</span>
        <textarea bind:value={notes} rows="7"></textarea>
      </label>
    </div>

    {#if errorMessage}
      <div class="entry-form__error mono" role="alert">{errorMessage}</div>
    {/if}

    <div class="entry-form__footer">
      <div class="entry-form__hint mono">
        required · <b>title password</b>
      </div>
      <Button variant="primary" type="submit" disabled={!canSubmit}>
        <Icon name={mode === 'edit' ? 'edit' : 'plus'} size={12} />
        {busy ? 'saving' : submitText}
      </Button>
    </div>
  </form>
</section>
