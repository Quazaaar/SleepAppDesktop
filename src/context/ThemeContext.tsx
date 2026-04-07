import React, { createContext, useContext, useEffect, useState } from "react";
import { load } from "@tauri-apps/plugin-store";
import { type AppThemeId, DEFAULT_THEME_ID, themePresets } from "../lib/theme";

export interface EscalationBgConfig {
  image: string;
  zoom: number;
  posX: number;
  posY: number;
}

export type EscalationBgs = Record<string, EscalationBgConfig>;

interface ThemeContextValue {
  themeId: AppThemeId;
  setTheme: (id: AppThemeId) => Promise<void>;
  escalationBgs: EscalationBgs;
  setEscalationBg: (level: string, config: EscalationBgConfig) => Promise<void>;
  clearEscalationBg: (level: string) => Promise<void>;
}

const ThemeContext = createContext<ThemeContextValue>({
  themeId: DEFAULT_THEME_ID,
  setTheme: async () => {},
  escalationBgs: {},
  setEscalationBg: async () => {},
  clearEscalationBg: async () => {},
});

export function useAppTheme() {
  return useContext(ThemeContext);
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [themeId, setThemeId] = useState<AppThemeId>(DEFAULT_THEME_ID);
  const [escalationBgs, setEscalationBgs] = useState<EscalationBgs>({});
  const [storeReady, setStoreReady] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json", { autoSave: true, defaults: {} });
        const saved = await store.get<AppThemeId>("app_theme");
        if (saved && saved in themePresets) {
          setThemeId(saved);
        }
        const bgs = await store.get<EscalationBgs>("escalation_bgs");
        if (bgs) {
          setEscalationBgs(bgs);
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

  useEffect(() => {
    const root = document.getElementById("root");
    if (!root) return;
    const appBg = escalationBgs["App"];
    if (appBg) {
      root.style.setProperty("--bg-custom-url", `url(${appBg.image})`);
      root.style.setProperty("--bg-custom-size", `${appBg.zoom * 100}%`);
      root.style.setProperty("--bg-custom-pos", `${appBg.posX}% ${appBg.posY}%`);
    } else {
      root.style.removeProperty("--bg-custom-url");
      root.style.removeProperty("--bg-custom-size");
      root.style.removeProperty("--bg-custom-pos");
    }
  }, [escalationBgs]);

  const setTheme = async (id: AppThemeId) => {
    setThemeId(id);
    try {
      const store = await load("settings.json", { autoSave: true, defaults: {} });
      await store.set("app_theme", id);
    } catch {
      // ignore
    }
  };

  const persistBgs = async (updated: EscalationBgs) => {
    try {
      const store = await load("settings.json", { autoSave: true, defaults: {} });
      await store.set("escalation_bgs", updated);
    } catch {
      // ignore
    }
  };

  const setEscalationBg = async (level: string, config: EscalationBgConfig) => {
    const updated = { ...escalationBgs, [level]: config };
    setEscalationBgs(updated);
    await persistBgs(updated);
  };

  const clearEscalationBg = async (level: string) => {
    const updated = { ...escalationBgs };
    delete updated[level];
    setEscalationBgs(updated);
    await persistBgs(updated);
  };

  if (!storeReady) return null;

  return (
    <ThemeContext.Provider value={{ themeId, setTheme, escalationBgs, setEscalationBg, clearEscalationBg }}>
      {children}
    </ThemeContext.Provider>
  );
}
