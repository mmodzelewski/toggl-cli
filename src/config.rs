use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::dirs::{find_global_config_dir, find_local_config, get_current_dir};

pub fn load_config() -> Result<Config> {
    let config = load_global_config()?;
    if config.is_none() {
        return Ok(Config::default());
    }
    let config = config.unwrap();

    let local_config = load_local_config()?;

    return Ok(Config {
        workspace_id: local_config
            .clone()
            .and_then(|lc| lc.workspace_id)
            .or(config.workspace_id),
        project_id: local_config
            .clone()
            .and_then(|lc| lc.project_id)
            .or(config.project_id),
    });
}

pub fn update_config(global: bool, new_config: Config) -> Result<()> {
    if global {
        let mut config = load_global_config()?.unwrap_or_default();
        config.update_project_id(new_config.project_id);
        config.update_workspace_id(new_config.workspace_id);
        save_global_config(&config)?;
    } else {
        let mut config = load_current_dir_config()?.unwrap_or_default();
        config.update_project_id(new_config.project_id);
        config.update_workspace_id(new_config.workspace_id);
        save_current_dir_config(&config)?;
    }
    return Ok(());
}

#[derive(Default, Clone, Deserialize, Serialize, Debug)]
pub struct Config {
    pub workspace_id: Option<u64>,
    pub project_id: Option<u64>,
}

impl Config {
    pub fn update_workspace_id(&mut self, workspace_id: Option<u64>) {
        if workspace_id.is_some() {
            self.workspace_id = workspace_id;
        }
    }

    pub fn update_project_id(&mut self, project_id: Option<u64>) {
        if project_id.is_some() {
            self.project_id = project_id;
        }
    }
}

fn load_global_config() -> Result<Option<Config>> {
    let config_dir = find_global_config_dir()?;
    let config_path = config_dir.join("config.toml");
    let config_exists = config_path
        .try_exists()
        .context("Couldn't read a global config file")?;
    if !config_exists {
        return Ok(None);
    }

    let config = fs::read_to_string(config_path).context("Couldn't read global config file.")?;
    let config = toml::from_str(&config).context("Couldn't parse global config file")?;
    return Ok(Some(config));
}

fn save_global_config(config: &Config) -> Result<()> {
    let config_dir = find_global_config_dir()?;
    let config_path = config_dir.join("config");
    let config_string = toml::to_string(&config).context("Couldn't serialize config")?;
    fs::write(&config_path, config_string).context("Could not save config file")?;
    return Ok(());
}

fn load_local_config() -> Result<Option<Config>> {
    return find_local_config(".toggl")?
        .map(|path| {
            fs::read_to_string(path)
                .context("Couldn't read config file")
                .and_then(|value| toml::from_str(&value).context("Couldn't parse config file"))
        })
        .transpose();
}

fn load_current_dir_config() -> Result<Option<Config>> {
    let current_dir = get_current_dir()?;
    let config_path = current_dir.join(".toggl");
    let exists = config_path.try_exists()?;
    if !exists {
        return Ok(None);
    }
    let config = fs::read_to_string(config_path).context("Couldn't read config file")?;
    let config = toml::from_str(&config).context("Couldn't parse config file")?;
    return Ok(Some(config));
}

fn save_current_dir_config(config: &Config) -> Result<()> {
    let config_dir = get_current_dir()?;
    let config_path = config_dir.join(".toggl");
    let config_string = toml::to_string(&config).context("Couldn't serialize config")?;
    fs::write(&config_path, config_string).context("Could not save config file")?;
    return Ok(());
}
