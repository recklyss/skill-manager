function normalizeBase(value: string | undefined): string {
  const trimmed = (value ?? "/api").trim();
  if (trimmed === "" || trimmed === "/") {
    return "";
  }
  return trimmed.endsWith("/") ? trimmed.slice(0, -1) : trimmed;
}

const apiBase = normalizeBase(import.meta.env.VITE_API_BASE);

// In Tauri, the Rust backend always binds to this fixed port.
// The frontend detects Tauri via the injected __TAURI_INTERNALS__ flag.
const TAURI_API_PORT = 18000;

let apiOrigin = "";

function isTauri(): boolean {
  return !!(window as any).__TAURI_INTERNALS__;
}

export function initApiOrigin(): void {
  if (isTauri()) {
    apiOrigin = `http://127.0.0.1:${TAURI_API_PORT}`;
    console.log("[skill-manager] Tauri mode, API at", apiOrigin);
  } else {
    apiOrigin = "";
    console.log("[skill-manager] Browser mode, relative API URLs");
  }
}

export function apiPath(path: string): string {
  return `${apiOrigin}${apiBase}${path}`;
}
