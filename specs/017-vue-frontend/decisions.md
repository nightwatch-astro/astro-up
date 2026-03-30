# Decisions Report: 017-vue-frontend

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: PrimeVue 4 as primary component library
**Choice**: PrimeVue. Best DataTable (virtual scroll, row expansion, selection, context menu). Evaluated Naive UI, Element Plus, Vuetify 3, Quasar, Radix Vue, shadcn-vue — none match PrimeVue's table capabilities.
**Alternatives rejected**: Quasar (4.5/5 — excellent for desktop but heavier and more opinionated), Naive UI (missing key table features), shadcn-vue (no DataTable at all)

### D2: Pinia from the start
**Choice**: Use Pinia for client-side state even though the app is simple.
**Reasoning**: Predictable state management from day one. Avoids refactoring composables into stores later. ~1KB overhead. Stores: FilterState, WizardState, UIPreferences, OperationProgress.

### D3: unplugin-icons + Iconify with Lucide primary
**Choice**: unplugin-icons for tree-shaken icons. Lucide as primary set, mix from others as needed.
**Reasoning**: Zero base cost, ~200B per icon. Can use Lucide for UI, Phosphor for status, Material for actions — without committing to one set. Superset of standalone icon libraries.

## Clarify-Phase Decisions (Interactive)

### C1: Hybrid layout — toolbar filters + separate wizard view
**Finding**: User discussed Options A (tabs) vs C (toolbar) and preferred a hybrid.
**Decision**: Main view is a single DataTable with toolbar filter chips (All / Installed / Outdated + Category dropdown + search). No tabs for software states. Setup wizard and settings are separate full-screen views. Toolbar approach keeps everything visible; wizard gets its own space.

### C2: Two wizard modes — Scan vs Set Up My Rig
**Finding**: User identified two distinct first-run journeys plus a power user skip.
**Decision**: First-run welcome screen offers three paths:
1. "Scan my system" — detect existing software
2. "Set up my rig" — hardware-based bundle installer (the killer feature)
3. "Skip" — straight to empty dashboard

"Set up my rig" is also accessible later from the main UI via a "Setup Wizard" menu item.

### C3: No predefined bundles — dependency graph is the bundle
**Finding**: User initially wanted Ubuntu-style bundles, but realized the manifest dependency graph already handles this. Selecting NINA auto-pulls ASCOM + ASTAP via `requires`. No need for hardcoded bundle definitions.
**Decision**: Wizard shows curated app list (catalog minus drivers/runtimes) grouped by category. User picks apps + hardware. Dependency resolver builds the full install list. Review step shows user picks vs auto-resolved deps clearly. Much simpler than maintaining bundle definitions.

### C4: Multiple hardware per category
**Finding**: Users control multiple mounts, have separate imaging + guiding cameras.
**Decision**: Wizard allows adding multiple items per hardware category. Each item maps to its driver package. The bundle includes ALL selected hardware drivers.

### C5: Additional libraries alongside PrimeVue
**Finding**: User asked about complementary libraries.
**Decision**: Added to stack:
- **VueUse** — utility composables (useLocalStorage, useDark, etc.)
- **@vueuse/motion** — animations for transitions, overlays, list filtering
- **VeeValidate + Zod** — form validation for settings and wizard
- **unplugin-icons + Iconify** — tree-shaken icons
- **Pinia** — client-side state management

### C6: No Vue Router — Pinia-driven view state
**Decision**: Three views (dashboard, settings, wizard) switched via Pinia store state. No URL routing needed in a desktop app. Simple `v-if` switching.

### C7: Row expansion for package details and actions
**Decision**: Click a row → expand inline showing: publisher, license, homepage, dependencies, backup status, and action buttons (Install/Update/Backup/Restore/Uninstall). No modal dialogs for package details — keep the list context.

### C8: Empty dashboard shows skip button
**Decision**: First-run welcome has two large buttons (Scan / Set Up My Rig) and a smaller "Skip" link at the bottom. Power users go straight to the empty dashboard which also has a "Scan" CTA.

## Questions I Would Have Asked

### Q1: Should the wizard remember previous hardware selections?
**My decision**: Yes — store in Pinia + persist to config. When re-opening the wizard, show previous selections as defaults. Users add new hardware, they shouldn't re-enter everything.

### Q2: Should the dashboard show a "last scanned" timestamp?
**My decision**: Yes — subtle text in the toolbar showing "Last scanned: 2 hours ago" with a refresh button. Helps users know how fresh the data is.

### Q3: Should we support drag-and-drop for install order in the wizard?
**My decision**: No — dependency resolution handles ordering automatically. Users shouldn't need to think about install order. That's the engine's job (spec 012).
