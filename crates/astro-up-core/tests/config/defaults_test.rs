use std::path::PathBuf;

use astro_up_core::config::{PathsConfig, load_config};

#[test]
fn load_config_with_empty_db_returns_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let paths = PathsConfig {
        download_dir: dir.path().join("downloads"),
        cache_dir: dir.path().join("cache"),
        data_dir: dir.path().join("data"),
        ..PathsConfig::default()
    };
    let log_file = dir.path().join("astro-up.log");

    let config = load_config(&db_path, paths.clone(), log_file.clone(), &[]).unwrap();

    assert_eq!(
        config.catalog.url,
        "https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db"
    );
    assert_eq!(config.catalog.cache_ttl.as_secs(), 86400);
    assert_eq!(config.paths.download_dir, paths.download_dir);
    assert_eq!(config.paths.cache_dir, paths.cache_dir);
    assert_eq!(config.paths.data_dir, paths.data_dir);
    assert_eq!(config.network.proxy, None);
    assert_eq!(config.network.timeout.as_secs(), 30);
    assert!(config.network.user_agent.starts_with("astro-up/"));
    assert!(config.updates.auto_check);
    assert_eq!(config.updates.check_interval.as_secs(), 86400);
    assert_eq!(config.logging.level.to_string(), "info");
    assert!(!config.logging.log_to_file);
    assert_eq!(config.logging.log_file, log_file);
    assert!(!config.telemetry.enabled);

    // Snapshot the config shape (excluding platform-dependent paths)
    let mut value = serde_json::to_value(&config).unwrap();
    // Redact platform-dependent paths for stable snapshots
    for path in [
        "paths.download_dir",
        "paths.cache_dir",
        "paths.data_dir",
        "logging.log_file",
    ] {
        let parts: Vec<&str> = path.split('.').collect();
        if let Some(section) = value.get_mut(parts[0]) {
            if let Some(field) = section.get_mut(parts[1]) {
                *field = serde_json::Value::String("[platform_path]".to_string());
            }
        }
    }
    insta::assert_json_snapshot!("default_config", value);
}

#[test]
fn load_config_with_no_db_file_creates_it() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("nonexistent.db");
    let paths = PathsConfig {
        download_dir: PathBuf::from("/tmp/dl"),
        cache_dir: PathBuf::from("/tmp/cache"),
        data_dir: PathBuf::from("/tmp/data"),
        ..PathsConfig::default()
    };

    assert!(!db_path.exists());
    let _config = load_config(&db_path, paths, PathBuf::from("/tmp/app.log"), &[]).unwrap();
    assert!(db_path.exists());
}

#[test]
fn known_keys_discovers_all_fields() {
    let config = astro_up_core::config::AppConfig::default();
    let keys = config.known_keys();

    assert!(keys.contains(&"catalog.url".to_string()));
    assert!(keys.contains(&"network.timeout".to_string()));
    assert!(keys.contains(&"logging.level".to_string()));
    assert!(keys.contains(&"telemetry.enabled".to_string()));
    assert_eq!(keys.len(), 18); // 18 leaf fields (14 original + 4 download config)
}
