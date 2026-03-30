# Research: 005-manifest-catalog

**Date**: 2026-03-30

## R1: Minisign Verification Crate

**Decision**: `minisign-verify` 0.2.5
**Rationale**: Purpose-built for verify-only. Zero dependencies, clean API, streaming support for large files. Same author (jedisct1) as the minisign tool.
**Alternatives considered**:
- `minisign` 0.9.1 — full implementation including signing. Drags in `scrypt`, `rpassword`, `ct-codecs`, `getrandom`. CLI-oriented API (`quiet`/`output` params). Unnecessary binary size increase for a verify-only use case.

**API pattern**:
```rust
use minisign_verify::{PublicKey, Signature};

const MINISIGN_PUBLIC_KEY: &str = "RWT3Z/NUV2mo2nf2YDHF/Iyz9NFR7+gkUHa0rTlAcIBBxg+eqG3LUItj";

let pk = PublicKey::from_base64(MINISIGN_PUBLIC_KEY)?;
let sig = Signature::from_file("catalog.db.minisig")?;
let data = std::fs::read("catalog.db")?;
pk.verify(&data, &sig, false)?;
```

Streaming also available via `pk.verify_stream(&sig)` + `StreamVerifier::update()` + `finalize()`.

## R2: HTTP Client (reqwest)

**Decision**: `reqwest` 0.13 with tokio runtime
**Rationale**: De facto Rust HTTP client (blessed.rs, lib.rs). Already an indirect dependency via Tauri. Supports custom headers (`If-None-Match`), status code inspection (304), timeouts, connection pooling.

**ETag/conditional fetch pattern**:
```rust
let mut req = client.get(&catalog_url);
if let Some(etag) = cached_etag {
    req = req.header("If-None-Match", etag);
}
let response = req.send().await?;
match response.status() {
    StatusCode::NOT_MODIFIED => Ok(CatalogFetchResult::Unchanged),
    StatusCode::OK => {
        let etag = response.headers().get("ETag").map(|v| v.to_str().unwrap().to_string());
        let bytes = response.bytes().await?;
        Ok(CatalogFetchResult::Updated { bytes, etag })
    }
    status => Err(CatalogError::FetchFailed(status)),
}
```

## R3: Rusqlite FTS5

**Decision**: `rusqlite` 0.35 with `bundled` feature (already a dependency)
**Rationale**: Already in the project. Bundled SQLite includes FTS5 by default. Read-only access via `OpenFlags::SQLITE_OPEN_READ_ONLY`.

**FTS5 query pattern**:
```rust
let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
let mut stmt = conn.prepare(
    "SELECT p.id, p.name, p.slug, p.category, p.type, p.description, p.aliases
     FROM packages_fts f
     JOIN packages p ON p.rowid = f.rowid
     WHERE packages_fts MATCH ?1
     ORDER BY rank"
)?;
```

**Read-only open**:
```rust
use rusqlite::OpenFlags;
let conn = Connection::open_with_flags(
    "catalog.db",
    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
)?;
```

## R4: PID Lockfile

**Decision**: Manual implementation using `std::fs` + `sysinfo` crate for cross-platform PID checking
**Rationale**: No established Rust crate for apt-style PID lockfiles. The logic is simple: write PID to file, check on acquire, clean up on drop. `sysinfo` provides cross-platform process existence checking.
**Alternative considered**: `fd-lock` — file descriptor locking, but doesn't survive process crashes (OS releases the lock). PID-based is better for our use case.

**Pattern**:
```rust
struct PidLock { path: PathBuf }

impl PidLock {
    fn acquire(path: &Path) -> Result<Self, LockError> {
        if path.exists() {
            let pid = std::fs::read_to_string(path)?.trim().parse::<u32>()?;
            if process_exists(pid) {
                return Err(LockError::AlreadyRunning(pid));
            }
            // Stale lock — process is dead
            std::fs::remove_file(path)?;
        }
        std::fs::write(path, std::process::id().to_string())?;
        Ok(Self { path: path.to_owned() })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) { let _ = std::fs::remove_file(&self.path); }
}
```

## R5: JSON Sidecar for Catalog Metadata

**Decision**: Simple JSON file (`catalog.db.meta`) using serde_json (already a dependency)
**Rationale**: Stores ETag and `fetched_at` timestamp. Read before deciding whether to fetch. No new dependencies.

**Schema**:
```json
{
  "etag": "\"abc123\"",
  "fetched_at": "2026-03-30T12:00:00Z"
}
```
