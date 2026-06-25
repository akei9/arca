import { afterAll, describe, expect, it, vi } from 'vitest';
import { buildAuditFindings, scoreAudit } from './audit';
import type { EntryDto } from './ipc';

const millisecondsPerDay = 1000 * 60 * 60 * 24;
const referenceTime = new Date('2026-06-22T12:00:00.000Z');
const fresh = new Date(referenceTime.getTime() - 30 * millisecondsPerDay).toISOString();
const stale = new Date(referenceTime.getTime() - 181 * millisecondsPerDay).toISOString();

vi.useFakeTimers();
vi.setSystemTime(referenceTime);
afterAll(() => vi.useRealTimers());

function entry(overrides: Partial<EntryDto>): EntryDto {
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
      'duplicate_username',
      'duplicate_username',
      'stale_entry',
      'missing_url',
      'untagged',
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
    }
  });
});

describe('scoreAudit', () => {
  it('returns zero for empty vaults and clamps poor scores at zero', () => {
    expect(scoreAudit(0, 0, 0)).toBe('0');
    expect(scoreAudit(3, 50, 10)).toBe('0');
  });

  it('penalizes high severity findings more heavily', () => {
    expect(scoreAudit(5, 2, 0)).toBe('92');
    expect(scoreAudit(5, 2, 2)).toBe('56');
  });
});
