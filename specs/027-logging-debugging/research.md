# Research: Logging and Debugging

**Date**: 2026-04-07

## Summary

No new libraries or technologies required. All implementation uses existing project infrastructure.

## Decisions

### Tracing instrumentation pattern

- **Decision**: Use `#[tracing::instrument(skip_all, fields(...))]` for public functions; `trace!` event macros for tight loops
- **Rationale**: `instrument` creates spans with automatic enter/exit. `skip_all` avoids Debug bound requirements. Tight loops use events to avoid span creation overhead.
- **Alternatives considered**: `log` crate (rejected — project already uses tracing, Tauri uses tracing internally), custom macro wrapper (rejected — unnecessary abstraction)

### Error handling pattern for unwrap replacement

- **Decision**: Replace with `?` + `.map_err()` using existing `CoreError` variants. Add new variants only when no existing variant fits.
- **Rationale**: Consistent with existing error handling pattern. `CoreError` already has 27 variants covering most failure modes.
- **Alternatives considered**: `anyhow::Context` in core (rejected — core uses typed errors via thiserror, not anyhow), panic-free lint (rejected — too broad, Mutex::lock unwrap is intentional)

### Frontend logging utility

- **Decision**: Thin wrapper around existing LogPanel store. No new dependencies.
- **Rationale**: LogPanel already supports level filtering, 1000-entry cap, and backend log forwarding. Adding a logger utility just provides a convenient API for components/composables.
- **Alternatives considered**: `loglevel` npm package (rejected — adds dependency for trivial wrapper), `console.*` methods (rejected — spec requires LogPanel integration, not browser console)

### VueQuery global error handling

- **Decision**: Add `onError` to individual mutations + global `QueryClient` `onError` as safety net
- **Rationale**: Per-mutation handlers give specific error messages. Global handler catches anything missed.
- **Alternatives considered**: Only global handler (rejected — generic messages less helpful), error boundary only (rejected — doesn't catch async mutation errors)
