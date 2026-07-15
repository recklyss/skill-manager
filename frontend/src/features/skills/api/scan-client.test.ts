import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { okJson } from "../../../test/fetch";
import { getScanHarnesses, scanSkill } from "./scan-client";

const fetchMock = vi.fn();

describe("scan api client", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    fetchMock.mockReset();
    vi.unstubAllGlobals();
  });

  it("loads scannable harnesses", async () => {
    fetchMock.mockResolvedValue(okJson({
      harnesses: [
        { harness: "claude", label: "Claude", cliAvailable: true, scannable: true },
      ],
    }));

    const result = await getScanHarnesses();

    expect(result.harnesses[0]?.harness).toBe("claude");
    expect(fetchMock).toHaveBeenCalledWith("/api/scan/harnesses");
  });

  it("posts harness id when scanning a skill", async () => {
    fetchMock.mockResolvedValue(okJson({
      skillName: "demo",
      isSafe: true,
      maxSeverity: "SAFE",
      findingsCount: 0,
      findings: [],
      analyzersUsed: ["claude_scanner"],
      durationSeconds: 0.1,
    }));

    await scanSkill("demo", { harness: "claude" });

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/scan/skills/demo",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ harness: "claude" }),
      }),
    );
  });
});
