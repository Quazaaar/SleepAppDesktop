import { useEffect, useMemo, useState } from "react";
import {
  Stack,
  Group,
  Paper,
  Text,
  Title,
  TextInput,
  SegmentedControl,
  ActionIcon,
  Badge,
  Collapse,
  Loader,
} from "@mantine/core";
import {
  IconSearch,
  IconChevronDown,
  IconChevronRight,
  IconTrash,
  IconPlus,
  IconCategory,
} from "@tabler/icons-react";
import {
  getAppCategories,
  setAppCategory,
  getTitleKeywordRules,
  addTitleKeywordRule,
  deleteTitleKeywordRule,
} from "../lib/commands";
import type { AppCategoryEntry, TitleKeywordRule, AppCategory } from "../lib/types";

const BROWSER_APPS = ["chrome", "firefox", "msedge", "brave", "opera", "arc", "chromium"];

function isBrowser(appName: string): boolean {
  return BROWSER_APPS.some((b) => appName.toLowerCase().includes(b));
}

function formatLastSeen(lastSeen: string | null): string {
  if (!lastSeen) return "Never";
  try {
    const d = new Date(lastSeen);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    if (diffSecs < 60) return "Just now";
    const diffMins = Math.floor(diffSecs / 60);
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  } catch {
    return lastSeen;
  }
}

const CATEGORY_ORDER: AppCategory[] = ["uncategorized", "productive", "neutral", "distracting"];

const CATEGORY_COLOR: Record<string, string> = {
  productive: "green",
  neutral: "gray",
  distracting: "red",
  uncategorized: "orange",
};

const CATEGORY_DOT: Record<string, string> = {
  productive: "#40c057",
  neutral: "#868e96",
  distracting: "#fa5252",
  uncategorized: "#fd7e14",
};

const SEG_DATA = [
  { label: "P", value: "productive" },
  { label: "N", value: "neutral" },
  { label: "D", value: "distracting" },
];

export default function Apps() {
  const [entries, setEntries] = useState<AppCategoryEntry[]>([]);
  const [search, setSearch] = useState("");
  const [expandedApp, setExpandedApp] = useState<string | null>(null);
  const [keywordRules, setKeywordRules] = useState<TitleKeywordRule[]>([]);
  const [newKeyword, setNewKeyword] = useState("");
  const [newKeywordCategory, setNewKeywordCategory] = useState<AppCategory>("distracting");
  const [loading, setLoading] = useState(true);

  const loadData = async () => {
    try {
      const [cats, rules] = await Promise.all([getAppCategories(), getTitleKeywordRules()]);
      setEntries(cats);
      setKeywordRules(rules);
    } catch {
      // ignore — backend may not be running in dev
    } finally {
      setLoading(false);
    }
  };

  const reloadRules = async () => {
    try {
      setKeywordRules(await getTitleKeywordRules());
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const uncatCount = useMemo(
    () => entries.filter((e) => e.category === "uncategorized").length,
    [entries]
  );

  const grouped = useMemo(() => {
    const lower = search.toLowerCase();
    const filtered = search
      ? entries.filter((e) => e.app_name.toLowerCase().includes(lower))
      : entries;

    const map: Record<string, AppCategoryEntry[]> = {};
    for (const cat of CATEGORY_ORDER) {
      map[cat] = [];
    }
    for (const entry of filtered) {
      const cat = CATEGORY_ORDER.includes(entry.category as AppCategory)
        ? entry.category
        : "uncategorized";
      map[cat].push(entry);
    }
    return map;
  }, [entries, search]);

  const handleCategoryChange = async (appName: string, newCat: string) => {
    // Optimistic update
    setEntries((prev) =>
      prev.map((e) => (e.app_name === appName ? { ...e, category: newCat } : e))
    );
    try {
      await setAppCategory(appName, newCat);
    } catch {
      // Revert on failure
      await loadData();
    }
  };

  const handleDeleteRule = async (id: number | null) => {
    if (id === null) return;
    try {
      await deleteTitleKeywordRule(id);
      await reloadRules();
    } catch {
      // ignore
    }
  };

  const handleAddRule = async () => {
    if (!expandedApp || !newKeyword.trim()) return;
    try {
      await addTitleKeywordRule(expandedApp, newKeyword.trim(), newKeywordCategory);
      setNewKeyword("");
      await reloadRules();
    } catch {
      // ignore
    }
  };

  if (loading) {
    return (
      <Stack align="center" pt="xl">
        <Loader size="sm" />
        <Text size="sm" c="dimmed">Loading apps...</Text>
      </Stack>
    );
  }

  return (
    <Stack gap="md">
      {/* Header */}
      <div>
        <Group gap="xs" mb={4}>
          <IconCategory size={20} />
          <Title order={3}>Apps</Title>
        </Group>
        {uncatCount > 0 && (
          <Text size="sm" c="orange">
            {uncatCount} uncategorized {uncatCount === 1 ? "app" : "apps"} need attention
          </Text>
        )}
      </div>

      {/* Search */}
      <TextInput
        placeholder="Search apps..."
        leftSection={<IconSearch size={16} />}
        value={search}
        onChange={(e) => setSearch(e.currentTarget.value)}
      />

      {/* Grouped list */}
      {CATEGORY_ORDER.map((cat) => {
        const group = grouped[cat];
        if (!group || group.length === 0) return null;

        return (
          <div key={cat}>
            {/* Section header */}
            <Group gap="xs" mb="xs">
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: "50%",
                  backgroundColor: CATEGORY_DOT[cat],
                  flexShrink: 0,
                }}
              />
              <Text size="sm" fw={600} tt="capitalize" c="dimmed">
                {cat}
              </Text>
              <Text size="xs" c="dimmed">
                ({group.length})
              </Text>
            </Group>

            <Stack gap="xs">
              {group.map((entry) => {
                const browser = isBrowser(entry.app_name);
                const isExpanded = expandedApp === entry.app_name;
                const appRules = keywordRules.filter(
                  (r) => r.app_name.toLowerCase() === entry.app_name.toLowerCase()
                );

                return (
                  <div key={entry.app_name}>
                    <Paper withBorder p="xs" radius="sm">
                      <Group justify="space-between" wrap="nowrap">
                        {/* Left: name + last seen */}
                        <div style={{ minWidth: 0, flex: 1 }}>
                          <Text fw={500} size="sm" truncate>
                            {entry.app_name}
                          </Text>
                          <Text size="xs" c="dimmed">
                            {formatLastSeen(entry.last_seen)}
                          </Text>
                        </div>

                        {/* Right: segment control + browser expand */}
                        <Group gap="xs" wrap="nowrap">
                          <SegmentedControl
                            data={SEG_DATA}
                            size="xs"
                            value={
                              entry.category === "uncategorized" ? "" : entry.category
                            }
                            onChange={(val) => handleCategoryChange(entry.app_name, val)}
                            w={140}
                          />
                          {browser && (
                            <ActionIcon
                              variant="subtle"
                              color="gray"
                              size="sm"
                              onClick={() =>
                                setExpandedApp(isExpanded ? null : entry.app_name)
                              }
                              aria-label={isExpanded ? "Collapse rules" : "Expand rules"}
                            >
                              {isExpanded ? (
                                <IconChevronDown size={14} />
                              ) : (
                                <IconChevronRight size={14} />
                              )}
                            </ActionIcon>
                          )}
                        </Group>
                      </Group>
                    </Paper>

                    {/* Browser keyword rules expansion */}
                    {browser && (
                      <Collapse in={isExpanded}>
                        <Paper
                          withBorder
                          p="sm"
                          radius="sm"
                          mt={2}
                          style={{ borderTopLeftRadius: 0, borderTopRightRadius: 0 }}
                        >
                          <Stack gap="xs">
                            <Text size="xs" fw={500} c="dimmed">
                              Title Keyword Rules
                            </Text>

                            {appRules.length === 0 && (
                              <Text size="xs" c="dimmed" fs="italic">
                                No rules yet — add keywords to override category by window title
                              </Text>
                            )}

                            {appRules.map((rule) => (
                              <Group key={rule.id ?? rule.keyword} justify="space-between" wrap="nowrap">
                                <Text size="xs" style={{ flex: 1 }} truncate>
                                  {rule.keyword}
                                </Text>
                                <Group gap="xs" wrap="nowrap">
                                  <Badge
                                    size="xs"
                                    color={CATEGORY_COLOR[rule.category] ?? "gray"}
                                    variant="light"
                                  >
                                    {rule.category}
                                  </Badge>
                                  <ActionIcon
                                    variant="subtle"
                                    color="red"
                                    size="xs"
                                    onClick={() => handleDeleteRule(rule.id)}
                                    aria-label="Delete rule"
                                  >
                                    <IconTrash size={12} />
                                  </ActionIcon>
                                </Group>
                              </Group>
                            ))}

                            {/* Add rule row */}
                            <Group gap="xs" wrap="nowrap" mt={4}>
                              <TextInput
                                placeholder="Keyword (e.g. github)"
                                size="xs"
                                value={newKeyword}
                                onChange={(e) => setNewKeyword(e.currentTarget.value)}
                                onKeyDown={(e) => {
                                  if (e.key === "Enter") handleAddRule();
                                }}
                                style={{ flex: 1 }}
                              />
                              <SegmentedControl
                                data={SEG_DATA}
                                size="xs"
                                value={newKeywordCategory}
                                onChange={(v) => setNewKeywordCategory(v as AppCategory)}
                                w={100}
                              />
                              <ActionIcon
                                variant="filled"
                                color="blue"
                                size="sm"
                                onClick={handleAddRule}
                                disabled={!newKeyword.trim()}
                                aria-label="Add rule"
                              >
                                <IconPlus size={14} />
                              </ActionIcon>
                            </Group>
                          </Stack>
                        </Paper>
                      </Collapse>
                    )}
                  </div>
                );
              })}
            </Stack>
          </div>
        );
      })}

      {entries.length === 0 && !loading && (
        <Text size="sm" c="dimmed" ta="center" py="xl">
          No apps tracked yet. Start the tracker to begin recording app usage.
        </Text>
      )}
    </Stack>
  );
}
