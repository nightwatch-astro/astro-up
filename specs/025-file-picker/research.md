# Research: 025-file-picker

## Dialog Plugin Frontend API

**Decision**: Use `@tauri-apps/plugin-dialog` JavaScript bindings (`open()` and `save()` functions)
**Rationale**: Official Tauri plugin, already installed on Rust side, cross-platform, simple API
**Alternatives considered**: Custom Tauri command wrapping Rust `rfd` crate — rejected because the JS API already provides everything needed with no Rust changes

### API Surface (from context7 docs)

```typescript
import { open, save } from '@tauri-apps/plugin-dialog';

// Directory picker
const dir = await open({
  directory: true,
  defaultPath: '/current/path',
  title: 'Select Directory'
});
// Returns: string | null (null on cancel)

// File save dialog with filter
const file = await save({
  defaultPath: 'app.log',
  filters: [{ name: 'Log files', extensions: ['log'] }],
  title: 'Select Log File'
});
// Returns: string | null (null on cancel)
```

### Key behaviors
- Returns `null` when user cancels — no error thrown
- `defaultPath` opens dialog at that location if it exists
- `directory: true` on `open()` switches to directory-only mode
- `filters` array supports multiple filter groups with named extensions
- Cross-platform: Windows, macOS, Linux, Android, iOS

## PrimeVue InputGroup Pattern

**Decision**: Use `InputGroup` + `InputGroupAddon` with `Button` for the browse button layout
**Rationale**: Standard PrimeVue pattern for input + action button, consistent with project's design system
**Alternatives considered**: Custom flex layout — rejected for not matching PrimeVue conventions

### Usage pattern
```vue
<InputGroup>
  <InputText v-model="value" />
  <Button icon="pi pi-folder-open" @click="browse" />
</InputGroup>
```

## Existing Infrastructure

| Component | Status | Action needed |
|-----------|--------|---------------|
| `tauri-plugin-dialog` (Rust) | Installed in `Cargo.toml` | None |
| Dialog plugin init (`lib.rs`) | Registered `.plugin(tauri_plugin_dialog::init())` | None |
| Dialog permissions | `dialog:default` in capabilities | None |
| `@tauri-apps/plugin-dialog` (npm) | **NOT installed** | `pnpm add @tauri-apps/plugin-dialog` |
| InputGroup component | Available in PrimeVue 4 | Import in components |

## No Unresolved Questions

All NEEDS CLARIFICATION items from Technical Context resolved via existing project state and documentation.
