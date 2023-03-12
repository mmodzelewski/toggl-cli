use std::{collections::HashMap, fmt::Display, fs};

use anyhow::{Context, Ok, Result};
use chrono::Utc;
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

    let auth = read_env_variables()?;
    let client = TogglClient::new(auth)?;

    match args.command {
        Some(command) => match command {
            Command::Start => todo!(),
            Command::Stop => client.stop_current_entry()?,
            Command::Status => client.print_current_entry()?,
            Command::Recent => client.print_recent_entries()?,
            Command::Restart => client.restart()?,
        },
        None => client.print_recent_entries()?,
    }

    return Ok(());
}

struct TogglClient {
    base_url: String,
    client: Client,
    auth: Auth,
}

impl TogglClient {
    fn new(auth: Auth) -> Result<TogglClient> {
        return Ok(TogglClient {
            base_url: "https://api.track.toggl.com/api/v9/".to_string(),
            client: Client::new(),
            auth,
        });
    }

    fn request(&self, method: Method, path: String) -> RequestBuilder {
        self.client
            .request(method, (&self.base_url).to_string() + &path)
            .basic_auth(&self.auth.username, Some(&self.auth.password))
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
}

fn read_env_variables() -> Result<Auth> {
    let dirs = ProjectDirs::from("dev", "Modzelewski", "Toggl Cli")
        .context("Could not retrieve home directory")?;
    let env_file =
        fs::read_to_string(dirs.config_dir().join("config")).context("config file should exist")?;
    let variables: HashMap<&str, &str> = env_file
        .lines()
        .filter_map(|line| line.split_once("="))
        .collect();

    return Ok(Auth {
        username: variables
            .get("USERNAME")
            .context("Username must be provided")?
            .to_string(),
        password: variables
            .get("PASSWORD")
            .context("Password must be provided")?
            .to_string(),
    });
}

struct Auth {
    username: String,
    password: String,
}

#[derive(Subcommand, Debug)]
enum Command {
    Start,
    Stop,
    Status,
    Recent,
    Restart,
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

        let duration = &self.duration;
        if let Some(_) = &self.stop {
            write!(f, " - {} seconds", duration)?;
        }
        return write!(f, "");
    }
}

fn default_if_empty<'a>(text: &'a String, default: &'a String) -> &'a String {
    if text.is_empty() {
        return default;
    }
    return text;
}
