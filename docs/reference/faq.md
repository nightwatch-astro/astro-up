# FAQ

## General

### Does Astro-Up work on Linux or macOS?

No. Astro-Up is Windows-only because the astrophotography software it manages (ASCOM drivers, N.I.N.A., SharpCap, etc.) is Windows-only. Detection uses Windows-specific methods (registry, PE files, ASCOM profiles, WMI).

### Does it work offline?

Astro-Up requires internet access to sync the catalog and download installers. It caches the catalog locally so browsing and detection work offline after the first sync.

### Can I use it on multiple imaging PCs?

Yes. Each installation is independent — install Astro-Up on each PC and it detects what's installed locally. Configuration and backups are per-machine.

### Is it free?

Yes. Astro-Up is free and open source under the Apache 2.0 license.

## Software Catalog

### How often is the catalog updated?

The catalog is updated whenever new packages or detection configs are added to the [astro-up-manifests](https://github.com/nightwatch-astro/astro-up-manifests) repository. Version data is checked periodically by CI.

### My software isn't in the catalog

You can [request it](https://github.com/nightwatch-astro/astro-up-manifests/issues) or [add it yourself](/guide/adding-manifests).

### What are package IDs?

Each package has a unique ID using `vendor-product` convention (e.g., `nina-app`, `zwo-asi-camera-driver`). Use these in CLI commands. Browse the catalog in the GUI or via `astro-up list`.

## Updates

### What happens if an update breaks something?

Astro-Up backs up your configuration before every update. Restore with the Backup view in the GUI or `astro-up restore <package>`. The backup covers config files — for the app itself, reinstall the previous version from the vendor.

### Can I skip a version?

There's no explicit "skip" feature. Astro-Up shows the latest available version. If you don't want to update, simply don't run the update for that package.

## Configuration

### Where are settings stored?

All settings live in `%APPDATA%\nightwatch\astro-up\config.toml`. You can edit this file directly, use the GUI Settings panel, or use `astro-up config --edit`. See the [full reference](/reference/config).

### Can I use different settings per imaging PC?

Yes. Each PC has its own config file. Settings are not synced between machines.
