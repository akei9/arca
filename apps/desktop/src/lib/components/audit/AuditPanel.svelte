<script lang="ts">
  import { AUDIT_FINDING_COPY, type AuditFinding, type AuditFindingTitle } from '../../audit';
  import { getAuditState } from '../../stores/audit.svelte';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Tag } from '../primitives';

  type AuditBucket = 'weak' | 'reused' | 'aging' | 'review';

  const auditState = $derived(getAuditState());
  const score = $derived(Number(auditState.score));
  const weakCount = $derived(countByTitle('weak_password'));
  const reusedCount = $derived(countByTitle('reused_password'));
  const agingCount = $derived(countByTitle('stale_entry'));
  const reviewCount = $derived(Math.max(0, auditState.findingCount - weakCount - reusedCount - agingCount));
  const attentionSummary = $derived.by(() => {
    const parts = [
      weakCount > 0 ? `${weakCount} weak` : null,
      reusedCount > 0 ? `${reusedCount} reused` : null,
      agingCount > 0 ? `${agingCount} aging` : null,
      reviewCount > 0 ? `${reviewCount} review` : null,
    ].filter(Boolean);

    return parts.join(', ');
  });
  const headline = $derived(
    score >= 85 ? 'vault health is strong.' : score >= 65 ? 'vault health needs review.' : 'vault health needs attention.',
  );
  const summary = $derived(
    vaultState.entries.length === 0
      ? 'Add entries to start measuring password health and vault hygiene.'
      : `${auditState.healthyCount} of ${vaultState.entries.length} entries are healthy. ${auditState.findingCount} findings need attention${attentionSummary ? ` - ${attentionSummary}.` : '.'}`,
  );

  function openEntry(entry: (typeof vaultState.entries)[number]) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }

  function countByTitle(title: AuditFindingTitle): number {
    return auditState.findings.filter((finding) => finding.title === title).length;
  }

  function findingBucket(title: AuditFindingTitle): AuditBucket {
    switch (title) {
      case 'weak_password':
        return 'weak';
      case 'reused_password':
        return 'reused';
      case 'stale_entry':
        return 'aging';
      default:
        return 'review';
    }
  }

  function bucketDotClass(title: AuditFindingTitle): string {
    return `row__sev row__sev--${findingBucket(title)}`;
  }

  function bucketTagClass(title: AuditFindingTitle): string {
    return `audit-tag audit-tag--${findingBucket(title)}`;
  }

  function bucketLabel(title: AuditFindingTitle): string {
    return findingBucket(title);
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
      <span>last_scan · <b>live</b></span>
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
      <div class="audit__stat audit__stat--weak">
        <div class="audit__stat-n">{weakCount}</div>
        <div class="audit__stat-k">weak</div>
      </div>
      <div class="audit__stat audit__stat--reused">
        <div class="audit__stat-n">{reusedCount}</div>
        <div class="audit__stat-k">reused</div>
      </div>
      <div class="audit__stat audit__stat--aging">
        <div class="audit__stat-n">{agingCount}</div>
        <div class="audit__stat-k">aging</div>
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
                <span class={bucketDotClass(finding.title)}></span>
              </div>
              <div class="row__main">
                <div class="row__title">{finding.entry.title}</div>
                <div class="row__sub">{findingDetails(finding)}</div>
              </div>
              <Tag class={bucketTagClass(finding.title)} value={bucketLabel(finding.title)} />
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
