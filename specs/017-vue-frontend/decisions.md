# Decisions Report: 017-vue-frontend
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: No Vue Router — simple ref-based navigation
**Choice**: `const view = ref<'dashboard' | 'settings' | 'custom'>('dashboard')` with v-if switching.
**Reasoning**: Three views don't justify a router. No URL-based navigation needed in a desktop app. Simpler state management.

### D2: VueQuery wraps all Tauri calls
**Choice**: Every Tauri invoke() is wrapped in a VueQuery composable (useQuery/useMutation).
**Reasoning**: VueQuery handles loading states, error states, caching, and invalidation. Without it, every component would need manual loading/error handling.

### D3: PrimeVue only — no shadcn-vue or other libraries
**Choice**: PrimeVue 4 for all UI components.
**Reasoning**: PrimeVue has DataTable, forms, dialogs, toasts — everything needed. Adding shadcn-vue or Headless UI would create style inconsistencies and increase bundle size.

### D4: Status badge colors are semantic, not customizable
**Choice**: Fixed color mapping — green/blue/orange/red/gray.
**Reasoning**: Consistent visual language. Users learn the colors once. No need for theme customization in v1.

## Clarify-Phase Decisions

### C1: Empty state shows welcome + scan CTA
**Decision**: Fresh install with no detected software shows a centered welcome message with a prominent "Scan for Software" button. Not an empty table.

### C2: GitHub token input uses password masking
**Decision**: The Settings form masks the GitHub token by default (dots), with a toggle to reveal. This prevents shoulder-surfing but allows verification.

### C3: No keyboard shortcuts in this spec
**Decision**: Keyboard shortcuts (Ctrl+U for update, etc.) belong in the Tauri shell spec (016) via tauri-plugin-global-shortcut. The frontend just responds to events.

### C4: Progress overlay, not separate page
**Decision**: Updates show as an overlay/drawer on top of the dashboard, not a separate view. Users can still see the dashboard while updates run.

## Questions I Would Have Asked

### Q1: Should the dashboard auto-refresh on a timer?
**My decision**: No auto-refresh. Users click "Check for Updates" explicitly. Background checks (if enabled) update the tray badge, and opening the window triggers a fresh query via VueQuery.
**Impact if wrong**: Low — VueQuery's staleTime can add auto-refresh later.

### Q2: Should we support multiple languages (i18n)?
**My decision**: English only for v1. i18n adds significant complexity (translation files, RTL support). Astro software community is overwhelmingly English-speaking.
**Impact if wrong**: High if targeting non-English markets. But astrophotography is a niche with English-dominant documentation.
