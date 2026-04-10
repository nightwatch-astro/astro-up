export interface AppConfig {
  ui: UiConfig;
  startup: StartupConfig;
  notifications: NotificationsConfig;
  backup_policy: BackupPolicyConfig;
  catalog: CatalogConfig;
  paths: PathsConfig;
  network: NetworkConfig;
  updates: UpdateConfig;
  logging: LogConfig;
  telemetry: TelemetryConfig;
}

export type ScanInterval = "manual" | "on_startup" | "hourly" | "daily" | "weekly";

export interface UiConfig {
  theme: "dark" | "light" | "system";
  font_size: "small" | "medium" | "large";
  auto_scan_on_launch: boolean;
  scan_interval: ScanInterval;
  default_install_scope: "user" | "machine";
  default_install_method: "silent" | "interactive";
  auto_check_updates: boolean;
  check_interval: string;
  auto_notify_updates: boolean;
  auto_install_updates: boolean;
}

export interface StartupConfig {
  start_at_login: boolean;
  start_minimized: boolean;
  minimize_to_tray_on_close: boolean;
}

export interface NotificationsConfig {
  enabled: boolean;
  display_duration: number;
  show_errors: boolean;
  show_warnings: boolean;
  show_update_available: boolean;
  show_operation_complete: boolean;
}

export interface BackupPolicyConfig {
  scheduled_enabled: boolean;
  schedule: "daily" | "weekly" | "monthly";
  max_per_package: number;
  max_total_size_mb: number;
  max_age_days: number;
}

export interface CatalogConfig {
  url: string;
  cache_ttl: string;
}

export interface NetworkConfig {
  proxy: string | null;
  connect_timeout: string;
  timeout: string;
  user_agent: string;
  download_speed_limit: number;
}

export interface PathsConfig {
  download_dir: string;
  cache_dir: string;
  data_dir: string;
  keep_installers: boolean;
  purge_installers_after_days: number;
}

export interface UpdateConfig {
  auto_check: boolean;
  check_interval: string;
}

export interface LogConfig {
  level: "error" | "warn" | "info" | "debug" | "trace";
  log_to_file: boolean;
  log_file: string;
  max_age_days: number;
}

export interface TelemetryConfig {
  enabled: boolean;
}
