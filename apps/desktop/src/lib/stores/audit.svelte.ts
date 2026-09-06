import {
  buildAuditFindings,
  filterAuditableEntries,
  scoreAudit,
  type AuditableEntry,
  type AuditFinding,
} from '../audit';
import { revealEntryPassword, type EntryDto } from '../ipc';
import { vaultState } from './vault.svelte';

export interface AuditStateSnapshot {
  entryCount: number;
  findings: AuditFinding[];
  findingCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  flaggedEntryCount: number;
  healthyCount: number;
  score: string;
}

let loadedAuditState = $state<AuditStateSnapshot>(snapshotForEntries([]));
let loadedAuditFingerprint = $state('');
let activeRefresh = 0;

export function getAuditState(): AuditStateSnapshot {
  const entries = filterAuditableEntries(vaultState.entries);
  const fingerprint = fingerprintEntries(entries);

  if (fingerprint === loadedAuditFingerprint) {
    return loadedAuditState;
  }

  return snapshotForEntries(entries);
}

export async function refreshAuditState(): Promise<void> {
  const entries = filterAuditableEntries(vaultState.entries);
  const fingerprint = fingerprintEntries(entries);

  if (vaultState.locked || fingerprint === loadedAuditFingerprint) {
    return;
  }

  const refreshId = ++activeRefresh;
  loadedAuditState = snapshotForEntries(entries);

  const entriesWithPasswords = await Promise.all(
    entries.map(async (entry): Promise<AuditableEntry> => ({
      ...entry,
      password: await revealEntryPassword(entry.id),
    })),
  );

  if (
    refreshId !== activeRefresh ||
    vaultState.locked ||
    fingerprint !== fingerprintEntries(filterAuditableEntries(vaultState.entries))
  ) {
    return;
  }

  loadedAuditState = snapshotForEntries(entriesWithPasswords);
  loadedAuditFingerprint = fingerprint;
}

function snapshotForEntries(entries: AuditableEntry[]): AuditStateSnapshot {
  const findings = buildAuditFindings(entries);
  const highCount = findings.filter((finding) => finding.severity === 'high').length;
  const mediumCount = findings.filter((finding) => finding.severity === 'medium').length;
  const lowCount = findings.filter((finding) => finding.severity === 'low').length;
  const flaggedEntryCount = new Set(findings.map((finding) => finding.entry.id)).size;
  const healthyCount = Math.max(0, entries.length - flaggedEntryCount);

  return {
    entryCount: entries.length,
    findings,
    findingCount: findings.length,
    highCount,
    mediumCount,
    lowCount,
    flaggedEntryCount,
    healthyCount,
    score: scoreAudit(entries.length, healthyCount),
  };
}

function fingerprintEntries(entries: EntryDto[]): string {
  return `${vaultState.vaultPath}::${entries
    .map((entry) => `${entry.id}:${entry.updatedAt}:${entry.revisionCount}`)
    .join('|')}`;
}
