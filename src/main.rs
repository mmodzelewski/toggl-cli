use std::{collections::HashMap, fmt::Display, fs};

use anyhow::{Context, Ok, Result};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::CONTENT_TYPE,
    Method,
};
use serde::Deserialize;

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
        let time_entries: Vec<TimeEntry> = self
            .request(Method::GET, "me/time_entries".to_string())
            .send()?
            .json()?;

        for time_entry in time_entries {
            println!("{}", time_entry);
        }

        return Ok(());
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
}

#[derive(Debug, Deserialize)]
struct TimeEntry {
    id: u64,
    workspace_id: u64,
    description: Option<String>,
    start: String,
    stop: Option<String>,
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

        return write!(f, "{}: {} - {}", description, &self.start, stop);
    }
}

fn default_if_empty<'a>(text: &'a String, default: &'a String) -> &'a String {
    if text.is_empty() {
        return default;
    }
    return text;
}
