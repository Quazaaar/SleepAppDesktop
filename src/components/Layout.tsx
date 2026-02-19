import { useState } from "react";
import {
  AppShell,
  NavLink,
  Title,
  Group,
  Text,
  ActionIcon,
  Tooltip,
} from "@mantine/core";
import {
  IconDashboard,
  IconTimeline,
  IconSettings,
  IconMoon,
  IconLayoutSidebarLeftCollapse,
  IconLayoutSidebarLeftExpand,
} from "@tabler/icons-react";
import { useLocation, useNavigate } from "react-router-dom";
import TitleBar from "./TitleBar";

interface LayoutProps {
  children: React.ReactNode;
}

const NAV_ITEMS = [
  { label: "Dashboard", icon: IconDashboard, path: "/" },
  { label: "Timeline", icon: IconTimeline, path: "/timeline" },
  { label: "Settings", icon: IconSettings, path: "/settings" },
];

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [collapsed, setCollapsed] = useState(false);

  return (
    <AppShell
      navbar={{ width: collapsed ? 56 : 200, breakpoint: "sm" }}
      header={{ height: 32 }}
      padding="md"
      classNames={{ navbar: "app-navbar" }}
    >
      <AppShell.Header p={0} style={{ border: "none" }}>
        <TitleBar />
      </AppShell.Header>

      <AppShell.Navbar
        p="sm"
        style={{
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        {/* ── Logo / title row ── */}
        <Group
          mb="md"
          px="xs"
          gap="xs"
          wrap="nowrap"
          justify="left"
          style={{ minHeight: 28 }}
        >
          <IconMoon size={20} style={{ flexShrink: 0 }} />
          <div
            className={`nav-title-text ${
              collapsed ? "nav-title-text--hidden" : "nav-title-text--visible"
            }`}
          >
            <Title order={4} style={{ whiteSpace: "nowrap" }}>
              Sleep App
            </Title>
          </div>
        </Group>

        {/* ── Nav links ── */}
        {NAV_ITEMS.map(({ label, icon: Icon, path }) => (
          <Tooltip
            key={path}
            label={label}
            position="right"
            disabled={!collapsed}
            withArrow
          >
            <NavLink
              className={collapsed ? "nav-link-collapsed" : undefined}
              label={collapsed ? undefined : label}
              leftSection={<Icon size={18} />}
              active={location.pathname === path}
              onClick={() => navigate(path)}
              style={{
                borderRadius: "var(--mantine-radius-md)",
                justifyContent: collapsed ? "center" : undefined,
              }}
            />
          </Tooltip>
        ))}

        {/* ── Spacer ── */}
        <div style={{ flex: 1 }} />

        {/* ── Collapse / expand toggle ── */}
        <Tooltip
          label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          position="right"
          withArrow
        >
          <ActionIcon
            variant="subtle"
            color="gray"
            size="lg"
            onClick={() => setCollapsed((c) => !c)}
            style={{ alignSelf: "center" }}
            mb={4}
          >
            {collapsed ? (
              <IconLayoutSidebarLeftExpand size={18} />
            ) : (
              <IconLayoutSidebarLeftCollapse size={18} />
            )}
          </ActionIcon>
        </Tooltip>

        {/* ── Version text (fades in/out) ── */}
        <div
          className={`nav-version-text ${
            collapsed
              ? "nav-version-text--hidden"
              : "nav-version-text--visible"
          }`}
        >
          <Text size="xs" c="dimmed" ta="center" py="xs">
            v0.1.0
          </Text>
        </div>
      </AppShell.Navbar>

      <AppShell.Main>{children}</AppShell.Main>
    </AppShell>
  );
}