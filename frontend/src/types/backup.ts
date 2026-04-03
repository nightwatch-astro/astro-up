export interface BackupListEntry {
  archive_path: string;
  package_id: string;
  version: string;
  created_at: string;
  file_count: number;
  total_size: number;
}

export interface BackupContents {
  metadata: BackupListEntry;
  files: BackupFile[];
}

export interface BackupFile {
  name: string;
  size: number;
  modified: string;
}

export interface FileChangeSummary {
  files: FileChange[];
}

export interface FileChange {
  name: string;
  action: "overwrite" | "unchanged" | "new" | "missing";
  current_size: number | null;
  current_modified: string | null;
  backup_size: number;
  backup_modified: string;
}
