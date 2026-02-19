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
} from "@mantine/core";
import { DonutChart } from "@mantine/charts";
import { IconPlayerPlay, IconPlayerPause } from "@tabler/icons-react";
import { getCurrentApp, getDailyStats, toggleTracking } from "../lib/commands";
import type { CurrentAppInfo, DailyStats } from "../lib/types";

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
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
  const today = new Date().toISOString().slice(0, 10);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        setCurrentApp(await getCurrentApp());
      } catch {
        setCurrentApp(null);
      }
    }, 5000);
    // Fetch immediately
    getCurrentApp().then(setCurrentApp).catch(() => setCurrentApp(null));
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const fetchStats = () =>
      getDailyStats(today)
        .then(setStats)
        .catch(() => {});
    fetchStats();
    const interval = setInterval(fetchStats, 15000);
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
    </Stack>
  );
}
