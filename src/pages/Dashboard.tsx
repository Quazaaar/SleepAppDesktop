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
import { useMotionValue, useSpring } from "framer-motion";
import { getCurrentApp, getDailyStats, toggleTracking, getEscalationSettings, pauseEscalation, getTracking, getAppCategories } from "../lib/commands";
import type { CurrentAppInfo, DailyStats, EscalationSettings, AppCategoryEntry, AppUsageStat } from "../lib/types";

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
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

function computeProductivityScore(
  appUsage: AppUsageStat[],
  categories: AppCategoryEntry[],
  escSettings: EscalationSettings | null
): number {
  const catMap = new Map<string, string>();
  for (const entry of categories) {
    catMap.set(entry.app_name.toLowerCase(), entry.category);
  }

  const productiveWeight = escSettings?.distracting_multiplier ?? 1.5;
  const distractingWeight = escSettings?.productive_multiplier ?? 0.5;

  let score = 0;
  for (const app of appUsage) {
    const minutes = app.total_duration_secs / 60;
    const category = catMap.get(app.app_name.toLowerCase()) ?? "uncategorized";
    switch (category) {
      case "productive":
        score += minutes * productiveWeight;
        break;
      case "distracting":
        score += minutes * distractingWeight;
        break;
      default:
        score += minutes;
        break;
    }
  }
  return score;
}

function getCurrentMultiplier(
  currentApp: CurrentAppInfo | null,
  categories: AppCategoryEntry[],
  escSettings: EscalationSettings | null
): number {
  if (!currentApp) return 1.0;
  const cat = categories.find(
    (c) => c.app_name.toLowerCase() === currentApp.app_name.toLowerCase()
  )?.category ?? "uncategorized";
  switch (cat) {
    case "productive": return escSettings?.distracting_multiplier ?? 1.5;
    case "distracting": return escSettings?.productive_multiplier ?? 0.5;
    default: return 1.0;
  }
}

export default function Dashboard() {
  const [currentApp, setCurrentApp] = useState<CurrentAppInfo | null>(null);
  const [stats, setStats] = useState<DailyStats | null>(null);
  const [isTracking, setIsTracking] = useState(true);
  const [escSettings, setEscSettings] = useState<EscalationSettings | null>(null);
  const [categories, setCategories] = useState<AppCategoryEntry[]>([]);
  const [displayScore, setDisplayScore] = useState("0.00");
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
    const fetchCategories = () =>
      getAppCategories()
        .then(setCategories)
        .catch(() => {});
    fetchStats();
    fetchCategories();
    loadEscSettings();
    const interval = setInterval(() => {
      fetchStats();
      fetchCategories();
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

  // -- Crash score animation --
  const targetScore = computeProductivityScore(
    stats?.app_usage ?? [],
    categories,
    escSettings
  );
  const motionScore = useMotionValue(0);
  const springScore = useSpring(motionScore, { stiffness: 50, damping: 20 });

  useEffect(() => {
    motionScore.set(targetScore);
  }, [targetScore, motionScore]);

  useEffect(() => {
    const unsubscribe = springScore.on("change", (v) => {
      setDisplayScore(v.toFixed(2));
    });
    return unsubscribe;
  }, [springScore]);

  const currentMultiplier = getCurrentMultiplier(currentApp, categories, escSettings);

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

      <SimpleGrid cols={2}>
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

        <Card shadow="sm" padding="sm" radius="md" withBorder>
          <div className="crash-score-container">
            <div className="crash-score-number">
              {displayScore}
            </div>
            <div className="crash-score-multiplier">
              {currentMultiplier.toFixed(1)}x
            </div>
            <div className="crash-score-label">
              productivity score
            </div>
          </div>
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
              valueFormatter={formatDuration}
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
