export interface ThemeDefinition {
  id: string;
  label: string;
  labelZh: string;
  tokens: Record<string, string>;
}

const DARK_TOKENS: Record<string, string> = {
  /* Surfaces */
  "--color-bg": "#0b0c0f",
  "--color-surface": "#1c1d21",
  "--color-surface-raised": "#24252a",
  "--color-surface-sunken": "#15161a",
  "--color-sidebar-bg": "#1a1b1f",
  /* Borders */
  "--color-border": "#2a2b2f",
  "--color-border-strong": "#3a3b40",
  /* Text */
  "--color-text": "#e8e6e1",
  "--color-text-muted": "#8a8680",
  "--color-text-subtle": "#65625d",
  "--color-text-inverted": "#ffffff",
  /* Accent */
  "--color-accent": "#529cca",
  "--color-accent-strong": "#4184b0",
  "--color-accent-soft": "rgba(82, 156, 202, 0.14)",
  "--color-accent-softer": "rgba(82, 156, 202, 0.08)",
  /* Status */
  "--color-success": "#6bc2a4",
  "--color-success-soft": "rgba(107, 194, 164, 0.12)",
  "--color-danger": "#f08d79",
  "--color-danger-soft": "rgba(240, 141, 121, 0.14)",
  "--color-warning": "#f3c969",
  "--color-warning-soft": "rgba(243, 201, 105, 0.16)",
  /* Scrollbars */
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "rgba(255, 255, 255, 0.02)",
  "--scrollbar-thumb": "rgba(154, 164, 178, 0.28)",
  "--scrollbar-thumb-hover": "rgba(178, 188, 200, 0.44)",
  "--scrollbar-thumb-active": "rgba(208, 216, 226, 0.58)",
  "--scrollbar-corner": "rgba(9, 10, 13, 0.01)",
  /* Shadows */
  "--shadow-sm": "0 1px 2px rgba(0, 0, 0, 0.2)",
  "--shadow-md": "0 4px 12px rgba(0, 0, 0, 0.25)",
  "--shadow-panel": "0 12px 28px rgba(0, 0, 0, 0.28)",
  "--shadow-lift": "0 10px 32px rgba(0, 0, 0, 0.5), 0 2px 8px rgba(0, 0, 0, 0.32)",
};

const LIGHT_TOKENS: Record<string, string> = {
  /* Surfaces */
  "--color-bg": "#ffffff",
  "--color-surface": "#f4f4f4",
  "--color-surface-raised": "#ffffff",
  "--color-surface-sunken": "#ebebeb",
  "--color-sidebar-bg": "#f8f8f8",
  /* Borders */
  "--color-border": "#e0e0e0",
  "--color-border-strong": "#c4c4c4",
  /* Text */
  "--color-text": "#191919",
  "--color-text-muted": "#5d5d5d",
  "--color-text-subtle": "#8b8b8b",
  "--color-text-inverted": "#ffffff",
  /* Accent */
  "--color-accent": "#3174a8",
  "--color-accent-strong": "#256694",
  "--color-accent-soft": "rgba(49, 116, 168, 0.10)",
  "--color-accent-softer": "rgba(49, 116, 168, 0.05)",
  /* Status */
  "--color-success": "#3d9a74",
  "--color-success-soft": "rgba(61, 154, 116, 0.10)",
  "--color-danger": "#d9705c",
  "--color-danger-soft": "rgba(217, 112, 92, 0.12)",
  "--color-warning": "#d4a843",
  "--color-warning-soft": "rgba(212, 168, 67, 0.14)",
  /* Scrollbars */
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "rgba(0, 0, 0, 0.03)",
  "--scrollbar-thumb": "rgba(0, 0, 0, 0.15)",
  "--scrollbar-thumb-hover": "rgba(0, 0, 0, 0.25)",
  "--scrollbar-thumb-active": "rgba(0, 0, 0, 0.35)",
  "--scrollbar-corner": "rgba(0, 0, 0, 0.01)",
  /* Shadows */
  "--shadow-sm": "0 1px 2px rgba(0, 0, 0, 0.05)",
  "--shadow-md": "0 4px 12px rgba(0, 0, 0, 0.07)",
  "--shadow-panel": "0 12px 28px rgba(0, 0, 0, 0.10)",
  "--shadow-lift": "0 10px 32px rgba(0, 0, 0, 0.18), 0 2px 8px rgba(0, 0, 0, 0.10)",
};

const HIGH_CONTRAST_TOKENS: Record<string, string> = {
  /* Surfaces */
  "--color-bg": "#000000",
  "--color-surface": "#111111",
  "--color-surface-raised": "#1a1a1a",
  "--color-surface-sunken": "#080808",
  "--color-sidebar-bg": "#0a0a0a",
  /* Borders */
  "--color-border": "#ffffff",
  "--color-border-strong": "#ffffff",
  /* Text */
  "--color-text": "#ffffff",
  "--color-text-muted": "#ffffff",
  "--color-text-subtle": "#cccccc",
  "--color-text-inverted": "#000000",
  /* Accent — bright yellow for maximum visibility */
  "--color-accent": "#ffcc00",
  "--color-accent-strong": "#ffdd33",
  "--color-accent-soft": "rgba(255, 204, 0, 0.20)",
  "--color-accent-softer": "rgba(255, 204, 0, 0.10)",
  /* Status */
  "--color-success": "#00ff88",
  "--color-success-soft": "rgba(0, 255, 136, 0.15)",
  "--color-danger": "#ff4444",
  "--color-danger-soft": "rgba(255, 68, 68, 0.20)",
  "--color-warning": "#ffcc00",
  "--color-warning-soft": "rgba(255, 204, 0, 0.20)",
  /* Scrollbars */
  "--scrollbar-size": "8px",
  "--scrollbar-size-thin": "6px",
  "--scrollbar-track": "rgba(255, 255, 255, 0.05)",
  "--scrollbar-thumb": "rgba(255, 255, 255, 0.35)",
  "--scrollbar-thumb-hover": "rgba(255, 255, 255, 0.50)",
  "--scrollbar-thumb-active": "rgba(255, 255, 255, 0.65)",
  "--scrollbar-corner": "rgba(0, 0, 0, 0.01)",
  /* Shadows */
  "--shadow-sm": "0 0 0 1px rgba(255, 255, 255, 0.3)",
  "--shadow-md": "0 0 0 2px rgba(255, 255, 255, 0.4)",
  "--shadow-panel": "0 0 0 2px rgba(255, 255, 255, 0.5)",
  "--shadow-lift": "0 0 0 3px rgba(255, 255, 255, 0.6)",
};

const OCEAN_TOKENS: Record<string, string> = {
  /* Surfaces — user palette: #E3FDFD, #CBF1F5, #A6E3E9, #71C9CE */
  "--color-bg": "#E3FDFD",
  "--color-surface": "#CBF1F5",
  "--color-surface-raised": "#ffffff",
  "--color-surface-sunken": "#d5f0f3",
  "--color-sidebar-bg": "#daf5f7",
  /* Borders */
  "--color-border": "#A6E3E9",
  "--color-border-strong": "#71C9CE",
  /* Text */
  "--color-text": "#0d3b3f",
  "--color-text-muted": "#3d6b6f",
  "--color-text-subtle": "#71C9CE",
  "--color-text-inverted": "#ffffff",
  /* Accent — teal-based */
  "--color-accent": "#71C9CE",
  "--color-accent-strong": "#5BA8AD",
  "--color-accent-soft": "rgba(113, 201, 206, 0.18)",
  "--color-accent-softer": "rgba(113, 201, 206, 0.09)",
  /* Status */
  "--color-success": "#2d8a6e",
  "--color-success-soft": "rgba(45, 138, 110, 0.12)",
  "--color-danger": "#d4604a",
  "--color-danger-soft": "rgba(212, 96, 74, 0.12)",
  "--color-warning": "#c4942e",
  "--color-warning-soft": "rgba(196, 148, 46, 0.14)",
  /* Scrollbars */
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "rgba(13, 59, 63, 0.03)",
  "--scrollbar-thumb": "rgba(13, 59, 63, 0.15)",
  "--scrollbar-thumb-hover": "rgba(13, 59, 63, 0.25)",
  "--scrollbar-thumb-active": "rgba(13, 59, 63, 0.35)",
  "--scrollbar-corner": "rgba(13, 59, 63, 0.01)",
  /* Shadows */
  "--shadow-sm": "0 1px 2px rgba(13, 59, 63, 0.06)",
  "--shadow-md": "0 4px 12px rgba(13, 59, 63, 0.08)",
  "--shadow-panel": "0 12px 28px rgba(13, 59, 63, 0.12)",
  "--shadow-lift": "0 10px 32px rgba(13, 59, 63, 0.20), 0 2px 8px rgba(13, 59, 63, 0.12)",
};

const MEADOW_TOKENS: Record<string, string> = {
  /* Surfaces — palette: #FDF6ED, #DCCFC0, #A1BC98, #778873 */
  "--color-bg": "#FDF6ED",
  "--color-surface": "#f5ede3",
  "--color-surface-raised": "#ffffff",
  "--color-surface-sunken": "#ede3d8",
  "--color-sidebar-bg": "#f7f1e9",
  /* Borders */
  "--color-border": "#DCCFC0",
  "--color-border-strong": "#b8a898",
  /* Text */
  "--color-text": "#2d2a25",
  "--color-text-muted": "#6b5f52",
  "--color-text-subtle": "#9b8e80",
  "--color-text-inverted": "#ffffff",
  /* Accent — sage green */
  "--color-accent": "#A1BC98",
  "--color-accent-strong": "#778873",
  "--color-accent-soft": "rgba(161, 188, 152, 0.18)",
  "--color-accent-softer": "rgba(161, 188, 152, 0.09)",
  /* Status */
  "--color-success": "#778873",
  "--color-success-soft": "rgba(119, 136, 115, 0.12)",
  "--color-danger": "#c9705e",
  "--color-danger-soft": "rgba(201, 112, 94, 0.12)",
  "--color-warning": "#b8953a",
  "--color-warning-soft": "rgba(184, 149, 58, 0.14)",
  /* Scrollbars */
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "rgba(45, 42, 37, 0.03)",
  "--scrollbar-thumb": "rgba(45, 42, 37, 0.14)",
  "--scrollbar-thumb-hover": "rgba(45, 42, 37, 0.24)",
  "--scrollbar-thumb-active": "rgba(45, 42, 37, 0.34)",
  "--scrollbar-corner": "rgba(45, 42, 37, 0.01)",
  /* Shadows */
  "--shadow-sm": "0 1px 2px rgba(45, 42, 37, 0.05)",
  "--shadow-md": "0 4px 12px rgba(45, 42, 37, 0.07)",
  "--shadow-panel": "0 12px 28px rgba(45, 42, 37, 0.10)",
  "--shadow-lift": "0 10px 32px rgba(45, 42, 37, 0.16), 0 2px 8px rgba(45, 42, 37, 0.10)",
};

export const THEMES: ThemeDefinition[] = [
  { id: "dark", label: "Dark", labelZh: "深色", tokens: DARK_TOKENS },
  { id: "light", label: "Light", labelZh: "浅色", tokens: LIGHT_TOKENS },
  { id: "high-contrast", label: "High Contrast", labelZh: "高对比度", tokens: HIGH_CONTRAST_TOKENS },
  { id: "ocean", label: "Ocean", labelZh: "海洋", tokens: OCEAN_TOKENS },
  { id: "meadow", label: "Meadow", labelZh: "草甸", tokens: MEADOW_TOKENS },
];

const DEFAULT_THEME_ID = "dark";
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

// Inject at module load so the CSS is ready before the first paint
// (the inline script in index.html sets data-theme synchronously).
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
