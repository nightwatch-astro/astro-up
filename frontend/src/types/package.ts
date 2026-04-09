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
  icon_base64?: string | null;
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
  | { type: "Installed"; version: string; method: string; install_path?: string | null }
  | { type: "InstalledUnknownVersion"; method: string; install_path?: string | null }
  | { type: "NotInstalled" }
  | { type: "Unavailable"; reason: string };

export interface BackupConfig {
  config_paths: string[];
}

export type InstallMethod =
  | "exe"
  | "msi"
  | "inno_setup"
  | "nsis"
  | "wix"
  | "burn"
  | "zip"
  | "portable"
  | "download_only";

export type InstallScope = "machine" | "user" | "either";

export type InstallElevation = "required" | "prohibited" | "self";

export interface InstallConfig {
  method: InstallMethod;
  zip_wrapped: boolean;
  zip_inner_path?: string | null;
  scope?: InstallScope | null;
  elevation?: InstallElevation | null;
  upgrade_behavior?: string | null;
  install_modes: string[];
  success_codes: number[];
  pre_install: string[];
  post_install: string[];
  switches?: Record<string, unknown> | null;
  known_exit_codes: Record<string, string>;
  timeout?: number | null;
}

export interface PackageWithStatus extends PackageSummary {
  installed_version?: string | null;
  latest_version?: string;
  update_available?: boolean;
  detection?: DetectionResult;
  backup?: BackupConfig | null;
  install?: InstallConfig | null;
}
