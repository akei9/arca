import { afterAll, describe, expect, it, vi } from 'vitest';
import { AUDIT_FINDING_COPY, buildAuditFindings, filterAuditableEntries, scoreAudit } from './audit';
import type { AuditableEntry } from './audit';

const millisecondsPerDay = 1000 * 60 * 60 * 24;
const referenceTime = new Date('2026-06-22T12:00:00.000Z');
const fresh = new Date(referenceTime.getTime() - 30 * millisecondsPerDay).toISOString();
const stale = new Date(referenceTime.getTime() - 181 * millisecondsPerDay).toISOString();

vi.useFakeTimers();
vi.setSystemTime(referenceTime);
afterAll(() => vi.useRealTimers());

function entry(overrides: Partial<AuditableEntry>): AuditableEntry {
  return {
    id: 'entry-id',
    title: 'Entry',
    username: 'user',
    password: null,
    collection: 'work',
    url: 'https://example.test',
    notes: null,
    tags: ['work'],
    createdAt: fresh,
    updatedAt: fresh,
    revisionCount: 0,
    ...overrides,
  };
}

describe('buildAuditFindings', () => {
  it('detects metadata, age, duplicate username, and password findings', () => {
    const findings = buildAuditFindings([
      entry({
        id: 'weak',
        title: 'Weak',
        username: 'same-user',
        password: 'short',
        url: null,
        tags: [],
        updatedAt: stale,
      }),
      entry({
        id: 'reused-a',
        title: 'Reused A',
        username: 'same-user',
        password: 'shared-password-with-length',
      }),
      entry({
        id: 'reused-b',
        title: 'Reused B',
        username: 'other-user',
        password: 'shared-password-with-length',
      }),
    ]);

    expect(findings.map((finding) => finding.title)).toEqual([
      'reused_password',
      'reused_password',
      'weak_password',
      'duplicate_url',
      'duplicate_url',
      'duplicate_username',
      'duplicate_username',
      'stale_entry',
      'missing_url',
      'untagged',
    ]);
  });

  it('detects local URL and organization hygiene findings', () => {
    const findings = buildAuditFindings([
      entry({
        id: 'http-a',
        title: 'HTTP A',
        username: 'http-a-user',
        url: 'http://example.test/login?session=ignored',
        collection: null,
      }),
      entry({
        id: 'http-b',
        title: 'HTTP B',
        username: 'http-b-user',
        url: 'http://example.test/login',
      }),
    ]);

    expect(findings.map((finding) => finding.title)).toEqual([
      'duplicate_url',
      'duplicate_url',
      'insecure_url',
      'insecure_url',
      'missing_collection',
    ]);
  });

  it('keeps plaintext passwords out of passive finding text', () => {
    const secret = 'shared-password-with-length';
    const findings = buildAuditFindings([
      entry({ id: 'first', password: secret }),
      entry({ id: 'second', password: secret }),
    ]);

    for (const finding of findings) {
      expect(finding.key).not.toContain(secret);
      expect(finding.title).not.toContain(secret);
      expect(finding.meta).not.toContain(secret);
      expect(AUDIT_FINDING_COPY[finding.title].label).not.toContain(secret);
      expect(AUDIT_FINDING_COPY[finding.title].action).not.toContain(secret);
    }
  });

  it('treats reused passwords as high-risk findings', () => {
    const secret = 'shared-password-with-length';
    const findings = buildAuditFindings([
      entry({ id: 'first', password: secret }),
      entry({ id: 'second', password: secret }),
    ]);

    expect(findings.filter((finding) => finding.title === 'reused_password')).toEqual([
      expect.objectContaining({ severity: 'high', meta: 'loaded_secret' }),
      expect.objectContaining({ severity: 'high', meta: 'loaded_secret' }),
    ]);
  });

  it('excludes archived entries from audit scope', () => {
    const entries = [
      entry({ id: 'active', collection: 'work' }),
      entry({ id: 'archived', collection: 'archive' }),
      entry({ id: 'archived-spaced', collection: ' Archive ' }),
    ];

    expect(filterAuditableEntries(entries).map((auditedEntry) => auditedEntry.id)).toEqual(['active']);
  });

  it('ignores archived passwords when detecting reused active passwords', () => {
    const secret = 'shared-password-with-length';
    const findings = buildAuditFindings(
      filterAuditableEntries([
        entry({ id: 'active', password: secret }),
        entry({ id: 'archived', password: secret, collection: 'archive' }),
      ]),
    );

    expect(findings.some((finding) => finding.title === 'reused_password')).toBe(false);
  });
});

describe('scoreAudit', () => {
  it('returns zero for empty vaults', () => {
    expect(scoreAudit(0, 0)).toBe('0');
  });

  it('scores the healthy entry ratio as a rounded percentage', () => {
    expect(scoreAudit(6, 1)).toBe('17');
    expect(scoreAudit(100, 82)).toBe('82');
  });
});
