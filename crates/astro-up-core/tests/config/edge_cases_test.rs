use std::path::PathBuf;

use astro_up_core::config::{
    ConfigStore, PathsConfig, config_get, config_list, config_reset, config_set, load_config,
};

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
fn config_get_unset_key_returns_default() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    let val = config_get(&store, &config, "updates.auto_check").unwrap();
    assert_eq!(val, "true");
}

#[test]
fn config_reset_unset_key_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (paths, log_file) = test_paths(dir.path());
    let _config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    // Should not error
    config_reset(&store, "network.timeout").unwrap();
}

#[test]
fn config_set_wrong_type_fails() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let store = ConfigStore::new(conn).unwrap();

    let result = config_set(&store, &config, "updates.auto_check", "42");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("boolean"), "got: {err}");
}

#[test]
fn config_list_empty_db_returns_all_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (paths, log_file) = test_paths(dir.path());
    let config = load_config(&db_path, paths, log_file, &[]).unwrap();

    let list = config_list(&config, &[]);
    assert_eq!(list.len(), 45);
    assert!(list.iter().all(|(_, _, overridden)| !overridden));
}

#[test]
fn corrupt_db_recovers_with_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (paths, log_file) = test_paths(dir.path());

    // Write garbage to the db file
    std::fs::write(&db_path, b"this is not a sqlite database").unwrap();

    let config = load_config(&db_path, paths, log_file, &[]).unwrap();

    // Should have recovered with defaults
    assert_eq!(config.network.timeout.as_secs(), 30);

    // Corrupt file should be renamed
    assert!(dir.path().join("test.corrupt").exists());
}
