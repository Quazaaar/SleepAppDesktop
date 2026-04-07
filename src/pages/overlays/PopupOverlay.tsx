import { Card, Text, Stack, Group, Button } from "@mantine/core";
import { useEscalationState } from "../../hooks/useEscalationState";
import { acknowledgePopup } from "../../lib/commands";
import { useAppTheme } from "../../context/ThemeContext";

/**
 * Level 2 popup overlay — rendered in the small floating "escalation-popup"
 * window (320x140, no decorations, always-on-top).
 *
 * "Ok" button closes the popup and records the dismissal for the session.
 * The escalation engine continues ticking and may re-show or advance to Level 3.
 */
export default function PopupOverlay() {
  const { message } = useEscalationState();
  const { escalationBgs } = useAppTheme();
  const bg = escalationBgs["Level2"];

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
        ...(bg && {
          backgroundImage: `url(${bg.image})`,
          backgroundSize: `${bg.zoom * 100}%`,
          backgroundPosition: `${bg.posX}% ${bg.posY}%`,
          backgroundRepeat: "no-repeat",
        }),
      }}
    >
      <Group justify="space-between" align="center" wrap="nowrap" w="100%">
        <Stack gap={4}>
          <Text size="sm" fw={600} c="yellow.4">
            Still going?
          </Text>
          <Text size="xs" c="dimmed" style={{ lineHeight: 1.4 }}>
            {message || "Consider wrapping up soon."}
          </Text>
        </Stack>
        <Button
          size="xs"
          variant="subtle"
          color="gray"
          onClick={() => acknowledgePopup()}
          style={{ flexShrink: 0 }}
        >
          Ok
        </Button>
      </Group>
    </Card>
  );
}
