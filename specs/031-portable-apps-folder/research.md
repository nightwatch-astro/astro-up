# Research: Portable Apps Folder

## Default Path Selection

**Decision**: `{data_dir}/../apps/` → `{AppData}/Roaming/nightwatch/astro-up/apps/`

**Rationale**: Sibling to the existing `data/` directory. Uses `Roaming` AppData (not `Local`) so it roams with the user profile on domain-joined machines. The `directories` crate already resolves `data_dir` — placing `apps/` as a sibling keeps everything under the same `nightwatch/astro-up/` umbrella.

**Alternatives considered**:
- `Documents/AstroUp/` — too visible, pollutes Documents
- `Program Files/AstroUp/` — requires elevation
- `{Local AppData}/nightwatch/astro-up/apps/` — doesn't roam, but avoids syncing large files. Acceptable alternative.

## Zip Detection Strategy

**Decision**: Check file extension (`.zip`) and magic bytes (`PK\x03\x04`) to determine if download should be extracted vs copied.

**Rationale**: The `zip` crate already exists in the project. Some downloads are zip archives with no `.zip` extension. Checking magic bytes is reliable and cheap (4 bytes).

**Alternatives considered**:
- Extension-only: misses extensionless zips
- MIME type: not available from local file

## Install Request Plumbing

**Decision**: Set `InstallRequest.install_dir = Some({portable_apps_dir}/{package-id})` for download-only and portable packages. No changes to `InstallerService` constructor.

**Rationale**: `install_dir` already exists on `InstallRequest` and `resolve_install_dir` already checks it. This minimizes code changes — the GUI commands just set the field before calling the installer.

**Alternatives considered**:
- Add `portable_apps_dir` to `InstallerService` — more structural change, ties the service to a specific config concept
- Add a new `InstallMethod::PortableApp` — creates a new variant that manifests would need to adopt

## Explorer Open Removal

**Decision**: Remove the `explorer.exe` spawn from `handle_download_only`. The UI already shows the destination path via `PackageComplete.download_path` — clicking that in the operations dock opens the folder.

**Rationale**: The Explorer spawn is a workaround for having no proper install location. Once files land in the portable dir, the UI path display is sufficient.
