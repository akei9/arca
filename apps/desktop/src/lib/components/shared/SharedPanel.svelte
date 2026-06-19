<script lang="ts">
  import { Icon } from '../icons';
  import { Button } from '../primitives';

  const outgoingShares: [] = [];
  const incomingShares: [] = [];
</script>

<section class="page shared-panel" aria-labelledby="shared-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="shared-title" class="page__title">shared<em>.</em></h1>
    <div class="page__meta mono">
      <span>active · <b>{outgoingShares.length + incomingShares.length}</b></span>
      <span>incoming · <b>{incomingShares.length}</b></span>
    </div>
  </div>

  <div class="shared">
    <div class="audit__list-title">shared by you <span class="entries__rule"></span></div>
    {#if outgoingShares.length > 0}
      {#each outgoingShares as share}
        <div>{share}</div>
      {/each}
    {:else}
      <div class="shared-empty">
        <b>no active outgoing shares</b>
        <small>Arca is still local-only here. Secure sharing transport has not been wired into vault-core yet.</small>
      </div>
    {/if}

    <div class="audit__list-title">shared with you <span class="entries__rule"></span></div>
    {#if incomingShares.length > 0}
      {#each incomingShares as share}
        <div>{share}</div>
      {/each}
    {:else}
      <div class="shared-empty">
        <b>nothing incoming</b>
        <small>When secure share import lands, received secrets will appear in this section.</small>
      </div>
    {/if}

    <div class="shared__actions">
      <Button variant="ghost" disabled>
        <Icon name="share" size={12} />
        new secure share
      </Button>
      <p class="shared__note mono">local-only vault · secure share transport pending</p>
    </div>
  </div>
</section>
