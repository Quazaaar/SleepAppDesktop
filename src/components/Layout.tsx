import { AppShell, NavLink, Title, Group } from "@mantine/core";
import {
  IconDashboard,
  IconTimeline,
  IconSettings,
} from "@tabler/icons-react";
import { useLocation, useNavigate } from "react-router-dom";

interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <AppShell
      navbar={{ width: 200, breakpoint: "sm" }}
      padding="md"
    >
      <AppShell.Navbar p="sm">
        <Group mb="md" px="xs">
          <Title order={4}>Sleep App</Title>
        </Group>
        <NavLink
          label="Dashboard"
          leftSection={<IconDashboard size={18} />}
          active={location.pathname === "/"}
          onClick={() => navigate("/")}
        />
        <NavLink
          label="Timeline"
          leftSection={<IconTimeline size={18} />}
          active={location.pathname === "/timeline"}
          onClick={() => navigate("/timeline")}
        />
        <NavLink
          label="Settings"
          leftSection={<IconSettings size={18} />}
          active={location.pathname === "/settings"}
          onClick={() => navigate("/settings")}
        />
      </AppShell.Navbar>
      <AppShell.Main>{children}</AppShell.Main>
    </AppShell>
  );
}
