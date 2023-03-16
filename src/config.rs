use std::{collections::HashMap, fs, path::PathBuf};
use directories::ProjectDirs;
use anyhow::{Result, Context};

fn get_config_dir<'a>() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "Modzelewski", "Toggl Cli")
        .context("Could not retrieve home directory")?;

    let config_dir = dirs.config_dir();

    let exists = config_dir
        .try_exists()
        .context("Could not access config directory")?;
    if !exists {
        fs::create_dir_all(config_dir)?;
    }
    return Ok(config_dir.to_owned());
}

pub fn get_config() -> Result<Config> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config");

    let config_exists = config_path
        .try_exists()
        .context("Couldn't read a config file")?;

    if !config_exists {
        return Ok(Config::default());
    }

    let config = fs::read_to_string(config_path)
        .context("Config file not found. Please use login command to set API token.")?;

    let parsed_config: HashMap<&str, &str> = config
        .lines()
        .filter_map(|line| line.split_once("="))
        .collect();

    let api_token = parsed_config
        .get("API_TOKEN")
        .map(|value| value.to_string());

    let workspace_id = parsed_config
        .get("DEFAULT_WORKSPACE_ID")
        .map(|id| id.parse::<u64>().context("Could not parse workspace_id"))
        .transpose()?;

    return Ok(Config {
        api_token,
        workspace_id,
    });
}

pub fn set_api_token(api_token: &str) -> Result<()> {
    let mut config = get_config()?;
    config.api_token = Some(api_token.to_string());

    save_config(&config)?;

    return Ok(());
}

fn save_config(config: &Config) -> Result<()> {
    let mut variables = HashMap::new();

    if let Some(api_token) = &config.api_token {
        variables.insert("API_TOKEN", api_token.to_owned());
    }
    if let Some(ref workspace_id) = &config.workspace_id {
        variables.insert("DEFAULT_WORKSPACE_ID", workspace_id.to_string());
    }

    let new_config = variables
        .iter()
        .map(|(key, value)| String::new() + key + "=" + value)
        .collect::<Vec<_>>()
        .join("\n");

    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config");
    fs::write(&config_path, new_config).context("Could not save config file")?;
    return Ok(());
}

pub fn set_workspace_id(workspace_id: u64) -> Result<()> {
    let mut config = get_config()?;
    config.workspace_id = Some(workspace_id);

    save_config(&config)?;

    return Ok(());
}

pub struct Config {
    pub api_token: Option<String>,
    pub workspace_id: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        return Config {
            api_token: None,
            workspace_id: None,
        };
    }
}
