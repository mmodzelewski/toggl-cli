use anyhow::{anyhow, Context, Ok, Result};
use directories::ProjectDirs;
use std::{env, fs, path::PathBuf};

pub fn find_global_config_dir() -> Result<PathBuf> {
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

pub fn find_local_config(config_name: &str) -> Result<Option<PathBuf>> {
    let start = env::current_dir().context("Could not get current dir")?;

    let user_dirs =
        directories::UserDirs::new().ok_or_else(|| anyhow!("Could not retrieve home dir"))?;
    let home = user_dirs.home_dir().to_owned();

    if !start.starts_with(&home) {
        return Ok(None);
    }

    let mut current = start;
    let mut found: Option<PathBuf> = None;

    loop {
        let path = current.join(config_name);
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

    return Ok(found);
}

pub fn get_current_dir() -> Result<PathBuf> {
    let current = env::current_dir().context("Couldn't get current dir")?;

    let user_dirs = directories::UserDirs::new().ok_or_else(|| anyhow!("Couldn't get home dir"))?;
    let home = user_dirs.home_dir().to_owned();

    if !current.starts_with(&home) {
        return Err(anyhow!(
            "Current dir is not in a user's home folder. Cannot creat local config."
        ));
    }

    return Ok(current);
}
