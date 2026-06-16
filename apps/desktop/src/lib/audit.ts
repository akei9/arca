import type { EntryDto } from './ipc';

export type AuditSeverity = 'high' | 'medium' | 'low';

export interface AuditFinding {
  key: string;
  severity: AuditSeverity;
  title: string;
  entry: EntryDto;
  meta: string;
}

export function buildAuditFindings(entries: EntryDto[]): AuditFinding[] {
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
      results.push(finding('duplicate-username', 'medium', 'duplicate_username', entry, 'duplicate_detected'));
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

export function scoreAudit(entryCount: number, findingCount: number, high: number): string {
  if (entryCount === 0) {
    return '0';
  }

  return Math.max(0, 100 - high * 18 - findingCount * 4).toString();
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
