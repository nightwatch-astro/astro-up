use garde::Validate;

use crate::error::CoreError;

use super::model::AppConfig;
use super::store::ConfigStore;
use super::{get_field_value, set_field};

/// Get the effective value of a config key (stored override or default).
pub fn config_get(
    store: &ConfigStore,
    config: &AppConfig,
    key: &str,
) -> Result<String, CoreError> {
    if !config.is_known_key(key) {
        return Err(CoreError::ConfigUnknownKey {
            key: key.to_string(),
            valid_keys: config.known_keys(),
        });
    }

    if let Some(stored) = store.get(key)? {
        return Ok(stored);
    }

    get_field_value(config, key).ok_or_else(|| CoreError::ConfigUnknownKey {
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
) -> Result<(), CoreError> {
    if !config.is_known_key(key) {
        return Err(CoreError::ConfigUnknownKey {
            key: key.to_string(),
            valid_keys: config.known_keys(),
        });
    }

    let mut temp = config.clone();
    set_field(&mut temp, key, value)?;
    temp.validate().map_err(CoreError::from)?;

    store.set(key, value)?;
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
pub fn config_reset(store: &ConfigStore, key: &str) -> Result<(), CoreError> {
    store.reset(key)?;
    Ok(())
}
