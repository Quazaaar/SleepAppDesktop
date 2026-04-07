import { useCallback, useRef, useState } from "react";
import {
  Card,
  Stack,
  Title,
  Text,
  SegmentedControl,
  Group,
  Button,
  Slider,
  Badge,
  Modal,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconUpload,
  IconTrash,
  IconEdit,
  IconPlayerPlay,
  IconPlayerStop,
} from "@tabler/icons-react";
import { useAppTheme } from "../context/ThemeContext";
import type { AppThemeId } from "../lib/theme";
import { useEscalationPreview } from "../hooks/useEscalationPreview";

const BG_TARGETS = [
  {
    key: "App",
    label: "Application Background",
    desc: "Main dashboard and app background",
    previewWidth: 400,
    previewHeight: 250,
  },
  {
    key: "Level2",
    label: "Level 2 — Popup",
    desc: "Small floating notification (320×140)",
    previewWidth: 384,
    previewHeight: 168,
  },
  {
    key: "Level3",
    label: "Level 3 — Side Panel",
    desc: "Right-side panel (~30% screen width, full height)",
    previewWidth: 200,
    previewHeight: 380,
  },
  {
    key: "Level4",
    label: "Level 4 — Fullscreen",
    desc: "Full-screen overlay",
    previewWidth: 400,
    previewHeight: 225,
  },
] as const;

interface EditorState {
  level: (typeof BG_TARGETS)[number];
  image: string;
  zoom: number;
  posX: number;
  posY: number;
}

function BgPositionEditor({
  editor,
  onUpdate,
  onSave,
  onCancel,
}: {
  editor: EditorState;
  onUpdate: (patch: Partial<EditorState>) => void;
  onSave: () => void;
  onCancel: () => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);
  const dragStart = useRef({ x: 0, y: 0, posX: 0, posY: 0 });

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      dragging.current = true;
      dragStart.current = {
        x: e.clientX,
        y: e.clientY,
        posX: editor.posX,
        posY: editor.posY,
      };

      const handleMouseMove = (ev: MouseEvent) => {
        if (!dragging.current || !containerRef.current) return;
        const rect = containerRef.current.getBoundingClientRect();
        const dx = ev.clientX - dragStart.current.x;
        const dy = ev.clientY - dragStart.current.y;
        const newPosX = Math.max(0, Math.min(100, dragStart.current.posX - (dx / rect.width) * 100));
        const newPosY = Math.max(0, Math.min(100, dragStart.current.posY - (dy / rect.height) * 100));
        onUpdate({ posX: newPosX, posY: newPosY });
      };

      const handleMouseUp = () => {
        dragging.current = false;
        window.removeEventListener("mousemove", handleMouseMove);
        window.removeEventListener("mouseup", handleMouseUp);
      };

      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
    },
    [editor.posX, editor.posY, onUpdate]
  );

  return (
    <Stack gap="md">
      <div
        ref={containerRef}
        onMouseDown={handleMouseDown}
        style={{
          width: editor.level.previewWidth,
          height: editor.level.previewHeight,
          maxWidth: "100%",
          borderRadius: "var(--mantine-radius-md)",
          overflow: "hidden",
          border: "1px solid var(--glass-border)",
          backgroundImage: `url(${editor.image})`,
          backgroundSize: `${editor.zoom * 100}%`,
          backgroundPosition: `${editor.posX}% ${editor.posY}%`,
          backgroundRepeat: "no-repeat",
          cursor: "grab",
          position: "relative",
          margin: "0 auto",
        }}
      >
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            pointerEvents: "none",
          }}
        >
          <Text size="sm" c="white" fw={600} style={{ opacity: 0.7 }}>
            {editor.level.label}
          </Text>
        </div>
      </div>

      <div>
        <Text size="sm" fw={500} mb="xs">Zoom</Text>
        <Slider
          min={1}
          max={3}
          step={0.05}
          value={editor.zoom}
          onChange={(v) => onUpdate({ zoom: v })}
          label={(v) => `${v.toFixed(1)}x`}
          marks={[
            { value: 1, label: "1x" },
            { value: 2, label: "2x" },
            { value: 3, label: "3x" },
          ]}
          mb="lg"
        />
      </div>

      <Group justify="flex-end">
        <Button variant="subtle" onClick={onCancel}>Cancel</Button>
        <Button onClick={onSave}>Save</Button>
      </Group>
    </Stack>
  );
}

export default function Theme() {
  const { themeId, setTheme, escalationBgs, setEscalationBg, clearEscalationBg } = useAppTheme();
  const { activePreview, handlePreview, clearPreview } = useEscalationPreview();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [pickingLevel, setPickingLevel] = useState<string | null>(null);
  const [editor, setEditor] = useState<EditorState | null>(null);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !pickingLevel) return;

    if (file.size > 2 * 1024 * 1024) {
      notifications.show({
        title: "Image too large",
        message: "Please select an image under 2MB.",
        color: "red",
      });
      e.target.value = "";
      return;
    }

    const level = BG_TARGETS.find((l) => l.key === pickingLevel);
    if (!level) return;

    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result as string;
      const existing = escalationBgs[pickingLevel];
      setEditor({
        level,
        image: dataUrl,
        zoom: existing?.zoom ?? 1,
        posX: existing?.posX ?? 50,
        posY: existing?.posY ?? 50,
      });
    };
    reader.readAsDataURL(file);
    e.target.value = "";
  };

  const openFilePicker = (levelKey: string) => {
    setPickingLevel(levelKey);
    fileInputRef.current?.click();
  };

  const openEditor = (levelKey: string) => {
    const level = BG_TARGETS.find((l) => l.key === levelKey);
    const config = escalationBgs[levelKey];
    if (!level || !config) return;
    setEditor({
      level,
      image: config.image,
      zoom: config.zoom,
      posX: config.posX,
      posY: config.posY,
    });
  };

  const handleEditorSave = () => {
    if (!editor) return;
    setEscalationBg(editor.level.key, {
      image: editor.image,
      zoom: editor.zoom,
      posX: editor.posX,
      posY: editor.posY,
    });
    setEditor(null);
  };

  return (
    <Stack>
      <Title order={2}>Theme</Title>

      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        style={{ display: "none" }}
        onChange={handleFileSelect}
      />

      {/* Theme Preset */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Title order={4} mb="xs">Theme Preset</Title>
        <Text size="sm" c="dimmed" mb="sm">
          Choose a base theme for the app
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

      {/* Escalation Backgrounds */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Title order={4} mb="xs">Custom Backgrounds</Title>
        <Text size="sm" c="dimmed" mb="md">
          Set a custom background image for the app and each escalation level
        </Text>

        <Stack gap="md">
          {BG_TARGETS.map((level) => {
            const config = escalationBgs[level.key];
            return (
              <Card key={level.key} padding="sm" radius="sm" withBorder>
                <Group justify="space-between" align="center">
                  <div style={{ flex: 1 }}>
                    <Text size="sm" fw={600}>{level.label}</Text>
                    <Text size="xs" c="dimmed">{level.desc}</Text>
                  </div>

                  {config && (
                    <div
                      style={{
                        width: 64,
                        height: 40,
                        borderRadius: 4,
                        overflow: "hidden",
                        border: "1px solid var(--glass-border)",
                        backgroundImage: `url(${config.image})`,
                        backgroundSize: `${config.zoom * 100}%`,
                        backgroundPosition: `${config.posX}% ${config.posY}%`,
                        backgroundRepeat: "no-repeat",
                        flexShrink: 0,
                      }}
                    />
                  )}

                  <Group gap="xs">
                    <Button
                      size="xs"
                      variant="light"
                      leftSection={<IconUpload size={14} />}
                      onClick={() => openFilePicker(level.key)}
                    >
                      {config ? "Replace" : "Choose Image"}
                    </Button>
                    {config && (
                      <>
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconEdit size={14} />}
                          onClick={() => openEditor(level.key)}
                        >
                          Edit
                        </Button>
                        <Button
                          size="xs"
                          variant="light"
                          color="red"
                          leftSection={<IconTrash size={14} />}
                          onClick={() => clearEscalationBg(level.key)}
                        >
                          Remove
                        </Button>
                      </>
                    )}
                  </Group>
                </Group>
              </Card>
            );
          })}
        </Stack>
      </Card>

      {/* Escalation Preview */}
      <Card shadow="sm" padding="lg" radius="md" withBorder>
        <Group justify="space-between" mb="xs">
          <Title order={4}>Escalation Preview</Title>
          {activePreview && (
            <Badge size="sm" color="yellow" variant="light">
              {activePreview} active
            </Badge>
          )}
        </Group>
        <Text size="sm" c="dimmed" mb="sm">
          Preview each escalation level to see how they appear
        </Text>
        <Group gap="xs">
          <Button
            size="xs"
            variant="light"
            leftSection={<IconPlayerPlay size={14} />}
            onClick={() => handlePreview("Level1")}
            disabled={activePreview === "Level1"}
          >
            Level 1 — Toast
          </Button>
          <Button
            size="xs"
            variant="light"
            leftSection={<IconPlayerPlay size={14} />}
            onClick={() => handlePreview("Level2")}
            disabled={activePreview === "Level2"}
          >
            Level 2 — Popup
          </Button>
          <Button
            size="xs"
            variant="light"
            leftSection={<IconPlayerPlay size={14} />}
            onClick={() => handlePreview("Level3")}
            disabled={activePreview === "Level3"}
          >
            Level 3 — Panel
          </Button>
          <Button
            size="xs"
            variant="light"
            leftSection={<IconPlayerPlay size={14} />}
            onClick={() => handlePreview("Level4")}
            disabled={activePreview === "Level4"}
          >
            Level 4 — Fullscreen
          </Button>
          {activePreview && (
            <Button
              size="xs"
              variant="filled"
              color="red"
              leftSection={<IconPlayerStop size={14} />}
              onClick={clearPreview}
            >
              Close Preview
            </Button>
          )}
        </Group>
      </Card>

      {/* Edit Position Modal */}
      <Modal
        opened={editor !== null}
        onClose={() => setEditor(null)}
        title={editor ? `Edit Position — ${editor.level.label}` : ""}
        size="lg"
        centered
      >
        {editor && (
          <BgPositionEditor
            editor={editor}
            onUpdate={(patch) => setEditor((prev) => prev ? { ...prev, ...patch } : prev)}
            onSave={handleEditorSave}
            onCancel={() => setEditor(null)}
          />
        )}
      </Modal>
    </Stack>
  );
}
