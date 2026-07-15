import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ScanRow } from "./ScanRow";
import type { SkillListRow } from "../../model/types";
import type { SkillScanState } from "../../model/use-skill-scan";
import type { SkillsCopy } from "../../i18n";

const row: SkillListRow = {
  skillRef: "shared:trace-lens",
  name: "Trace Lens",
  description: "Trace review workflow",
  displayStatus: "Managed",
  actions: { canManage: false, canStopManaging: true, canDelete: true },
  cells: [],
};

const copy: SkillsCopy["scan"]["view"] = {
  tableAria: "Skills scan table",
  select: "Select",
  action: "Action",
  configureAria: "Configure LLM scan",
  configure: "Configure",
  selectHarness: "Select harness",
  selectHarnessAria: "Select a harness above to scan",
  scanning: "Scanning",
  viewResult: "View Result",
  rescan: "Re-scan",
  retry: "Retry",
  scan: "Scan",
  deselectSkill: (name) => `Deselect ${name}`,
  selectSkill: (name) => `Select ${name}`,
  scanningSkill: (name) => `Scanning ${name}`,
  viewResultFor: (name) => `View scan results for ${name}`,
  rescanFor: (name) => `Re-scan ${name}`,
  retryScanFor: (name) => `Retry scan for ${name}`,
  scanSkill: (name) => `Scan ${name}`,
  bulkAria: "Scan bulk actions",
  selected: (count) => `${count} selected`,
  scanAll: "Scan all",
  scanSelected: "Scan selected",
  rescanAll: "Re-scan all",
  rescanSelected: "Re-scan selected",
  clearSelection: "Clear selection",
};

const doneState: SkillScanState = {
  status: "done",
  error: null,
  completedAt: Date.now(),
  result: {
    skillName: "Trace Lens",
    isSafe: true,
    maxSeverity: "SAFE",
    findingsCount: 0,
    findings: [],
    analyzersUsed: ["claude_scanner"],
    durationSeconds: 1.2,
  },
};

describe("ScanRow", () => {
  it("shows view and re-scan actions when a scan is complete", () => {
    const onScanSkill = vi.fn();
    const onViewResult = vi.fn();

    render(
      <table>
        <tbody>
          <ScanRow
            row={row}
            canScan
            checked={false}
            scanState={doneState}
            copy={copy}
            onOpenSkill={vi.fn()}
            onToggleChecked={vi.fn()}
            onScanSkill={onScanSkill}
            onViewResult={onViewResult}
          />
        </tbody>
      </table>,
    );

    fireEvent.click(screen.getByRole("button", { name: "View scan results for Trace Lens" }));
    expect(onViewResult).toHaveBeenCalledWith("shared:trace-lens");

    fireEvent.click(screen.getByRole("button", { name: "Re-scan Trace Lens" }));
    expect(onScanSkill).toHaveBeenCalledWith("shared:trace-lens");
  });
});
