<script lang="ts">
  import type { EntryDto } from '../../ipc';
  import { uiState } from '../../stores/ui.svelte';
  import { vaultState } from '../../stores/vault.svelte';
  import { Icon } from '../icons';
  import { Button, Tag } from '../primitives';

  type AuditSeverity = 'high' | 'medium' | 'low';

  interface AuditFinding {
    key: string;
    severity: AuditSeverity;
    title: string;
    entry: EntryDto;
    meta: string;
  }

  const findings = $derived(buildFindings(vaultState.entries));
  const highCount = $derived(findings.filter((finding) => finding.severity === 'high').length);
  const mediumCount = $derived(findings.filter((finding) => finding.severity === 'medium').length);
  const lowCount = $derived(findings.filter((finding) => finding.severity === 'low').length);
  const auditScore = $derived(scoreFor(vaultState.entries.length, findings.length, highCount));

  function openEntry(entry: EntryDto) {
    vaultState.selectedEntry = entry;
    uiState.view = 'detail';
  }

  function buildFindings(entries: EntryDto[]): AuditFinding[] {
    const results: AuditFinding[] = [];
    const usernames = groupBy(entries, (entry) => normalize(entry.username));
    const loadedPasswords = groupBy(
      entries.filter((entry) => entry.password),
      (entry) => entry.password ?? '',
    );

    for (const entry of entries) {
      if (!entry.url) {
        results.push(finding('missing-url', 'low', 'missing_url', entry, 'metadata'));
      }

      if (entry.tags.length === 0) {
        results.push(finding('untagged', 'low', 'untagged', entry, 'metadata'));
      }

      if (isStale(entry.updatedAt)) {
        results.push(finding('stale', 'medium', 'stale_entry', entry, modified(entry)));
      }

      if (entry.username && (usernames.get(normalize(entry.username))?.length ?? 0) > 1) {
        results.push(finding('duplicate-username', 'medium', 'duplicate_username', entry, entry.username));
      }

      if (entry.password && entry.password.length < 12) {
        results.push(finding('weak-password', 'high', 'weak_password', entry, 'loaded_secret'));
      }

      if (entry.password && (loadedPasswords.get(entry.password)?.length ?? 0) > 1) {
        results.push(finding('reused-password', 'high', 'reused_password', entry, 'loaded_secret'));
      }
    }

    return results.sort((a, b) => severityRank(a.severity) - severityRank(b.severity) || a.title.localeCompare(b.title));
  }

  function finding(
    type: string,
    severity: AuditSeverity,
    title: string,
    entry: EntryDto,
    meta: string,
  ): AuditFinding {
    return {
      key: `${type}:${entry.id}`,
      severity,
      title,
      entry,
      meta,
    };
  }

  function groupBy(entries: EntryDto[], keyFor: (entry: EntryDto) => string): Map<string, EntryDto[]> {
    const groups = new Map<string, EntryDto[]>();

    for (const entry of entries) {
      const key = keyFor(entry);

      if (!key) {
        continue;
      }

      groups.set(key, [...(groups.get(key) ?? []), entry]);
    }

    return groups;
  }

  function scoreFor(entryCount: number, findingCount: number, high: number): string {
    if (entryCount === 0) {
      return '0';
    }

    return Math.max(0, 100 - high * 18 - findingCount * 4).toString();
  }

  function isStale(updatedAt: string): boolean {
    const time = Date.parse(updatedAt);

    if (Number.isNaN(time)) {
      return false;
    }

    return Date.now() - time > 1000 * 60 * 60 * 24 * 180;
  }

  function modified(entry: EntryDto): string {
    const time = Date.parse(entry.updatedAt);

    if (Number.isNaN(time)) {
      return 'unknown';
    }

    return new Date(time).toISOString().slice(0, 10);
  }

  function normalize(value: string): string {
    return value.trim().toLowerCase();
  }

  function severityRank(severity: AuditSeverity): number {
    return severity === 'high' ? 0 : severity === 'medium' ? 1 : 2;
  }
</script>

<section class="audit-panel" aria-labelledby="audit-title">
  <div class="detail-head">
    <div class="detail__title-wrap">
      <div class="detail__crumbs mono">vault &nbsp;/&nbsp; <b>audit</b></div>
      <h1 id="audit-title" class="detail__title">audit<em>.</em></h1>
      <div class="detail__meta mono">
        <Tag value={`${auditScore}/100`} />
        <span>entries · <b>{vaultState.entries.length}</b></span>
        <span>findings · <b>{findings.length}</b></span>
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
        <b>{highCount}</b>
      </div>
      <div>
        <span>medium</span>
        <b>{mediumCount}</b>
      </div>
      <div>
        <span>low</span>
        <b>{lowCount}</b>
      </div>
    </section>

    <div class="audit-list" role="list">
      {#if findings.length > 0}
        {#each findings as finding (finding.key)}
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
