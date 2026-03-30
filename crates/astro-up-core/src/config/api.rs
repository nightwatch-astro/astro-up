use garde::Validate;

use super::model::AppConfig;
use super::store::ConfigStore;
use super::{get_field_value, set_field, ConfigError};

/// Get the effective value of a config key (stored override or default).
pub fn config_get(
    store: &ConfigStore,
    config: &AppConfig,
    key: &str,
) -> Result<String, ConfigError> {
    if !config.is_known_key(key) {
        return Err(ConfigError::UnknownKey {
            key: key.to_string(),
            valid_keys: config.known_keys(),
        });
    }

    // Check stored value first
    if let Some(stored) = store.get(key).map_err(ConfigError::Store)? {
        return Ok(stored);
    }

    // Fall back to default
    get_field_value(config, key).ok_or_else(|| ConfigError::UnknownKey {
        key: key.to_string(),
        valid_keys: config.known_keys(),
    })
}

/// Set a config value. Validates before persisting.
pub fn config_set(
    store: &ConfigStore,
    config: &AppConfig,
    key: &str,
    value: &str,
) -> Result<(), ConfigError> {
    if !config.is_known_key(key) {
        return Err(ConfigError::UnknownKey {
            key: key.to_string(),
            valid_keys: config.known_keys(),
        });
    }

    // Build a temporary config with the proposed change and validate
    let mut temp = config.clone();
    set_field(&mut temp, key, value)?;
    temp.validate().map_err(ConfigError::Validation)?;

    // Persist
    store.set(key, value).map_err(ConfigError::Store)?;
    Ok(())
}

/// List all config keys with their effective values.
/// Returns `(key, value, is_overridden)` tuples.
pub fn config_list(
    config: &AppConfig,
    stored: &[(String, String)],
) -> Vec<(String, String, bool)> {
    let keys = config.known_keys();
    keys.into_iter()
        .map(|key| {
            if let Some((_, val)) = stored.iter().find(|(k, _)| k == &key) {
                (key, val.clone(), true)
            } else {
                let val = get_field_value(config, &key).unwrap_or_default();
                (key, val, false)
            }
        })
        .collect()
}

/// Reset a config key to its default (remove stored override).
pub fn config_reset(store: &ConfigStore, key: &str) -> Result<(), ConfigError> {
    // We don't validate the key here — reset on unknown key is harmless
    store.reset(key).map_err(ConfigError::Store)?;
    Ok(())
}
