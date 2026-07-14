import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import {
  type ThemeDefinition,
  getDefaultThemeId,
  getThemeById,
  isValidThemeId,
} from "./themes";

const STORAGE_KEY = "skill-manager-theme";

function readStoredTheme(): string {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value && isValidThemeId(value)) {
      return value;
    }
  } catch {
    // localStorage unavailable — use default
  }
  return getDefaultThemeId();
}

function applyTheme(themeId: string): void {
  document.documentElement.setAttribute("data-theme", themeId);
  try {
    localStorage.setItem(STORAGE_KEY, themeId);
  } catch {
    // localStorage unavailable — ignore
  }
}

interface ThemeContextValue {
  theme: string;
  currentTheme: ThemeDefinition;
  setTheme: (themeId: string) => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<string>(() => readStoredTheme());

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  const setTheme = useCallback((next: string) => {
    if (isValidThemeId(next)) {
      setThemeState(next);
    }
  }, []);

  const currentTheme = useMemo(
    () => getThemeById(theme) ?? getThemeById(getDefaultThemeId())!,
    [theme],
  );

  const value = useMemo(
    () => ({ theme, currentTheme, setTheme }),
    [theme, currentTheme, setTheme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return ctx;
}
