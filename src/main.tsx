import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { HashRouter } from "react-router-dom";
import App from "./App";
import { ThemeProvider, useAppTheme } from "./context/ThemeContext";
import { themePresets } from "./lib/theme";

import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "@mantine/charts/styles.css";
import "@mantine/dates/styles.css";
import "./App.css";

function ThemedMantineProvider({ children }: { children: React.ReactNode }) {
  const { themeId } = useAppTheme();
  const preset = themePresets[themeId];
  return (
    <MantineProvider theme={preset.mantineTheme} forceColorScheme={preset.colorScheme}>
      {children}
    </MantineProvider>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <ThemedMantineProvider>
        <Notifications />
        <HashRouter>
          <App />
        </HashRouter>
      </ThemedMantineProvider>
    </ThemeProvider>
  </React.StrictMode>
);
