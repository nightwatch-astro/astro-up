# Quickstart: Portable Apps Folder

## What Changes

After this feature, `download_only` packages (17 in catalog: firmware tools, NINA caches, utilities) are placed in an organized directory instead of dumped in a temporary download folder.

## User Flow

1. Open Astro-Up, navigate to a download-only package (e.g., iOptron Upgrade Utility)
2. Click **Install**
3. The app downloads the file and copies/extracts it to `C:\Users\{you}\AppData\Roaming\nightwatch\astro-up\apps\{package-id}\`
4. The operations dock shows the destination path — click to open in Explorer
5. The package appears in the Installed view with the install path visible in Technical tab

## Configuring the Directory

Settings > Paths > **Portable Apps Directory**

Default: `C:\Users\{you}\AppData\Roaming\nightwatch\astro-up\apps\`

Change it to any writable directory (e.g., `D:\AstroTools\`). New installs use the new path; existing apps stay where they are.

## Files Touched

| Area | Files | Change |
|------|-------|--------|
| Config model | `config/model.rs`, `defaults.rs`, `mod.rs` | Add `portable_apps_dir` field |
| Install handlers | `install/mod.rs` | Modify `handle_download_only` to copy/extract; update `handle_portable_install` target |
| GUI commands | `commands.rs` | Set `install_dir` on `InstallRequest` for download-only/portable |
| Frontend | `PathsSection.vue`, `config.ts`, `validation.ts`, `SettingsView.vue` | Add path input field |
