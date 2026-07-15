import { postJson, fetchJson } from "../../../api/http";
import type { ScanResult } from "./scan-types";

export interface ScanHarnessOption {
  harness: string;
  label: string;
  cliAvailable: boolean;
  scannable: boolean;
}

export interface ScanHarnessListResponse {
  harnesses: ScanHarnessOption[];
}

export async function getScanHarnesses(): Promise<ScanHarnessListResponse> {
  return fetchJson<ScanHarnessListResponse>("/scan/harnesses");
}

export async function scanSkill(
  skillRef: string,
  options: ScanSkillOptions,
): Promise<ScanResult> {
  return postJson<ScanResult>(
    `/scan/skills/${encodeURIComponent(skillRef)}`,
    options,
  );
}

export interface ScanSkillOptions {
  harness: string;
}
