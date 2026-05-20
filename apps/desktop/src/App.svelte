<script lang="ts">
  import { vaultState } from './lib/stores/vault.svelte';
</script>

<main class="shell">
  <section class="panel" aria-label="Arca desktop shell">
    <div class="brand">ARCA</div>
    <div class="status">
      <span>{vaultState.locked ? 'LOCKED' : 'UNLOCKED'}</span>
      <span>{vaultState.entries.length} entries</span>
    </div>
  </section>
</main>
