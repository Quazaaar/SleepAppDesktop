import { useEffect, useRef, useState } from "react";
import { Card, Title, Text, Switch, Slider, Stack, Group, Loader, Button, Badge } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { IconPlayerPlay, IconPlayerStop } from "@tabler/icons-react";
import { getEscalationSettings, setEscalationSettings, showEscalationWindow } from "../lib/commands";
import type { EscalationSettings } from "../lib/types";
import { TimelineBar } from "./TimelineBar";

export function EscalationSettingsCard() {
  const [settings, setSettings] = useState<EscalationSettings | null>(null);
  const [saving, setSaving] = useState(false);
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

  useEffect(() => {
    getEscalationSettings()
      .then(setSettings)
      .catch((e) => {
        notifications.show({
          title: "Error",
          message: `Failed to load escalation settings: ${String(e)}`,
          color: "red",
        });
      });
  }, []);

  const handleSave = async (updated: EscalationSettings) => {
    setSaving(true);
    try {
      await setEscalationSettings(updated);
      notifications.show({
        title: "Saved",
        message: "Escalation settings updated",
        color: "green",
        autoClose: 2000,
      });
    } catch (e) {
      notifications.show({
        title: "Error",
        message: `Failed to save escalation settings: ${String(e)}`,
        color: "red",
      });
    } finally {
      setSaving(false);
    }
  };

  const handleToggleEnabled = async (enabled: boolean) => {
    if (!settings) return;
    const updated = { ...settings, enabled };
    setSettings(updated);
    await handleSave(updated);
  };

  const handleTimelineDrag = (greenEnd: number, yellowEnd: number) => {
    if (!settings) return;
    setSettings({ ...settings, green_end_hour: greenEnd, yellow_end_hour: yellowEnd });
  };

  const handleTimelineDragEnd = async (greenEnd: number, yellowEnd: number) => {
    if (!settings) return;
    const updated = { ...settings, green_end_hour: greenEnd, yellow_end_hour: yellowEnd };
    setSettings(updated);
    await handleSave(updated);
  };

  const handleSensitivityChange = async (value: number) => {
    if (!settings) return;
    const updated = { ...settings, sensitivity: value };
    setSettings(updated);
    await handleSave(updated);
  };

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Group justify="space-between" mb="xs">
        <Title order={4}>Escalation Schedule</Title>
        {saving && <Loader size="xs" />}
      </Group>

      <Text size="sm" c="dimmed" mb="md">
        Configure when the escalation system activates based on time of day. The green zone is safe
        working hours — escalation only fires in yellow and red zones. Sensitivity controls how
        quickly escalation advances through levels.
      </Text>

      {settings === null ? (
        <Loader size="sm" />
      ) : (
        <Stack gap="lg">
          <Switch
            label="Enable Escalation"
            checked={settings.enabled}
            onChange={(e) => handleToggleEnabled(e.currentTarget.checked)}
          />

          <div>
            <Text size="sm" fw={500} mb="sm">
              Time Zones
            </Text>
            <TimelineBar
              greenEndHour={settings.green_end_hour}
              yellowEndHour={settings.yellow_end_hour}
              onChange={handleTimelineDrag}
              onChangeEnd={handleTimelineDragEnd}
            />
          </div>

          <div>
            <Text size="sm" fw={500} mb="xs">
              Escalation Speed
            </Text>
            <Slider
              min={0}
              max={1}
              step={0.1}
              value={settings.sensitivity}
              onChangeEnd={handleSensitivityChange}
              marks={[
                { value: 0, label: "Gentle" },
                { value: 0.5, label: "Moderate" },
                { value: 1, label: "Aggressive" },
              ]}
              mb="xl"
            />
          </div>

          <div>
            <Group justify="space-between" mb="xs">
              <Text size="sm" fw={500}>
                Preview Levels
              </Text>
              {activePreview && (
                <Badge size="sm" color="yellow" variant="light">
                  {activePreview} active
                </Badge>
              )}
            </Group>
            <Group gap="xs">
              <Button
                size="xs"
                variant="light"
                leftSection={<IconPlayerPlay size={14} />}
                onClick={() => handlePreview("Level1")}
                disabled={activePreview === "Level1"}
              >
                Level 1 — Toast
              </Button>
              <Button
                size="xs"
                variant="light"
                leftSection={<IconPlayerPlay size={14} />}
                onClick={() => handlePreview("Level2")}
                disabled={activePreview === "Level2"}
              >
                Level 2 — Popup
              </Button>
              <Button
                size="xs"
                variant="light"
                leftSection={<IconPlayerPlay size={14} />}
                onClick={() => handlePreview("Level3")}
                disabled={activePreview === "Level3"}
              >
                Level 3 — Panel
              </Button>
              <Button
                size="xs"
                variant="light"
                leftSection={<IconPlayerPlay size={14} />}
                onClick={() => handlePreview("Level4")}
                disabled={activePreview === "Level4"}
              >
                Level 4 — Fullscreen
              </Button>
              {activePreview && (
                <Button
                  size="xs"
                  variant="filled"
                  color="red"
                  leftSection={<IconPlayerStop size={14} />}
                  onClick={clearPreview}
                >
                  Close Preview
                </Button>
              )}
            </Group>
          </div>
        </Stack>
      )}
    </Card>
  );
}
