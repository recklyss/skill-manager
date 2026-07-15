import type { SkillListRow } from "./types";

export type SkillBucket = "disabled" | "single" | "selective" | "enabled";

function interactiveCells(row: SkillListRow) {
  return row.cells.filter((cell) => cell.interactive);
}

function countEnabledInteractive(row: SkillListRow): number {
  return interactiveCells(row).filter((cell) => cell.state === "enabled").length;
}

export function bucketForRow(row: SkillListRow): SkillBucket {
  const interactive = interactiveCells(row);
  if (interactive.length === 0) {
    return "enabled";
  }
  const enabledCount = countEnabledInteractive(row);
  if (enabledCount === 0) {
    return "disabled";
  }
  if (interactive.every((cell) => cell.state === "enabled")) {
    return "enabled";
  }
  if (enabledCount === 1) {
    return "single";
  }
  return "selective";
}

export interface BucketedRows {
  disabled: SkillListRow[];
  single: SkillListRow[];
  selective: SkillListRow[];
  enabled: SkillListRow[];
}

export function bucketRows(rows: SkillListRow[]): BucketedRows {
  const result: BucketedRows = { disabled: [], single: [], selective: [], enabled: [] };
  for (const row of rows) {
    result[bucketForRow(row)].push(row);
  }
  return result;
}
