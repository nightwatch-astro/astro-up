# Quickstart: Vue Frontend Views

## Prerequisites

- Node.js (via mise)
- pnpm
- Rust toolchain (for Tauri dev server)
- Tauri system dependencies (see CLAUDE.md CI section)

## Setup

```sh
# Install new frontend dependencies
pnpm --dir frontend add vue-router@^4 @vueuse/core@^12 valibot@^1

# Verify
just dev  # starts Tauri dev server with Vite HMR
```

## Development

```sh
just dev          # Tauri + Vite dev server (hot-reload)
just test         # Run all tests (Rust + Vue)
just lint         # Clippy + ESLint
just fmt          # Rust fmt

# Frontend only
pnpm --dir frontend test    # Vitest
pnpm --dir frontend lint    # ESLint
pnpm --dir frontend build   # Production build
```

## Key Files

| File | Purpose |
|------|---------|
| `frontend/src/router/index.ts` | Route definitions (hash mode) |
| `frontend/src/composables/useInvoke.ts` | Tauri command wrappers (existing) |
| `frontend/src/composables/useOperations.ts` | Operation state + single-op guard |
| `frontend/src/composables/useKeyboard.ts` | Keyboard shortcuts |
| `frontend/src/mocks/` | Mock data for stubbed commands |
| `frontend/src/validation/config.ts` | Valibot schemas for settings |
| `frontend/src/types/` | TypeScript types mirroring Rust models |

## Architecture

```
Tauri Commands (Rust)
    ↓ invoke()
useInvoke.ts (VueQuery wrappers)
    ↓ useQuery / useMutation
Views & Components (Vue 3 + PrimeVue)
    ↓ events
useOperations.ts (operation state)
    ↓ listen("core-event")
OperationsDock.vue + LogPanel.vue
```

Mock data flows through the same `useInvoke.ts` layer — composables check a mock registry and return mock data when the real command is a stub.
