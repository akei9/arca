<script lang="ts">
  import type { AuditSeverity } from '../../audit';
  import { getAuditState } from '../../stores/audit.svelte';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Tag } from '../primitives';

  const auditState = $derived(getAuditState());
  const score = $derived(Number(auditState.score));
  const weakCount = $derived(countByTitle('weak_password'));
  const reusedCount = $derived(countByTitle('reused_password'));
  const agingCount = $derived(countByTitle('stale_entry'));
  const flaggedEntryCount = $derived(new Set(auditState.findings.map((finding) => finding.entry.id)).size);
  const healthyCount = $derived(Math.max(0, vaultState.entries.length - flaggedEntryCount));
  const headline = $derived(
    score >= 85 ? 'vault health is strong.' : score >= 65 ? 'vault health needs review.' : 'vault health needs attention.',
  );
  const summary = $derived(
    vaultState.entries.length === 0
      ? 'Add entries to start measuring password health and vault hygiene.'
      : `${healthyCount} of ${vaultState.entries.length} entries are healthy. ${auditState.findingCount} findings need attention — ${weakCount} weak, ${reusedCount} reused, ${agingCount} aging past six months.`,
  );

  function openEntry(entry: (typeof vaultState.entries)[number]) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }

  function countByTitle(title: string): number {
    return auditState.findings.filter((finding) => finding.title === title).length;
  }

  function severityVariant(severity: AuditSeverity): 'out' | 'vault' | 'slate' {
    switch (severity) {
      case 'high':
        return 'out';
      case 'medium':
        return 'vault';
      case 'low':
        return 'slate';
    }
  }

  function severityLabel(severity: AuditSeverity): string {
    switch (severity) {
      case 'high':
        return 'high';
      case 'medium':
        return 'review';
      case 'low':
        return 'minor';
    }
  }

  function severityDotClass(severity: AuditSeverity): string {
    switch (severity) {
      case 'high':
        return 'row__sev row__sev--high';
      case 'medium':
        return 'row__sev row__sev--med';
      case 'low':
        return 'row__sev row__sev--low';
    }
  }
</script>

<section class="page audit-page" aria-labelledby="audit-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="audit-title" class="page__title">audit<em>.</em></h1>
    <div class="page__meta mono">
      <span>entries · <b>{vaultState.entries.length}</b></span>
      <span>findings · <b>{auditState.findingCount}</b></span>
    </div>
  </div>

  <div class="audit">
    <div class="audit__score">
      <div
        class="audit__ring"
        style={`background: conic-gradient(var(--vault) 0% ${score}%, var(--bg-inset) ${score}% 100%);`}
        aria-hidden="true"
      >
        <div class="audit__ring-core">
          <div class="audit__ring-num">{score}<small>/100</small></div>
        </div>
      </div>

      <div class="audit__score-copy">
        <h3>{headline}</h3>
        <p>{summary}</p>
      </div>
    </div>

    <div class="audit__stats">
      <div class="audit__stat">
        <div class="audit__stat-n audit__stat-n--warn">{weakCount}</div>
        <div class="audit__stat-k">weak</div>
      </div>
      <div class="audit__stat">
        <div class="audit__stat-n audit__stat-n--warn">{reusedCount}</div>
        <div class="audit__stat-k">reused</div>
      </div>
      <div class="audit__stat">
        <div class="audit__stat-n">{agingCount}</div>
        <div class="audit__stat-k">aging</div>
      </div>
      <div class="audit__stat">
        <div class="audit__stat-n audit__stat-n--ok">{healthyCount}</div>
        <div class="audit__stat-k">healthy</div>
      </div>
    </div>

    <div class="audit__list-title">
      needs attention
      <span class="entries__rule"></span>
      <span class="entries__count">{auditState.findingCount.toString().padStart(2, '0')}</span>
    </div>

    {#if auditState.findingCount > 0}
      <div class="entries audit__entries" role="list">
        {#each auditState.findings as finding (finding.key)}
          <button type="button" class="row" onclick={() => openEntry(finding.entry)}>
            <div class="row__bullet">
              <span class={severityDotClass(finding.severity)}></span>
            </div>
            <div class="row__main">
              <div class="row__title">{finding.entry.title}</div>
              <div class="row__sub">{finding.title.replaceAll('_', ' ')} · {finding.meta}</div>
            </div>
            <Tag variant={severityVariant(finding.severity)} value={severityLabel(finding.severity)} />
          </button>
        {/each}
      </div>
    {:else}
      <div class="shared-empty">
        <b>no findings</b>
        <small>Your loaded vault is in good shape. Add more entries or rescan after updates to spot regressions.</small>
      </div>
    {/if}
  </div>
</section>
