import type { BackupListEntry, BackupContents, FileChange } from "../types/backup";

export const mockBackups: BackupListEntry[] = [
  { archive_path: "backups/nina-3.1.2-20260402.zip", package_id: "nina", version: "3.1.2", created_at: "2026-04-02T12:30:00Z", file_count: 12, total_size: 2516582 },
  { archive_path: "backups/phd2-2.6.13-20260401.zip", package_id: "phd2", version: "2.6.13", created_at: "2026-04-01T10:00:00Z", file_count: 3, total_size: 49152 },
  { archive_path: "backups/nina-3.1.1-20260320.zip", package_id: "nina", version: "3.1.1", created_at: "2026-03-20T09:15:00Z", file_count: 11, total_size: 2202009 },
  { archive_path: "backups/ascom-6.6-20260315.zip", package_id: "ascom", version: "6.6", created_at: "2026-03-15T14:20:00Z", file_count: 1, total_size: 348160 },
  { archive_path: "backups/nina-3.1.0-20260228.zip", package_id: "nina", version: "3.1.0", created_at: "2026-02-28T08:00:00Z", file_count: 10, total_size: 1992294 },
];

export const mockBackupContents: BackupContents = {
  metadata: mockBackups[0],
  files: [
    { name: "Profiles/Default.json", size: 12083, modified: "2026-03-15T09:10:00Z" },
    { name: "Profiles/Imaging.json", size: 8089, modified: "2026-03-15T09:10:00Z" },
    { name: "Profiles/Guiding.json", size: 3174, modified: "2026-03-15T09:10:00Z" },
    { name: "Settings/dock-layout.xml", size: 2150, modified: "2026-02-20T00:00:00Z" },
    { name: "Settings/filters.xml", size: 1843, modified: "2026-02-18T00:00:00Z" },
    { name: "Settings/equipment.xml", size: 4608, modified: "2026-02-15T00:00:00Z" },
    { name: "Settings/sequences.xml", size: 890, modified: "2026-02-10T00:00:00Z" },
    { name: "Settings/camera.xml", size: 1228, modified: "2026-02-10T00:00:00Z" },
    { name: "Settings/guider.xml", size: 980, modified: "2026-02-10T00:00:00Z" },
    { name: "Settings/focuser.xml", size: 760, modified: "2026-02-10T00:00:00Z" },
    { name: "HorizonDefinitions/home.hrz", size: 1433, modified: "2026-03-15T09:10:00Z" },
    { name: "Templates/narrowband.json", size: 4301, modified: "2026-03-15T09:10:00Z" },
  ],
};

export const mockRestorePreview: FileChange[] = [
  { name: "Profiles/Default.json", action: "overwrite", current_size: 12697, current_modified: "2026-03-30T14:22:00Z", backup_size: 12083, backup_modified: "2026-03-15T09:10:00Z" },
  { name: "Profiles/Imaging.json", action: "overwrite", current_size: 8396, current_modified: "2026-03-28T22:15:00Z", backup_size: 8089, backup_modified: "2026-03-15T09:10:00Z" },
  { name: "Profiles/Guiding.json", action: "overwrite", current_size: 3174, current_modified: "2026-03-25T18:00:00Z", backup_size: 3174, backup_modified: "2026-03-15T09:10:00Z" },
  { name: "Settings/dock-layout.xml", action: "unchanged", current_size: 2150, current_modified: "2026-02-20T00:00:00Z", backup_size: 2150, backup_modified: "2026-02-20T00:00:00Z" },
  { name: "Settings/filters.xml", action: "unchanged", current_size: 1843, current_modified: "2026-02-18T00:00:00Z", backup_size: 1843, backup_modified: "2026-02-18T00:00:00Z" },
  { name: "Settings/equipment.xml", action: "unchanged", current_size: 4608, current_modified: "2026-02-15T00:00:00Z", backup_size: 4608, backup_modified: "2026-02-15T00:00:00Z" },
  { name: "Settings/sequences.xml", action: "unchanged", current_size: 890, current_modified: "2026-02-10T00:00:00Z", backup_size: 890, backup_modified: "2026-02-10T00:00:00Z" },
  { name: "Settings/camera.xml", action: "unchanged", current_size: 1228, current_modified: "2026-02-10T00:00:00Z", backup_size: 1228, backup_modified: "2026-02-10T00:00:00Z" },
  { name: "Settings/guider.xml", action: "unchanged", current_size: 980, current_modified: "2026-02-10T00:00:00Z", backup_size: 980, backup_modified: "2026-02-10T00:00:00Z" },
  { name: "Settings/focuser.xml", action: "unchanged", current_size: 760, current_modified: "2026-02-10T00:00:00Z", backup_size: 760, backup_modified: "2026-02-10T00:00:00Z" },
  { name: "HorizonDefinitions/home.hrz", action: "new", current_size: null, current_modified: null, backup_size: 1433, backup_modified: "2026-03-15T09:10:00Z" },
  { name: "Templates/narrowband.json", action: "new", current_size: null, current_modified: null, backup_size: 4301, backup_modified: "2026-03-15T09:10:00Z" },
];
