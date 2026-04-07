import { useEffect, useRef, useState } from "react";
import { notifications } from "@mantine/notifications";
import { showEscalationWindow } from "../lib/commands";

export function useEscalationPreview() {
  const [activePreview, setActivePreview] = useState<string | null>(null);
  const previewTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearPreview = () => {
    if (previewTimer.current) {
      clearTimeout(previewTimer.current);
      previewTimer.current = null;
    }
    if (activePreview && activePreview !== "Level1") {
      showEscalationWindow("None").catch(() => {});
    }
    setActivePreview(null);
  };

  const handlePreview = (level: string) => {
    clearPreview();
    setActivePreview(level);

    if (level === "Level1") {
      notifications.show({
        title: "Escalation — Level 1",
        message: "This is what a Level 1 toast notification looks like.",
        color: "yellow",
        autoClose: 5000,
      });
      previewTimer.current = setTimeout(() => setActivePreview(null), 5000);
    } else {
      showEscalationWindow(level).catch(() => {});
      previewTimer.current = setTimeout(() => {
        showEscalationWindow("None").catch(() => {});
        setActivePreview(null);
      }, 8000);
    }
  };

  useEffect(() => {
    return () => {
      if (previewTimer.current) clearTimeout(previewTimer.current);
    };
  }, []);

  return { activePreview, handlePreview, clearPreview };
}
