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
            Command::Stop => todo!(),
            Command::Status => print_current_entry(&client)?,
        },
        None => print_latest_entries(&client)?,
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
}

fn print_latest_entries(client: &TogglClient) -> Result<()> {
    let json: Vec<TimeEntry> = client
        .request(Method::GET, "me/time_entries".to_string())
        .send()?
        .json()?;

    for time_entry in json {
        println!("{}", time_entry);
    }

    return Ok(());
}

fn print_current_entry(client: &TogglClient) -> Result<()> {
    let response: Option<TimeEntry> = client
        .request(Method::GET, "me/time_entries/current".to_string())
        .send()?
        .json()?;

    if let Some(time_entry) = response {
        println!("{}", time_entry);
    } else {
        println!("There are no active time entries");
    }

    return Ok(());
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
}

#[derive(Debug, Deserialize)]
struct TimeEntry {
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
