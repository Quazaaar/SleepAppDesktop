import React, { createContext, useContext, useEffect, useState } from "react";
import { load } from "@tauri-apps/plugin-store";
import { type AppThemeId, DEFAULT_THEME_ID, themePresets } from "../lib/theme";

interface ThemeContextValue {
  themeId: AppThemeId;
  setTheme: (id: AppThemeId) => Promise<void>;
}

const ThemeContext = createContext<ThemeContextValue>({
  themeId: DEFAULT_THEME_ID,
  setTheme: async () => {},
});

export function useAppTheme() {
  return useContext(ThemeContext);
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [themeId, setThemeId] = useState<AppThemeId>(DEFAULT_THEME_ID);
  const [storeReady, setStoreReady] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json", { autoSave: true, defaults: {} });
        const saved = await store.get<AppThemeId>("app_theme");
        if (saved && saved in themePresets) {
          setThemeId(saved);
        }
      } catch {
        // Tauri not available (web-only dev) or store error — use default
      }
      setStoreReady(true);
    })();
  }, []);

  useEffect(() => {
    const root = document.getElementById("root");
    if (!root) return;
    Object.values(themePresets).forEach((preset) => {
      root.classList.remove(preset.cssClass);
    });
    root.classList.add(themePresets[themeId].cssClass);
  }, [themeId]);

  const setTheme = async (id: AppThemeId) => {
    setThemeId(id);
    try {
      const store = await load("settings.json", { autoSave: true, defaults: {} });
      await store.set("app_theme", id);
    } catch {
      // ignore
    }
  };

  if (!storeReady) return null;

  return (
    <ThemeContext.Provider value={{ themeId, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}
