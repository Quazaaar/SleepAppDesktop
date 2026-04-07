import { useEffect, useState } from "react";
import { Card, Text, Stack, Group, ActionIcon, Button } from "@mantine/core";
import { IconX, IconCopy } from "@tabler/icons-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Window } from "@tauri-apps/api/window";
import { getLatestWrapUpNote } from "../../lib/commands";
import type { WrapUpNote } from "../../lib/types";

export default function ResumeOverlay() {
  const [note, setNote] = useState<WrapUpNote | null>(null);

  useEffect(() => {
    getLatestWrapUpNote()
      .then((n) => {
        if (n) setNote(n);
        else getCurrentWindow().close();
      })
      .catch(() => getCurrentWindow().close());
  }, []);

  if (!note) return null;

  const dateLabel = new Date(note.created_at).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });

  const handleOpen = async () => {
    const main = new Window("main");
    await main.show();
    await main.setFocus();
    await getCurrentWindow().close();
  };

  const handleDismiss = () => {
    getCurrentWindow().close();
  };

  const handleCopy = async () => {
    const parts: string[] = [];
    if (note.working_on.trim()) {
      parts.push(`Working on:\n${note.working_on}`);
    }
    if (note.next_steps.trim()) {
      parts.push(`Next steps:\n${note.next_steps}`);
    }
    try {
      await navigator.clipboard.writeText(parts.join("\n\n"));
    } catch {
      // silently ignore clipboard errors
    }
  };

  return (
    <Card
      padding="md"
      radius="md"
      data-tauri-drag-region
      style={{
        background: "rgba(18, 18, 28, 0.97)",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        height: "100vh",
        cursor: "grab",
        overflow: "auto",
      }}
    >
      <Group justify="space-between" mb="xs">
        <Text size="sm" c="dimmed">
          Notes from {dateLabel}
        </Text>
        <ActionIcon variant="subtle" size="sm" onClick={handleDismiss} aria-label="Dismiss">
          <IconX size={14} />
        </ActionIcon>
      </Group>

      <div onClick={handleOpen} style={{ cursor: "pointer" }}>
        {note.working_on.trim() && (
          <Stack gap={2} mb="xs">
            <Text size="xs" c="dimmed">
              Working on
            </Text>
            <Text size="sm">{note.working_on}</Text>
          </Stack>
        )}

        {note.next_steps.trim() && (
          <Stack gap={2} mb="xs">
            <Text size="xs" c="dimmed">
              Next steps
            </Text>
            <Text size="sm">{note.next_steps}</Text>
          </Stack>
        )}
      </div>

      <Button
        size="xs"
        variant="light"
        leftSection={<IconCopy size={12} />}
        mt="xs"
        onClick={handleCopy}
      >
        Copy Notes
      </Button>
    </Card>
  );
}
