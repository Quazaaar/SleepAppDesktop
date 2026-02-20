import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { EscalationLevel, EscalationStatePayload } from "../lib/types";

export function useEscalationState() {
  const [state, setState] = useState<EscalationStatePayload>({
    level: "None" as EscalationLevel,
    message: "",
  });

  useEffect(() => {
    const unlisten = listen<EscalationStatePayload>(
      "escalation-state-changed",
      (event) => setState(event.payload)
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return state;
}
