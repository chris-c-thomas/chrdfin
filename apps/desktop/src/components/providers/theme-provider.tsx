import { invoke } from "@tauri-apps/api/core";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type JSX,
  type ReactNode,
} from "react";

export type Theme = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

interface ThemeContextValue {
  theme: Theme;
  resolvedTheme: ResolvedTheme;
  setTheme: (theme: Theme) => void;
  isLoading: boolean;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "dark";
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function resolveTheme(theme: Theme): ResolvedTheme {
  return theme === "system" ? getSystemTheme() : theme;
}

function applyDomTheme(resolved: ResolvedTheme): void {
  const html = document.documentElement;
  if (resolved === "dark") html.classList.add("dark");
  else html.classList.remove("dark");
}

function isTheme(value: unknown): value is Theme {
  return value === "light" || value === "dark" || value === "system";
}

export interface ThemeProviderProps {
  children: ReactNode;
  defaultTheme?: Theme;
}

/**
 * Theme state provider.
 *
 * Persists via Tauri commands `get_theme` / `set_theme` (DuckDB
 * `app_settings` table). No localStorage. Tauri commands not yet
 * registered fall through silently — the app renders the default theme.
 */
export function ThemeProvider({
  children,
  defaultTheme = "dark",
}: ThemeProviderProps): JSX.Element {
  const [theme, setThemeState] = useState<Theme>(defaultTheme);
  const [isLoading, setIsLoading] = useState(true);

  // Initial read from DuckDB (best-effort).
  useEffect(() => {
    let cancelled = false;
    invoke<unknown>("get_theme")
      .then((stored) => {
        if (cancelled) return;
        if (isTheme(stored)) setThemeState(stored);
      })
      .catch(() => {
        // Command not yet registered or DB unavailable — keep default.
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Apply DOM class whenever the resolved theme changes.
  const resolvedTheme = resolveTheme(theme);
  useEffect(() => {
    applyDomTheme(resolvedTheme);
  }, [resolvedTheme]);

  // Re-resolve on system preference change in "system" mode.
  useEffect(() => {
    if (theme !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: light)");
    const handler = () => applyDomTheme(getSystemTheme());
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  const setTheme = useCallback((next: Theme) => {
    setThemeState(next);
    invoke("set_theme", { theme: next }).catch(() => {
      // Persistence failure is non-fatal; UI still reflects the new theme.
    });
  }, []);

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme, isLoading }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used inside <ThemeProvider>");
  return ctx;
}
