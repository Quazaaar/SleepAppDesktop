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

  const handleProductiveMultiplierChange = (value: number) => {
    if (!settings) return;
    setSettings({ ...settings, productive_multiplier: value });
  };

  const handleProductiveMultiplierSave = async (value: number) => {
    if (!settings) return;
    const updated = { ...settings, productive_multiplier: value };
    setSettings(updated);
    await handleSave(updated);
  };

  const handleDistractingMultiplierChange = (value: number) => {
    if (!settings) return;
    setSettings({ ...settings, distracting_multiplier: value });
  };

  const handleDistractingMultiplierSave = async (value: number) => {
    if (!settings) return;
    const updated = { ...settings, distracting_multiplier: value };
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
                { value: 1, label: "Assertive" },
              ]}
              mb="xl"
            />
          </div>

          {settings.enabled && (
            <div>
              <Text size="sm" fw={500} mb={4}>
                Category Speed Multipliers
              </Text>
              <Text size="xs" c="dimmed" mb="sm">
                How much app categories affect escalation speed
              </Text>

              <Stack gap="lg">
                <div>
                  <Text size="xs" fw={500} mb="xs">
                    Productive slowdown
                  </Text>
                  <Slider
                    min={0.1}
                    max={1.0}
                    step={0.1}
                    value={settings.productive_multiplier}
                    onChange={handleProductiveMultiplierChange}
                    onChangeEnd={handleProductiveMultiplierSave}
                    label={(v) => v >= 1.0 ? "Normal" : `${(1 / v).toFixed(1)}x slower`}
                    marks={[
                      { value: 0.1, label: "10x" },
                      { value: 0.5, label: "2x slower" },
                      { value: 1.0, label: "Normal" },
                    ]}
                    mb="xl"
                  />
                </div>

                <div>
                  <Text size="xs" fw={500} mb="xs">
                    Distracting speedup
                  </Text>
                  <Slider
                    min={1.0}
                    max={3.0}
                    step={0.1}
                    value={settings.distracting_multiplier}
                    onChange={handleDistractingMultiplierChange}
                    onChangeEnd={handleDistractingMultiplierSave}
                    label={(v) => `${v.toFixed(1)}x faster`}
                    marks={[
                      { value: 1.0, label: "Normal" },
                      { value: 2.0, label: "2x faster" },
                      { value: 3.0, label: "3x faster" },
                    ]}
                    mb="xl"
                  />
                </div>
              </Stack>
            </div>
          )}
        </Stack>
      )}
    </Card>
  );
}
