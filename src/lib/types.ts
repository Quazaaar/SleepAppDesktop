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
