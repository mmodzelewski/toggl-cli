use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::{collections::HashMap, env, fs, path::PathBuf};

fn get_config_dir() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "Modzelewski", "Toggl Cli")
        .context("Could not retrieve home directory")?;

    let config_dir = dirs.config_dir().to_owned();

    let exists = config_dir
        .try_exists()
        .context("Could not access config directory")?;
    if !exists {
        fs::create_dir_all(&config_dir)?;
    }
    return Ok(config_dir);
}

#[derive(Default, Clone)]
pub struct Config {
    pub api_token: Option<String>,
    pub workspace_id: Option<u64>,
    pub project_id: Option<u64>,
}

impl Config {
    pub fn set_api_token(&mut self, api_token: &str) {
        self.api_token = Some(api_token.to_string());
    }

    pub fn set_workspace_id(&mut self, workspace_id: u64) {
        self.workspace_id = Some(workspace_id);
    }

    pub fn load() -> Result<Config> {
        let config_dir = get_config_dir()?;
        let config_path = config_dir.join("config");

        let config_exists = config_path
            .try_exists()
            .context("Couldn't read a config file")?;

        let local_config = Config::load_local();

        if !config_exists {
            if local_config.is_none() {
                return Ok(Config::default());
            } else {
                return Ok(local_config.unwrap());
            }
        }

        return Config::parse_config(config_path).map(|conf| Config {
            api_token: local_config
                .clone()
                .and_then(|lc| lc.api_token)
                .or(conf.api_token),
            workspace_id: local_config
                .clone()
                .and_then(|lc| lc.workspace_id)
                .or(conf.workspace_id),
            project_id: local_config
                .clone()
                .and_then(|lc| lc.project_id)
                .or(conf.project_id),
        });
    }

    fn load_local() -> Option<Config> {
        return Config::find_local_config_dir()
            .map(|path| Config::parse_config(path))
            .transpose()
            .ok()
            .flatten();
    }

    fn parse_config(path: PathBuf) -> Result<Config> {
        let config = fs::read_to_string(path)
            .context("Config file not found. Please use login command to set API token.")?;

        let parsed_config: HashMap<&str, &str> = config
            .lines()
            .filter_map(|line| line.split_once("="))
            .collect();

        let api_token = parsed_config
            .get("API_TOKEN")
            .map(|value| value.to_string());

        let workspace_id = parsed_config
            .get("WORKSPACE_ID")
            .map(|id| id.parse::<u64>().context("Could not parse workspace_id"))
            .transpose()?;

        let project_id = parsed_config
            .get("PROJECT_ID")
            .map(|id| id.parse::<u64>().context("Could not parse project_id"))
            .transpose()?;

        return Ok(Config {
            api_token,
            workspace_id,
            project_id,
        });
    }

    fn find_local_config_dir() -> Option<PathBuf> {
        let start = env::current_dir().ok();
        if start.is_none() {
            return None;
        }
        let start = start.unwrap();

        let user_dirs = directories::UserDirs::new();
        if user_dirs.is_none() {
            return None;
        }
        let home = user_dirs.unwrap().home_dir().to_owned();

        if !start.starts_with(&home) {
            return None;
        }

        let mut current = start;
        let mut found: Option<PathBuf> = None;

        loop {
            let path = current.join(".toggl");
            let exists = path.try_exists().ok().unwrap_or(false);
            if exists {
                found = Some(path);
                break;
            }

            if current == home {
                break;
            }

            current.pop();
        }

        return found;
    }

    pub fn save(self: &Config) -> Result<()> {
        let mut variables = HashMap::new();

        if let Some(api_token) = &self.api_token {
            variables.insert("API_TOKEN", api_token.to_owned());
        }
        if let Some(ref workspace_id) = &self.workspace_id {
            variables.insert("WORKSPACE_ID", workspace_id.to_string());
        }

        let new_config = variables
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<_>>()
            .join("\n");

        let config_dir = get_config_dir()?;
        let config_path = config_dir.join("config");
        fs::write(&config_path, new_config).context("Could not save config file")?;
        return Ok(());
    }
}
