# Data Model: Vue Frontend Views

## TypeScript Types

### Package Types (mirror Rust `astro-up-core` types)

```typescript
interface PackageSummary {
  id: string;
  name: string;
  slug: string;
  description: string | null;
  publisher: string | null;
  homepage: string | null;
  category: Category;
  software_type: SoftwareType;
  license: string | null;
  aliases: string[];
  tags: string[];
  dependencies: string[];
  manifest_version: number;
}

type Category =
  | "Capture" | "Guiding" | "Platesolving" | "Equipment"
  | "Focusing" | "Planetarium" | "Viewers" | "Prerequisites"
  | "Usb" | "Driver";

type SoftwareType =
  | "Application" | "Driver" | "Runtime" | "Database"
  | "UsbDriver" | "Resource";

interface VersionEntry {
  package_id: string;
  version: string;
  url: string;
  sha256: string | null;
  discovered_at: string; // ISO datetime
  release_notes_url: string | null;
  pre_release: boolean;
}

interface SearchResult {
  package: PackageSummary;
  rank: number;
}
```

### Detection & Installation Status

```typescript
type DetectionResult =
  | { type: "Installed"; version: string; method: string }
  | { type: "InstalledUnknownVersion"; method: string }
  | { type: "NotInstalled" }
  | { type: "Unavailable"; reason: string };

// Combined view for the frontend
interface PackageWithStatus extends PackageSummary {
  installed_version: string | null;
  latest_version: string;
  update_available: boolean;
  detection: DetectionResult;
}
```

### Backup Types

```typescript
interface BackupListEntry {
  archive_path: string;
  package_id: string;
  version: string;
  created_at: string; // ISO datetime
  file_count: number;
  total_size: number; // bytes
}

interface BackupContents {
  metadata: BackupListEntry;
  files: BackupFile[];
}

interface BackupFile {
  name: string;         // relative path within backup
  size: number;         // bytes
  modified: string;     // ISO datetime
}

interface FileChangeSummary {
  files: FileChange[];
}

interface FileChange {
  name: string;
  action: "overwrite" | "unchanged" | "new" | "missing";
  current_size: number | null;
  current_modified: string | null;
  backup_size: number;
  backup_modified: string;
}
```

### Config Types (mirror Rust `AppConfig`)

```typescript
interface AppConfig {
  general: GeneralConfig;
  startup: StartupConfig;
  notifications: NotificationsConfig;
  backup: BackupPolicyConfig;
  catalog: CatalogConfig;
  network: NetworkConfig;
  paths: PathsConfig;
  logging: LogConfig;
}

interface GeneralConfig {
  theme: "dark" | "light" | "system";
  font_size: "small" | "medium" | "large";
  auto_scan_on_launch: boolean;
  default_install_scope: "user" | "machine";
  default_install_method: "silent" | "interactive";
  auto_check_updates: boolean;
  check_interval: string; // humantime duration
  auto_notify_updates: boolean;
  auto_install_updates: boolean;
}

interface StartupConfig {
  start_at_login: boolean;
  start_minimized: boolean;
  minimize_to_tray_on_close: boolean;
}

interface NotificationsConfig {
  enabled: boolean;
  display_duration: number; // seconds, 0 = never auto-dismiss
  show_errors: boolean;
  show_warnings: boolean;
  show_update_available: boolean;
  show_operation_complete: boolean;
}

interface BackupPolicyConfig {
  scheduled_enabled: boolean;
  schedule: "daily" | "weekly" | "monthly";
  max_per_package: number; // 0 = unlimited
  max_total_size_mb: number; // 0 = unlimited
  max_age_days: number; // 0 = never expire
}

interface CatalogConfig {
  url: string;
  cache_ttl: string; // humantime duration
}

interface NetworkConfig {
  proxy: string | null;
  connect_timeout: string;
  timeout: string;
  download_speed_limit: number; // bytes/s, 0 = unlimited
}

interface PathsConfig {
  download_dir: string;
  cache_dir: string;
  keep_installers: boolean;
  purge_installers_after_days: number;
}

interface LogConfig {
  level: "error" | "warn" | "info" | "debug" | "trace";
  log_to_file: boolean;
  log_file: string;
}
```

### Operation Types

```typescript
interface Operation {
  id: string;
  label: string;
  progress: number; // 0-100
  status: "running" | "complete" | "failed" | "cancelled";
  steps: OperationStep[];
}

interface OperationStep {
  timestamp: string;
  message: string;
  level: "info" | "warn" | "error";
}
```

### Config Snapshot Types

```typescript
interface ConfigSnapshot {
  id: string;          // UUID
  timestamp: string;   // ISO datetime
  config: AppConfig;
}
```

### Log Types

```typescript
interface LogEntry {
  timestamp: string;
  level: "error" | "warn" | "info" | "debug" | "trace";
  target: string;      // module path
  message: string;
}
```

## State Management

### Reactive State (not persisted)

| State | Scope | Source |
|-------|-------|--------|
| Current page / route | Global | vue-router |
| Active operation | Global | `useOperations` composable |
| Log panel visible | Global | Reactive ref, toggled by status bar |
| Log panel entries | Global | Array of LogEntry, max 1000, from event listener |
| Log level filter | Session | Reactive ref, not persisted |
| Search text (per page) | Per-page | Reactive ref, preserved in session |
| Active category filter | Catalog | Reactive ref |
| Selected package (detail) | Detail | Route param `:id` |

### Persisted State

| State | Storage | Source |
|-------|---------|--------|
| App config | SQLite via Tauri | `get_config` / `save_config` |
| Config snapshots | localStorage | `useStorage` from @vueuse |
| Window position/size | Tauri plugin | `@tauri-apps/plugin-window-state` |
| Theme preference | Part of AppConfig | `general.theme` |

### Server State (VueQuery cache)

| Query Key | Command | Stale Time | Invalidated By |
|-----------|---------|------------|----------------|
| `["software", filter]` | `list_software` | 5 min | scan, install, update |
| `["catalog-search", q]` | `search_catalog` | 1 min | — |
| `["updates"]` | `check_for_updates` | 5 min | scan, install, update |
| `["config"]` | `get_config` | Infinity | save_config |
| `["backups"]` | mock | 5 min | backup, restore, delete |
| `["backup-contents", a]` | mock | Infinity | — |
| `["backup-preview", a]` | mock | Infinity | — |
