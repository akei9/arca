import { buildAuditFindings, scoreAudit, type AuditFinding } from '../audit';
import { vaultState } from './vault.svelte';

export interface AuditStateSnapshot {
  findings: AuditFinding[];
  findingCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  score: string;
}

const auditState = $derived.by((): AuditStateSnapshot => {
  const findings = buildAuditFindings(vaultState.entries);
  const highCount = findings.filter((finding) => finding.severity === 'high').length;
  const mediumCount = findings.filter((finding) => finding.severity === 'medium').length;
  const lowCount = findings.filter((finding) => finding.severity === 'low').length;

  return {
    findings,
    findingCount: findings.length,
    highCount,
    mediumCount,
    lowCount,
    score: scoreAudit(vaultState.entries.length, findings.length, highCount),
  };
});

export function getAuditState(): AuditStateSnapshot {
  return auditState;
}
