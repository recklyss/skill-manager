import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ScanResult } from "../api/scan-types";
import { useSkillScan } from "./use-skill-scan";

const scanClient = vi.hoisted(() => ({
  scanSkill: vi.fn(),
  getScanHarnesses: vi.fn(),
}));

vi.mock("../api/scan-client", () => scanClient);

const scanResult: ScanResult = {
  skillName: "Trace Lens",
  isSafe: true,
  maxSeverity: "SAFE",
  findingsCount: 0,
  findings: [],
  analyzersUsed: ["claude_scanner"],
  durationSeconds: 1.2,
};

describe("useSkillScan", () => {
  beforeEach(() => {
    window.localStorage.clear();
    scanClient.scanSkill.mockReset();
    scanClient.getScanHarnesses.mockReset();
    scanClient.getScanHarnesses.mockResolvedValue({
      harnesses: [
        {
          harness: "claude",
          label: "Claude",
          cliAvailable: true,
          scannable: true,
        },
      ],
    });
  });

  it("keeps an in-flight scan alive when the consuming page unmounts", async () => {
    let resolveScan: (result: ScanResult) => void = () => undefined;
    scanClient.scanSkill.mockReturnValue(new Promise<ScanResult>((resolve) => {
      resolveScan = resolve;
    }));

    const first = renderHook(() => useSkillScan());
    await waitFor(() => expect(first.result.current.selectedHarness).toBe("claude"));

    let pendingScan: Promise<void> = Promise.resolve();
    act(() => {
      pendingScan = first.result.current.scanSkill("shared:trace-lens");
    });
    await waitFor(() => {
      expect(first.result.current.getScanState("shared:trace-lens").status).toBe("scanning");
    });

    first.unmount();
    await act(async () => {
      resolveScan(scanResult);
      await pendingScan;
    });

    const second = renderHook(() => useSkillScan());
    await waitFor(() => {
      expect(second.result.current.getScanState("shared:trace-lens").status).toBe("done");
    });
    expect(second.result.current.getScanState("shared:trace-lens").result?.skillName).toBe("Trace Lens");
  });

  it("re-scans a completed skill and replaces the cached report", async () => {
    const firstResult: ScanResult = {
      ...scanResult,
      skillName: "Trace Lens",
      durationSeconds: 1.2,
    };
    const secondResult: ScanResult = {
      ...scanResult,
      skillName: "Trace Lens",
      durationSeconds: 2.4,
    };

    scanClient.scanSkill
      .mockResolvedValueOnce(firstResult)
      .mockResolvedValueOnce(secondResult);

    const { result } = renderHook(() => useSkillScan());
    await waitFor(() => expect(result.current.selectedHarness).toBe("claude"));

    await act(async () => {
      await result.current.scanSkill("shared:trace-lens");
    });
    await waitFor(() => {
      expect(result.current.getScanState("shared:trace-lens").status).toBe("done");
    });
    expect(result.current.getScanState("shared:trace-lens").result?.durationSeconds).toBe(1.2);

    await act(async () => {
      await result.current.scanSkill("shared:trace-lens");
    });
    await waitFor(() => {
      expect(result.current.getScanState("shared:trace-lens").result?.durationSeconds).toBe(2.4);
    });
    expect(scanClient.scanSkill).toHaveBeenCalledTimes(2);

    const cached = JSON.parse(
      window.localStorage.getItem("skillmgr.securityReport.cache.v1") ?? "{}",
    ) as Record<string, { result: ScanResult }>;
    expect(cached["shared:trace-lens"]?.result.durationSeconds).toBe(2.4);
  });
});
