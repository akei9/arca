import type { EntryDto } from './ipc';

export type AuditSeverity = 'high' | 'medium' | 'low';
export type AuditFindingTitle =
  | 'weak_password'
  | 'reused_password'
  | 'insecure_url'
  | 'duplicate_url'
  | 'duplicate_username'
  | 'stale_entry'
  | 'missing_url'
  | 'missing_collection'
  | 'untagged';

export interface AuditFinding {
  key: string;
  severity: AuditSeverity;
  title: AuditFindingTitle;
  entry: EntryDto;
  meta: string;
}

export type AuditableEntry = EntryDto & {
  password?: string | null;
};

export interface AuditFindingCopy {
  label: string;
  action: string;
}

export const AUDIT_FINDING_COPY: Record<AuditFindingTitle, AuditFindingCopy> = {
  weak_password: {
    label: 'weak password',
    action: 'Generate a stronger replacement and update this entry.',
  },
  reused_password: {
    label: 'reused password',
    action: 'Use a unique password for this account.',
  },
  insecure_url: {
    label: 'HTTP URL',
    action: 'Switch this entry to HTTPS when the service supports it.',
  },
  duplicate_url: {
    label: 'duplicate URL',
    action: 'Confirm these entries are intentionally separate accounts.',
  },
  duplicate_username: {
    label: 'duplicate username',
    action: 'Confirm this login is intentional for both entries.',
  },
  stale_entry: {
    label: 'stale entry',
    action: 'Review whether this credential still needs rotation.',
  },
  missing_url: {
    label: 'missing URL',
    action: 'Add the service URL so search and audit context stay useful.',
  },
  missing_collection: {
    label: 'missing collection',
    action: 'Assign a collection so this entry has a clear home.',
  },
  untagged: {
    label: 'missing tags',
    action: 'Add tags to make this entry easier to find later.',
  },
};

/** Builds audit findings from metadata plus any explicitly loaded password values. */
export function buildAuditFindings(entries: AuditableEntry[]): AuditFinding[] {
  const results: AuditFinding[] = [];
  const usernames = groupBy(entries, (entry) => normalize(entry.username));
  const urls = groupBy(entries, (entry) => normalizeUrl(entry.url));
  const loadedPasswords = groupBy(
    entries.filter(hasLoadedPassword),
    (entry) => entry.password,
    { includeEmptyKey: true },
  );

  for (const entry of entries) {
    if (!entry.url) {
      results.push(finding('missing-url', 'low', 'missing_url', entry, 'metadata'));
    } else if (isHttpUrl(entry.url)) {
      results.push(finding('insecure-url', 'medium', 'insecure_url', entry, 'http'));
    }

    if (entry.url && (urls.get(normalizeUrl(entry.url))?.length ?? 0) > 1) {
      results.push(finding('duplicate-url', 'medium', 'duplicate_url', entry, 'duplicate_detected'));
    }

    if (!entry.collection?.trim()) {
      results.push(finding('missing-collection', 'low', 'missing_collection', entry, 'metadata'));
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

    if (hasValue(entry.password) && entry.password.length < 12) {
      results.push(finding('weak-password', 'high', 'weak_password', entry, 'loaded_secret'));
    }

    if (hasValue(entry.password) && (loadedPasswords.get(entry.password)?.length ?? 0) > 1) {
      results.push(finding('reused-password', 'high', 'reused_password', entry, 'loaded_secret'));
    }
  }

  return results.sort((a, b) => severityRank(a.severity) - severityRank(b.severity) || a.title.localeCompare(b.title));
}

/** Removes archived entries from audit scope. */
export function filterAuditableEntries<T extends EntryDto>(entries: T[]): T[] {
  return entries.filter(isAuditableEntry);
}

/** Scores audit health as the rounded percentage of healthy entries. */
export function scoreAudit(entryCount: number, healthyEntryCount: number): string {
  if (entryCount === 0) {
    return '0';
  }

  return Math.round((Math.max(0, healthyEntryCount) / entryCount) * 100).toString();
}

function isAuditableEntry(entry: EntryDto): boolean {
  return normalize(entry.collection ?? '') !== 'archive';
}

function finding(
  type: string,
  severity: AuditSeverity,
  title: AuditFindingTitle,
  entry: AuditableEntry,
  meta: string,
): AuditFinding {
  return {
    key: `${type}:${entry.id}`,
    severity,
    title,
    entry: metadataOnlyEntry(entry),
    meta,
  };
}

function metadataOnlyEntry(entry: AuditableEntry): EntryDto {
  const { password: _password, ...metadata } = entry;
  return metadata;
}

function groupBy<T extends EntryDto>(
  entries: T[],
  keyFor: (entry: T) => string,
  options?: { includeEmptyKey?: boolean },
): Map<string, T[]> {
  const groups = new Map<string, T[]>();

  for (const entry of entries) {
    const key = keyFor(entry);

    if (!options?.includeEmptyKey && !key) {
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

function normalizeUrl(value: string | null | undefined): string {
  if (!value) {
    return '';
  }

  try {
    const url = new URL(value);
    url.hash = '';
    url.search = '';
    return `${url.protocol}//${url.host}${url.pathname.replace(/\/$/, '')}`.toLowerCase();
  } catch {
    return normalize(value).replace(/\/$/, '');
  }
}

function isHttpUrl(value: string): boolean {
  try {
    return new URL(value).protocol === 'http:';
  } catch {
    return normalize(value).startsWith('http://');
  }
}

function hasLoadedPassword(entry: AuditableEntry): entry is EntryDto & { password: string } {
  return hasValue(entry.password);
}

function hasValue<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined;
}

function severityRank(severity: AuditSeverity): number {
  return severity === 'high' ? 0 : severity === 'medium' ? 1 : 2;
}
