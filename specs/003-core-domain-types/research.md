# Research: Core Domain Types

## trait-variant for async dyn dispatch

**Decision**: Use `trait_variant::make` instead of `async-trait`.

**Rationale**: Generates two traits from one definition — native async (zero-cost for static dispatch) + dyn-safe variant (boxed futures for engine registry). By dtolnay, 1.5M downloads, 0 dependencies.

**Alternatives considered**:
- `async-trait`: boxes ALL async calls, even static dispatch. Legacy approach.
- Enum dispatch (no traits): simpler but less extensible. Closed set of detectors/installers would work, but traits allow mocking in tests.
- Manual `Pin<Box<dyn Future>>` return types: verbose, no crate needed but error-prone.

## strum for enum derives

**Decision**: Use `strum` 0.26 with `serialize_all = "snake_case"`.

**Rationale**: One `#[derive(Display, EnumString, EnumIter)]` gives `FromStr`, `Display`, and iteration. Combined with `#[strum(serialize_all = "snake_case")]`, enum variants serialize/deserialize as snake_case strings matching our TOML/JSON format.

**Key pattern**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Category {
    Capture,
    Guiding,
    // ...
}
```

Both serde and strum use the same `snake_case` convention — no mismatch.

## Lenient version parsing

**Decision**: Custom `Version` wrapper with `semver::Version` + raw string fallback.

**Rationale**: ~30% of astrophotography software uses non-semver versions (NINA 3.1.2.3001, SharpCap 4.1.12288.0, ASCOM 6.6 SP2). Strict semver rejects all of these.

**Coercion rules**:
1. Try `semver::Version::parse(raw)` — if valid, use as-is
2. Strip 4th component: `3.1.2.3001` → `3.1.2`
3. Strip non-numeric suffix: `6.6 SP2` → `6.6.0` with pre-release `SP2`
4. Pad missing components: `3.1` → `3.1.0`
5. If all fail: store `raw` only, `parsed = None`

**Ordering**: `parsed.cmp()` when both sides have parsed versions, `raw.cmp()` (lexicographic) as fallback. Mixed (one parsed, one raw) always sorts parsed before raw.

## Event serialization for Tauri IPC

**Decision**: Adjacently tagged enum with `#[serde(tag = "type", content = "data")]`.

**Rationale**: Produces `{"type": "download_progress", "data": {...}}` which the Vue frontend can cleanly `switch (event.type)` on. Internally tagged (`#[serde(tag = "type")]`) requires all variants to be structs. Externally tagged (default) produces `{"DownloadProgress": {...}}` which is awkward in TypeScript.

## thiserror vs anyhow in core

**Decision**: `thiserror` only in `astro-up-core`. No `anyhow` in the library crate.

**Rationale**: Library crates should expose typed errors that consumers can match on. `anyhow` erases the type, making downstream handling impossible. The CLI uses `color-eyre` (wraps `anyhow`-style), the GUI uses `anyhow`. Both convert `CoreError` → their error type at the boundary.
