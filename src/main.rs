use std::{collections::HashMap, fmt::Display, fs, path::PathBuf};

use anyhow::{anyhow, Context, Ok, Result};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::CONTENT_TYPE,
    Method,
};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Login { ref token }) = args.command {
        set_api_token(token)?;
        let config = get_config()?;
        let client = TogglClient::new(config)?;
        set_workspace_id(client.get_default_workspace_id()?)?;
        return Ok(());
    }

    let config = get_config()?;
    if let Config {
        api_token: None,
        workspace_id: _,
    } = config
    {
        return Err(anyhow!("Missing API token. Use login command to set it"));
    }
    let client = TogglClient::new(config)?;

    match args.command {
        Some(command) => match command {
            Command::Start { description } => client.start(description)?,
            Command::Stop => client.stop_current_entry()?,
            Command::Status => client.print_current_entry()?,
            Command::Recent => client.print_recent_entries()?,
            Command::Restart => client.restart()?,
            Command::Login { token: _ } => unreachable!(),
        },
        None => client.print_recent_entries()?,
    }

    return Ok(());
}

struct TogglClient {
    base_url: String,
    client: Client,
    config: Config,
}

impl TogglClient {
    fn new(config: Config) -> Result<TogglClient> {
        return Ok(TogglClient {
            base_url: "https://api.track.toggl.com/api/v9/".to_string(),
            client: Client::new(),
            config,
        });
    }

    fn request(&self, method: Method, path: String) -> RequestBuilder {
        self.client
            .request(method, (&self.base_url).to_string() + &path)
            .basic_auth(&self.config.api_token.as_ref().unwrap(), Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
    }

    fn print_recent_entries(&self) -> Result<()> {
        let time_entries = self.get_recent_entries()?;
        for time_entry in time_entries {
            println!("{}", time_entry);
        }

        return Ok(());
    }

    fn get_recent_entries(&self) -> Result<Vec<TimeEntry>> {
        return self
            .request(Method::GET, "me/time_entries".to_string())
            .send()?
            .json()
            .context("Could not get time entries");
    }

    fn print_current_entry(&self) -> Result<()> {
        let maybe_time_entry = self.get_current_entry()?;
        if let Some(time_entry) = maybe_time_entry {
            println!("{}", time_entry);
        } else {
            println!("There are no active time entries");
        }

        return Ok(());
    }

    fn get_current_entry(&self) -> Result<Option<TimeEntry>> {
        return self
            .request(Method::GET, "me/time_entries/current".to_string())
            .send()?
            .json()
            .context("Could not get time entry");
    }

    fn stop_current_entry(&self) -> Result<()> {
        let maybe_time_entry = self.get_current_entry()?;
        if let Some(time_entry) = maybe_time_entry {
            let time_entry: TimeEntry = self
                .request(
                    Method::PATCH,
                    format!(
                        "workspaces/{}/time_entries/{}/stop",
                        time_entry.workspace_id, time_entry.id
                    ),
                )
                .send()?
                .json()
                .context("Could not stop the current time entry")?;
            println!("Stopped time entry: {}", time_entry)
        }
        return Ok(());
    }

    fn restart(&self) -> Result<()> {
        let recent_entries = self.get_recent_entries()?;
        let maybe_last_one = recent_entries.first();
        if let Some(last_one) = maybe_last_one {
            let new_time_entry = NewTimeEntry::from_time_entry(last_one)?;
            let stared_entry: TimeEntry = self
                .request(
                    Method::POST,
                    format!("workspaces/{}/time_entries", last_one.workspace_id),
                )
                .json(&new_time_entry)
                .send()?
                .json()
                .context("Could not start a time entry")?;
            println!("Time entry started: {}", stared_entry);
        } else {
            println!("No recent time entry to restart");
        }
        return Ok(());
    }

    fn start(&self, description: Option<String>) -> Result<()> {
        let now = Utc::now();
        let workspace_id = self
            .config
            .workspace_id
            .context("workspace id should be set")?;
        let new_time_entry = NewTimeEntry {
            workspace_id,
            created_with: "toggl-cli".to_string(),
            description,
            project_id: None,
            start: format!("{:?}", now),
            duration: -1 * now.timestamp(),
        };

        let stared_entry: TimeEntry = self
            .request(
                Method::POST,
                format!("workspaces/{}/time_entries", workspace_id),
            )
            .json(&new_time_entry)
            .send()?
            .json()
            .context("Could not start a time entry")?;
        println!("Time entry started: {}", stared_entry);
        return Ok(());
    }

    fn get_default_workspace_id(&self) -> Result<u64> {
        return self
            .request(Method::GET, "me".to_string())
            .send()?
            .json::<UserData>()
            .map(|data| data.default_workspace_id)
            .context("Could not get user data");
    }
}

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

fn get_config() -> Result<Config> {
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

fn set_api_token(api_token: &str) -> Result<()> {
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

fn set_workspace_id(workspace_id: u64) -> Result<()> {
    let mut config = get_config()?;
    config.workspace_id = Some(workspace_id);

    save_config(&config)?;

    return Ok(());
}

struct Config {
    api_token: Option<String>,
    workspace_id: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        return Config {
            api_token: None,
            workspace_id: None,
        };
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    Start {
        description: Option<String>,
    },
    Stop,
    Status,
    Recent,
    Restart,
    Login {
        #[arg(long)]
        token: String,
    },
}

#[derive(Debug, Deserialize)]
struct UserData {
    default_workspace_id: u64,
}

#[derive(Debug, Deserialize)]
struct TimeEntry {
    id: u64,
    workspace_id: u64,
    description: Option<String>,
    project_id: Option<u64>,
    start: String,
    stop: Option<String>,
    duration: i64,
}

#[derive(Serialize)]
struct NewTimeEntry {
    workspace_id: u64,
    created_with: String,
    description: Option<String>,
    project_id: Option<u64>,
    start: String,
    duration: i64,
}

impl NewTimeEntry {
    fn from_time_entry(time_entry: &TimeEntry) -> Result<NewTimeEntry> {
        let now = Utc::now();
        return Ok(NewTimeEntry {
            workspace_id: time_entry.workspace_id,
            created_with: "toggl-cli".to_string(),
            description: time_entry.description.to_owned(),
            project_id: time_entry.project_id.to_owned(),
            start: format!("{:?}", now),
            duration: -1 * now.timestamp(),
        });
    }
}

impl Display for TimeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let empty_description = "no description".to_string();
        let description = match &self.description {
            Some(desc) => default_if_empty(desc, &empty_description),
            None => &empty_description,
        };

        let still_running = "in progress".to_string();
        let stop = match &self.stop {
            Some(time) => time,
            None => &still_running,
        };
        write!(f, "{}: {} - {}", description, &self.start, stop)?;

        let duration = Duration::seconds(self.duration);
        if let Some(_) = &self.stop {
            write!(f, " - {}", format_duration(&duration))?;
        }
        return write!(f, "");
    }
}

fn format_duration(duration: &Duration) -> String {
    let mut result = String::new();
    let hours = duration.num_hours();
    if hours > 0 {
        let hours_part = format!("{} h ", hours);
        result += hours_part.as_str();
    }

    let minutes = duration.num_minutes() % 60;
    let minutes_part = format!("{} min", minutes);
    result += minutes_part.as_str();

    return result;
}

fn default_if_empty<'a>(text: &'a String, default: &'a String) -> &'a String {
    if text.is_empty() {
        return default;
    }
    return text;
}
