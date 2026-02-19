import { createTheme, Card, AppShell, Modal } from "@mantine/core";

export type AppThemeId = "glass-dark" | "glass-light" | "solid-minimal";

export interface ThemePreset {
  id: AppThemeId;
  label: string;
  colorScheme: "dark" | "light";
  mantineTheme: ReturnType<typeof createTheme>;
  cssClass: string;
}

const glassComponents = {
  Card: Card.extend({
    styles: (_theme: unknown, props: { withBorder?: boolean }) => ({
      root: {
        background: "var(--glass-bg)",
        backdropFilter: "var(--glass-blur)",
        WebkitBackdropFilter: "var(--glass-blur)",
        border: "1px solid var(--glass-border)",
        boxShadow: "var(--glass-shadow)",
        ...(props.withBorder && { borderColor: "var(--glass-border)" }),
      },
    }),
  }),
  AppShell: AppShell.extend({
    styles: () => ({
      navbar: {
        background: "var(--nav-bg)",
        backdropFilter: "var(--glass-blur)",
        WebkitBackdropFilter: "var(--glass-blur)",
        borderRight: "1px solid var(--glass-border)",
      },
    }),
  }),
  Modal: Modal.extend({
    styles: () => ({
      content: {
        background: "var(--glass-bg)",
        backdropFilter: "var(--glass-blur)",
        WebkitBackdropFilter: "var(--glass-blur)",
        border: "1px solid var(--glass-border)",
        boxShadow: "var(--glass-shadow)",
      },
      overlay: {
        backdropFilter: "blur(4px)",
      },
    }),
  }),
};

const glassDarkTheme = createTheme({
  // Replace Mantine's default grey dark palette with blue-tinted shades
  // so inputs, buttons, and surfaces match the navy background
  colors: {
    dark: [
      "#c5cfe8", // [0] lightest — primary text
      "#a8b8d8", // [1]
      "#8a9fc0", // [2]
      "#5a6e8a", // [3]
      "#3a4f6e", // [4] borders
      "#2a3f5e", // [5]
      "#1e3050", // [6] input / surface backgrounds
      "#142240", // [7] deeper surface / navbar
      "#0e1a33", // [8]
      "#081228", // [9] darkest
    ],
  },
  components: glassComponents,
});

const glassLightTheme = createTheme({
  components: glassComponents,
});

export const themePresets: Record<AppThemeId, ThemePreset> = {
  "glass-dark": {
    id: "glass-dark",
    label: "Glass Dark",
    colorScheme: "dark",
    mantineTheme: glassDarkTheme,
    cssClass: "theme-glass-dark",
  },
  "glass-light": {
    id: "glass-light",
    label: "Glass Light",
    colorScheme: "light",
    mantineTheme: glassLightTheme,
    cssClass: "theme-glass-light",
  },
  "solid-minimal": {
    id: "solid-minimal",
    label: "Solid Minimal",
    colorScheme: "dark",
    mantineTheme: glassDarkTheme,
    cssClass: "theme-solid-minimal",
  },
};

export const DEFAULT_THEME_ID: AppThemeId = "glass-dark";
