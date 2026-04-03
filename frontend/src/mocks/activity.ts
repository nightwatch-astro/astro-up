export interface ActivityEntry {
  id: string;
  type: "install" | "update" | "scan" | "backup" | "restore";
  name: string;
  detail: string;
  timestamp: string;
}

export const mockActivity: ActivityEntry[] = [
  { id: "1", type: "update", name: "N.I.N.A. updated to 3.2.0", detail: "Capture \u00b7 Silent install", timestamp: "2026-04-02T12:30:00Z" },
  { id: "2", type: "install", name: "PHD2 installed (2.6.13)", detail: "Guiding \u00b7 InnoSetup", timestamp: "2026-04-01T10:00:00Z" },
  { id: "3", type: "scan", name: "Full scan completed", detail: "12 packages found, 3 updates available", timestamp: "2026-04-01T08:00:00Z" },
  { id: "4", type: "install", name: "ASCOM Platform installed (6.6 SP2)", detail: "Prerequisites \u00b7 MSI", timestamp: "2026-03-30T14:00:00Z" },
];
