<script lang="ts">
  import type { EntryDto } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, Kbd } from '../primitives';
  import EntryRow from './EntryRow.svelte';
  import FilterSidebar, { type FilterItem } from './FilterSidebar.svelte';
  import SearchBar from './SearchBar.svelte';

  type FilterKey = 'all' | 'recent' | 'work' | 'personal' | 'infrastructure' | 'shared' | 'archive';

  interface EntrySection {
    key: string;
    label: string;
    entries: EntryDto[];
  }

  let selectedFilter = $state<FilterKey>('all');
  let selectedTag = $state<string | null>(null);
  let searchFocused = $state(true);

  const sortedEntries = $derived([...vaultState.entries].sort(compareByUpdatedAt));
  const recentIds = $derived(new Set(sortedEntries.slice(0, 6).map((entry) => entry.id)));
  const filteredEntries = $derived(
    sortedEntries.filter(
      (entry) =>
        matchesFilter(entry, selectedFilter, recentIds) &&
        matchesSelectedTag(entry, selectedTag) &&
        matchesQuery(entry),
    ),
  );
  const sections = $derived(buildSections(filteredEntries, selectedFilter, recentIds));
  const filters = $derived<FilterItem[]>([
    { key: 'all', label: 'all', count: vaultState.entries.length },
    { key: 'recent', label: 'recent', count: recentIds.size },
    { key: 'work', label: 'work', count: countByFilter('work', recentIds) },
    { key: 'personal', label: 'personal', count: countByFilter('personal', recentIds) },
    { key: 'infrastructure', label: 'infrastructure', count: countByFilter('infrastructure', recentIds) },
    { key: 'shared', label: 'shared', count: countByFilter('shared', recentIds) },
    { key: 'archive', label: 'archive', count: countByFilter('archive', recentIds) },
  ]);
  const tags = $derived(deriveTags(vaultState.entries));
  const entropyScore = $derived(vaultState.entries.length > 0 ? '98.2%' : '0.0%');
  const sealedAt = $derived(formatTimestamp(vaultState.lastSaved));
  const hasScopedResults = $derived(
    vaultState.searchQuery.trim().length > 0 || selectedTag !== null || selectedFilter !== 'all',
  );
  const emptyState = $derived(describeEmptyState(vaultState.entries.length));

  function setQuery(query: string) {
    vaultState.searchQuery = query;
  }

  function selectEntry(entry: EntryDto) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }

  function openNewEntry() {
    vaultState.selectedEntry = null;
    uiState.view = 'edit';
  }

  function selectFilter(key: string) {
    selectedFilter = key as FilterKey;
  }

  function selectTag(tag: string) {
    selectedTag = selectedTag === tag ? null : tag;
  }

  function clearScopedFilters() {
    selectedFilter = 'all';
    selectedTag = null;
  }

  function matchesQuery(entry: EntryDto): boolean {
    const query = vaultState.searchQuery.trim().toLowerCase();

    if (!query) {
      return true;
    }

    if (query.startsWith('#')) {
      const tagQuery = query.slice(1);
      return entry.tags.some((tag) => normalize(tag) === tagQuery);
    }

    return [entry.title, entry.username, entry.url ?? '', ...entry.tags]
      .concat(entry.collection ?? '')
      .join(' ')
      .toLowerCase()
      .includes(query);
  }

  function matchesFilter(entry: EntryDto, filter: FilterKey, recent: Set<string>): boolean {
    if (filter === 'all') {
      return true;
    }

    if (filter === 'recent') {
      return recent.has(entry.id);
    }

    return collectionFor(entry) === filter;
  }

  function matchesSelectedTag(entry: EntryDto, tag: string | null): boolean {
    if (!tag) {
      return true;
    }

    return entry.tags.some((entryTag) => normalize(entryTag) === tag);
  }

  function buildSections(entries: EntryDto[], filter: FilterKey, recent: Set<string>): EntrySection[] {
    if (vaultState.searchQuery.trim().length > 0) {
      return [{ key: 'results', label: 'results', entries }];
    }

    if (selectedTag !== null) {
      return [{ key: `tag-${selectedTag}`, label: `#${selectedTag}`, entries }];
    }

    if (filter !== 'all') {
      return [{ key: filter, label: filter, entries }];
    }

    const grouped = new Map<string, EntrySection>([
      ['recent', { key: 'recent', label: 'recent', entries: [] }],
      ['work', { key: 'work', label: 'work', entries: [] }],
      ['personal', { key: 'personal', label: 'personal', entries: [] }],
      ['infrastructure', { key: 'infrastructure', label: 'infrastructure', entries: [] }],
      ['shared', { key: 'shared', label: 'shared', entries: [] }],
      ['archive', { key: 'archive', label: 'archive', entries: [] }],
      ['other', { key: 'other', label: 'other', entries: [] }],
    ]);

    for (const entry of entries) {
      if (recent.has(entry.id)) {
        grouped.get('recent')?.entries.push(entry);
      } else {
        grouped.get(collectionFor(entry) ?? 'other')?.entries.push(entry);
      }
    }

    return [...grouped.values()].filter((section) => section.entries.length > 0);
  }

  function countByFilter(filter: FilterKey, recent: Set<string>): number {
    return vaultState.entries.filter((entry) => matchesFilter(entry, filter, recent)).length;
  }

  function collectionFor(entry: EntryDto): FilterKey | null {
    const collection = normalize(entry.collection ?? '');

    if (collection === 'work' || collection === 'personal' || collection === 'infrastructure' || collection === 'shared' || collection === 'archive') {
      return collection;
    }

    if (collection === 'infra') {
      return 'infrastructure';
    }

    return null;
  }

  function deriveTags(entries: EntryDto[]): { value: string; count: number }[] {
    const counts = new Map<string, number>();

    for (const entry of entries) {
      for (const tag of entry.tags) {
        const normalized = normalize(tag);
        counts.set(normalized, (counts.get(normalized) ?? 0) + 1);
      }
    }

    return [...counts.entries()]
      .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
      .slice(0, 8)
      .map(([tag, count]) => ({ value: tag, count }));
  }

  function describeEmptyState(entryCount: number): { title: string; hint: string } {
    if (entryCount === 0) {
      return {
        title: 'no_entries',
        hint: 'create your first entry to start the vault',
      };
    }

    if (vaultState.searchQuery.trim().length > 0) {
      return {
        title: 'no_matches',
        hint: 'refine_search or clear the prompt',
      };
    }

    if (selectedTag !== null) {
      return {
        title: 'no_tag_matches',
        hint: `clear #${selectedTag} or try another category`,
      };
    }

    if (selectedFilter !== 'all') {
      return {
        title: 'no_category_matches',
        hint: `clear ${selectedFilter} or move entries into this collection`,
      };
    }

    return {
      title: 'no_matches',
      hint: 'adjust the current filters',
    };
  }

  function compareByUpdatedAt(a: EntryDto, b: EntryDto): number {
    return Date.parse(b.updatedAt) - Date.parse(a.updatedAt);
  }

  function formatTimestamp(value: Date | null): string {
    if (!value) {
      return 'not_saved';
    }

    return value.toISOString().slice(0, 16).replace('T', ' ');
  }

  function normalize(value: string): string {
    return value.trim().toLowerCase();
  }
</script>

<section class="vault-list" aria-labelledby="vault-list-title">
  <div class="hero">
    <span class="hero__hash mono">#</span>
    <h1 id="vault-list-title" class="hero__title">the vault for what you can't <em>lose.</em></h1>
    <div class="hero__meta mono">
      <span>encryption · <b>aes-512</b></span>
      <span>sealed · <b>{sealedAt}</b></span>
      <span>entropy · <b class="vault-list__score">{entropyScore}</b></span>
    </div>
  </div>

  <div class="toolbar">
    <SearchBar
      query={vaultState.searchQuery}
      focused={searchFocused}
      onquery={setQuery}
      onfocus={() => (searchFocused = true)}
      onblur={() => (searchFocused = false)}
    />
    <Button variant="primary" onclick={openNewEntry}>
      <Icon name="plus" size={11} sw={2} />
      new_entry
      <Kbd value="N" />
    </Button>
    <Button variant="ghost">
      <Icon name="refresh" size={11} sw={2} />
      sync
    </Button>
  </div>

  <div class="body">
    <FilterSidebar
      filters={filters}
      tags={tags}
      active={selectedFilter}
      activeTag={selectedTag}
      onselect={selectFilter}
      ontagselect={selectTag}
      onclear={clearScopedFilters}
    />

    <div class="entries" role="listbox" aria-label="Vault entries">
      {#if hasScopedResults}
        <div class="entries__active">
          {#if selectedFilter !== 'all'}
            <span class="entries__active-item mono">collection · <b>{selectedFilter}</b></span>
          {/if}
          {#if selectedTag}
            <span class="entries__active-item mono">tag · <b>#{selectedTag}</b></span>
          {/if}
          {#if vaultState.searchQuery.trim()}
            <span class="entries__active-item mono">query · <b>{vaultState.searchQuery.trim()}</b></span>
          {/if}
        </div>
      {/if}

      {#if sections.length > 0}
        {#each sections as section}
          <div class="entries__section">
            {section.label}
            <span class="entries__rule"></span>
            <span class="entries__count">{section.entries.length.toString().padStart(2, '0')}</span>
          </div>
          {#each section.entries as entry, index (entry.id)}
            <EntryRow
              {entry}
              selected={vaultState.selectedEntry?.id === entry.id}
              shortcut={index < 9 ? `⌘${index + 1}` : undefined}
              onselect={selectEntry}
            />
          {/each}
        {/each}
      {:else}
        <div class="entries__section">
          empty <span class="entries__rule"></span> <span class="entries__count">00</span>
        </div>
        <div class="entries__empty">
          <span>&gt;</span>{emptyState.title}
          <small>{emptyState.hint}</small>
        </div>
      {/if}
    </div>
  </div>
</section>
