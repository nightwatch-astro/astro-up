/** Adjacently tagged event from astro-up-core: { type: "...", data: {...} } */
export type CoreEvent =
  | { type: "check_started"; data: { id: string } }
  | { type: "check_progress"; data: { id: string; progress: number } }
  | { type: "check_complete"; data: { id: string } }
  | { type: "download_started"; data: { id: string; url: string } }
  | {
      type: "download_progress";
      data: {
        id: string;
        progress: number;
        bytes_downloaded: number;
        total_bytes: number;
        speed: number;
        elapsed: { secs: number; nanos: number };
        estimated_remaining: { secs: number; nanos: number } | null;
      };
    }
  | { type: "download_complete"; data: { id: string } }
  | { type: "backup_started"; data: { id: string } }
  | {
      type: "backup_progress";
      data: { id: string; files_processed: number; total_files: number };
    }
  | { type: "backup_complete"; data: { id: string } }
  | { type: "restore_started"; data: { id: string } }
  | { type: "restore_complete"; data: { id: string } }
  | { type: "install_started"; data: { id: string } }
  | { type: "install_complete"; data: { id: string } }
  | { type: "install_failed"; data: { id: string; error: string } }
  | { type: "install_reboot_required"; data: { id: string } }
  | { type: "manual_download_required"; data: { id: string; url: string } }
  | { type: "error"; data: { id: string; error: string } }
  | { type: "scan_started"; data: Record<string, never> }
  | { type: "scan_progress"; data: { progress: number; current_id: string } }
  | { type: "scan_complete"; data: { total_found: number } }
  | { type: "plan_ready"; data: { total: number; skipped: number } }
  | {
      type: "package_started";
      data: { package_id: string; step_count: number };
    }
  | {
      type: "package_complete";
      data: { package_id: string; status: string; error?: string; download_path?: string };
    }
  | {
      type: "package_skipped";
      data: { package_id: string; reason: string };
    }
  | {
      type: "process_blocking";
      data: { package_id: string; process_name: string; pid: number };
    }
  | {
      type: "orchestration_complete";
      data: { succeeded: number; failed: number; skipped: number };
    };

export interface OperationId {
  id: string;
}

export interface CoreError {
  message: string;
  code: string;
}

export interface UpdateAvailable {
  id: string;
  current_version: string;
  latest_version: string;
}

export interface ErrorLogEntry {
  timestamp: Date;
  severity: "error" | "warning";
  summary: string;
  detail: string;
}
