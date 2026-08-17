import { buildAuditFindings, filterAuditableEntries, scoreAudit, type AuditFinding } from '../audit';
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

const auditState = $derived.by((): AuditStateSnapshot => {
  const entries = filterAuditableEntries(vaultState.entries);
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
});

export function getAuditState(): AuditStateSnapshot {
  return auditState;
}
