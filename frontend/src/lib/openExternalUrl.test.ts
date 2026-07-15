import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const openUrl = vi.fn();

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl,
}));

describe("openExternalUrl", () => {
  const originalOpen = window.open;

  beforeEach(() => {
    openUrl.mockReset();
    window.open = vi.fn();
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  });

  afterEach(() => {
    window.open = originalOpen;
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  });

  it("opens via window.open in browser mode", async () => {
    const { openExternalUrl } = await import("./openExternalUrl");

    await openExternalUrl("https://example.com/docs");

    expect(window.open).toHaveBeenCalledWith(
      "https://example.com/docs",
      "_blank",
      "noopener,noreferrer",
    );
    expect(openUrl).not.toHaveBeenCalled();
  });

  it("opens via Tauri opener in desktop mode", async () => {
    (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    const { openExternalUrl } = await import("./openExternalUrl");

    await openExternalUrl("https://clis.dev/cli/gh");

    expect(openUrl).toHaveBeenCalledWith("https://clis.dev/cli/gh");
    expect(window.open).not.toHaveBeenCalled();
  });

  it("ignores blank URLs", async () => {
    const { openExternalUrl } = await import("./openExternalUrl");

    await openExternalUrl("   ");

    expect(window.open).not.toHaveBeenCalled();
    expect(openUrl).not.toHaveBeenCalled();
  });
});
