import { useEffect, useRef } from "react";
import { notifications } from "@mantine/notifications";
import type { EscalationLevel } from "../../lib/types";
import { showEscalationWindow } from "../../lib/commands";

interface Props {
  level: EscalationLevel;
  message: string;
}

/**
 * Central dispatcher for escalation level changes.
 *
 * - Level1: shows a persistent Mantine toast in the bottom-right corner.
 * - Level2/3/4: calls showEscalationWindow to open the appropriate overlay window.
 * - None/Done: closes all overlay windows.
 *
 * Returns null (no DOM output).
 */
export default function EscalationToastHandler({ level, message }: Props) {
  const toastShownRef = useRef(false);

  useEffect(() => {
    if (level === "Level1") {
      if (!toastShownRef.current) {
        notifications.show({
          id: "escalation-level1",
          title: "Time Check",
          message,
          autoClose: false,
          withCloseButton: true,
          position: "bottom-right",
          color: "yellow",
          style: { maxWidth: 320 },
        });
        toastShownRef.current = true;
      }
    } else {
      // Hide the Level1 toast when level advances away from it.
      if (toastShownRef.current) {
        notifications.hide("escalation-level1");
        toastShownRef.current = false;
      }

      // Open the appropriate overlay window for Level2/3/4, or close all for None/Done.
      if (level !== "None") {
        showEscalationWindow(level).catch(console.error);
      }
    }
  }, [level, message]);

  return null;
}
