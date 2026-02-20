import { useEffect, useState } from "react";
import {
  Card,
  Stack,
  Title,
  Text,
  Switch,
  Button,
  TextInput,
  Group,
  Select,
  NumberInput,
  Table,
  ActionIcon,
  Modal,
  SegmentedControl,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconPlayerPlay, IconPlus, IconTrash } from "@tabler/icons-react";
import {
  getReminderRules,
  saveReminderRule,
  deleteReminderRule,
  toggleReminderRule,
  testReminderNotification,
  getIgnoredApps,
  setIgnoredApps,
  syncNow,
  setSyncConfig,
  getSyncStatus,
} from "../lib/commands";
import type { ReminderRule, SyncStatus } from "../lib/types";
import { notifications } from "@mantine/notifications";
import { load } from "@tauri-apps/plugin-store";
import { useAppTheme } from "../context/ThemeContext";
import type { AppThemeId } from "../lib/theme";
import { EscalationSettingsCard } from "../components/EscalationSettingsCard";

export default function Settings() {
  const { themeId, setTheme } = useAppTheme();
  const [rules, setRules] = useState<ReminderRule[]>([]);
  const [ignoredApps, setIgnoredAppsState] = useState<string[]>([]);
  const [newIgnored, setNewIgnored] = useState("");
  const [opened, { open, close }] = useDisclosure(false);

  // Sync state
  const [syncUrl, setSyncUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);
  const [syncing, setSyncing] = useState(false);

  // New rule form state
  const [ruleType, setRuleType] = useState<string>("break_reminder");
  const [ruleAppName, setRuleAppName] = useState("");
  const [ruleThreshold, setRuleThreshold] = useState<number>(30);
  const [ruleMessage, setRuleMessage] = useState("");

  const loadData = async () => {
    try {
      setRules(await getReminderRules());
    } catch {
      // ignore
    }
    try {
      setIgnoredAppsState(await getIgnoredApps());
    } catch {
      // ignore
    }
    try {
      setSyncStatus(await getSyncStatus());
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    loadData();
    // Load sync config from store
    (async () => {
      try {
        const store = await load("settings.json");
        const url = await store.get<string>("sync_url");
        const key = await store.get<string>("api_key");
        if (url) setSyncUrl(url);
        if (key) setApiKey(key);
      } catch {
        // store not available yet
      }
    })();
  }, []);

  const handleToggleRule = async (ruleId: number, enabled: boolean) => {
    await toggleReminderRule(ruleId, enabled);
    await loadData();
  };

  const handleDeleteRule = async (ruleId: number) => {
    await deleteReminderRule(ruleId);
    await loadData();
  };

  const handleAddRule = async () => {
    await saveReminderRule({
      id: null,
      rule_type: ruleType,
      app_name: ruleType === "app_limit" ? ruleAppName : null,
      threshold_minutes: ruleThreshold,
      message: ruleMessage,
      enabled: true,
    });
    close();
    setRuleType("break_reminder");
    setRuleAppName("");
    setRuleThreshold(30);
    setRuleMessage("");
    await loadData();
  };

  const handleSaveSyncConfig = async () => {
    try {
      await setSyncConfig(syncUrl, apiKey);
      const store = await load("settings.json");
      await store.set("sync_url", syncUrl);
      await store.set("api_key", apiKey);
      await store.save();
      setSyncStatus(await getSyncStatus());
      notifications.show({ title: "Saved", message: "Sync configuration saved", color: "green" });
    } catch (e) {
      notifications.show({ title: "Error", message: String(e), color: "red" });
    }
  };

  const handleSyncNow = async () => {
    setSyncing(true);
    try {
      const count = await syncNow();
      setSyncStatus(await getSyncStatus());
      notifications.show({
        title: "Sync Complete",
        message: count > 0 ? `Synced ${count} sessions` : "No new sessions to sync",
        color: "green",
      });
    } catch (e) {
      notifications.show({ title: "Sync Failed", message: String(e), color: "red" });
    } finally {
      setSyncing(false);
    }
  };

  const handleAddIgnored = async () => {
    if (newIgnored.trim()) {
      const updated = [...ignoredApps, newIgnored.trim()];
      await setIgnoredApps(updated);
      setIgnoredAppsState(updated);
      setNewIgnored("");
    }
  };

  const handleRemoveIgnored = async (app: string) => {
    const updated = ignoredApps.filter((a) => a !== app);
    await setIgnoredApps(updated);
    setIgnoredAppsState(updated);
  };

  return (
    <Stack>
      <Title order={2}>Settings</Title>

      {/* Appearance */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Title order={4} mb="md">
          Appearance
        </Title>
        <Text size="sm" c="dimmed" mb="sm">
          Choose a theme for the app
        </Text>
        <SegmentedControl
          fullWidth
          value={themeId}
          onChange={(val) => setTheme(val as AppThemeId)}
          data={[
            { value: "glass-dark", label: "Glass Dark" },
            { value: "glass-light", label: "Glass Light" },
            { value: "solid-minimal", label: "Solid Minimal" },
          ]}
        />
      </Card>

      {/* Escalation Settings */}
      <EscalationSettingsCard />

      {/* Ignored Apps */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Title order={4} mb="md">
          Ignored Apps
        </Title>
        <Text size="sm" c="dimmed" mb="sm">
          These apps will not be tracked
        </Text>
        <Group mb="sm">
          <TextInput
            placeholder="App name (e.g., Explorer)"
            value={newIgnored}
            onChange={(e) => setNewIgnored(e.currentTarget.value)}
            style={{ flex: 1 }}
          />
          <Button size="sm" onClick={handleAddIgnored}>
            Add
          </Button>
        </Group>
        {ignoredApps.map((app) => (
          <Group key={app} mb="xs">
            <Text size="sm" style={{ flex: 1 }}>
              {app}
            </Text>
            <ActionIcon
              color="red"
              variant="light"
              onClick={() => handleRemoveIgnored(app)}
            >
              <IconTrash size={14} />
            </ActionIcon>
          </Group>
        ))}
      </Card>

      {/* Reminder Rules */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group justify="space-between" mb="md">
          <Title order={4}>Reminder Rules</Title>
          <Button
            size="sm"
            leftSection={<IconPlus size={14} />}
            onClick={open}
          >
            Add Rule
          </Button>
        </Group>

        {rules.length > 0 ? (
          <Table>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Type</Table.Th>
                <Table.Th>App</Table.Th>
                <Table.Th>Threshold</Table.Th>
                <Table.Th>Message</Table.Th>
                <Table.Th>Enabled</Table.Th>
                <Table.Th />
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {rules.map((rule) => (
                <Table.Tr key={rule.id}>
                  <Table.Td>{rule.rule_type}</Table.Td>
                  <Table.Td>{rule.app_name ?? "—"}</Table.Td>
                  <Table.Td>{rule.threshold_minutes}m</Table.Td>
                  <Table.Td>
                    <Text size="sm" lineClamp={1} maw={200}>
                      {rule.message}
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    <Switch
                      checked={rule.enabled}
                      onChange={(e) =>
                        handleToggleRule(rule.id!, e.currentTarget.checked)
                      }
                    />
                  </Table.Td>
                  <Table.Td>
                    <Group gap="xs" wrap="nowrap">
                      <ActionIcon
                        color="blue"
                        variant="light"
                        title="Test this reminder"
                        onClick={() =>
                          testReminderNotification(rule.message).catch((e) =>
                            notifications.show({
                              title: "Error",
                              message: String(e),
                              color: "red",
                            })
                          )
                        }
                      >
                        <IconPlayerPlay size={14} />
                      </ActionIcon>
                      <ActionIcon
                        color="red"
                        variant="light"
                        onClick={() => handleDeleteRule(rule.id!)}
                      >
                        <IconTrash size={14} />
                      </ActionIcon>
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        ) : (
          <Text c="dimmed">No reminder rules configured</Text>
        )}
      </Card>

      {/* Cloud Sync */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Title order={4} mb="md">
          Cloud Sync
        </Title>
        <Text size="sm" c="dimmed" mb="sm">
          Configure cloud sync to back up your data
        </Text>
        <TextInput
          label="API URL"
          placeholder="http://localhost:3000"
          value={syncUrl}
          onChange={(e) => setSyncUrl(e.currentTarget.value)}
          mb="sm"
        />
        <TextInput
          label="API Key"
          placeholder="Your API key"
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.currentTarget.value)}
          mb="sm"
        />
        <Group>
          <Button variant="light" onClick={handleSaveSyncConfig}>
            Save Config
          </Button>
          <Button
            onClick={handleSyncNow}
            loading={syncing}
            disabled={!syncStatus?.configured}
          >
            Sync Now
          </Button>
        </Group>
        {syncStatus?.last_sync_time && (
          <Text size="xs" c="dimmed" mt="sm">
            Last sync: {new Date(syncStatus.last_sync_time).toLocaleString()}
          </Text>
        )}
      </Card>

      {/* Add Rule Modal */}
      <Modal opened={opened} onClose={close} title="Add Reminder Rule">
        <Stack>
          <Select
            label="Rule Type"
            data={[
              { value: "break_reminder", label: "Break Reminder" },
              { value: "app_limit", label: "App Time Limit" },
            ]}
            value={ruleType}
            onChange={(v) => setRuleType(v ?? "break_reminder")}
          />
          {ruleType === "app_limit" && (
            <TextInput
              label="App Name"
              placeholder="e.g., Chrome"
              value={ruleAppName}
              onChange={(e) => setRuleAppName(e.currentTarget.value)}
            />
          )}
          <NumberInput
            label="Threshold (minutes)"
            value={ruleThreshold}
            onChange={(v) => setRuleThreshold(typeof v === "number" ? v : 30)}
            min={1}
          />
          <TextInput
            label="Reminder Message"
            placeholder="Time to take a break!"
            value={ruleMessage}
            onChange={(e) => setRuleMessage(e.currentTarget.value)}
          />
          <Button onClick={handleAddRule}>Save Rule</Button>
        </Stack>
      </Modal>
    </Stack>
  );
}
