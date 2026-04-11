#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Round-trip integration test: create fixture catalog with full detection configs,
//! read via SqliteCatalogReader::detection_config(), verify all fields match
//! including deserialized fallback chain.

use rusqlite::{Connection, params};

use astro_up_core::catalog::reader::SqliteCatalogReader;
use astro_up_core::catalog::types::PackageId;
use astro_up_core::types::DetectionMethod;

/// Create an in-memory-like temp catalog with the new detection schema and return its path.
fn create_test_catalog(dir: &std::path::Path) -> std::path::PathBuf {
    let db_path = dir.join("catalog.db");
    let conn = Connection::open(&db_path).unwrap();

    conn.execute_batch(
        "
        CREATE TABLE packages (
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
        CREATE TABLE versions (
            package_id TEXT NOT NULL REFERENCES packages(id),
            version TEXT NOT NULL,
            url TEXT NOT NULL,
            sha256 TEXT,
            discovered_at TEXT NOT NULL,
            release_notes_url TEXT,
            pre_release INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (package_id, version)
        );
        CREATE TABLE detection (
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
        CREATE TABLE install (
            package_id TEXT PRIMARY KEY REFERENCES packages(id),
            method TEXT NOT NULL,
            scope TEXT,
            elevation TEXT,
            switches TEXT,
            exit_codes TEXT,
            success_codes TEXT
        );
        CREATE TABLE meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE VIRTUAL TABLE packages_fts USING fts5(
            name, description, tags, aliases, publisher,
            content='packages', content_rowid='rowid'
        );
        ",
    )
    .unwrap();

    // Insert a package
    conn.execute(
        "INSERT INTO packages (id, manifest_version, name, description, publisher, homepage, category, [type], slug, license, tags, aliases, dependencies)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            "nina",
            1,
            "N.I.N.A.",
            "Nighttime Imaging N Astronomy",
            "Stefan Berg",
            "https://nighttime-imaging.eu",
            "capture",
            "application",
            "N.I.N.A.",
            "GPL-3.0",
            r#"["imaging"]"#,
            r#"["nina"]"#,
            "[]"
        ],
    )
    .unwrap();

    // Populate FTS
    conn.execute_batch(
        "INSERT INTO packages_fts(rowid, name, description, tags, aliases, publisher)
         SELECT rowid, name, description, tags, aliases, publisher FROM packages;",
    )
    .unwrap();

    // Insert version
    conn.execute(
        "INSERT INTO versions (package_id, version, url, sha256, discovered_at, release_notes_url, pre_release)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "nina",
            "3.1.2",
            "https://example.com/nina-3.1.2.exe",
            "abc123",
            "2026-03-25T12:00:00Z",
            Option::<String>::None,
            0
        ],
    )
    .unwrap();

    // Insert detection with full fields including fallback_config JSON.
    // Registry keys use absolute paths matching the format expected by registry::detect().
    let fallback_json = serde_json::json!({
        "method": "pe_file",
        "file_path": "C:\\Program Files\\NINA\\NINA.exe",
        "version_regex": "^(\\d+\\.\\d+\\.\\d+)"
    });
    conn.execute(
        "INSERT INTO detection (package_id, method, file_path, registry_key, registry_value,
                                version_regex, product_code, upgrade_code, inf_provider,
                                device_class, inf_name, fallback_config)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            "nina",
            "registry",
            Option::<String>::None,
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NINA 2_is1",
            "DisplayVersion",
            Option::<String>::None,
            "{B3A4F860-DA18-4B76-8E4A-3E29C2C01738}",
            Option::<String>::None,
            Option::<String>::None,
            Option::<String>::None,
            Option::<String>::None,
            fallback_json.to_string()
        ],
    )
    .unwrap();

    // Insert install config
    conn.execute(
        "INSERT INTO install (package_id, method, elevation, switches)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            "nina",
            "inno_setup",
            Option::<String>::None,
            r#"{"silent":"/VERYSILENT /NORESTART /SUPPRESSMSGBOXES"}"#,
        ],
    )
    .unwrap();

    // Meta
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

    db_path
}

#[test]
fn detection_config_roundtrip_with_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = create_test_catalog(dir.path());

    let reader = SqliteCatalogReader::open(&db_path).unwrap();
    let id: PackageId = "nina".parse().unwrap();

    let config = reader.detection_config(&id).unwrap();
    assert!(config.is_some(), "detection config should exist for nina");

    let config = config.unwrap();
    assert_eq!(config.method, DetectionMethod::Registry);
    assert_eq!(
        config.registry_key.as_deref(),
        Some(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NINA 2_is1")
    );
    assert_eq!(config.registry_value.as_deref(), Some("DisplayVersion"));
    assert!(config.file_path.is_none());
    assert_eq!(
        config.product_code.as_deref(),
        Some("{B3A4F860-DA18-4B76-8E4A-3E29C2C01738}")
    );

    // Verify fallback chain deserialized correctly
    let fallback = config.fallback.as_ref().expect("fallback should exist");
    assert_eq!(fallback.method, DetectionMethod::PeFile);
    assert_eq!(
        fallback.file_path.as_deref(),
        Some("C:\\Program Files\\NINA\\NINA.exe")
    );
    assert_eq!(
        fallback.version_regex.as_deref(),
        Some("^(\\d+\\.\\d+\\.\\d+)")
    );
    assert!(fallback.fallback.is_none(), "no nested fallback");
}

#[test]
fn detection_config_missing_package_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = create_test_catalog(dir.path());

    let reader = SqliteCatalogReader::open(&db_path).unwrap();
    let id: PackageId = "nonexistent".parse().unwrap();

    let config = reader.detection_config(&id).unwrap();
    assert!(config.is_none());
}

#[test]
fn detection_config_no_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("catalog.db");
    let conn = Connection::open(&db_path).unwrap();

    conn.execute_batch(
        "
        CREATE TABLE packages (
            id TEXT PRIMARY KEY, manifest_version INTEGER NOT NULL,
            name TEXT NOT NULL, description TEXT, publisher TEXT, homepage TEXT,
            category TEXT NOT NULL, [type] TEXT NOT NULL, slug TEXT NOT NULL,
            license TEXT, tags TEXT, aliases TEXT, dependencies TEXT, icon_base64 TEXT
        );
        CREATE TABLE versions (
            package_id TEXT NOT NULL, version TEXT NOT NULL, url TEXT NOT NULL,
            sha256 TEXT, discovered_at TEXT NOT NULL, release_notes_url TEXT,
            pre_release INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (package_id, version)
        );
        CREATE TABLE detection (
            package_id TEXT PRIMARY KEY, method TEXT NOT NULL, file_path TEXT,
            registry_key TEXT, registry_value TEXT, version_regex TEXT,
            product_code TEXT, upgrade_code TEXT, inf_provider TEXT,
            device_class TEXT, inf_name TEXT, fallback_config TEXT
        );
        CREATE TABLE install (
            package_id TEXT PRIMARY KEY, method TEXT NOT NULL,
            scope TEXT, elevation INTEGER NOT NULL DEFAULT 0,
            switches TEXT, exit_codes TEXT, success_codes TEXT
        );
        CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE VIRTUAL TABLE packages_fts USING fts5(
            name, description, tags, aliases, publisher,
            content='packages', content_rowid='rowid'
        );
        ",
    )
    .unwrap();

    conn.execute(
        "INSERT INTO packages VALUES (?1,1,'PHD2','Guiding','Open PHD','https://phd2.org','guiding','application','PHD2','BSD',NULL,NULL,NULL,NULL)",
        params!["phd2"],
    ).unwrap();
    conn.execute_batch(
        "INSERT INTO packages_fts(rowid, name, description, tags, aliases, publisher) SELECT rowid, name, description, tags, aliases, publisher FROM packages;",
    ).unwrap();
    conn.execute(
        "INSERT INTO versions VALUES ('phd2','2.6.13','https://example.com/phd2.exe',NULL,'2026-01-01T00:00:00Z',NULL,0)",
        [],
    ).unwrap();
    conn.execute(
        r"INSERT INTO detection (package_id, method, registry_key, registry_value) VALUES ('phd2','registry','HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\PHDGuiding2_is1','DisplayVersion')",
        [],
    ).unwrap();
    conn.execute("INSERT INTO meta VALUES ('schema_version','1')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO meta VALUES ('compiled_at','2026-03-30T12:00:00Z')",
        [],
    )
    .unwrap();

    let reader = SqliteCatalogReader::open(&db_path).unwrap();
    let id: PackageId = "phd2".parse().unwrap();
    let config = reader.detection_config(&id).unwrap().unwrap();

    assert_eq!(config.method, DetectionMethod::Registry);
    assert_eq!(
        config.registry_key.as_deref(),
        Some(
            r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\PHDGuiding2_is1"
        )
    );
    assert!(config.fallback.is_none());
    assert!(config.file_path.is_none());
    assert!(config.version_regex.is_none());
}

#[test]
fn list_all_with_detection_populates_config() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = create_test_catalog(dir.path());

    let reader = SqliteCatalogReader::open(&db_path).unwrap();
    let software_list = reader.list_all_with_detection().unwrap();

    assert_eq!(software_list.len(), 1);
    let nina = &software_list[0];
    assert!(nina.detection.is_some());
    let det = nina.detection.as_ref().unwrap();
    assert_eq!(det.method, DetectionMethod::Registry);
    assert!(det.fallback.is_some());

    // Verify install config is also populated
    let install = nina.install.as_ref().expect("install config should exist");
    assert_eq!(
        install.method,
        astro_up_core::types::InstallMethod::InnoSetup
    );
    assert!(install.elevation.is_none()); // elevation = 0 → None
    let switches = install.switches.as_ref().expect("switches should exist");
    assert!(!switches.silent.is_empty());
}

#[test]
fn wmi_apps_detection_method_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("catalog.db");
    let conn = Connection::open(&db_path).unwrap();

    conn.execute_batch(
        "
        CREATE TABLE packages (
            id TEXT PRIMARY KEY, manifest_version INTEGER NOT NULL,
            name TEXT NOT NULL, description TEXT, publisher TEXT, homepage TEXT,
            category TEXT NOT NULL, [type] TEXT NOT NULL, slug TEXT NOT NULL,
            license TEXT, tags TEXT, aliases TEXT, dependencies TEXT, icon_base64 TEXT
        );
        CREATE TABLE versions (
            package_id TEXT NOT NULL, version TEXT NOT NULL, url TEXT NOT NULL,
            sha256 TEXT, discovered_at TEXT NOT NULL, release_notes_url TEXT,
            pre_release INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (package_id, version)
        );
        CREATE TABLE detection (
            package_id TEXT PRIMARY KEY, method TEXT NOT NULL, file_path TEXT,
            registry_key TEXT, registry_value TEXT, version_regex TEXT,
            product_code TEXT, upgrade_code TEXT, inf_provider TEXT,
            device_class TEXT, inf_name TEXT, fallback_config TEXT
        );
        CREATE TABLE install (
            package_id TEXT PRIMARY KEY, method TEXT NOT NULL,
            scope TEXT, elevation INTEGER NOT NULL DEFAULT 0,
            switches TEXT, exit_codes TEXT, success_codes TEXT
        );
        CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE VIRTUAL TABLE packages_fts USING fts5(
            name, description, tags, aliases, publisher,
            content='packages', content_rowid='rowid'
        );
        ",
    )
    .unwrap();

    // Insert ASI Studio-like package with wmi_apps primary + file_exists fallback
    conn.execute(
        "INSERT INTO packages VALUES (?1,1,'ZWO ASIStudio','ZWO imaging suite','ZWO','https://zwo.com','capture','application','zwo-asistudio','Proprietary',NULL,'[\"ASIStudio\"]',NULL,NULL)",
        params!["zwo-asistudio"],
    ).unwrap();
    conn.execute_batch(
        "INSERT INTO packages_fts(rowid, name, description, tags, aliases, publisher) SELECT rowid, name, description, tags, aliases, publisher FROM packages;",
    ).unwrap();
    conn.execute(
        "INSERT INTO versions VALUES ('zwo-asistudio','1.8.0','https://example.com/asistudio.exe',NULL,'2026-01-01T00:00:00Z',NULL,0)",
        [],
    ).unwrap();

    let fallback_json = serde_json::json!({
        "method": "file_exists",
        "file_path": "C:\\Program Files\\ASIStudio\\ASIStudio.exe"
    });
    conn.execute(
        "INSERT INTO detection (package_id, method, fallback_config) VALUES ('zwo-asistudio','wmi_apps',?1)",
        params![fallback_json.to_string()],
    ).unwrap();
    conn.execute("INSERT INTO meta VALUES ('schema_version','1')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO meta VALUES ('compiled_at','2026-03-30T12:00:00Z')",
        [],
    )
    .unwrap();

    let reader = SqliteCatalogReader::open(&db_path).unwrap();
    let id: PackageId = "zwo-asistudio".parse().unwrap();
    let config = reader.detection_config(&id).unwrap();
    assert!(
        config.is_some(),
        "wmi_apps detection config should be parsed"
    );

    let config = config.unwrap();
    assert_eq!(config.method, DetectionMethod::WmiApps);

    // Verify fallback chain
    let fallback = config.fallback.as_ref().expect("fallback should exist");
    assert_eq!(fallback.method, DetectionMethod::FileExists);
    assert_eq!(
        fallback.file_path.as_deref(),
        Some("C:\\Program Files\\ASIStudio\\ASIStudio.exe")
    );
}

#[test]
fn file_exists_detection_method_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("catalog.db");
    let conn = Connection::open(&db_path).unwrap();

    conn.execute_batch(
        "
        CREATE TABLE packages (
            id TEXT PRIMARY KEY, manifest_version INTEGER NOT NULL,
            name TEXT NOT NULL, description TEXT, publisher TEXT, homepage TEXT,
            category TEXT NOT NULL, [type] TEXT NOT NULL, slug TEXT NOT NULL,
            license TEXT, tags TEXT, aliases TEXT, dependencies TEXT, icon_base64 TEXT
        );
        CREATE TABLE versions (
            package_id TEXT NOT NULL, version TEXT NOT NULL, url TEXT NOT NULL,
            sha256 TEXT, discovered_at TEXT NOT NULL, release_notes_url TEXT,
            pre_release INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (package_id, version)
        );
        CREATE TABLE detection (
            package_id TEXT PRIMARY KEY, method TEXT NOT NULL, file_path TEXT,
            registry_key TEXT, registry_value TEXT, version_regex TEXT,
            product_code TEXT, upgrade_code TEXT, inf_provider TEXT,
            device_class TEXT, inf_name TEXT, fallback_config TEXT
        );
        CREATE TABLE install (
            package_id TEXT PRIMARY KEY, method TEXT NOT NULL,
            scope TEXT, elevation INTEGER NOT NULL DEFAULT 0,
            switches TEXT, exit_codes TEXT, success_codes TEXT
        );
        CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE VIRTUAL TABLE packages_fts USING fts5(
            name, description, tags, aliases, publisher,
            content='packages', content_rowid='rowid'
        );
        ",
    )
    .unwrap();

    // Insert ASTAP database-like package with file_exists detection
    conn.execute(
        "INSERT INTO packages VALUES (?1,1,'ASTAP D50 Star Database','Large star database','ASTAP','https://hnsky.org','platesolving','database','astap-d50','Freeware',NULL,NULL,NULL,NULL)",
        params!["astap-d50"],
    ).unwrap();
    conn.execute_batch(
        "INSERT INTO packages_fts(rowid, name, description, tags, aliases, publisher) SELECT rowid, name, description, tags, aliases, publisher FROM packages;",
    ).unwrap();
    conn.execute(
        "INSERT INTO versions VALUES ('astap-d50','2024.01.01','https://example.com/d50.exe',NULL,'2026-01-01T00:00:00Z',NULL,0)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO detection (package_id, method, file_path) VALUES ('astap-d50','file_exists','{config_dir}/astap/d50')",
        [],
    ).unwrap();
    conn.execute("INSERT INTO meta VALUES ('schema_version','1')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO meta VALUES ('compiled_at','2026-03-30T12:00:00Z')",
        [],
    )
    .unwrap();

    let reader = SqliteCatalogReader::open(&db_path).unwrap();
    let id: PackageId = "astap-d50".parse().unwrap();
    let config = reader.detection_config(&id).unwrap();
    assert!(
        config.is_some(),
        "file_exists detection config should be parsed"
    );

    let config = config.unwrap();
    assert_eq!(config.method, DetectionMethod::FileExists);
    assert_eq!(config.file_path.as_deref(), Some("{config_dir}/astap/d50"));
    assert!(config.fallback.is_none());
}
