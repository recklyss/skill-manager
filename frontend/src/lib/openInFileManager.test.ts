import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const revealItemInDir = vi.fn();

vi.mock("@tauri-apps/plugin-opener", () => ({
  revealItemInDir,
}));

describe("openInFileManager", () => {
  beforeEach(() => {
    revealItemInDir.mockReset();
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  });

  afterEach(() => {
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  });

  it("is a no-op in browser mode", async () => {
    const { openInFileManager } = await import("./openInFileManager");

    await openInFileManager("/tmp/skill-manager/shared");

    expect(revealItemInDir).not.toHaveBeenCalled();
  });

  it("reveals the folder via the Tauri opener in desktop mode", async () => {
    (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    const { openInFileManager } = await import("./openInFileManager");

    await openInFileManager("/tmp/skill-manager/shared");

    expect(revealItemInDir).toHaveBeenCalledWith("/tmp/skill-manager/shared");
  });

  it("ignores blank paths", async () => {
    (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    const { openInFileManager } = await import("./openInFileManager");

    await openInFileManager("   ");

    expect(revealItemInDir).not.toHaveBeenCalled();
  });
});
