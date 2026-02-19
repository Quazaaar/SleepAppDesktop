import { Text } from "@mantine/core";

async function minimize() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().minimize();
}

async function maximize() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().toggleMaximize();
}

async function close() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().close();
}

export default function TitleBar() {
  return (
    <div data-tauri-drag-region className="titlebar">
      <Text size="xs" fw={500} c="dimmed" style={{ pointerEvents: "none" }}>
        Sleep App
      </Text>
      <div className="titlebar-controls">
        <button className="titlebar-btn" onClick={minimize}>─</button>
        <button className="titlebar-btn" onClick={maximize}>□</button>
        <button className="titlebar-btn titlebar-btn-close" onClick={close}>✕</button>
      </div>
    </div>
  );
}
