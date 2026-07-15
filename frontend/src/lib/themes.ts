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
  WCAG_AAA_LARGE,
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

/** Top 10 palettes from https://colorhunt.co/palettes/popular (July 2026). */
const COLORHUNT_THEMES: ThemeDefinition[] = [
  {
    id: "blush-rose",
    label: "Blush Rose",
    labelZh: "腮红玫瑰",
    category: "colorhunt",
    palette: ["#FBEFEF", "#FFE2E2", "#F5CBCB", "#C5B3D3"],
    tokens: composeTokens(
      {
        "--color-bg": "#FBEFEF",
        "--color-surface": "#FFFFFF",
        "--color-surface-raised": "#FFFFFF",
        "--color-surface-sunken": "#FFE2E2",
        "--color-sidebar-bg": "#FBEFEF",
        "--color-border": "#AB8181",
        "--color-border-strong": "#9A7373",
        "--color-text": "#3A2D38",
        "--color-text-muted": "#604B5B",
        "--color-text-subtle": "#A894A0",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#6B4B88",
        "--color-accent-strong": "#5A3D75",
        "--color-accent-soft": rgba("#6B4B88", 0.14),
        "--color-accent-softer": rgba("#6B4B88", 0.07),
      },
      false,
    ),
  },
  {
    id: "navy-earth",
    label: "Navy Earth",
    labelZh: "海军大地",
    category: "colorhunt",
    palette: ["#0A2947", "#F3E4C9", "#D3D4C0", "#8B5E3C"],
    tokens: composeTokens(
      {
        "--color-bg": "#0A2947",
        "--color-surface": "#123456",
        "--color-surface-raised": "#1A3D63",
        "--color-surface-sunken": "#081F35",
        "--color-sidebar-bg": "#0A2947",
        "--color-border": "#688AAA",
        "--color-border-strong": "#7E9CBE",
        "--color-text": "#F3E4C9",
        "--color-text-muted": "#D3D4C0",
        "--color-text-subtle": "#A8A898",
        "--color-text-inverted": "#0A2947",
        "--color-accent": "#D7AA88",
        "--color-accent-strong": "#E8BE9C",
        "--color-accent-soft": rgba("#D7AA88", 0.18),
        "--color-accent-softer": rgba("#D7AA88", 0.09),
      },
      true,
    ),
  },
  {
    id: "ocean-depths",
    label: "Ocean Depths",
    labelZh: "深海蓝",
    category: "colorhunt",
    palette: ["#293681", "#4274D9", "#95CCDD", "#D0E7E6"],
    tokens: composeTokens(
      {
        "--color-bg": "#D0E7E6",
        "--color-surface": "#E8F4F3",
        "--color-surface-raised": "#F0F9F8",
        "--color-surface-sunken": "#B8D9D8",
        "--color-sidebar-bg": "#D0E7E6",
        "--color-border": "#518899",
        "--color-border-strong": "#4A7F90",
        "--color-text": "#1A2456",
        "--color-text-muted": "#334676",
        "--color-text-subtle": "#6A7A9E",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#2052B7",
        "--color-accent-strong": "#1A459E",
        "--color-accent-soft": rgba("#2052B7", 0.14),
        "--color-accent-softer": rgba("#2052B7", 0.07),
      },
      false,
    ),
  },
  {
    id: "sage-grove",
    label: "Sage Grove",
    labelZh: "鼠尾草绿",
    category: "colorhunt",
    palette: ["#659287", "#88BDA4", "#B1D3B9", "#E6F2DD"],
    tokens: composeTokens(
      {
        "--color-bg": "#E6F2DD",
        "--color-surface": "#F2F9EE",
        "--color-surface-raised": "#F8FCF6",
        "--color-surface-sunken": "#D4E8CA",
        "--color-sidebar-bg": "#E6F2DD",
        "--color-border": "#6F9177",
        "--color-border-strong": "#5E8066",
        "--color-text": "#2A4038",
        "--color-text-muted": "#375548",
        "--color-text-subtle": "#7A9588",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#226350",
        "--color-accent-strong": "#1A5242",
        "--color-accent-soft": rgba("#226350", 0.14),
        "--color-accent-softer": rgba("#226350", 0.07),
      },
      false,
    ),
  },
  {
    id: "earthy-sage",
    label: "Earthy Sage",
    labelZh: "大地鼠尾草",
    category: "colorhunt",
    palette: ["#778873", "#A1BC98", "#DCCFC0", "#FDF6ED"],
    tokens: composeTokens(
      {
        "--color-bg": "#FDF6ED",
        "--color-surface": "#FFFFFF",
        "--color-surface-raised": "#FFFFFF",
        "--color-surface-sunken": "#F0E8DC",
        "--color-sidebar-bg": "#FDF6ED",
        "--color-border": "#9A8D7E",
        "--color-border-strong": "#867A6C",
        "--color-text": "#2E3828",
        "--color-text-muted": "#4A5840",
        "--color-text-subtle": "#8A9680",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#44603C",
        "--color-accent-strong": "#364E30",
        "--color-accent-soft": rgba("#44603C", 0.14),
        "--color-accent-softer": rgba("#44603C", 0.07),
      },
      false,
    ),
  },
  {
    id: "cherry-cream",
    label: "Cherry Cream",
    labelZh: "樱桃奶油",
    category: "colorhunt",
    palette: ["#FFFAF3", "#FFF2DB", "#FFE5BF", "#F62440"],
    tokens: composeTokens(
      {
        "--color-bg": "#FFFAF3",
        "--color-surface": "#FFFFFF",
        "--color-surface-raised": "#FFFFFF",
        "--color-surface-sunken": "#FFF2DB",
        "--color-sidebar-bg": "#FFFAF3",
        "--color-border": "#A78D67",
        "--color-border-strong": "#947A56",
        "--color-text": "#2C1818",
        "--color-text-muted": "#6B4545",
        "--color-text-subtle": "#9A7070",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#A80E24",
        "--color-accent-strong": "#8E0C1F",
        "--color-accent-soft": rgba("#A80E24", 0.12),
        "--color-accent-softer": rgba("#A80E24", 0.06),
      },
      false,
    ),
  },
  {
    id: "olive-harvest",
    label: "Olive Harvest",
    labelZh: "橄榄丰收",
    category: "colorhunt",
    palette: ["#FFEED6", "#A5AF79", "#827148", "#E8A07C"],
    tokens: composeTokens(
      {
        "--color-bg": "#FFEED6",
        "--color-surface": "#FFF6EC",
        "--color-surface-raised": "#FFFAF5",
        "--color-surface-sunken": "#F5E0C0",
        "--color-sidebar-bg": "#FFEED6",
        "--color-border": "#98886C",
        "--color-border-strong": "#82765E",
        "--color-text": "#3D3428",
        "--color-text-muted": "#5B4E38",
        "--color-text-subtle": "#8A7D68",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#68572E",
        "--color-accent-strong": "#544622",
        "--color-accent-soft": rgba("#68572E", 0.14),
        "--color-accent-softer": rgba("#68572E", 0.07),
      },
      false,
    ),
  },
  {
    id: "berry-sunset",
    label: "Berry Sunset",
    labelZh: "浆果日落",
    category: "colorhunt",
    palette: ["#5E244E", "#AA1C41", "#E68457", "#FFE8B4"],
    tokens: composeTokens(
      {
        "--color-bg": "#5E244E",
        "--color-surface": "#6E2E5C",
        "--color-surface-raised": "#7E386A",
        "--color-surface-sunken": "#4E1A40",
        "--color-sidebar-bg": "#5E244E",
        "--color-border": "#D888AE",
        "--color-border-strong": "#E8A0BE",
        "--color-text": "#FFF2BE",
        "--color-text-muted": "#FFF2C2",
        "--color-text-subtle": "#C4A878",
        "--color-text-inverted": "#5E244E",
        "--color-accent": "#FFBE91",
        "--color-accent-strong": "#FFD0AB",
        "--color-accent-soft": rgba("#FFBE91", 0.16),
        "--color-accent-softer": rgba("#FFBE91", 0.08),
      },
      true,
    ),
  },
  {
    id: "midnight-flame",
    label: "Midnight Flame",
    labelZh: "午夜火焰",
    category: "colorhunt",
    palette: ["#000000", "#233D4D", "#FE7F2D", "#EAECF0"],
    tokens: composeTokens(
      {
        "--color-bg": "#1A2830",
        "--color-surface": "#233D4D",
        "--color-surface-raised": "#2E5060",
        "--color-surface-sunken": "#122028",
        "--color-sidebar-bg": "#1A2830",
        "--color-border": "#829DB0",
        "--color-border-strong": "#96B0C2",
        "--color-text": "#EAECF0",
        "--color-text-muted": "#E4E8F0",
        "--color-text-subtle": "#889098",
        "--color-text-inverted": "#1A2830",
        "--color-accent": "#FFA957",
        "--color-accent-strong": "#FFBD73",
        "--color-accent-soft": rgba("#FFA957", 0.16),
        "--color-accent-softer": rgba("#FFA957", 0.08),
      },
      true,
    ),
  },
  {
    id: "forest-gold",
    label: "Forest Gold",
    labelZh: "森林金",
    category: "colorhunt",
    palette: ["#FFBF00", "#FFF78D", "#467235", "#283F24"],
    tokens: composeTokens(
      {
        "--color-bg": "#283F24",
        "--color-surface": "#354F2E",
        "--color-surface-raised": "#426038",
        "--color-surface-sunken": "#1E301A",
        "--color-sidebar-bg": "#283F24",
        "--color-border": "#8AB679",
        "--color-border-strong": "#A0CC8E",
        "--color-text": "#FFFFEB",
        "--color-text-muted": "#FFFFEA",
        "--color-text-subtle": "#A8AC60",
        "--color-text-inverted": "#283F24",
        "--color-accent": "#FFC708",
        "--color-accent-strong": "#FFD633",
        "--color-accent-soft": rgba("#FFC708", 0.16),
        "--color-accent-softer": rgba("#FFC708", 0.08),
      },
      true,
    ),
  },
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
