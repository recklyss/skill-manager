function normalizeBase(value: string | undefined): string {
  const trimmed = (value ?? "/api").trim();
  if (trimmed === "" || trimmed === "/") {
    return "";
  }
  return trimmed.endsWith("/") ? trimmed.slice(0, -1) : trimmed;
}

function resolveApiOrigin(): string {
  // In Tauri, the Rust backend injects the server URL at runtime.
  const tauriOrigin = (window as any).__SKILL_MANAGER_API_ORIGIN__ as string | undefined;
  if (tauriOrigin) {
    return tauriOrigin;
  }
  // In dev (Vite proxy) or when served from the same origin, use relative paths.
  return "";
}

const apiOrigin = resolveApiOrigin();
const apiBase = normalizeBase(import.meta.env.VITE_API_BASE);

export function apiPath(path: string): string {
  return `${apiOrigin}${apiBase}${path}`;
}
