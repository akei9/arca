<script lang="ts">
  import { getAuditState } from '../../stores/audit.svelte';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, Tag } from '../primitives';

  const auditState = $derived(getAuditState());

  function openEntry(entry: (typeof vaultState.entries)[number]) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }
</script>

<section class="audit-panel" aria-labelledby="audit-title">
  <div class="detail-head">
    <div class="detail__title-wrap">
      <div class="detail__crumbs mono">vault &nbsp;/&nbsp; <b>audit</b></div>
      <h1 id="audit-title" class="detail__title">audit<em>.</em></h1>
      <div class="detail__meta mono">
        <Tag value={`${auditState.score}/100`} />
        <span>entries · <b>{vaultState.entries.length}</b></span>
        <span>findings · <b>{auditState.findingCount}</b></span>
      </div>
    </div>
    <div class="detail__actions">
      <Button variant="ghost" size="sm" onclick={() => (uiState.view = 'list')}>back_to_vault</Button>
    </div>
  </div>

  <div class="audit-body">
    <section class="audit-summary" aria-label="Audit summary">
      <div>
        <span>high</span>
        <b>{auditState.highCount}</b>
      </div>
      <div>
        <span>medium</span>
        <b>{auditState.mediumCount}</b>
      </div>
      <div>
        <span>low</span>
        <b>{auditState.lowCount}</b>
      </div>
      <div class="audit-summary__note">
        <span>coverage</span>
        <b>loaded_secret</b>
      </div>
    </section>

    <div class="audit-list" role="list">
      {#if auditState.findingCount > 0}
        {#each auditState.findings as finding (finding.key)}
          <button type="button" class="audit-finding" onclick={() => openEntry(finding.entry)}>
            <span class={`audit-finding__severity audit-finding__severity--${finding.severity}`}>
              {finding.severity}
            </span>
            <span class="audit-finding__main">
              <b>{finding.title}</b>
              <span>{finding.entry.title}</span>
            </span>
            <span class="audit-finding__meta mono">{finding.meta}</span>
            <Icon name="external" size={12} />
          </button>
        {/each}
      {:else}
        <div class="audit-empty">
          <span>&gt;</span>
          no_findings
        </div>
      {/if}
    </div>
  </div>
</section>
