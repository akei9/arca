<script lang="ts">
  import { AUDIT_FINDING_COPY, type AuditFinding, type AuditSeverity } from '../../audit';
  import { getAuditState } from '../../stores/audit.svelte';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Tag } from '../primitives';

  const auditState = $derived(getAuditState());
  const score = $derived(Number(auditState.score));
  const headline = $derived(
    score >= 85 ? 'vault health is strong.' : score >= 65 ? 'vault health needs review.' : 'vault health needs attention.',
  );
  const summary = $derived(
    vaultState.entries.length === 0
      ? 'Add entries to start measuring password health and vault hygiene.'
      : `${auditState.healthyCount} of ${vaultState.entries.length} entries are healthy. ${auditState.findingCount} findings need attention — ${auditState.highCount} high, ${auditState.mediumCount} review, ${auditState.lowCount} hygiene.`,
  );

  function openEntry(entry: (typeof vaultState.entries)[number]) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
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

  function severityDotClass(severity: AuditSeverity): string {
    return `row__sev row__sev--${severity}`;
  }

  function severityTagClass(severity: AuditSeverity): string {
    return `audit-tag audit-tag--${severity}`;
  }

  function severityLabel(severity: AuditSeverity): string {
    switch (severity) {
      case 'high':
        return 'high';
      case 'medium':
        return 'review';
      case 'low':
        return 'hygiene';
    }
  }

  function findingTitle(title: keyof typeof AUDIT_FINDING_COPY): string {
    return AUDIT_FINDING_COPY[title].label;
  }

  function findingAction(title: keyof typeof AUDIT_FINDING_COPY): string {
    return AUDIT_FINDING_COPY[title].action;
  }

  function findingMeta(meta: string): string {
    return meta.replaceAll('_', ' ');
  }

  function findingDetails(finding: AuditFinding): string {
    return `${findingTitle(finding.title)} · ${findingMeta(finding.meta)} · ${findingAction(finding.title)}`;
  }
</script>

<section class="page audit-page" aria-labelledby="audit-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="audit-title" class="page__title">audit<em>.</em></h1>
    <div class="page__meta mono">
      <span>entries · <b>{vaultState.entries.length}</b></span>
      <span>findings · <b>{auditState.findingCount}</b></span>
      <span>scope · <b>local</b></span>
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
      <div class="audit__stat audit__stat--high">
        <div class="audit__stat-n">{auditState.highCount}</div>
        <div class="audit__stat-k">high</div>
      </div>
      <div class="audit__stat audit__stat--review">
        <div class="audit__stat-n">{auditState.mediumCount}</div>
        <div class="audit__stat-k">review</div>
      </div>
      <div class="audit__stat audit__stat--hygiene">
        <div class="audit__stat-n">{auditState.lowCount}</div>
        <div class="audit__stat-k">hygiene</div>
      </div>
      <div class="audit__stat audit__stat--healthy">
        <div class="audit__stat-n">{auditState.healthyCount}</div>
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
          <div role="listitem">
            <button type="button" class="row audit-row" onclick={() => openEntry(finding.entry)}>
              <div class="row__bullet">
                <span class={severityDotClass(finding.severity)}></span>
              </div>
              <div class="row__main">
                <div class="row__title">{finding.entry.title}</div>
                <div class="row__sub">{findingDetails(finding)}</div>
              </div>
              <Tag variant={severityVariant(finding.severity)} class={severityTagClass(finding.severity)} value={severityLabel(finding.severity)} />
            </button>
          </div>
        {/each}
      </div>
    {:else}
      <div class="audit-empty">
        <span>no findings</span>
        <small>vault health checks are clear</small>
      </div>
    {/if}
  </div>
</section>
