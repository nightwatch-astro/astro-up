use std::path::PathBuf;

use astro_up_core::config::{
    ConfigStore, PathsConfig, config_get, config_list, config_reset, config_set, load_config,
};

fn test_setup() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    (dir, db_path)
}

fn test_paths(dir: &std::path::Path) -> (PathsConfig, PathBuf) {
    (
        PathsConfig {
            download_dir: dir.join("downloads"),
            cache_dir: dir.join("cache"),
            data_dir: dir.join("data"),
        },
        dir.join("astro-up.log"),
    )
}

#[test]
fn config_set_get_roundtrip() {
    let (dir, db_path) = test_setup();
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    config_set(&store, &config, "network.timeout", "60s").unwrap();
    let val = config_get(&store, &config, "network.timeout").unwrap();
    assert_eq!(val, "60s");
}

#[test]
fn config_get_returns_default_when_not_set() {
    let (dir, db_path) = test_setup();
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    let val = config_get(&store, &config, "network.timeout").unwrap();
    assert_eq!(val, "30s");
}

#[test]
fn config_reset_reverts_to_default() {
    let (dir, db_path) = test_setup();
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    config_set(&store, &config, "network.timeout", "60s").unwrap();
    config_reset(&store, "network.timeout").unwrap();
    let val = config_get(&store, &config, "network.timeout").unwrap();
    assert_eq!(val, "30s");
}

#[test]
fn config_list_shows_overrides() {
    let (dir, db_path) = test_setup();
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    config_set(&store, &config, "network.timeout", "60s").unwrap();
    let stored = store.list().unwrap();
    let list = config_list(&config, &stored);

    let timeout_entry = list
        .iter()
        .find(|(k, _, _)| k == "network.timeout")
        .unwrap();
    assert_eq!(timeout_entry.1, "60s");
    assert!(timeout_entry.2); // is_overridden

    let url_entry = list.iter().find(|(k, _, _)| k == "catalog.url").unwrap();
    assert!(!url_entry.2); // is default
}

#[test]
fn config_set_unknown_key_fails() {
    let (dir, db_path) = test_setup();
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    let result = config_set(&store, &config, "nonexistent.field", "value");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unknown config key"), "got: {err}");
}

#[test]
fn config_set_invalid_value_does_not_persist() {
    let (dir, db_path) = test_setup();
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    // Zero duration should fail validation
    let result = config_set(&store, &config, "network.timeout", "0s");
    assert!(result.is_err());

    // Should NOT be persisted
    assert_eq!(store.get("network.timeout").unwrap(), None);
}
