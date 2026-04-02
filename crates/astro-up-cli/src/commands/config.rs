use color_eyre::eyre::{Result, eyre};

use crate::ConfigAction;
use crate::output::OutputMode;
use crate::output::json::print_json;

/// T030: Manage application configuration.
pub async fn handle_config(action: ConfigAction, mode: &OutputMode) -> Result<()> {
    match action {
        ConfigAction::Init => handle_config_init(mode),
        ConfigAction::Show => handle_config_show(mode),
    }
}

fn handle_config_init(mode: &OutputMode) -> Result<()> {
    let config_dir = directories::ProjectDirs::from("com", "nightwatch", "astro-up")
        .map(|dirs| dirs.config_dir().to_owned())
        .ok_or_else(|| eyre!("could not determine config directory"))?;

    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        if *mode == OutputMode::Json {
            return print_json(&serde_json::json!({
                "path": config_path.display().to_string(),
                "status": "already_exists"
            }));
        }
        println!("Config file already exists: {}", config_path.display());
        return Ok(());
    }

    let default_config = astro_up_core::config::AppConfig::default();
    let toml = toml::to_string_pretty(&default_config)?;
    std::fs::write(&config_path, toml)?;

    if *mode == OutputMode::Json {
        return print_json(&serde_json::json!({
            "path": config_path.display().to_string(),
            "status": "created"
        }));
    }
    println!("Created config file: {}", config_path.display());
    Ok(())
}

fn handle_config_show(mode: &OutputMode) -> Result<()> {
    let config = astro_up_core::config::AppConfig::default();

    if *mode == OutputMode::Json {
        return print_json(&config);
    }

    let toml = toml::to_string_pretty(&config)?;
    println!("{toml}");
    Ok(())
}
