<script lang="ts">
  import { AUDIT_FINDING_COPY, type AuditSeverity } from '../../audit';
  import { getAuditState } from '../../stores/audit.svelte';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Tag } from '../primitives';

  const auditState = $derived(getAuditState());
  const score = $derived(Number(auditState.score));
  const highCount = $derived(countBySeverity('high'));
  const mediumCount = $derived(countBySeverity('medium'));
  const lowCount = $derived(countBySeverity('low'));
  const flaggedEntryCount = $derived(new Set(auditState.findings.map((finding) => finding.entry.id)).size);
  const healthyCount = $derived(Math.max(0, vaultState.entries.length - flaggedEntryCount));
  const findingGroups = $derived(
    [
      {
        severity: 'high' as AuditSeverity,
        label: 'critical',
        findings: auditState.findings.filter((finding) => finding.severity === 'high'),
      },
      {
        severity: 'medium' as AuditSeverity,
        label: 'review',
        findings: auditState.findings.filter((finding) => finding.severity === 'medium'),
      },
      {
        severity: 'low' as AuditSeverity,
        label: 'hygiene',
        findings: auditState.findings.filter((finding) => finding.severity === 'low'),
      },
    ].filter((group) => group.findings.length > 0),
  );
  const headline = $derived(
    score >= 85 ? 'vault health is strong.' : score >= 65 ? 'vault health needs review.' : 'vault health needs attention.',
  );
  const summary = $derived(
    vaultState.entries.length === 0
      ? 'Add entries to start measuring password health and vault hygiene.'
      : `${healthyCount} of ${vaultState.entries.length} entries are clear. ${auditState.findingCount} findings need attention — ${highCount} high, ${mediumCount} review, ${lowCount} hygiene.`,
  );

  function openEntry(entry: (typeof vaultState.entries)[number]) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }

  function countBySeverity(severity: AuditSeverity): number {
    return auditState.findings.filter((finding) => finding.severity === severity).length;
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

  function findingTitle(title: keyof typeof AUDIT_FINDING_COPY): string {
    return AUDIT_FINDING_COPY[title].label;
  }

  function findingAction(title: keyof typeof AUDIT_FINDING_COPY): string {
    return AUDIT_FINDING_COPY[title].action;
  }

  function findingMeta(meta: string): string {
    return meta.replaceAll('_', ' ');
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
      <div class="audit__stat">
        <div class="audit__stat-n audit__stat-n--warn">{highCount}</div>
        <div class="audit__stat-k">high</div>
      </div>
      <div class="audit__stat">
        <div class="audit__stat-n audit__stat-n--warn">{mediumCount}</div>
        <div class="audit__stat-k">review</div>
      </div>
      <div class="audit__stat">
        <div class="audit__stat-n">{lowCount}</div>
        <div class="audit__stat-k">hygiene</div>
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
        {#each findingGroups as group (group.severity)}
          <div class="audit__group-title">
            {group.label}
            <span>{group.findings.length}</span>
          </div>
          {#each group.findings as finding (finding.key)}
            <div role="listitem">
              <button type="button" class="row audit-row" onclick={() => openEntry(finding.entry)}>
                <div class="row__bullet">
                  <span class={severityDotClass(finding.severity)}></span>
                </div>
                <div class="row__main">
                  <div class="row__title">{finding.entry.title}</div>
                  <div class="row__sub">{findingTitle(finding.title)} · {findingAction(finding.title)}</div>
                </div>
                <div class="audit-row__meta">{findingMeta(finding.meta)}</div>
                <Tag variant={severityVariant(finding.severity)} value={severityLabel(finding.severity)} />
              </button>
            </div>
          {/each}
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
