<script lang="ts">
  import type { EntryDto } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, IconButton, Tag } from '../primitives';
  import FieldStack from './FieldStack.svelte';
  import MetricStrip from './MetricStrip.svelte';
  import NotePanel from './NotePanel.svelte';

  const entry = $derived(vaultState.selectedEntry ?? vaultState.entries[0] ?? null);

  function backToList() {
    uiState.view = 'list';
  }

  function editEntry() {
    uiState.view = 'edit';
  }

  function modified(entry: EntryDto): string {
    const time = Date.parse(entry.updatedAt);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    return new Date(time).toISOString().slice(0, 16).replace('T', ' ');
  }
</script>

{#if entry}
  <section class="entry-detail" aria-labelledby="entry-detail-title">
    <div class="detail-head">
      <div class="detail__title-wrap">
        <div class="detail__crumbs mono">
          <button type="button" class="detail__crumb-link" onclick={backToList}>vault</button>
          &nbsp;/&nbsp; recent &nbsp;/&nbsp; <b>{entry.title}</b>
        </div>
        <h1 id="entry-detail-title" class="detail__title">{entry.title}<em>.</em></h1>
        <div class="detail__meta mono">
          <Tag value="⌘1" />
          <span>id · <b>{entry.id}</b></span>
          <span>modified · <b>{modified(entry)}</b></span>
          <span>auth · <b>password</b></span>
        </div>
      </div>
      <div class="detail__actions">
        <Button variant="ghost" size="sm" onclick={editEntry}>
          <Icon name="edit" size={12} />
          edit
        </Button>
        <Button variant="ghost" size="sm">
          <Icon name="share" size={12} />
          share
        </Button>
        <IconButton label={`Delete ${entry.title}`}>
          <Icon name="trash" size={13} />
        </IconButton>
      </div>
    </div>

    <div class="detail-body">
      <NotePanel notes={entry.notes} tags={entry.tags} />
      <FieldStack {entry} />
      <MetricStrip {entry} />
    </div>
  </section>
{:else}
  <section class="vault-placeholder" aria-labelledby="entry-detail-empty-title">
    <p class="vault-placeholder__eyebrow">detail</p>
    <h1 id="entry-detail-empty-title">no_entry</h1>
    <p>select_entry_from_vault</p>
    <Button variant="ghost" onclick={backToList}>back_to_vault</Button>
  </section>
{/if}
