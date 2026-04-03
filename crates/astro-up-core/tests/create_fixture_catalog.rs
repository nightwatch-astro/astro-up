//! Helper to create test fixture catalog.db.
//! Run with: cargo test -p astro-up-core --test create_fixture_catalog -- --ignored

use rusqlite::{Connection, params};

#[test]
#[ignore] // Run manually to regenerate fixture
fn create_fixture_catalog() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/catalog/catalog.db"
    );

    // Remove existing
    let _ = std::fs::remove_file(path);

    let conn = Connection::open(path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .unwrap();

    // Create schema (matches astro-up-compiler schema.rs)
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS packages (
            id TEXT PRIMARY KEY,
            manifest_version INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            publisher TEXT,
            homepage TEXT,
            category TEXT NOT NULL,
            sw_typee TEXT NOT NULL,
            slug TEXT NOT NULL,
            license TEXT,
            tags TEXT,
            aliases TEXT,
            dependencies TEXT,
            icon_base64 TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_packages_category ON packages(category);
        CREATE INDEX IF NOT EXISTS idx_packages_sw_typee ON packages(sw_typee);
        CREATE INDEX IF NOT EXISTS idx_packages_slug ON packages(slug);

        CREATE TABLE IF NOT EXISTS versions (
            package_id TEXT NOT NULL REFERENCES packages(id),
            version TEXT NOT NULL,
            url TEXT NOT NULL,
            sha256 TEXT,
            discovered_at TEXT NOT NULL,
            release_notes_url TEXT,
            pre_release INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (package_id, version)
        );
        CREATE INDEX IF NOT EXISTS idx_versions_package ON versions(package_id);

        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS packages_fts USING fts5(
            name, description, tags, aliases, publisher,
            content='packages', content_rowid='rowid'
        );
        ",
    )
    .unwrap();

    // Insert test packages
    let packages = vec![
        (
            "nina",
            1,
            "N.I.N.A.",
            "Nighttime Imaging N Astronomy - astrophotography imaging suite",
            "Stefan Berg",
            "https://nighttime-imaging.eu",
            "capture",
            "application",
            "N.I.N.A.",
            "GPL-3.0",
            r#"["imaging","capture","sequencer"]"#,
            r#"["nina","nighttimeimaging"]"#,
            "[]",
        ),
        (
            "phd2",
            1,
            "PHD2",
            "Push Here Dummy - autoguiding software",
            "Open PHD Guiding",
            "https://openphdguiding.org",
            "guiding",
            "application",
            "PHD2",
            "BSD-3-Clause",
            r#"["guiding","autoguide"]"#,
            r#"["phd","pushhereddummy"]"#,
            "[]",
        ),
        (
            "ascom-platform",
            1,
            "ASCOM Platform",
            "ASCOM astronomy device interface standard",
            "ASCOM Initiative",
            "https://ascom-standards.org",
            "prerequisites",
            "runtime",
            "ASCOM Platform",
            "MIT",
            r#"["ascom","drivers","platform"]"#,
            r#"["ascom"]"#,
            "[]",
        ),
        (
            "astap",
            1,
            "ASTAP",
            "Astrometric STAcking Program - plate solver and stacker",
            "Han Kleijn",
            "https://www.hnsky.org/astap.htm",
            "platesolving",
            "application",
            "ASTAP",
            "Freeware",
            r#"["platesolving","stacking"]"#,
            r#"["astap"]"#,
            "[]",
        ),
        (
            "sharpcap",
            1,
            "SharpCap",
            "Astronomy camera capture tool with live stacking",
            "Robin Glover",
            "https://www.sharpcap.co.uk",
            "capture",
            "application",
            "SharpCap",
            "Proprietary",
            r#"["capture","livestacking"]"#,
            r#"["sharpcap"]"#,
            r#"["ascom-platform"]"#,
        ),
    ];

    for (id, mv, name, desc, pub_, hp, cat, sw_type, slug, lic, tags, aliases, deps) in &packages {
        conn.execute(
            "INSERT INTO packages (id, manifest_version, name, description, publisher, homepage, category, sw_typee, slug, license, tags, aliases, dependencies)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![id, mv, name, desc, pub_, hp, cat, sw_type, slug, lic, tags, aliases, deps],
        )
        .unwrap();
    }

    // Populate FTS5 index
    conn.execute_batch(
        "INSERT INTO packages_fts(rowid, name, description, tags, aliases, publisher)
         SELECT rowid, name, description, tags, aliases, publisher FROM packages;",
    )
    .unwrap();

    // Insert versions
    let versions = vec![
        (
            "nina",
            "3.1.2.9001",
            "https://github.com/daleghent/nina/releases/download/3.1.2.9001/NINA-3.1.2.9001-setup.exe",
            Some("abc123"),
            "2026-03-25T12:00:00Z",
            Some("https://github.com/daleghent/nina/releases/tag/3.1.2.9001"),
            false,
        ),
        (
            "nina",
            "3.1.1.9001",
            "https://github.com/daleghent/nina/releases/download/3.1.1.9001/NINA-3.1.1.9001-setup.exe",
            None,
            "2026-03-15T10:00:00Z",
            None,
            false,
        ),
        (
            "nina",
            "4.0.0.9001-beta1",
            "https://github.com/daleghent/nina/releases/download/4.0.0.9001-beta1/NINA-4.0.0.9001-beta1-setup.exe",
            None,
            "2026-03-28T08:00:00Z",
            None,
            true,
        ),
        (
            "phd2",
            "2.6.13",
            "https://github.com/OpenPHDGuiding/phd2/releases/download/v2.6.13/phd2-2.6.13-setup.exe",
            None,
            "2026-02-10T09:00:00Z",
            None,
            false,
        ),
        (
            "ascom-platform",
            "6.6.2",
            "https://github.com/ASCOMInitiative/ASCOMPlatform/releases/download/v6.6.2/ASCOMPlatform66.exe",
            None,
            "2026-01-05T14:00:00Z",
            None,
            false,
        ),
    ];

    for (pid, ver, url, sha, disc, rn, pre) in &versions {
        conn.execute(
            "INSERT INTO versions (package_id, version, url, sha256, discovered_at, release_notes_url, pre_release)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![pid, ver, url, sha, disc, rn, *pre as i32],
        )
        .unwrap();
    }

    // Write meta
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        params!["schema_version", "1"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        params!["compiled_at", "2026-03-30T12:00:00Z"],
    )
    .unwrap();

    println!("Created fixture catalog at {path}");
    println!("Now sign it with: rsign sign {path} -s <secret-key-file> -t 'test fixture'");
}
