use anyhow::{Context, Error, Ok, Result};
use std::{collections::HashMap, fmt::Display, fs, str::FromStr};

use crate::dirs::{find_global_config_dir, find_local_config, get_current_dir};

pub fn load_config() -> Result<Config> {
    let config = load_global_config()?;
    if config.is_none() {
        return Ok(Config::default());
    }
    let config = config.unwrap();

    let local_config = load_local_config()?;

    return Ok(Config {
        api_token: local_config
            .clone()
            .and_then(|lc| lc.api_token)
            .or(config.api_token),
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
        config.update_api_token(new_config.api_token);
        config.update_project_id(new_config.project_id);
        config.update_workspace_id(new_config.workspace_id);
        save_global_config(&config)?;
    } else {
        let mut config = load_current_dir_config()?.unwrap_or_default();
        if new_config.api_token.is_some() {
            println!("API token can only be set globally. Use --global option.");
        }
        config.update_project_id(new_config.project_id);
        config.update_workspace_id(new_config.workspace_id);
        save_current_dir_config(&config)?;
    }
    return Ok(());
}

#[derive(Default, Clone)]
pub struct Config {
    pub api_token: Option<String>,
    pub workspace_id: Option<u64>,
    pub project_id: Option<u64>,
}

impl Config {
    pub fn update_api_token(&mut self, api_token: Option<String>) {
        if api_token.is_some() {
            self.api_token = api_token;
        }
    }

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

impl FromStr for Config {
    type Err = Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let config: HashMap<&str, &str> = value
            .lines()
            .filter_map(|line| line.split_once("="))
            .collect();

        let api_token = config.get("API_TOKEN").map(|value| value.to_string());

        let workspace_id = config
            .get("WORKSPACE_ID")
            .map(|id| id.parse::<u64>().context("Could not parse workspace_id"))
            .transpose()?;

        let project_id = config
            .get("PROJECT_ID")
            .map(|id| id.parse::<u64>().context("Could not parse project_id"))
            .transpose()?;

        return Ok(Config {
            api_token,
            workspace_id,
            project_id,
        });
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut variables = HashMap::new();

        if let Some(api_token) = &self.api_token {
            variables.insert("API_TOKEN", api_token.to_owned());
        }
        if let Some(ref workspace_id) = &self.workspace_id {
            variables.insert("WORKSPACE_ID", workspace_id.to_string());
        }
        if let Some(ref project_id) = &self.project_id {
            variables.insert("PROJECT_ID", project_id.to_string());
        }

        let new_config = variables
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<_>>()
            .join("\n");

        return write!(f, "{}", new_config);
    }
}

fn load_global_config() -> Result<Option<Config>> {
    let config_dir = find_global_config_dir()?;
    let config_path = config_dir.join("config");
    let config_exists = config_path
        .try_exists()
        .context("Couldn't read a global config file")?;
    if !config_exists {
        return Ok(None);
    }

    let config = fs::read_to_string(config_path)
        .context("Couldn't read global config file.")?
        .parse::<Config>()
        .context("Couldn't parse global config file")?;
    return Ok(Some(config));
}

fn save_global_config(config: &Config) -> Result<()> {
    let config_dir = find_global_config_dir()?;
    let config_path = config_dir.join("config");
    fs::write(&config_path, config.to_string()).context("Could not save config file")?;
    return Ok(());
}

fn load_local_config() -> Result<Option<Config>> {
    return find_local_config(".toggl")?
        .map(|path| {
            fs::read_to_string(path)
                .context("Couldn't read config file")
                .and_then(|value| {
                    value
                        .parse::<Config>()
                        .context("Couldn't parse config file")
                })
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
    let config = fs::read_to_string(config_path)
        .context("Couldn't read config file")?
        .parse::<Config>()
        .context("Couldn't parse config file")?;
    return Ok(Some(config));
}

fn save_current_dir_config(config: &Config) -> Result<()> {
    let config_dir = get_current_dir()?;
    let config_path = config_dir.join(".toggl");
    fs::write(&config_path, config.to_string()).context("Could not save config file")?;
    return Ok(());
}
