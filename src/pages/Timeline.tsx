import { useEffect, useState } from "react";
import {
  Card,
  Stack,
  Title,
  Text,
  Group,
  Badge,
  Table,
} from "@mantine/core";
import { DatePickerInput, type DateValue } from "@mantine/dates";
import { getActivityTimeline } from "../lib/commands";
import type { ActivitySession } from "../lib/types";

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

export default function Timeline() {
  const [date, setDate] = useState<Date | null>(new Date());
  const [sessions, setSessions] = useState<ActivitySession[]>([]);

  const dateStr = date
    ? date.toISOString().slice(0, 10)
    : new Date().toISOString().slice(0, 10);

  useEffect(() => {
    getActivityTimeline(dateStr)
      .then(setSessions)
      .catch(() => setSessions([]));
  }, [dateStr]);

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={2}>Timeline</Title>
        <DatePickerInput
          value={date}
          onChange={(v: DateValue) => setDate(v instanceof Date ? v : v ? new Date(v) : null)}
          maxDate={new Date()}
          w={200}
        />
      </Group>

      <Card shadow="sm" padding="lg" radius="md" withBorder>
        {sessions.length > 0 ? (
          <Table striped highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>App</Table.Th>
                <Table.Th>Window</Table.Th>
                <Table.Th>Start</Table.Th>
                <Table.Th>End</Table.Th>
                <Table.Th>Duration</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {sessions.map((session, i) => (
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
                    <Text size="sm">
                      {formatDuration(session.duration_secs)}
                    </Text>
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
