import { Card, Text, Stack } from "@mantine/core";
import { useEscalationState } from "../../hooks/useEscalationState";

/**
 * Level 2 popup overlay — rendered in the small floating "escalation-popup"
 * window (320x140, no decorations, always-on-top).
 *
 * No dismiss button: the window is closed automatically when the escalation
 * engine advances to Level 3.
 */
export default function PopupOverlay() {
  const { message } = useEscalationState();

  return (
    <Card
      padding="sm"
      radius="md"
      data-tauri-drag-region
      style={{
        background: "rgba(20, 20, 30, 0.97)",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        height: "100vh",
        display: "flex",
        alignItems: "center",
        cursor: "grab",
      }}
    >
      <Stack gap={4}>
        <Text size="sm" fw={600} c="yellow.4">
          Still going?
        </Text>
        <Text size="xs" c="dimmed" style={{ lineHeight: 1.4 }}>
          {message || "Consider wrapping up soon."}
        </Text>
      </Stack>
    </Card>
  );
}
