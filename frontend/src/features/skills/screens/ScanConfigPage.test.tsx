import { fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "../../../App";
import { SCAN_HARNESS_KEY } from "../model/use-skill-scan";
import { createRouteFetchMock, okJson } from "../../../test/fetch";
import { skillsPayload } from "../../../test/fixtures/skills";
import { renderWithRouter, stubDesktopMatchMedia } from "../../../test/render";

const fetchMock = vi.fn();

const harnessesResponse = {
  harnesses: [
    { harness: "claude", label: "Claude", cliAvailable: true, scannable: true },
    { harness: "codex", label: "Codex", cliAvailable: true, scannable: true },
    { harness: "cursor", label: "Cursor", cliAvailable: false, scannable: false },
  ],
};

describe("ScanConfigPage", () => {
  beforeEach(() => {
    window.localStorage.clear();
    stubDesktopMatchMedia();
    fetchMock.mockImplementation(
      createRouteFetchMock([
        { match: "/api/skills", response: skillsPayload() },
        { match: "/api/scan/harnesses", response: harnessesResponse },
        { match: "/api/scan/configs", response: { configs: [], activeId: null } },
        { match: "/api/mcp/servers", response: { entries: [], columns: [] } },
        { match: "/api/settings", response: { harnesses: [] } },
        { match: "/api/slash-commands", response: { commands: [], reviewCommands: [] } },
      ], () => okJson({})),
    );
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    fetchMock.mockReset();
    vi.unstubAllGlobals();
  });

  it("renders harness picker instead of redirecting", async () => {
    renderWithRouter(<App />, { route: "/scan-config" });

    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "Scan settings" })).toBeInTheDocument(),
    );
    expect(screen.getByRole("radio", { name: /Claude/i })).toBeChecked();
    expect(screen.getByRole("radio", { name: /Cursor/i })).toBeDisabled();
    expect(screen.getByText("CLI not installed")).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Run scans on Skills → In use → Scan view" }),
    ).toHaveAttribute("href", "/skills/use");
  });

  it("persists selected harness to localStorage", async () => {
    renderWithRouter(<App />, { route: "/scan-config" });

    await waitFor(() =>
      expect(screen.getByRole("radio", { name: /Claude/i })).toBeChecked(),
    );

    fireEvent.click(screen.getByRole("radio", { name: /Codex/i }));
    expect(window.localStorage.getItem(SCAN_HARNESS_KEY)).toBe("codex");
  });
});
