import { useState } from "react";
import { Stack, Title, Text, Textarea, Button } from "@mantine/core";
import { motion } from "framer-motion";
import { IconClock } from "@tabler/icons-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEscalationState } from "../../hooks/useEscalationState";
import { saveWrapUpNote } from "../../lib/commands";
import { useAppTheme } from "../../context/ThemeContext";

/**
 * Level 3 side panel overlay — rendered in the tall "escalation-panel" window
 * that covers ~30% of the right side of the screen (no decorations, always-on-top).
 *
 * Slides in from the right using framer-motion. Shows a wrap-up note form below
 * the escalation warning. Submitting closes the panel but does NOT dismiss the
 * escalation — Level 4 (Fullscreen) will still fire. This is critical: calling
 * dismissEscalation() here would stop the escalation cycle prematurely.
 *
 * Empty submit closes the window without saving a note record.
 */
export default function PanelOverlay() {
  const { message } = useEscalationState();
  const { escalationBgs } = useAppTheme();
  const bg = escalationBgs["Level3"];
  const [workingOn, setWorkingOn] = useState("");
  const [nextSteps, setNextSteps] = useState("");
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit() {
    setSubmitting(true);
    try {
      if (workingOn.trim() || nextSteps.trim()) {
        await saveWrapUpNote(workingOn.trim(), nextSteps.trim());
      }
      // Close ONLY this panel window — do NOT call dismissEscalation().
      // L4 escalation must still fire after panel closes.
      await getCurrentWindow().close();
    } catch (err) {
      console.error("[PanelOverlay] submit failed:", err);
      setSubmitting(false);
    }
  }

  return (
    <motion.div
      initial={{ x: "100%" }}
      animate={{ x: 0 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      style={{
        height: "100vh",
        background: "rgba(0, 0, 0, 0.92)",
        borderLeft: "1px solid rgba(255, 200, 0, 0.3)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "2rem",
        ...(bg && {
          backgroundImage: `url(${bg.image})`,
          backgroundSize: `${bg.zoom * 100}%`,
          backgroundPosition: `${bg.posX}% ${bg.posY}%`,
          backgroundRepeat: "no-repeat",
        }),
      }}
    >
      <Stack align="stretch" gap="lg" style={{ maxWidth: 320, width: "100%" }}>
        <Stack align="center" gap="sm">
          <IconClock size={48} color="#ffd43b" stroke={1.5} />
          <Title order={2} ta="center" c="white" style={{ lineHeight: 1.2 }}>
            Time to Wrap Up
          </Title>
          <Text ta="center" c="dimmed" size="sm" style={{ lineHeight: 1.6 }}>
            {message || "You've been working late. Start your wrap-up."}
          </Text>
        </Stack>

        <div
          style={{
            width: "100%",
            height: 4,
            borderRadius: 2,
            background: "linear-gradient(90deg, #ffd43b 0%, #ff6b35 100%)",
          }}
        />

        <Textarea
          label="What were you working on?"
          placeholder="Capture where you left off..."
          value={workingOn}
          onChange={(e) => setWorkingOn(e.currentTarget.value)}
          minRows={2}
          autosize
          styles={{
            label: { color: "rgba(255,255,255,0.7)" },
          }}
        />

        <Textarea
          label="What's next?"
          placeholder="What will you start with tomorrow?"
          value={nextSteps}
          onChange={(e) => setNextSteps(e.currentTarget.value)}
          minRows={2}
          autosize
          styles={{
            label: { color: "rgba(255,255,255,0.7)" },
          }}
        />

        <Button
          variant="filled"
          color="yellow"
          onClick={handleSubmit}
          loading={submitting}
        >
          Submit &amp; Close
        </Button>
      </Stack>
    </motion.div>
  );
}
