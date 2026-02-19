import { invoke } from "@tauri-apps/api/core";
import type {
  CurrentAppInfo,
  DailyStats,
  ActivitySession,
  ReminderRule,
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
