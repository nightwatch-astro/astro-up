# Software Catalog

Astro-Up ships with a curated catalog of astrophotography software. The catalog is a pre-built SQLite database hosted on GitHub Releases, synced to your machine on first launch and periodically thereafter.

## What's in the Catalog

The catalog currently tracks 90+ packages across categories:

- **Capture suites** — N.I.N.A., Sequence Generator Pro, Voyager, APT
- **Plate solvers** — ASTAP, PlateSolve2, All-Sky Plate Solver
- **Planetarium** — Stellarium, Cartes du Ciel, TheSkyX
- **Image processing** — PixInsight, Siril, DeepSkyStacker, APP
- **Guiding** — PHD2, MetaGuide
- **ASCOM drivers** — Platform, device-specific drivers
- **Utilities** — SharpCap, FireCapture, EQMOD, Green Swamp Server

Each package entry includes an ID, display name, category, publisher, website, detection config, and version history with download URLs and checksums.

## Syncing

The catalog syncs automatically:

- On first launch
- Periodically based on the configured `cache_ttl` (default: 12 hours)
- Manually via the GUI's **Settings > Catalog > Re-download Now** button
- Via CLI: `astro-up sync --force`

## Cache

The catalog database is cached locally in your app data directory:

```
%APPDATA%\nightwatch\astro-up\catalog.db
```

Clear the cache via Settings or CLI: `astro-up clear --cache`
