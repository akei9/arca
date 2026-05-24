import type { EntryDto } from '../ipc';

export const vaultState = $state({
  locked: true,
  entries: [] as EntryDto[],
  selectedEntry: null as EntryDto | null,
  searchQuery: '',
  vaultName: '',
  vaultPath: '',
  lastSaved: null as Date | null,
});
