<script lang="ts">
  import { deleteEntry, type EntryDto } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, IconButton, Tag } from '../primitives';
  import FieldStack from './FieldStack.svelte';
  import MetricStrip from './MetricStrip.svelte';
  import NotePanel from './NotePanel.svelte';

  const entry = $derived(vaultState.selectedEntry ?? vaultState.entries[0] ?? null);
  let confirmDeleteOpen = $state(false);
  let deleteBusy = $state(false);
  let deleteError = $state('');

  function backToList() {
    uiState.view = 'list';
  }

  function editEntry() {
    uiState.view = 'edit';
  }

  function requestDelete() {
    confirmDeleteOpen = true;
    deleteError = '';
  }

  function cancelDelete() {
    confirmDeleteOpen = false;
    deleteError = '';
  }

  async function confirmDelete(entry: EntryDto) {
    deleteBusy = true;
    deleteError = '';

    try {
      await deleteEntry(entry.id);
      vaultState.entries = vaultState.entries.filter((item) => item.id !== entry.id);

      if (vaultState.selectedEntry?.id === entry.id) {
        vaultState.selectedEntry = null;
      }

      vaultState.lastSaved = new Date();
      uiState.view = 'list';
    } catch (error) {
      deleteError = messageFromError(error);
    } finally {
      deleteBusy = false;
    }
  }

  function modified(entry: EntryDto): string {
    const time = Date.parse(entry.updatedAt);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    return new Date(time).toISOString().slice(0, 16).replace('T', ' ');
  }

  function messageFromError(error: unknown): string {
    if (typeof error === 'object' && error !== null && 'message' in error) {
      return String(error.message);
    }

    return 'Unable to delete entry';
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
        <IconButton label={`Delete ${entry.title}`} onclick={requestDelete} disabled={deleteBusy}>
          <Icon name="trash" size={13} />
        </IconButton>
      </div>
    </div>

    {#if confirmDeleteOpen}
      <div
        class="detail-confirm modal"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="delete-entry-title"
        aria-describedby="delete-entry-message"
      >
        <div>
          <div id="delete-entry-title" class="modal__q">delete_entry</div>
          <p id="delete-entry-message" class="detail-confirm__copy">
            {entry.title} will be removed from this vault.
          </p>
          {#if deleteError}
            <div class="detail-confirm__error mono error" role="alert">{deleteError}</div>
          {/if}
        </div>
        <div class="modal__row">
          <Button variant="ghost" size="sm" onclick={cancelDelete} disabled={deleteBusy}>cancel</Button>
          <Button variant="danger" size="sm" onclick={() => confirmDelete(entry)} disabled={deleteBusy}>
            <Icon name="trash" size={12} />
            {deleteBusy ? 'deleting' : 'delete'}
          </Button>
        </div>
      </div>
    {/if}

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
