import { useRef, useCallback } from "react";
import { Text, Stack, Group } from "@mantine/core";

interface TimelineBarProps {
  greenEndHour: number;
  yellowEndHour: number;
  onChange: (greenEnd: number, yellowEnd: number) => void;
  onChangeEnd?: (greenEnd: number, yellowEnd: number) => void;
}

function formatHour(hour: number): string {
  if (hour === 0 || hour === 24) return "12am";
  if (hour === 12) return "12pm";
  if (hour < 12) return `${hour}am`;
  return `${hour - 12}pm`;
}

export function TimelineBar({ greenEndHour, yellowEndHour, onChange, onChangeEnd }: TimelineBarProps) {
  const barRef = useRef<HTMLDivElement>(null);
  const draggingRef = useRef<"green" | "yellow" | null>(null);

  const greenWidth = (greenEndHour / 24) * 100;
  const yellowWidth = ((yellowEndHour - greenEndHour) / 24) * 100;
  const redWidth = ((24 - yellowEndHour) / 24) * 100;

  const getHourFromEvent = useCallback((clientX: number): number => {
    if (!barRef.current) return 0;
    const rect = barRef.current.getBoundingClientRect();
    const relX = clientX - rect.left;
    const clamped = Math.max(0, Math.min(relX, rect.width));
    return Math.round((clamped / rect.width) * 24);
  }, []);

  const handleMouseDown = useCallback(
    (boundary: "green" | "yellow") => (e: React.MouseEvent) => {
      e.preventDefault();
      draggingRef.current = boundary;
    },
    []
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!draggingRef.current) return;
      const hour = getHourFromEvent(e.clientX);

      if (draggingRef.current === "green") {
        const newGreen = Math.max(1, Math.min(hour, yellowEndHour - 1));
        if (newGreen !== greenEndHour) onChange(newGreen, yellowEndHour);
      } else {
        const newYellow = Math.max(greenEndHour + 1, Math.min(hour, 23));
        if (newYellow !== yellowEndHour) onChange(greenEndHour, newYellow);
      }
    },
    [greenEndHour, yellowEndHour, onChange, getHourFromEvent]
  );

  const handleMouseUp = useCallback(() => {
    if (draggingRef.current && onChangeEnd) {
      onChangeEnd(greenEndHour, yellowEndHour);
    }
    draggingRef.current = null;
  }, [greenEndHour, yellowEndHour, onChangeEnd]);

  const handleMouseLeave = useCallback(() => {
    if (draggingRef.current && onChangeEnd) {
      onChangeEnd(greenEndHour, yellowEndHour);
    }
    draggingRef.current = null;
  }, [greenEndHour, yellowEndHour, onChangeEnd]);

  return (
    <Stack gap="xs">
      {/* Zone labels */}
      <Group gap="lg">
        <Group gap={6}>
          <div style={{ width: 10, height: 10, borderRadius: 2, background: "#4ade80", flexShrink: 0 }} />
          <Text size="xs" c="dimmed">Green: 12am – {formatHour(greenEndHour)}</Text>
        </Group>
        <Group gap={6}>
          <div style={{ width: 10, height: 10, borderRadius: 2, background: "#facc15", flexShrink: 0 }} />
          <Text size="xs" c="dimmed">Yellow: {formatHour(greenEndHour)} – {formatHour(yellowEndHour)}</Text>
        </Group>
        <Group gap={6}>
          <div style={{ width: 10, height: 10, borderRadius: 2, background: "#f87171", flexShrink: 0 }} />
          <Text size="xs" c="dimmed">Red: {formatHour(yellowEndHour)} – 12am</Text>
        </Group>
      </Group>

      {/* Bar container */}
      <div
        ref={barRef}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseLeave}
        style={{ position: "relative", userSelect: "none", cursor: "default" }}
      >
        {/* Colored bar segments */}
        <div
          style={{
            display: "flex",
            height: 40,
            borderRadius: 8,
            overflow: "hidden",
            border: "1px solid rgba(255,255,255,0.1)",
          }}
        >
          <div style={{ flex: greenWidth, background: "#4ade80", opacity: 0.85 }} />
          <div style={{ flex: yellowWidth, background: "#facc15", opacity: 0.85 }} />
          <div style={{ flex: redWidth, background: "#f87171", opacity: 0.85 }} />
        </div>

        {/* Green/Yellow boundary handle */}
        <div
          onMouseDown={handleMouseDown("green")}
          style={{
            position: "absolute",
            top: 0,
            left: `calc(${greenWidth}% - 6px)`,
            width: 12,
            height: 40,
            background: "rgba(255,255,255,0.9)",
            borderRadius: 4,
            cursor: "ew-resize",
            zIndex: 2,
            boxShadow: "0 0 4px rgba(0,0,0,0.4)",
          }}
          title={`Green/Yellow boundary: ${formatHour(greenEndHour)}`}
        />

        {/* Yellow/Red boundary handle */}
        <div
          onMouseDown={handleMouseDown("yellow")}
          style={{
            position: "absolute",
            top: 0,
            left: `calc(${(yellowEndHour / 24) * 100}% - 6px)`,
            width: 12,
            height: 40,
            background: "rgba(255,255,255,0.9)",
            borderRadius: 4,
            cursor: "ew-resize",
            zIndex: 2,
            boxShadow: "0 0 4px rgba(0,0,0,0.4)",
          }}
          title={`Yellow/Red boundary: ${formatHour(yellowEndHour)}`}
        />
      </div>

      {/* Hour labels */}
      <div style={{ display: "flex", justifyContent: "space-between", paddingLeft: 0 }}>
        {[0, 6, 12, 18, 24].map((h) => (
          <Text key={h} size="xs" c="dimmed" style={{ lineHeight: 1 }}>
            {formatHour(h)}
          </Text>
        ))}
      </div>
    </Stack>
  );
}
