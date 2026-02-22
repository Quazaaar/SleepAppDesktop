import { useEffect, useState } from "react";
import {
  Card,
  Group,
  Stack,
  Title,
  Text,
  Badge,
  Button,
  SimpleGrid,
  ActionIcon,
} from "@mantine/core";
import { DonutChart } from "@mantine/charts";
import { IconPlayerPlay, IconPlayerPause, IconX, IconCopy } from "@tabler/icons-react";
import { getCurrentApp, getDailyStats, toggleTracking, getEscalationSettings, pauseEscalation, getTracking, getLatestWrapUpNote } from "../lib/commands";
import type { CurrentAppInfo, DailyStats, EscalationSettings, WrapUpNote } from "../lib/types";

function ResumePopup({ note, onDismiss }: { note: WrapUpNote; onDismiss: () => void }) {
  const dateLabel = new Date(note.created_at).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });

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
      style={{
        position: "fixed",
        bottom: 24,
        right: 24,
        width: 320,
        zIndex: 1000,
        background: "rgba(18, 18, 28, 0.97)",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
      }}
      padding="md"
    >
      <Group justify="space-between" mb="xs">
        <Text size="sm" c="dimmed">
          Notes from {dateLabel}
        </Text>
        <ActionIcon variant="subtle" size="sm" onClick={onDismiss} aria-label="Dismiss">
          <IconX size={14} />
        </ActionIcon>
      </Group>

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

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function computeTonightHours(): number {
  const now = new Date();
  const tomorrow6am = new Date(now);
  tomorrow6am.setDate(tomorrow6am.getDate() + 1);
  tomorrow6am.setHours(6, 0, 0, 0);
  const diffMs = tomorrow6am.getTime() - now.getTime();
  const hours = Math.floor(diffMs / (1000 * 60 * 60));
  return Math.max(hours, 1);
}

const COLORS = [
  "blue",
  "cyan",
  "teal",
  "green",
  "lime",
  "yellow",
  "orange",
  "red",
  "pink",
  "grape",
  "violet",
  "indigo",
];

export default function Dashboard() {
  const [currentApp, setCurrentApp] = useState<CurrentAppInfo | null>(null);
  const [stats, setStats] = useState<DailyStats | null>(null);
  const [isTracking, setIsTracking] = useState(true);
  const [escSettings, setEscSettings] = useState<EscalationSettings | null>(null);
  const [resumeNote, setResumeNote] = useState<WrapUpNote | null>(null);
  const [resumeDismissed, setResumeDismissed] = useState(false);
  const d = new Date();
  const today = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;

  const loadEscSettings = async () => {
    try {
      setEscSettings(await getEscalationSettings());
    } catch {
      // ignore
    }
  };

  const handlePause = async (hours: number | null) => {
    try {
      await pauseEscalation(hours);
      await loadEscSettings();
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    getLatestWrapUpNote()
      .then((note) => {
        if (note) setResumeNote(note);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    const pollCurrentApp = async () => {
      try {
        setCurrentApp(await getCurrentApp());
      } catch {
        setCurrentApp(null);
      }
    };
    const pollTracking = () =>
      getTracking().then(setIsTracking).catch(() => {});
    pollCurrentApp();
    pollTracking();
    const interval = setInterval(() => {
      pollCurrentApp();
      pollTracking();
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const fetchStats = () =>
      getDailyStats(today)
        .then(setStats)
        .catch(() => {});
    fetchStats();
    loadEscSettings();
    const interval = setInterval(() => {
      fetchStats();
      loadEscSettings();
    }, 2000);
    return () => clearInterval(interval);
  }, [today]);

  const handleToggle = async () => {
    try {
      const tracking = await toggleTracking();
      setIsTracking(tracking);
    } catch {
      // ignore
    }
  };

  const chartData =
    stats?.app_usage.map((app, i) => ({
      name: app.app_name,
      value: app.total_duration_secs,
      color: COLORS[i % COLORS.length],
    })) ?? [];

  const isEscPaused =
    escSettings?.paused_until != null &&
    new Date(escSettings.paused_until) > new Date();

  const pausedUntilFormatted = escSettings?.paused_until
    ? new Date(escSettings.paused_until).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      })
    : null;

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={2}>Dashboard</Title>
        <Button
          variant="light"
          color={isTracking ? "red" : "green"}
          leftSection={
            isTracking ? (
              <IconPlayerPause size={16} />
            ) : (
              <IconPlayerPlay size={16} />
            )
          }
          onClick={handleToggle}
        >
          {isTracking ? "Pause" : "Resume"}
        </Button>
      </Group>

      <SimpleGrid cols={2}>
        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Text size="sm" c="dimmed" mb="xs">
            Currently Active
          </Text>
          {currentApp ? (
            <>
              <Title order={4}>{currentApp.app_name}</Title>
              <Text size="sm" c="dimmed" lineClamp={1}>
                {currentApp.window_title}
              </Text>
              <Badge mt="sm" variant="light">
                {formatDuration(currentApp.duration_secs)}
              </Badge>
            </>
          ) : (
            <Text c="dimmed">No app tracked yet</Text>
          )}
        </Card>

        <Card shadow="sm" padding="lg" radius="md" withBorder>
          <Text size="sm" c="dimmed" mb="xs">
            Today&apos;s Summary
          </Text>
          {stats ? (
            <>
              <Title order={4}>
                {formatDuration(stats.total_tracked_secs)}
              </Title>
              <Text size="sm" c="dimmed">
                Total tracked time
              </Text>
              <Text size="sm" mt="xs">
                Most used: <strong>{stats.most_used_app || "N/A"}</strong>
              </Text>
            </>
          ) : (
            <Text c="dimmed">No data yet</Text>
          )}
        </Card>
      </SimpleGrid>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group justify="space-between" align="center" mb="xs">
          <Text size="sm" c="dimmed">
            Escalation Controls
          </Text>
          {isEscPaused && pausedUntilFormatted && (
            <Badge color="yellow" variant="light">
              Paused until {pausedUntilFormatted}
            </Badge>
          )}
        </Group>
        {isEscPaused ? (
          <Button
            variant="light"
            color="green"
            size="sm"
            onClick={() => handlePause(null)}
          >
            Resume Escalation
          </Button>
        ) : (
          <Button.Group>
            <Button
              variant="light"
              color="orange"
              size="sm"
              onClick={() => handlePause(1)}
            >
              Pause 1h
            </Button>
            <Button
              variant="light"
              color="orange"
              size="sm"
              onClick={() => handlePause(2)}
            >
              Pause 2h
            </Button>
            <Button
              variant="light"
              color="orange"
              size="sm"
              onClick={() => handlePause(computeTonightHours())}
            >
              Pause Tonight
            </Button>
          </Button.Group>
        )}
      </Card>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Text size="sm" c="dimmed" mb="md">
          App Usage Breakdown
        </Text>
        {chartData.length > 0 ? (
          <Group justify="center">
            <DonutChart
              data={chartData}
              size={220}
              thickness={30}
              tooltipDataSource="segment"
              withLabelsLine
              withLabels
            />
          </Group>
        ) : (
          <Text c="dimmed" ta="center" py="xl">
            Start using apps to see your usage breakdown
          </Text>
        )}
      </Card>

      {resumeNote && !resumeDismissed && (
        <ResumePopup
          note={resumeNote}
          onDismiss={() => setResumeDismissed(true)}
        />
      )}
    </Stack>
  );
}
