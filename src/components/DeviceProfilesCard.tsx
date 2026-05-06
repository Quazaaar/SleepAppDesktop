import { useEffect, useMemo, useState } from "react";
import {
  ActionIcon,
  Badge,
  Button,
  Card,
  Group,
  Loader,
  Modal,
  Select,
  Stack,
  Table,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { notifications } from "@mantine/notifications";
import {
  IconDeviceDesktop,
  IconDeviceFloppy,
  IconEdit,
  IconPlus,
  IconRefresh,
  IconTrash,
} from "@tabler/icons-react";
import {
  createDeviceProfile,
  deleteDeviceProfile,
  listDeviceProfiles,
  renameDeviceProfile,
  saveActiveDeviceProfile,
  selectDeviceProfile,
  syncDeviceProfiles,
} from "../lib/commands";
import type { DeviceProfile, DeviceProfilesState } from "../lib/types";

interface DeviceProfilesCardProps {
  isLoggedIn: boolean;
  onProfileApplied: () => void;
}

type ModalMode = "create" | "rename";

function formatUpdatedAt(value: string) {
  try {
    return new Date(value).toLocaleString();
  } catch {
    return value;
  }
}

function profileSummary(profile: DeviceProfile) {
  const settings = profile.settings;
  return `${settings.reminder_rules.length} reminders, ${settings.ignored_apps.length} ignored, ${settings.app_categories.length} app categories`;
}

export function DeviceProfilesCard({
  isLoggedIn,
  onProfileApplied,
}: DeviceProfilesCardProps) {
  const [profilesState, setProfilesState] = useState<DeviceProfilesState>({
    profiles: [],
    active_profile_id: null,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [modalMode, setModalMode] = useState<ModalMode | null>(null);
  const [editingProfile, setEditingProfile] = useState<DeviceProfile | null>(null);
  const [profileName, setProfileName] = useState("");

  const activeProfile = useMemo(
    () =>
      profilesState.profiles.find(
        (profile) => profile.id === profilesState.active_profile_id
      ) ?? null,
    [profilesState]
  );

  const profileOptions = useMemo(
    () =>
      profilesState.profiles.map((profile) => ({
        value: profile.id,
        label: profile.name,
      })),
    [profilesState.profiles]
  );

  const refreshProfiles = async () => {
    setLoading(true);
    try {
      setProfilesState(await listDeviceProfiles());
    } catch (e) {
      notifications.show({
        title: "Profiles unavailable",
        message: String(e),
        color: "red",
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSyncProfiles = async (showNotification: boolean) => {
    if (!isLoggedIn) return;
    setSyncing(true);
    try {
      const nextState = await syncDeviceProfiles();
      setProfilesState(nextState);
      onProfileApplied();
      if (showNotification) {
        notifications.show({
          title: "Profiles synced",
          message: "Device profiles are up to date",
          color: "green",
        });
      }
    } catch (e) {
      if (showNotification) {
        notifications.show({
          title: "Profile sync failed",
          message: String(e),
          color: "red",
        });
      }
    } finally {
      setSyncing(false);
    }
  };

  useEffect(() => {
    refreshProfiles();
  }, []);

  useEffect(() => {
    if (isLoggedIn) {
      handleSyncProfiles(false);
    }
  }, [isLoggedIn]);

  const openCreateModal = () => {
    setModalMode("create");
    setEditingProfile(null);
    setProfileName("");
  };

  const openRenameModal = (profile: DeviceProfile) => {
    setModalMode("rename");
    setEditingProfile(profile);
    setProfileName(profile.name);
  };

  const closeModal = () => {
    setModalMode(null);
    setEditingProfile(null);
    setProfileName("");
  };

  const handleModalSave = async () => {
    const name = profileName.trim();
    if (!name || !modalMode) return;

    setSaving(true);
    try {
      const nextState =
        modalMode === "create"
          ? await createDeviceProfile(name)
          : await renameDeviceProfile(editingProfile!.id, name);
      setProfilesState(nextState);
      closeModal();
      onProfileApplied();
      notifications.show({
        title: modalMode === "create" ? "Profile created" : "Profile renamed",
        message: name,
        color: "green",
      });
    } catch (e) {
      notifications.show({
        title: "Profile update failed",
        message: String(e),
        color: "red",
      });
    } finally {
      setSaving(false);
    }
  };

  const handleSelectProfile = async (profileId: string | null) => {
    if (!profileId || profileId === profilesState.active_profile_id) return;

    setSaving(true);
    try {
      const nextState = await selectDeviceProfile(profileId);
      setProfilesState(nextState);
      onProfileApplied();
      const selected = nextState.profiles.find((profile) => profile.id === profileId);
      notifications.show({
        title: "Profile selected",
        message: selected?.name ?? "Settings applied",
        color: "green",
      });
    } catch (e) {
      notifications.show({
        title: "Profile switch failed",
        message: String(e),
        color: "red",
      });
    } finally {
      setSaving(false);
    }
  };

  const handleSaveActiveProfile = async () => {
    setSaving(true);
    try {
      const nextState = await saveActiveDeviceProfile();
      setProfilesState(nextState);
      notifications.show({
        title: "Profile saved",
        message: activeProfile?.name ?? "Current settings captured",
        color: "green",
      });
    } catch (e) {
      notifications.show({
        title: "Profile save failed",
        message: String(e),
        color: "red",
      });
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteProfile = async (profile: DeviceProfile) => {
    setSaving(true);
    try {
      const nextState = await deleteDeviceProfile(profile.id);
      setProfilesState(nextState);
      onProfileApplied();
      notifications.show({
        title: "Profile deleted",
        message: profile.name,
        color: "blue",
      });
    } catch (e) {
      notifications.show({
        title: "Profile delete failed",
        message: String(e),
        color: "red",
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card shadow="sm" padding="lg" radius="md" withBorder>
      <Group justify="space-between" align="center" mb="md">
        <Group gap="xs">
          <IconDeviceDesktop size={20} />
          <Title order={4}>Device Profiles</Title>
          {activeProfile && (
            <Badge size="sm" variant="light">
              {activeProfile.name}
            </Badge>
          )}
        </Group>
        {(loading || saving || syncing) && <Loader size="xs" />}
      </Group>

      <Stack gap="md">
        <Group align="end">
          <Select
            label="Current profile"
            data={profileOptions}
            value={profilesState.active_profile_id}
            onChange={handleSelectProfile}
            disabled={loading || saving}
            style={{ flex: 1 }}
          />
          <Button
            variant="light"
            leftSection={<IconDeviceFloppy size={14} />}
            onClick={handleSaveActiveProfile}
            loading={saving}
            disabled={!activeProfile}
          >
            Save
          </Button>
          <Button
            variant="light"
            leftSection={<IconRefresh size={14} />}
            onClick={() => handleSyncProfiles(true)}
            loading={syncing}
            disabled={!isLoggedIn}
          >
            Sync
          </Button>
          <Button leftSection={<IconPlus size={14} />} onClick={openCreateModal}>
            New
          </Button>
        </Group>

        {profilesState.profiles.length > 0 ? (
          <Table>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Profile</Table.Th>
                <Table.Th>Settings</Table.Th>
                <Table.Th>Updated</Table.Th>
                <Table.Th />
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {profilesState.profiles.map((profile) => (
                <Table.Tr key={profile.id}>
                  <Table.Td>
                    <Text size="sm" fw={profile.id === profilesState.active_profile_id ? 700 : 500}>
                      {profile.name}
                    </Text>
                    {profile.id === profilesState.active_profile_id && (
                      <Text size="xs" c="green">
                        this device
                      </Text>
                    )}
                  </Table.Td>
                  <Table.Td>
                    <Text size="xs" c="dimmed">
                      {profileSummary(profile)}
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    <Text size="xs" c="dimmed">
                      {formatUpdatedAt(profile.updated_at)}
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    <Group gap="xs" justify="flex-end" wrap="nowrap">
                      <ActionIcon
                        variant="light"
                        color="blue"
                        onClick={() => openRenameModal(profile)}
                        aria-label="Rename profile"
                      >
                        <IconEdit size={14} />
                      </ActionIcon>
                      <ActionIcon
                        variant="light"
                        color="red"
                        onClick={() => handleDeleteProfile(profile)}
                        disabled={profilesState.profiles.length <= 1}
                        aria-label="Delete profile"
                      >
                        <IconTrash size={14} />
                      </ActionIcon>
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        ) : (
          <Text size="sm" c="dimmed">
            No profiles found
          </Text>
        )}
      </Stack>

      <Modal
        opened={modalMode !== null}
        onClose={closeModal}
        title={modalMode === "rename" ? "Rename profile" : "New profile"}
      >
        <Stack>
          <TextInput
            label="Profile name"
            placeholder="Work"
            value={profileName}
            onChange={(event) => setProfileName(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                handleModalSave();
              }
            }}
            autoFocus
          />
          <Group justify="flex-end">
            <Button variant="subtle" onClick={closeModal}>
              Cancel
            </Button>
            <Button onClick={handleModalSave} loading={saving} disabled={!profileName.trim()}>
              Save
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Card>
  );
}
