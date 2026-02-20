import { useEffect, useState } from "react";
import { Card, Title, Text, Switch, Slider, Stack, Group, Loader } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { getEscalationSettings, setEscalationSettings } from "../lib/commands";
import type { EscalationSettings } from "../lib/types";
import { TimelineBar } from "./TimelineBar";

export function EscalationSettingsCard() {
  const [settings, setSettings] = useState<EscalationSettings | null>(null);
  const [saving, setSaving] = useState(false);

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

  const handleTimelineChange = async (greenEnd: number, yellowEnd: number) => {
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
              onChange={handleTimelineChange}
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
        </Stack>
      )}
    </Card>
  );
}
