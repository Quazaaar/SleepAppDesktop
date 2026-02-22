export interface ActivitySession {
  id: number | null;
  app_name: string;
  window_title: string;
  start_time: string;
  end_time: string;
  duration_secs: number;
  date: string;
}

export interface AppUsageStat {
  app_name: string;
  total_duration_secs: number;
  percentage: number;
  session_count: number;
}

export interface ReminderRule {
  id: number | null;
  rule_type: string;
  app_name: string | null;
  threshold_minutes: number;
  message: string;
  enabled: boolean;
}

export interface CurrentAppInfo {
  app_name: string;
  window_title: string;
  duration_secs: number;
}

export interface DailyStats {
  date: string;
  total_tracked_secs: number;
  app_usage: AppUsageStat[];
  most_used_app: string;
}

export interface SyncStatus {
  configured: boolean;
  last_sync_time: string | null;
}

export type EscalationLevel = "None" | "Level1" | "Level2" | "Level3" | "Level4" | "Done";

export interface EscalationStatePayload {
  level: EscalationLevel;
  message: string;
}

export interface EscalationSettings {
  green_end_hour: number;
  yellow_end_hour: number;
  sensitivity: number;
  enabled: boolean;
  paused_until: string | null;
  productive_multiplier: number;
  distracting_multiplier: number;
}

export type AppCategory = "productive" | "neutral" | "distracting" | "uncategorized";

export interface AppCategoryEntry {
  app_name: string;
  category: string;
  last_seen: string | null;
}

export interface TitleKeywordRule {
  id: number | null;
  app_name: string;
  keyword: string;
  category: string;
}

export interface WrapUpNote {
  session_key: string;
  working_on: string;
  next_steps: string;
  created_at: string;
}
