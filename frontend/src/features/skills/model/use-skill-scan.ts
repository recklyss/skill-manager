import { useCallback, useEffect, useSyncExternalStore } from "react";

import type { ScanResult } from "../api/scan-types";
import {
  scanSkill as scanSkillApi,
  getScanHarnesses,
  type ScanHarnessOption,
} from "../api/scan-client";

export type ScanStatus = "idle" | "scanning" | "done" | "error";

export interface SkillScanState {
  status: ScanStatus;
  result: ScanResult | null;
  error: string | null;
  completedAt: number | null;
}

export interface ScanStateMap {
  [skillRef: string]: SkillScanState;
}

const IDLE_STATE: SkillScanState = { status: "idle", result: null, error: null, completedAt: null };
const SCAN_REPORT_CACHE_KEY = "skillmgr.securityReport.cache.v1";
export const SCAN_HARNESS_KEY = "skillmgr.scan.harness.v1";

interface CachedScanReport {
  savedAt: number;
  result: ScanResult;
}

type CachedScanReportMap = Record<string, CachedScanReport>;

function readCachedScanReportEntries(): CachedScanReportMap {
  if (typeof window === "undefined") return {};
  try {
    const raw = window.localStorage.getItem(SCAN_REPORT_CACHE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as CachedScanReportMap;
    const next: CachedScanReportMap = {};
    let changed = false;
    for (const [skillRef, entry] of Object.entries(parsed)) {
      if (!entry || typeof entry.savedAt !== "number" || !entry.result) {
        changed = true;
        continue;
      }
      next[skillRef] = entry;
    }
    if (changed) {
      writeCachedScanReportEntries(next);
    }
    return next;
  } catch {
    window.localStorage.removeItem(SCAN_REPORT_CACHE_KEY);
    return {};
  }
}

function readCachedScanReports(): ScanStateMap {
  const entries = readCachedScanReportEntries();
  const next: ScanStateMap = {};
  for (const [skillRef, entry] of Object.entries(entries)) {
    next[skillRef] = { status: "done", result: entry.result, error: null, completedAt: entry.savedAt };
  }
  return next;
}

function writeCachedScanReportEntries(cache: CachedScanReportMap): void {
  if (typeof window === "undefined") return;
  if (Object.keys(cache).length === 0) {
    window.localStorage.removeItem(SCAN_REPORT_CACHE_KEY);
    return;
  }
  window.localStorage.setItem(SCAN_REPORT_CACHE_KEY, JSON.stringify(cache));
}

function cacheScanResult(skillRef: string, result: ScanResult, savedAt = Date.now()): void {
  const cached = readCachedScanReportEntries();
  writeCachedScanReportEntries({
    ...cached,
    [skillRef]: { savedAt, result },
  });
}

function clearCachedScanResult(skillRef: string): void {
  const cached = readCachedScanReportEntries();
  if (!cached[skillRef]) {
    return;
  }
  const { [skillRef]: _removed, ...rest } = cached;
  writeCachedScanReportEntries(rest);
}

function readStoredHarness(): string | null {
  if (typeof window === "undefined") return null;
  const value = window.localStorage.getItem(SCAN_HARNESS_KEY);
  return value && value.trim() ? value : null;
}

function writeStoredHarness(harness: string): void {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(SCAN_HARNESS_KEY, harness);
}

interface SkillScanStoreSnapshot {
  scanState: ScanStateMap;
  harnesses: ScanHarnessOption[];
  selectedHarness: string | null;
  harnessesLoaded: boolean;
}

let scanStoreSnapshot: SkillScanStoreSnapshot = {
  scanState: {},
  harnesses: [],
  selectedHarness: readStoredHarness(),
  harnessesLoaded: false,
};
let hydratedCachedReports = false;

const scanStoreListeners = new Set<() => void>();

function subscribeToScanStore(listener: () => void): () => void {
  scanStoreListeners.add(listener);
  return () => {
    scanStoreListeners.delete(listener);
  };
}

function getScanStoreSnapshot(): SkillScanStoreSnapshot {
  return scanStoreSnapshot;
}

function updateScanStore(
  updater: (current: SkillScanStoreSnapshot) => SkillScanStoreSnapshot,
): void {
  scanStoreSnapshot = updater(scanStoreSnapshot);
  for (const listener of scanStoreListeners) {
    listener();
  }
}

function hydrateCachedScanReports(): void {
  if (hydratedCachedReports) {
    return;
  }
  hydratedCachedReports = true;
  updateScanStore((current) => ({
    ...current,
    scanState: {
      ...readCachedScanReports(),
      ...current.scanState,
    },
  }));
}

function pickDefaultHarness(
  harnesses: ScanHarnessOption[],
  current: string | null,
): string | null {
  const scannable = harnesses.filter((entry) => entry.scannable);
  if (scannable.length === 0) {
    return null;
  }
  if (current && scannable.some((entry) => entry.harness === current)) {
    return current;
  }
  return scannable[0]?.harness ?? null;
}

export function useSkillScan() {
  const snapshot = useSyncExternalStore(
    subscribeToScanStore,
    getScanStoreSnapshot,
    getScanStoreSnapshot,
  );

  const refreshHarnesses = useCallback(async () => {
    try {
      const resp = await getScanHarnesses();
      updateScanStore((current) => {
        const selectedHarness = pickDefaultHarness(resp.harnesses, current.selectedHarness);
        if (selectedHarness) {
          writeStoredHarness(selectedHarness);
        }
        return {
          ...current,
          harnesses: resp.harnesses,
          selectedHarness,
          harnessesLoaded: true,
        };
      });
    } catch {
      updateScanStore((current) => ({ ...current, harnessesLoaded: true }));
    }
  }, []);

  useEffect(() => {
    void refreshHarnesses();
  }, [refreshHarnesses]);

  useEffect(() => {
    hydrateCachedScanReports();
  }, []);

  const getScanState = useCallback(
    (skillRef: string): SkillScanState => snapshot.scanState[skillRef] ?? IDLE_STATE,
    [snapshot.scanState],
  );

  const selectHarness = useCallback((harness: string) => {
    writeStoredHarness(harness);
    updateScanStore((current) => ({
      ...current,
      selectedHarness: harness,
    }));
  }, []);

  const scanSkill = useCallback(
    async (skillRef: string) => {
      if (!snapshot.selectedHarness) return;
      clearCachedScanResult(skillRef);
      updateScanStore((current) => ({
        ...current,
        scanState: {
          ...current.scanState,
          [skillRef]: { status: "scanning", result: null, error: null, completedAt: null },
        },
      }));
      try {
        const result = await scanSkillApi(skillRef, { harness: snapshot.selectedHarness });
        const completedAt = Date.now();
        cacheScanResult(skillRef, result, completedAt);
        updateScanStore((current) => ({
          ...current,
          scanState: {
            ...current.scanState,
            [skillRef]: { status: "done", result, error: null, completedAt },
          },
        }));
      } catch (e) {
        updateScanStore((current) => ({
          ...current,
          scanState: {
            ...current.scanState,
            [skillRef]: {
              status: "error",
              result: null,
              error: e instanceof Error ? e.message : String(e),
              completedAt: null,
            },
          },
        }));
      }
    },
    [snapshot.selectedHarness],
  );

  const selectedHarnessOption = snapshot.harnesses.find(
    (entry) => entry.harness === snapshot.selectedHarness,
  ) ?? null;

  return {
    scanState: snapshot.scanState,
    getScanState,
    scanSkill,
    harnesses: snapshot.harnesses,
    selectedHarness: snapshot.selectedHarness,
    selectedHarnessOption,
    selectHarness,
    harnessesLoaded: snapshot.harnessesLoaded,
    refreshHarnesses,
  };
}
