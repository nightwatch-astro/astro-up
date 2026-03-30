use std::path::PathBuf;

use astro_up_core::config::{ConfigStore, PathsConfig, load_config};

fn test_paths(dir: &std::path::Path) -> (PathsConfig, PathBuf) {
    (
        PathsConfig {
            download_dir: dir.join("downloads"),
            cache_dir: dir.join("cache"),
            data_dir: dir.join("data"),
            ..PathsConfig::default()
        },
        dir.join("astro-up.log"),
    )
}

#[test]
fn cli_overrides_sqlite_overrides_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (paths, log_file) = test_paths(dir.path());

    // Set SQLite value
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let store = ConfigStore::new(conn).unwrap();
        store.set("logging.level", "warn").unwrap();
    }

    // Load with CLI override — CLI wins
    let config = load_config(
        &db_path,
        paths.clone(),
        log_file.clone(),
        &[("logging.level", "debug")],
    )
    .unwrap();
    assert_eq!(config.logging.level.to_string(), "debug");

    // Load without CLI override — SQLite wins
    let config = load_config(&db_path, paths.clone(), log_file.clone(), &[]).unwrap();
    assert_eq!(config.logging.level.to_string(), "warn");

    // Reset SQLite — default wins
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let store = ConfigStore::new(conn).unwrap();
        store.reset("logging.level").unwrap();
    }
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    assert_eq!(config.logging.level.to_string(), "info");
}
