import * as v from "valibot";

export const GeneralSchema = v.object({
  theme: v.picklist(["dark", "light", "system"]),
  font_size: v.picklist(["small", "medium", "large"]),
  auto_scan_on_launch: v.boolean(),
  default_install_scope: v.picklist(["user", "machine"]),
  default_install_method: v.picklist(["silent", "interactive"]),
  auto_check_updates: v.boolean(),
  check_interval: v.pipe(v.string(), v.minLength(1, "Required")),
  auto_notify_updates: v.boolean(),
  auto_install_updates: v.boolean(),
});

export const StartupSchema = v.object({
  start_at_login: v.boolean(),
  start_minimized: v.boolean(),
  minimize_to_tray_on_close: v.boolean(),
});

export const NotificationsSchema = v.object({
  enabled: v.boolean(),
  display_duration: v.pipe(v.number(), v.minValue(0, "Must be >= 0")),
  show_errors: v.boolean(),
  show_warnings: v.boolean(),
  show_update_available: v.boolean(),
  show_operation_complete: v.boolean(),
});

export const BackupPolicySchema = v.object({
  scheduled_enabled: v.boolean(),
  schedule: v.picklist(["daily", "weekly", "monthly"]),
  max_per_package: v.pipe(v.number(), v.minValue(0)),
  max_total_size_mb: v.pipe(v.number(), v.minValue(0)),
  max_age_days: v.pipe(v.number(), v.minValue(0)),
});

export const CatalogSchema = v.object({
  url: v.pipe(v.string(), v.url("Must be a valid URL")),
  cache_ttl: v.pipe(v.string(), v.minLength(1, "Required")),
});

export const NetworkSchema = v.object({
  proxy: v.nullable(v.string()),
  connect_timeout: v.pipe(v.string(), v.minLength(1, "Required")),
  timeout: v.pipe(v.string(), v.minLength(1, "Required")),
  download_speed_limit: v.pipe(v.number(), v.minValue(0)),
});

export const PathsSchema = v.object({
  download_dir: v.pipe(v.string(), v.minLength(1, "Required")),
  cache_dir: v.pipe(v.string(), v.minLength(1, "Required")),
  keep_installers: v.boolean(),
  purge_installers_after_days: v.pipe(v.number(), v.minValue(0)),
});

export const LogSchema = v.object({
  level: v.picklist(["error", "warn", "info", "debug", "trace"]),
  log_to_file: v.boolean(),
  log_file: v.string(),
});

export const AppConfigSchema = v.object({
  general: GeneralSchema,
  startup: StartupSchema,
  notifications: NotificationsSchema,
  backup: BackupPolicySchema,
  catalog: CatalogSchema,
  network: NetworkSchema,
  paths: PathsSchema,
  logging: LogSchema,
});
