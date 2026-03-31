# Research: 013 Backup and Restore

**Date**: 2026-03-31

## Dependency Research

### zip crate (v2.x) — already added by spec 011

**Decision**: Use `zip = "2"` with `deflate` feature for creating and reading backup archives
**Rationale**: Already a project dependency (spec 011). Standard ZIP crate for Rust. `ZipWriter` for creation, `ZipArchive` for reading. Synchronous API — wrap in `spawn_blocking`.
**Alternatives considered**: `tar` (not portable to Windows users), `flate2` raw (no archive format)

Key API:
- `ZipWriter::new(file)` — create archive
- `zip.start_file("path/in/archive", options)` — add file entry (string path, not filesystem)
- `zip.write_all(bytes)` — write content (ZipWriter implements Write)
- `zip.add_directory("dir/", options)` — add directory entry
- `zip.finish()` — finalize (writes central directory, must call)
- `SimpleFileOptions::default().compression_method(Deflated)` — compression config
- `ZipArchive::new(file)` + `archive.by_name("metadata.json")` — read specific entry

### walkdir crate (v2.x) — new dependency

**Decision**: Use `walkdir = "2"` for recursive directory traversal during backup
**Rationale**: Lightweight (~500 lines), on blessed.rs and lib.rs. Backup paths are explicit manifest directories — no gitignore filtering needed, so `ignore` crate is overkill.
**Alternatives considered**: `ignore` (heavier, gitignore-aware — unnecessary), `std::fs::read_dir` recursive (manual, error-prone)

### sha2 crate (v0.10) — existing dependency

**Decision**: Use existing `sha2` for file hashing in restore preview
**Rationale**: Already in Cargo.toml from spec 010. Hash files at backup time, store in metadata.json. At restore, hash on-disk file and compare — reports changed/unchanged/new without re-reading the ZIP.

### Locked file detection — no crate needed

**Decision**: Use `File::open()` error handling for locked file detection
**Rationale**: On Windows, exclusively locked files return OS error 32 (`ERROR_SHARING_VIOLATION`). Standard `std::io::Error` handling, no platform crate needed. Skip and warn per FR-013.

## Cargo.toml Changes

```toml
# Add to [dependencies]
walkdir = "2"
# zip = "2" already present from spec 011 (or add if branching before 011 merge)
```
