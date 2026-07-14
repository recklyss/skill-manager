function normalizeBase(value: string | undefined): string {
  const trimmed = (value ?? "/api").trim();
  if (trimmed === "" || trimmed === "/") {
    return "";
  }
  return trimmed.endsWith("/") ? trimmed.slice(0, -1) : trimmed;
}

const apiBase = normalizeBase(import.meta.env.VITE_API_BASE);

let apiOrigin = "";

export async function initApiOrigin(): Promise<void> {
  // Try the Tauri IPC bridge first (most reliable).
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const url: string = await invoke("get_server_url");
    if (url) {
      apiOrigin = url;
      return;
    }
  } catch {
    // Not running in Tauri — use fallback.
  }

  // Fallback: check for injected window variable.
  const injected = (window as any).__SKILL_MANAGER_API_ORIGIN__ as string | undefined;
  if (injected) {
    apiOrigin = injected;
    return;
  }

  // Final fallback: relative URLs (traditional dev server or same-origin deploy).
  apiOrigin = "";
}

export function apiPath(path: string): string {
  return `${apiOrigin}${apiBase}${path}`;
}
