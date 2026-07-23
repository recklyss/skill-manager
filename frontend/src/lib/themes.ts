/**
 * Theme definitions and CSS token injection.
 *
 * WCAG 2.2 AAA contrast targets for token pairs:
 * - `--color-text`, `--color-text-muted` on `--color-bg` | `--color-surface` | `--color-surface-raised`: 7:1 (normal text)
 * - `--color-accent` on backgrounds: 4.5:1 (large text / links)
 * - `--color-text-inverted` on `--color-accent`: 7:1 (primary button labels)
 * - `--color-border` on adjacent surfaces: 3:1 (UI component visibility)
 * - `--color-success` | `--color-danger` | `--color-warning` on surfaces: 4.5:1 (semantic large text)
 *
 * Validated by `themes.contrast.test.ts`.
 */
import {
  adjustForSurfaces,
  contrastRatio,
  nudgeForContrast,
  WCAG_AAA_LARGE,
  WCAG_AAA_NORMAL,
  WCAG_AAA_UI,
} from "./contrast";

export type ThemeCategory = "default" | "colorhunt";

export interface ThemeDefinition {
  id: string;
  label: string;
  labelZh: string;
  category: ThemeCategory;
  /** Optional palette colors for multi-swatch preview in ThemeSelector. */
  palette?: string[];
  tokens: Record<string, string>;
}

const SHARED_LAYOUT_TOKENS: Record<string, string> = {
  "--scrollbar-size": "6px",
  "--scrollbar-size-thin": "4px",
  "--scrollbar-track": "transparent",
  "--scrollbar-corner": "transparent",
  "--shadow-sm": "none",
  "--shadow-md": "none",
  "--shadow-panel": "none",
  "--shadow-lift": "none",
};

function scrollbarTokens(isDark: boolean): Record<string, string> {
  if (isDark) {
    return {
      "--scrollbar-thumb": "rgba(255, 255, 255, 0.18)",
      "--scrollbar-thumb-hover": "rgba(255, 255, 255, 0.28)",
      "--scrollbar-thumb-active": "rgba(255, 255, 255, 0.38)",
    };
  }
  return {
    "--scrollbar-thumb": "rgba(0, 0, 0, 0.14)",
    "--scrollbar-thumb-hover": "rgba(0, 0, 0, 0.22)",
    "--scrollbar-thumb-active": "rgba(0, 0, 0, 0.30)",
  };
}

function semanticTokens(
  isDark: boolean,
  surfaces: string[],
): Record<string, string> {
  const bases = isDark
    ? {
        success: "#4ADE80",
        danger: "#F87171",
        warning: "#FBBF24",
        softAlpha: { success: 0.12, danger: 0.14, warning: 0.14 },
      }
    : {
        success: "#16A34A",
        danger: "#DC2626",
        warning: "#CA8A04",
        softAlpha: { success: 0.1, danger: 0.1, warning: 0.12 },
      };

  const success = adjustForSurfaces(bases.success, surfaces, WCAG_AAA_LARGE);
  const danger = adjustForSurfaces(bases.danger, surfaces, WCAG_AAA_LARGE);
  const warning = adjustForSurfaces(bases.warning, surfaces, WCAG_AAA_LARGE);

  return {
    "--color-success": success,
    "--color-success-soft": rgba(success, bases.softAlpha.success),
    "--color-danger": danger,
    "--color-danger-soft": rgba(danger, bases.softAlpha.danger),
    "--color-warning": warning,
    "--color-warning-soft": rgba(warning, bases.softAlpha.warning),
  };
}

function rgba(hex: string, alpha: number): string {
  const normalized = hex.replace("#", "");
  const r = Number.parseInt(normalized.slice(0, 2), 16);
  const g = Number.parseInt(normalized.slice(2, 4), 16);
  const b = Number.parseInt(normalized.slice(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function composeTokens(
  colors: Record<string, string>,
  isDark: boolean,
): Record<string, string> {
  const surfaces = [colors["--color-surface"], colors["--color-surface-raised"]].filter(
    (value): value is string => typeof value === "string" && value.startsWith("#"),
  );

  return {
    ...colors,
    ...semanticTokens(isDark, surfaces),
    ...scrollbarTokens(isDark),
    ...SHARED_LAYOUT_TOKENS,
  };
}

const DARK_TOKENS = composeTokens(
  {
    "--color-bg": "#111111",
    "--color-surface": "#1A1A1A",
    "--color-surface-raised": "#222222",
    "--color-surface-sunken": "#0D0D0D",
    "--color-sidebar-bg": "#111111",
    "--color-border": "#6C6C6C",
    "--color-border-strong": "#888888",
    "--color-text": "#F0F0F0",
    "--color-text-muted": "#ACACAC",
    "--color-text-subtle": "#666666",
    "--color-text-inverted": "#111111",
    "--color-accent": "#7EB3FF",
    "--color-accent-strong": "#9AC5FF",
    "--color-accent-soft": "rgba(126, 179, 255, 0.16)",
    "--color-accent-softer": "rgba(126, 179, 255, 0.08)",
  },
  true,
);

const LIGHT_TOKENS = composeTokens(
  {
    "--color-bg": "#F7F7F8",
    "--color-surface": "#FFFFFF",
    "--color-surface-raised": "#FFFFFF",
    "--color-surface-sunken": "#F0F0F1",
    "--color-sidebar-bg": "#FFFFFF",
    "--color-border": "#8F8F8F",
    "--color-border-strong": "#757575",
    "--color-text": "#111111",
    "--color-text-muted": "#535353",
    "--color-text-subtle": "#9A9A9A",
    "--color-text-inverted": "#FFFFFF",
    "--color-accent": "#1A3FA0",
    "--color-accent-strong": "#153280",
    "--color-accent-soft": "rgba(26, 63, 160, 0.12)",
    "--color-accent-softer": "rgba(26, 63, 160, 0.06)",
  },
  false,
);

/**
 * Colour helpers for deriving WCAG 2.2 AAA compliant token sets from a raw
 * palette. Each palette below supplies its natural background / surface /
 * primary hues; the builder nudges text, accent, and border tokens until they
 * satisfy the contrast targets enforced by `themes.contrast.test.ts`.
 */
function parseHex(hex: string): [number, number, number] {
  const n = hex.replace("#", "");
  return [0, 2, 4].map((i) => Number.parseInt(n.slice(i, i + 2), 16)) as [
    number,
    number,
    number,
  ];
}

function toHex(r: number, g: number, b: number): string {
  const clamp = (n: number) => Math.max(0, Math.min(255, Math.round(n)));
  return (
    "#" + [r, g, b].map((c) => clamp(c).toString(16).padStart(2, "0")).join("")
  );
}

/** Linear blend between two hex colours (`t` in 0..1 toward `b`). */
function mix(a: string, b: string, t: number): string {
  const [ar, ag, ab] = parseHex(a);
  const [br, bg, bb] = parseHex(b);
  return toHex(ar + (br - ar) * t, ag + (bg - ag) * t, ab + (bb - ab) * t);
}

/** Darken (dark themes) or lighten (light themes) a surface until `text` reads at `target`. */
function fitSurface(
  surface: string,
  text: string,
  isDark: boolean,
  target: number,
): string {
  const towards = isDark ? "#000000" : "#FFFFFF";
  let result = surface;
  for (let step = 1; step <= 20 && contrastRatio(text, result) < target; step += 1) {
    result = mix(surface, towards, step * 0.05);
  }
  return result;
}

interface ColorhuntInput {
  id: string;
  label: string;
  labelZh: string;
  isDark: boolean;
  bg: string;
  surface: string;
  primary: string;
  secondary: string;
  accentColor: string;
  /** Text override (defaults to white on dark themes, near-black on light). */
  text?: string;
}

function colorhuntTheme(input: ColorhuntInput): ThemeDefinition {
  const { isDark } = input;
  const text = input.text ?? (isDark ? "#FFFFFF" : "#1A1A1A");

  const bg = fitSurface(input.bg, text, isDark, WCAG_AAA_NORMAL);
  const surface = fitSurface(input.surface, text, isDark, WCAG_AAA_NORMAL);
  const surfaceRaised = fitSurface(
    mix(surface, "#FFFFFF", isDark ? 0.06 : 0.12),
    text,
    isDark,
    WCAG_AAA_NORMAL,
  );
  const surfaceSunken = mix(bg, "#000000", isDark ? 0.3 : 0.05);
  const surfaces = [bg, surface, surfaceRaised];

  const textMuted = adjustForSurfaces(mix(text, surface, 0.3), surfaces, WCAG_AAA_NORMAL);
  const textSubtle = mix(text, surface, 0.55);

  const accent = adjustForSurfaces(input.primary, surfaces, WCAG_AAA_LARGE);
  const accentStrong = adjustForSurfaces(accent, surfaces, 6);
  const invertedBase =
    contrastRatio("#FFFFFF", accent) >= contrastRatio("#111111", accent)
      ? "#FFFFFF"
      : "#111111";
  const textInverted = nudgeForContrast(invertedBase, accent, WCAG_AAA_NORMAL);

  const border = adjustForSurfaces(mix(text, bg, 0.55), surfaces, WCAG_AAA_UI);
  const borderStrong = adjustForSurfaces(border, surfaces, 4);

  return {
    id: input.id,
    label: input.label,
    labelZh: input.labelZh,
    category: "colorhunt",
    palette: [input.bg, input.primary, input.secondary, input.accentColor],
    tokens: composeTokens(
      {
        "--color-bg": bg,
        "--color-surface": surface,
        "--color-surface-raised": surfaceRaised,
        "--color-surface-sunken": surfaceSunken,
        "--color-sidebar-bg": bg,
        "--color-border": border,
        "--color-border-strong": borderStrong,
        "--color-text": text,
        "--color-text-muted": textMuted,
        "--color-text-subtle": textSubtle,
        "--color-text-inverted": textInverted,
        "--color-accent": accent,
        "--color-accent-strong": accentStrong,
        "--color-accent-soft": rgba(accent, isDark ? 0.16 : 0.14),
        "--color-accent-softer": rgba(accent, isDark ? 0.08 : 0.07),
      },
      isDark,
    ),
  };
}

/** Curated palettes (accent = each theme's Primary per its design-system mapping). */
const COLORHUNT_THEMES: ThemeDefinition[] = [
  colorhuntTheme({
    id: "autumn-forest",
    label: "Autumn Forest",
    labelZh: "秋日森林",
    isDark: true,
    bg: "#21250F",
    surface: "#414C2A",
    primary: "#9A6D18",
    secondary: "#725B14",
    accentColor: "#DCAE29",
  }),
  colorhuntTheme({
    id: "forest-green",
    label: "Forest Green",
    labelZh: "森林绿",
    isDark: true,
    bg: "#0E171C",
    surface: "#3C2E2B",
    primary: "#526951",
    secondary: "#7D977F",
    accentColor: "#B3C9B6",
  }),
  colorhuntTheme({
    id: "butterfly-nature",
    label: "Butterfly Nature",
    labelZh: "蝴蝶自然",
    isDark: true,
    bg: "#28241C",
    surface: "#5E8175",
    primary: "#B55A08",
    secondary: "#405740",
    accentColor: "#AAB8B4",
  }),
  colorhuntTheme({
    id: "industrial-gray",
    label: "Industrial Gray",
    labelZh: "工业灰",
    isDark: true,
    bg: "#141616",
    surface: "#3D3C3B",
    primary: "#746D67",
    secondary: "#A49F9D",
    accentColor: "#7F1D1A",
  }),
  colorhuntTheme({
    id: "wine-elegance",
    label: "Wine Elegance",
    labelZh: "酒红优雅",
    isDark: true,
    bg: "#230E0F",
    surface: "#541625",
    primary: "#910D3B",
    secondary: "#9D4060",
    accentColor: "#BC788D",
  }),
  colorhuntTheme({
    id: "tropical-bird",
    label: "Tropical Bird",
    labelZh: "热带鸟",
    isDark: true,
    bg: "#252D37",
    surface: "#D3D9DA",
    primary: "#356A8B",
    secondary: "#9A5E17",
    accentColor: "#8B150C",
  }),
  colorhuntTheme({
    id: "warm-home",
    label: "Warm Home",
    labelZh: "温暖家居",
    isDark: false,
    bg: "#E7CFB8",
    surface: "#D8BD63",
    primary: "#9B6F45",
    secondary: "#43718E",
    accentColor: "#3A2E2D",
    text: "#3A2E2D",
  }),
  colorhuntTheme({
    id: "ocean-night",
    label: "Ocean Night",
    labelZh: "海洋之夜",
    isDark: true,
    bg: "#11141B",
    surface: "#1C273E",
    primary: "#2C4972",
    secondary: "#4A74A1",
    accentColor: "#8FACCB",
  }),
  colorhuntTheme({
    id: "lavender-dream",
    label: "Lavender Dream",
    labelZh: "薰衣草之梦",
    isDark: true,
    bg: "#2D2C27",
    surface: "#594587",
    primary: "#7553C7",
    secondary: "#BCA4E9",
    accentColor: "#9D8957",
  }),
];

export const THEMES: ThemeDefinition[] = [
  { id: "light", label: "Light", labelZh: "浅色", category: "default", tokens: LIGHT_TOKENS },
  { id: "dark", label: "Dark", labelZh: "深色", category: "default", tokens: DARK_TOKENS },
  ...COLORHUNT_THEMES,
];

export const DEFAULT_THEMES = THEMES.filter((theme) => theme.category === "default");
export const COLORHUNT_THEME_OPTIONS = THEMES.filter((theme) => theme.category === "colorhunt");

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
