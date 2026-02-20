import { useEffect, useState, useMemo } from "react";
import {
  Card,
  Stack,
  Title,
  Text,
  Group,
  Badge,
  Table,
  UnstyledButton,
} from "@mantine/core";
import {
  IconChevronUp,
  IconChevronDown,
  IconSelector,
} from "@tabler/icons-react";
import { DatePickerInput, type DateValue } from "@mantine/dates";
import { getActivityTimeline } from "../lib/commands";
import type { ActivitySession } from "../lib/types";

type SortKey = "app_name" | "window_title" | "start_time" | "end_time" | "duration_secs";
type SortDir = "asc" | "desc";

function toLocalDateStr(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

interface SortableThProps {
  label: string;
  sortKey: SortKey;
  active: SortKey;
  dir: SortDir;
  onSort: (key: SortKey) => void;
}

function SortableTh({ label, sortKey, active, dir, onSort }: SortableThProps) {
  const isActive = active === sortKey;
  const Icon = isActive ? (dir === "asc" ? IconChevronUp : IconChevronDown) : IconSelector;

  return (
    <Table.Th>
      <UnstyledButton
        onClick={() => onSort(sortKey)}
        className="sort-th-btn"
      >
        <Group gap={4} wrap="nowrap">
          <span>{label}</span>
          <Icon size={14} style={{ opacity: isActive ? 1 : 0.4, flexShrink: 0 }} />
        </Group>
      </UnstyledButton>
    </Table.Th>
  );
}

export default function Timeline() {
  const [date, setDate] = useState<Date | null>(new Date());
  const [sessions, setSessions] = useState<ActivitySession[]>([]);
  const [sortKey, setSortKey] = useState<SortKey>("start_time");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const dateStr = toLocalDateStr(date ?? new Date());

  useEffect(() => {
    getActivityTimeline(dateStr)
      .then(setSessions)
      .catch(() => setSessions([]));
  }, [dateStr]);

  function handleSort(key: SortKey) {
    if (key === sortKey) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  }

  const sorted = useMemo(() => {
    return [...sessions].sort((a, b) => {
      let cmp = 0;
      if (sortKey === "duration_secs") {
        cmp = a.duration_secs - b.duration_secs;
      } else {
        cmp = a[sortKey].localeCompare(b[sortKey]);
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [sessions, sortKey, sortDir]);

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={2}>Timeline</Title>
        <DatePickerInput
          value={date}
          onChange={(v: DateValue) => {
            if (!v) { setDate(null); return; }
            const raw = v instanceof Date ? v : new Date(v);
            // Mantine builds dates from ISO strings via dayjs → UTC midnight.
            // Extract the UTC calendar parts and reconstruct as local midnight
            // so getDate() / getMonth() / getFullYear() return the picked day.
            setDate(new Date(raw.getUTCFullYear(), raw.getUTCMonth(), raw.getUTCDate()));
          }}
          maxDate={new Date()}
          weekendDays={[]}
          w={200}
        />
      </Group>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        {sorted.length > 0 ? (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <SortableTh label="App"      sortKey="app_name"      active={sortKey} dir={sortDir} onSort={handleSort} />
                <SortableTh label="Window"   sortKey="window_title"  active={sortKey} dir={sortDir} onSort={handleSort} />
                <SortableTh label="Start"    sortKey="start_time"    active={sortKey} dir={sortDir} onSort={handleSort} />
                <SortableTh label="End"      sortKey="end_time"      active={sortKey} dir={sortDir} onSort={handleSort} />
                <SortableTh label="Duration" sortKey="duration_secs" active={sortKey} dir={sortDir} onSort={handleSort} />
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {sorted.map((session, i) => (
                <Table.Tr key={session.id ?? i}>
                  <Table.Td>
                    <Badge variant="light" size="sm">
                      {session.app_name}
                    </Badge>
                  </Table.Td>
                  <Table.Td>
                    <Text size="sm" lineClamp={1} maw={250}>
                      {session.window_title}
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    <Text size="sm">{formatTime(session.start_time)}</Text>
                  </Table.Td>
                  <Table.Td>
                    <Text size="sm">{formatTime(session.end_time)}</Text>
                  </Table.Td>
                  <Table.Td>
                    <Text size="sm">{formatDuration(session.duration_secs)}</Text>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        ) : (
          <Text c="dimmed" ta="center" py="xl">
            No activity recorded for this date
          </Text>
        )}
      </Card>
    </Stack>
  );
}
