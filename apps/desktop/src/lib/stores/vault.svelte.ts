import type { CreateEntryDto, EntryDto } from '../ipc';

export const vaultState = $state({
  locked: true,
  entries: [] as EntryDto[],
  selectedEntry: null as EntryDto | null,
  entryDraft: null as Partial<CreateEntryDto> | null,
  entryDraftNonce: 0,
  searchQuery: '',
  vaultName: '',
  vaultPath: '',
  lastSaved: null as Date | null,
});

export function setEntryDraft(draft: Partial<CreateEntryDto>) {
  vaultState.entryDraft = draft;
  vaultState.entryDraftNonce += 1;
}

export function clearEntryDraft() {
  vaultState.entryDraft = null;
  vaultState.entryDraftNonce += 1;
}
