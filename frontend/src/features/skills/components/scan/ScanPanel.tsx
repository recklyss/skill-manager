import { ShieldAlert, ShieldCheck, Terminal, ChevronDown, ChevronRight, FileText } from "lucide-react";
import { useMemo, useState } from "react";
import type { ScanResult, ScanFinding } from "../../api/scan-types";
import { useSkillsCopy } from "../../i18n";

const SEVERITY_ORDER = ["CRITICAL", "HIGH", "MEDIUM", "LOW"];

function SeverityBadge({ severity }: { severity: string }) {
  return (
    <span className="scan-report__severity" data-severity={severity}>
      {severity}
    </span>
  );
}

function FindingRow({ finding, remediationLabel }: { finding: ScanFinding; remediationLabel: string }) {
  const [open, setOpen] = useState(false);
  const location = finding.filePath
    ? `${finding.filePath}${finding.lineNumber != null ? `:${finding.lineNumber}` : ""}`
    : null;

  return (
    <article className="scan-report-finding" data-open={open ? "true" : undefined}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="scan-report-finding__trigger"
      >
        <span className="scan-report-finding__chevron">
          {open ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </span>
        <SeverityBadge severity={finding.severity} />
        <span className="scan-report-finding__title">{finding.title}</span>
        {location ? (
          <span className="scan-report-finding__location">
            <FileText size={13} aria-hidden="true" />
            {location}
          </span>
        ) : null}
      </button>
      {open && (
        <div className="scan-report-finding__body">
          <div className="skill-detail__document-surface scan-report-finding__surface">
            <div className="markdown-content scan-report-finding__content">
              <p className="scan-report-finding__description">{finding.description}</p>
              {finding.remediation && (
                <p className="scan-report-finding__remediation">
                  <strong>{remediationLabel} </strong>{finding.remediation}
                </p>
              )}
              {finding.snippet && (
                <pre className="scan-report-finding__snippet">
                  {finding.snippet}
                </pre>
              )}
            </div>
          </div>
        </div>
      )}
    </article>
  );
}

export default function ScanPanel({
  result,
  harnessLabel,
}: {
  result: ScanResult;
  harnessLabel?: string | null;
}) {
  const copy = useSkillsCopy().scan.result;

  const sortedFindings = useMemo(
    () => [...result.findings].sort((a, b) => severityRank(a.severity) - severityRank(b.severity)),
    [result.findings],
  );
  const criticalCount = sortedFindings.filter((finding) => finding.severity === "CRITICAL").length;
  const hasCriticalFindings = criticalCount > 0;
  const hasFindings = result.findingsCount > 0;
  const headline = hasCriticalFindings
    ? copy.serious
    : hasFindings
      ? copy.nonSerious
      : copy.noProblems;
  const findingsLabel = copy.findingsCount(result.findingsCount);

  return (
    <section className="scan-report" aria-label={copy.reportAria}>
      <header className="scan-report__hero" data-safe={hasCriticalFindings ? "false" : "true"}>
        <div className="scan-report__status-icon">
          {hasCriticalFindings ? <ShieldAlert size={26} aria-hidden="true" /> : <ShieldCheck size={26} aria-hidden="true" />}
        </div>
        <div className="scan-report__headline">
          <h3>{headline}</h3>
          <p>
            {result.skillName} - {result.durationSeconds.toFixed(1)}s - {findingsLabel}
          </p>
        </div>
      </header>

      {harnessLabel ? (
        <div className="scan-report__llm">
          <div className="scan-report__llm-heading">
            <Terminal size={14} aria-hidden="true" />
            <span>{copy.harnessScanner}</span>
          </div>
          <div className="scan-report__llm-body">
            <div>{copy.harnessUsed}: <strong>{harnessLabel}</strong></div>
          </div>
        </div>
      ) : null}

      <div className="scan-report__findings">
        {sortedFindings.map((f) => (
          <FindingRow key={f.id} finding={f} remediationLabel={copy.remediation} />
        ))}
      </div>
    </section>
  );
}

function severityRank(severity: string) {
  const rank = SEVERITY_ORDER.indexOf(severity);
  return rank === -1 ? SEVERITY_ORDER.length : rank;
}
