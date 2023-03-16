use std::fmt::Display;

use anyhow::{Result, Context};
use chrono::{Duration, Utc};
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::CONTENT_TYPE,
    Method,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub struct TogglClient {
    base_url: String,
    client: Client,
    config: Config,
}

impl TogglClient {
    pub fn new(config: Config) -> Result<TogglClient> {
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

    pub fn print_recent_entries(&self) -> Result<()> {
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

    pub fn print_current_entry(&self) -> Result<()> {
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

    pub fn stop_current_entry(&self) -> Result<()> {
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

    pub fn restart(&self) -> Result<()> {
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

    pub fn start(&self, description: Option<String>) -> Result<()> {
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

    pub fn get_default_workspace_id(&self) -> Result<u64> {
        return self
            .request(Method::GET, "me".to_string())
            .send()?
            .json::<UserData>()
            .map(|data| data.default_workspace_id)
            .context("Could not get user data");
    }
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
