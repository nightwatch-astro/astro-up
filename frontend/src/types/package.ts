export type Category =
  | "Capture"
  | "Guiding"
  | "Platesolving"
  | "Equipment"
  | "Focusing"
  | "Planetarium"
  | "Viewers"
  | "Prerequisites"
  | "Usb"
  | "Driver";

export type SoftwareType =
  | "Application"
  | "Driver"
  | "Runtime"
  | "Database"
  | "UsbDriver"
  | "Resource";

export interface PackageSummary {
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

export interface VersionEntry {
  package_id: string;
  version: string;
  url: string;
  sha256: string | null;
  discovered_at: string;
  release_notes_url: string | null;
  pre_release: boolean;
}

export interface SearchResult {
  package: PackageSummary;
  rank: number;
}

export type DetectionResult =
  | { type: "Installed"; version: string; method: string }
  | { type: "InstalledUnknownVersion"; method: string }
  | { type: "NotInstalled" }
  | { type: "Unavailable"; reason: string };

export interface PackageWithStatus extends PackageSummary {
  installed_version?: string | null;
  latest_version?: string;
  update_available?: boolean;
  detection?: DetectionResult;
}
