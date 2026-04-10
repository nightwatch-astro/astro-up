#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Helper to create test fixture catalog.db.
//! Run with: cargo test -p astro-up-core --test create_fixture_catalog -- --ignored

use rusqlite::{Connection, params};

#[test]
#[ignore = "run manually to regenerate fixture"]
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
            [type] TEXT NOT NULL,
            slug TEXT NOT NULL,
            license TEXT,
            tags TEXT,
            aliases TEXT,
            dependencies TEXT,
            icon_base64 TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_packages_category ON packages(category);
        CREATE INDEX IF NOT EXISTS idx_packages_type ON packages([type]);
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

        CREATE TABLE IF NOT EXISTS detection (
            package_id TEXT PRIMARY KEY REFERENCES packages(id),
            method TEXT NOT NULL,
            file_path TEXT,
            registry_key TEXT,
            registry_value TEXT,
            version_regex TEXT,
            product_code TEXT,
            upgrade_code TEXT,
            inf_provider TEXT,
            device_class TEXT,
            inf_name TEXT,
            fallback_config TEXT
        );

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
            "INSERT INTO packages (id, manifest_version, name, description, publisher, homepage, category, [type], slug, license, tags, aliases, dependencies)
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
            params![pid, ver, url, sha, disc, rn, i32::from(*pre)],
        )
        .unwrap();
    }

    // Insert detection configs
    let detections = vec![
        (
            "nina",
            "registry",
            None::<&str>,           // file_path
            Some("NINA 2"),         // registry_key
            Some("DisplayVersion"), // registry_value
            None::<&str>,           // version_regex
            None::<&str>,           // product_code
            None::<&str>,           // upgrade_code
            None::<&str>,           // inf_provider
            None::<&str>,           // device_class
            None::<&str>,           // inf_name
            Some(r#"{"method":"pe_file","file_path":"C:\\Program Files\\NINA\\NINA.exe"}"#), // fallback_config
        ),
        (
            "phd2",
            "registry",
            None,
            Some("PHD2"),
            Some("DisplayVersion"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        (
            "ascom-platform",
            "registry",
            None,
            Some("ASCOM Platform 6"),
            Some("DisplayVersion"),
            Some(r"^(\d+\.\d+)"),
            Some("{B3A4F860-DA18-4B76-8E4A-3E29C2C01738}"),
            None,
            None,
            None,
            None,
            None,
        ),
        (
            "astap",
            "registry",
            None,
            Some("AppName=ASTAP, the Astrometric STAcking Program,~D52A8A79_is1"),
            Some("DisplayVersion"),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(r#"{"method":"pe_file","file_path":"C:\\Program Files\\astap\\astap.exe"}"#),
        ),
    ];

    for (
        pid,
        method,
        file_path,
        reg_key,
        reg_val,
        ver_regex,
        product_code,
        upgrade_code,
        inf_prov,
        dev_class,
        inf_name,
        fallback_cfg,
    ) in &detections
    {
        conn.execute(
            "INSERT INTO detection (package_id, method, file_path, registry_key, registry_value,
                                    version_regex, product_code, upgrade_code, inf_provider,
                                    device_class, inf_name, fallback_config)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                pid,
                method,
                file_path,
                reg_key,
                reg_val,
                ver_regex,
                product_code,
                upgrade_code,
                inf_prov,
                dev_class,
                inf_name,
                fallback_cfg
            ],
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
