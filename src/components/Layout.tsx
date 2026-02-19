import { AppShell, NavLink, Title, Group, Text } from "@mantine/core";
import {
  IconDashboard,
  IconTimeline,
  IconSettings,
  IconMoon,
} from "@tabler/icons-react";
import { useLocation, useNavigate } from "react-router-dom";
import TitleBar from "./TitleBar";

interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <AppShell navbar={{ width: 200, breakpoint: "sm" }} header={{ height: 32 }} padding="md">
      <AppShell.Header p={0} style={{ border: "none" }}>
        <TitleBar />
      </AppShell.Header>
      <AppShell.Navbar p="sm" style={{ display: "flex", flexDirection: "column" }}>
        <Group mb="md" px="xs">
          <IconMoon size={20} />
          <Title order={4}>Sleep App</Title>
        </Group>
        <NavLink
          label="Dashboard"
          leftSection={<IconDashboard size={18} />}
          active={location.pathname === "/"}
          onClick={() => navigate("/")}
          style={{ borderRadius: "var(--mantine-radius-md)" }}
        />
        <NavLink
          label="Timeline"
          leftSection={<IconTimeline size={18} />}
          active={location.pathname === "/timeline"}
          onClick={() => navigate("/timeline")}
          style={{ borderRadius: "var(--mantine-radius-md)" }}
        />
        <NavLink
          label="Settings"
          leftSection={<IconSettings size={18} />}
          active={location.pathname === "/settings"}
          onClick={() => navigate("/settings")}
          style={{ borderRadius: "var(--mantine-radius-md)" }}
        />
        <div style={{ flex: 1 }} />
        <Text size="xs" c="dimmed" ta="center" py="xs">
          v0.1.0
        </Text>
      </AppShell.Navbar>
      <AppShell.Main>{children}</AppShell.Main>
    </AppShell>
  );
}
