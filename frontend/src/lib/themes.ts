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

function semanticTokens(isDark: boolean): Record<string, string> {
  if (isDark) {
    return {
      "--color-success": "#4ADE80",
      "--color-success-soft": "rgba(74, 222, 128, 0.12)",
      "--color-danger": "#F87171",
      "--color-danger-soft": "rgba(248, 113, 113, 0.14)",
      "--color-warning": "#FBBF24",
      "--color-warning-soft": "rgba(251, 191, 36, 0.14)",
    };
  }
  return {
    "--color-success": "#16A34A",
    "--color-success-soft": "rgba(22, 163, 74, 0.10)",
    "--color-danger": "#DC2626",
    "--color-danger-soft": "rgba(220, 38, 38, 0.10)",
    "--color-warning": "#CA8A04",
    "--color-warning-soft": "rgba(202, 138, 4, 0.12)",
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
  return {
    ...colors,
    ...semanticTokens(isDark),
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
        "--color-border": "#F5CBCB",
        "--color-border-strong": "#E0A8A8",
        "--color-text": "#3A2D38",
        "--color-text-muted": "#7A6575",
        "--color-text-subtle": "#A894A0",
        "--color-text-inverted": "#FBEFEF",
        "--color-accent": "#9B7BB8",
        "--color-accent-strong": "#7E5F9E",
        "--color-accent-soft": rgba("#9B7BB8", 0.14),
        "--color-accent-softer": rgba("#9B7BB8", 0.07),
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
        "--color-border": "#1E4060",
        "--color-border-strong": "#2E5578",
        "--color-text": "#F3E4C9",
        "--color-text-muted": "#D3D4C0",
        "--color-text-subtle": "#A8A898",
        "--color-text-inverted": "#0A2947",
        "--color-accent": "#8B5E3C",
        "--color-accent-strong": "#A07048",
        "--color-accent-soft": rgba("#8B5E3C", 0.18),
        "--color-accent-softer": rgba("#8B5E3C", 0.09),
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
        "--color-border": "#95CCDD",
        "--color-border-strong": "#7AB5C8",
        "--color-text": "#1A2456",
        "--color-text-muted": "#3D5080",
        "--color-text-subtle": "#6A7A9E",
        "--color-text-inverted": "#F0F9F8",
        "--color-accent": "#4274D9",
        "--color-accent-strong": "#335FC0",
        "--color-accent-soft": rgba("#4274D9", 0.14),
        "--color-accent-softer": rgba("#4274D9", 0.07),
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
        "--color-border": "#B1D3B9",
        "--color-border-strong": "#88BDA4",
        "--color-text": "#2A4038",
        "--color-text-muted": "#4D6B5E",
        "--color-text-subtle": "#7A9588",
        "--color-text-inverted": "#F8FCF6",
        "--color-accent": "#4A8B78",
        "--color-accent-strong": "#3A7262",
        "--color-accent-soft": rgba("#4A8B78", 0.14),
        "--color-accent-softer": rgba("#4A8B78", 0.07),
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
        "--color-border": "#DCCFC0",
        "--color-border-strong": "#C4B5A4",
        "--color-text": "#2E3828",
        "--color-text-muted": "#5A6850",
        "--color-text-subtle": "#8A9680",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#5E7A56",
        "--color-accent-strong": "#4A6344",
        "--color-accent-soft": rgba("#5E7A56", 0.14),
        "--color-accent-softer": rgba("#5E7A56", 0.07),
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
        "--color-border": "#FFE5BF",
        "--color-border-strong": "#E8C898",
        "--color-text": "#2C1818",
        "--color-text-muted": "#6B4545",
        "--color-text-subtle": "#9A7070",
        "--color-text-inverted": "#FFFFFF",
        "--color-accent": "#F62440",
        "--color-accent-strong": "#D41A32",
        "--color-accent-soft": rgba("#F62440", 0.12),
        "--color-accent-softer": rgba("#F62440", 0.06),
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
        "--color-border": "#D4C4A8",
        "--color-border-strong": "#A5AF79",
        "--color-text": "#3D3428",
        "--color-text-muted": "#6B5E48",
        "--color-text-subtle": "#8A7D68",
        "--color-text-inverted": "#FFF6EC",
        "--color-accent": "#827148",
        "--color-accent-strong": "#6A5A38",
        "--color-accent-soft": rgba("#827148", 0.14),
        "--color-accent-softer": rgba("#827148", 0.07),
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
        "--color-border": "#8A3A60",
        "--color-border-strong": "#AA1C41",
        "--color-text": "#FFE8B4",
        "--color-text-muted": "#E8C898",
        "--color-text-subtle": "#C4A878",
        "--color-text-inverted": "#5E244E",
        "--color-accent": "#E68457",
        "--color-accent-strong": "#F09868",
        "--color-accent-soft": rgba("#E68457", 0.16),
        "--color-accent-softer": rgba("#E68457", 0.08),
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
        "--color-border": "#3A5568",
        "--color-border-strong": "#4A6578",
        "--color-text": "#EAECF0",
        "--color-text-muted": "#B8BCC4",
        "--color-text-subtle": "#889098",
        "--color-text-inverted": "#1A2830",
        "--color-accent": "#FE7F2D",
        "--color-accent-strong": "#FF9548",
        "--color-accent-soft": rgba("#FE7F2D", 0.16),
        "--color-accent-softer": rgba("#FE7F2D", 0.08),
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
        "--color-border": "#467235",
        "--color-border-strong": "#5A8C42",
        "--color-text": "#FFF78D",
        "--color-text-muted": "#D4D878",
        "--color-text-subtle": "#A8AC60",
        "--color-text-inverted": "#283F24",
        "--color-accent": "#FFBF00",
        "--color-accent-strong": "#FFD033",
        "--color-accent-soft": rgba("#FFBF00", 0.16),
        "--color-accent-softer": rgba("#FFBF00", 0.08),
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
