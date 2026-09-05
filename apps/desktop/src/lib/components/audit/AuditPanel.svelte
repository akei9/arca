<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { AUDIT_FINDING_COPY, type AuditFinding, type AuditFindingTitle, type AuditSeverity } from '../../audit';
  import { isEditableTarget } from '../../keyboard';
  import { getAuditState, refreshAuditState } from '../../stores/audit.svelte';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Tag } from '../primitives';

  type AuditBucket = 'weak' | 'reused' | 'aging' | 'review';

  let activeFindingKey = $state('');

  const auditState = $derived(getAuditState());
  const auditInputFingerprint = $derived(
    vaultState.entries.map((entry) => `${entry.id}:${entry.updatedAt}:${entry.revisionCount}`).join('|'),
  );
  const score = $derived(Number(auditState.score));
  const weakCount = $derived(countBySeverity('high'));
  const reusedCount = $derived(countByTitle('reused_password'));
  const agingCount = $derived(countByTitle('stale_entry'));
  const reviewCount = $derived(Math.max(0, auditState.findingCount - weakCount - reusedCount - agingCount));
  const scoreColor = $derived(score >= 85 ? 'var(--vault)' : score >= 65 ? '#D7833F' : 'var(--accent)');
  const findingPositions = $derived(
    new Map(auditState.findings.map((finding, index) => [finding.key, index + 1] as const)),
  );
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
    auditState.entryCount === 0
      ? vaultState.entries.length === 0
        ? 'Add entries to start measuring password health and vault hygiene.'
        : 'Move entries out of archive to include them in password health and vault hygiene.'
      : `${auditState.healthyCount} of ${auditState.entryCount} entries are healthy. ${auditState.findingCount} findings need attention${attentionSummary ? ` - ${attentionSummary}.` : '.'}`,
  );

  $effect(() => {
    if (auditInputFingerprint !== undefined && !vaultState.locked) {
      void refreshAuditState().catch(() => {
        uiState.notification = {
          kind: 'error',
          message: 'Unable to refresh audit findings',
        };
      });
    }
  });

  $effect(() => {
    if (auditState.findings.length === 0) {
      activeFindingKey = '';
      return;
    }

    if (!auditState.findings.some((finding) => finding.key === activeFindingKey)) {
      activeFindingKey = auditState.findings[0].key;
    }
  });

  $effect(() => {
    if (activeFindingKey) {
      void scrollActiveFindingIntoView(activeFindingKey);
    }
  });

  onMount(() => {
    function handleKeydown(event: KeyboardEvent) {
      const key = event.key.toLowerCase();

      if (event.repeat || event.altKey || event.metaKey || event.ctrlKey || isEditableTarget(event.target)) {
        return;
      }

      if (key === 'arrowdown') {
        event.preventDefault();
        moveActiveFinding(1);
        return;
      }

      if (key === 'arrowup') {
        event.preventDefault();
        moveActiveFinding(-1);
        return;
      }

      if (key === 'home') {
        event.preventDefault();
        setActiveFindingAt(0);
        return;
      }

      if (key === 'end') {
        event.preventDefault();
        setActiveFindingAt(auditState.findings.length - 1);
        return;
      }

      if (key === 'enter') {
        const active = auditState.findings.find((finding) => finding.key === activeFindingKey);

        if (active) {
          event.preventDefault();
          openEntry(active.entry);
        }
      }
    }

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  function openEntry(entry: (typeof vaultState.entries)[number]) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }

  function moveActiveFinding(offset: number) {
    if (auditState.findings.length === 0) {
      return;
    }

    const currentIndex = Math.max(0, auditState.findings.findIndex((finding) => finding.key === activeFindingKey));
    setActiveFindingAt(currentIndex + offset);
  }

  function setActiveFindingAt(index: number) {
    const finding = auditState.findings[Math.max(0, Math.min(index, auditState.findings.length - 1))];

    if (finding) {
      activeFindingKey = finding.key;
    }
  }

  async function scrollActiveFindingIntoView(findingKey: string) {
    await tick();
    const row = document.querySelector<HTMLElement>(`[data-audit-row-key="${CSS.escape(findingKey)}"]`);
    const shouldMoveFocus =
      document.activeElement instanceof HTMLElement &&
      document.activeElement.closest('[data-audit-list]') !== null;

    row?.scrollIntoView({ block: 'nearest' });

    if (shouldMoveFocus) {
      row?.focus({ preventScroll: true });
    }
  }

  function countByTitle(title: AuditFindingTitle): number {
    return auditState.findings.filter((finding) => finding.title === title).length;
  }

  function countBySeverity(severity: AuditSeverity): number {
    return auditState.findings.filter((finding) => finding.severity === severity).length;
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

  function findingMarker(finding: AuditFinding): string {
    return `#${findingPositions.get(finding.key) ?? 0}`;
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
        style={`--audit-score-color: ${scoreColor}; background: conic-gradient(var(--audit-score-color) 0% ${score}%, var(--bg-inset) ${score}% 100%);`}
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
      <div class="entries audit__entries" role="list" data-audit-list>
        {#each auditState.findings as finding (finding.key)}
          <div role="listitem">
            <button
              type="button"
              class={activeFindingKey === finding.key ? 'row row--selected audit-row' : 'row audit-row'}
              tabindex={activeFindingKey === finding.key ? 0 : -1}
              data-audit-row-key={finding.key}
              onclick={() => openEntry(finding.entry)}
            >
              <div class="row__bullet">
                <span class={bucketDotClass(finding.title)}></span>
              </div>
              <div class="row__main">
                <div class="row__title">{finding.entry.title}</div>
                <div class="row__sub">{findingDetails(finding)}</div>
              </div>
              <div class="audit-row__tags">
                <Tag value={findingMarker(finding)} />
                <Tag class={bucketTagClass(finding.title)} value={bucketLabel(finding.title)} />
              </div>
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
