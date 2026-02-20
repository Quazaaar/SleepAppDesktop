import { useState } from "react";
import { Card, Stack, Title, Text, Textarea, Button } from "@mantine/core";
import { motion } from "framer-motion";
import { useEscalationState } from "../../hooks/useEscalationState";
import { dismissEscalation } from "../../lib/commands";

/**
 * Level 4 fullscreen overlay — rendered in the maximized transparent
 * "escalation-fullscreen" window (always-on-top, no decorations).
 *
 * Dismissal IS the wrap-up notes form submission. Submitting the notes
 * calls dismissEscalation() which:
 *   1. Sets escalation engine to Done (no more reminders tonight).
 *   2. Closes the fullscreen window.
 *
 * Note: Note persistence is deferred to Phase 3 (SQLite wiring).
 */
export default function FullscreenOverlay() {
  const { message } = useEscalationState();
  const [workingOn, setWorkingOn] = useState("");
  const [nextSteps, setNextSteps] = useState("");
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit() {
    setSubmitting(true);
    // Phase 3 will persist these to SQLite.
    console.log("[FullscreenOverlay] Wrap-up notes:", { workingOn, nextSteps });
    try {
      await dismissEscalation();
    } catch (err) {
      console.error("[FullscreenOverlay] dismissEscalation failed:", err);
      setSubmitting(false);
    }
  }

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3, ease: "easeOut" }}
      style={{
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(0, 0, 0, 0.88)",
        padding: "2rem",
      }}
    >
      <Card
        padding="xl"
        radius="lg"
        style={{
          background: "rgba(18, 18, 28, 0.98)",
          border: "1px solid rgba(255, 255, 255, 0.12)",
          width: "100%",
          maxWidth: 560,
          boxShadow: "0 24px 48px rgba(0,0,0,0.6)",
        }}
      >
        <Stack gap="lg">
          <div>
            <Title order={2} c="white" mb={4}>
              Time to stop for tonight
            </Title>
            <Text c="dimmed" size="sm">
              {message || "Write your notes and call it a night."}
            </Text>
          </div>

          <div
            style={{
              height: 2,
              borderRadius: 1,
              background: "linear-gradient(90deg, #ffd43b 0%, #ff6b35 100%)",
            }}
          />

          <Textarea
            label="What were you working on?"
            placeholder="Capture where you left off so you can pick up tomorrow..."
            value={workingOn}
            onChange={(e) => setWorkingOn(e.currentTarget.value)}
            minRows={3}
            autosize
          />

          <Textarea
            label="What's next?"
            placeholder="What should you start with tomorrow?"
            value={nextSteps}
            onChange={(e) => setNextSteps(e.currentTarget.value)}
            minRows={3}
            autosize
          />

          <Button
            size="lg"
            variant="filled"
            color="yellow"
            onClick={handleSubmit}
            loading={submitting}
            style={{ marginTop: 8 }}
          >
            Submit &amp; Close
          </Button>
        </Stack>
      </Card>
    </motion.div>
  );
}
