import { isTauriRuntime } from "./openExternalUrl";

/**
 * Reveal a folder in the OS file manager (Finder on macOS, the default file
 * manager on Linux), selecting it. In browser mode there is no native file
 * manager, so the call is a no-op.
 *
 * Uses `revealItemInDir` rather than `openPath`: the former is covered by the
 * app's `opener:default` capability and has no path-scope restriction, whereas
 * `openPath` additionally requires a per-path scope to be configured.
 */
export async function openInFileManager(path: string): Promise<void> {
  const trimmed = path.trim();
  if (!trimmed) {
    return;
  }

  if (!isTauriRuntime()) {
    return;
  }

  try {
    const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
    await revealItemInDir(trimmed);
  } catch (error) {
    console.error("Failed to reveal path in file manager", error);
  }
}
