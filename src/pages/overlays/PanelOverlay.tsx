import { Stack, Title, Text } from "@mantine/core";
import { motion } from "framer-motion";
import { IconClock } from "@tabler/icons-react";
import { useEscalationState } from "../../hooks/useEscalationState";

/**
 * Level 3 side panel overlay — rendered in the tall "escalation-panel" window
 * that covers ~30% of the right side of the screen (no decorations, always-on-top).
 *
 * Slides in from the right using framer-motion. No dismiss button: the window
 * is closed automatically when the engine advances to Level 4.
 */
export default function PanelOverlay() {
  const { message } = useEscalationState();

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
      }}
    >
      <Stack align="center" gap="lg" style={{ maxWidth: 280 }}>
        <IconClock size={48} color="#ffd43b" stroke={1.5} />
        <Title order={2} ta="center" c="white" style={{ lineHeight: 1.2 }}>
          Time to Wrap Up
        </Title>
        <Text ta="center" c="dimmed" size="sm" style={{ lineHeight: 1.6 }}>
          {message || "You've been working late. Start your wrap-up."}
        </Text>
        <div
          style={{
            width: "100%",
            height: 4,
            borderRadius: 2,
            background: "linear-gradient(90deg, #ffd43b 0%, #ff6b35 100%)",
          }}
        />
        <Text size="xs" c="yellow.5" ta="center">
          One more escalation if you stay
        </Text>
      </Stack>
    </motion.div>
  );
}
