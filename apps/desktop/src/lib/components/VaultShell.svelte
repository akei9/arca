<script lang="ts">
  import { uiState } from '../stores/ui.svelte';
  import { AuditPanel } from './audit';
  import { EntryDetail, EntryForm } from './detail';
  import { GeneratorPanel } from './generator';
  import { SettingsPanel } from './settings';
  import { SharedPanel } from './shared';
  import { EntryList } from './vault';
</script>

{#if uiState.view === 'detail'}
  <EntryDetail />
{:else if uiState.view === 'edit'}
  <EntryForm />
{:else if uiState.view === 'list'}
  <EntryList />
{:else if uiState.view === 'settings'}
  <SettingsPanel />
{:else if uiState.view === 'audit'}
  <AuditPanel />
{:else if uiState.view === 'generator'}
  <GeneratorPanel />
{:else if uiState.view === 'shared'}
  <SharedPanel />
{:else}
  <EntryList />
{/if}
