import { invoke } from "@tauri-apps/api/core";
import type {
  CurrentAppInfo,
  DailyStats,
  ActivitySession,
  ReminderRule,
  SyncStatus,
  EscalationSettings,
} from "./types";

export async function getCurrentApp(): Promise<CurrentAppInfo> {
  return invoke("get_current_app");
}

export async function getDailyStats(date: string): Promise<DailyStats> {
  return invoke("get_daily_stats", { date });
}

export async function getActivityTimeline(
  date: string
): Promise<ActivitySession[]> {
  return invoke("get_activity_timeline", { date });
}

export async function toggleTracking(): Promise<boolean> {
  return invoke("toggle_tracking");
}

export async function getIgnoredApps(): Promise<string[]> {
  return invoke("get_ignored_apps");
}

export async function setIgnoredApps(apps: string[]): Promise<void> {
  return invoke("set_ignored_apps", { apps });
}

export async function getReminderRules(): Promise<ReminderRule[]> {
  return invoke("get_reminder_rules");
}

export async function saveReminderRule(rule: ReminderRule): Promise<void> {
  return invoke("save_reminder_rule", { rule });
}

export async function deleteReminderRule(ruleId: number): Promise<void> {
  return invoke("delete_reminder_rule", { ruleId });
}

export async function toggleReminderRule(
  ruleId: number,
  enabled: boolean
): Promise<void> {
  return invoke("toggle_reminder_rule", { ruleId, enabled });
}

export async function syncNow(): Promise<number> {
  return invoke("sync_now");
}

export async function setSyncConfig(
  syncUrl: string,
  apiKey: string
): Promise<void> {
  return invoke("set_sync_config", { syncUrl, apiKey });
}

export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke("get_sync_status");
}

export async function showEscalationWindow(level: string): Promise<void> {
  return invoke("show_escalation_window", { level });
}

export async function dismissEscalation(): Promise<void> {
  return invoke("dismiss_escalation");
}

export async function getEscalationSettings(): Promise<EscalationSettings> {
  return invoke("get_escalation_settings");
}

export async function setEscalationSettings(settings: EscalationSettings): Promise<void> {
  return invoke("set_escalation_settings", { settings });
}
