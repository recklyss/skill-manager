export interface ThemeDefinition {
  id: string;
  label: string;
  labelZh: string;
  tokens: Record<string, string>;
}

const DARK_TOKENS: Record<string, string> = {
  "--color-bg": "#111111",
  "--color-surface": "#1A1A1A",
  "--color-surface-raised": "#222222",
  "--color-surface-sunken": "#0D0D0D",
  "--color-sidebar-bg": "#111111",
  "--color-border": "#2E2E2E",
  "--color-border-strong": "#404040",
  "--color-text": "#F0F0F0",
  "--color-text-muted": "#8A8A8A",
  "--color-text-subtle": "#666666",
  "--color-text-inverted": "#111111",
  "--color-accent": "#4D8DF5",
  "--color-accent-strong": "#3B7AE0",
  "--color-accent-soft": "rgba(77, 141, 245, 0.16)",
  "--color-accent-softer": "rgba(77, 141, 245, 0.08)",
  "--color-success": "#4ADE80",
  "--color-success-soft": "rgba(74, 222, 128, 0.12)",
  "--color-danger": "#F87171",
  "--color-danger-soft": "rgba(248, 113, 113, 0.14)",
  "--color-warning": "#FBBF24",
  "--color-warning-soft": "rgba(251, 191, 36, 0.14)",
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "transparent",
  "--scrollbar-thumb": "rgba(255, 255, 255, 0.18)",
  "--scrollbar-thumb-hover": "rgba(255, 255, 255, 0.28)",
  "--scrollbar-thumb-active": "rgba(255, 255, 255, 0.38)",
  "--scrollbar-corner": "transparent",
  "--shadow-sm": "none",
  "--shadow-md": "none",
  "--shadow-panel": "none",
  "--shadow-lift": "none",
};

const LIGHT_TOKENS: Record<string, string> = {
  "--color-bg": "#F7F7F8",
  "--color-surface": "#FFFFFF",
  "--color-surface-raised": "#FFFFFF",
  "--color-surface-sunken": "#F0F0F1",
  "--color-sidebar-bg": "#FFFFFF",
  "--color-border": "#E5E5E5",
  "--color-border-strong": "#C8C8C8",
  "--color-text": "#111111",
  "--color-text-muted": "#6B6B6B",
  "--color-text-subtle": "#9A9A9A",
  "--color-text-inverted": "#FFFFFF",
  "--color-accent": "#2563EB",
  "--color-accent-strong": "#1D4ED8",
  "--color-accent-soft": "rgba(37, 99, 235, 0.12)",
  "--color-accent-softer": "rgba(37, 99, 235, 0.06)",
  "--color-success": "#16A34A",
  "--color-success-soft": "rgba(22, 163, 74, 0.10)",
  "--color-danger": "#DC2626",
  "--color-danger-soft": "rgba(220, 38, 38, 0.10)",
  "--color-warning": "#CA8A04",
  "--color-warning-soft": "rgba(202, 138, 4, 0.12)",
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "transparent",
  "--scrollbar-thumb": "rgba(0, 0, 0, 0.14)",
  "--scrollbar-thumb-hover": "rgba(0, 0, 0, 0.22)",
  "--scrollbar-thumb-active": "rgba(0, 0, 0, 0.30)",
  "--scrollbar-corner": "transparent",
  "--shadow-sm": "none",
  "--shadow-md": "none",
  "--shadow-panel": "none",
  "--shadow-lift": "none",
};

export const THEMES: ThemeDefinition[] = [
  { id: "light", label: "Light", labelZh: "浅色", tokens: LIGHT_TOKENS },
  { id: "dark", label: "Dark", labelZh: "深色", tokens: DARK_TOKENS },
];

const DEFAULT_THEME_ID = "light";
const STYLE_ELEMENT_ID = "theme-styles";

function buildThemeCSS(): string {
  return THEMES.map((t) => {
    const props = Object.entries(t.tokens)
      .map(([key, value]) => `    ${key}: ${value};`)
      .join("\n");
    return `:root[data-theme="${t.id}"] {\n${props}\n}`;
  }).join("\n");
}

export function injectThemeStyles(): void {
  if (typeof document === "undefined") return;
  const existing = document.getElementById(STYLE_ELEMENT_ID);
  if (existing) return;
  const style = document.createElement("style");
  style.id = STYLE_ELEMENT_ID;
  style.textContent = buildThemeCSS();
  document.head.appendChild(style);
}

injectThemeStyles();

export function getDefaultThemeId(): string {
  return DEFAULT_THEME_ID;
}

export function getThemeById(id: string): ThemeDefinition | undefined {
  return THEMES.find((t) => t.id === id);
}

export function isValidThemeId(id: string): boolean {
  return THEMES.some((t) => t.id === id);
}
