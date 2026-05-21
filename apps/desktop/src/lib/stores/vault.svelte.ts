import type { EntryDto } from '../ipc';

export const vaultState = $state({
  locked: true,
  entries: [] as EntryDto[],
  selectedEntry: null as EntryDto | null,
  searchQuery: '',
  vaultName: '',
  lastSaved: null as Date | null,
});
