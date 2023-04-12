use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Local, Utc};
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

    fn request(&self, method: Method, path: String) -> Result<RequestBuilder> {
        if self.config.api_token.is_none() {
            return Err(anyhow!("Missing API token. Use login command to set it"));
        }
        let builder = self
            .client
            .request(method, (&self.base_url).to_string() + &path)
            .basic_auth(&self.config.api_token.as_ref().unwrap(), Some("api_token"))
            .header(CONTENT_TYPE, "application/json");

        return Ok(builder);
    }

    pub fn print_recent_entries(&self) -> Result<()> {
        let time_entries = self.get_recent_entries()?;
        let today = Local::now().date_naive();
        let today_entries = time_entries
            .iter()
            .filter(|entry| parse_date_time(&entry.start).unwrap().date_naive() == today)
            .collect::<Vec<_>>();

        if today_entries.len() > 0 {
            print!(" -- Today -- ");
            let total = today_entries
                .iter()
                .map(|entry| {
                    if entry.stop.is_some() {
                        entry.duration
                    } else {
                        Utc::now().timestamp() - parse_date_time(&entry.start).unwrap().timestamp()
                    }
                })
                .sum::<i64>();
            let duration = Duration::seconds(total);
            let hours = duration.num_hours();
            let minutes = duration.num_minutes() - hours * 60;
            println!("⌛{} hours {:02} minutes", hours, minutes);
        }

        for time_entry in today_entries {
            println!("{}", time_entry);
        }

        let older_entries = time_entries
            .iter()
            .filter(|entry| parse_date_time(&entry.start).unwrap().date_naive() != today)
            .take(10)
            .collect::<Vec<_>>();

        if older_entries.len() > 0 {
            println!(" -- Older -- ");
            for time_entry in older_entries {
                println!("{}", time_entry);
            }
        }

        return Ok(());
    }

    fn get_recent_entries(&self) -> Result<Vec<TimeEntry>> {
        return self
            .request(Method::GET, "me/time_entries".to_string())?
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
            .request(Method::GET, "me/time_entries/current".to_string())?
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
                )?
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
                )?
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

    pub fn start(&self, description: Option<String>, project_id: Option<u64>) -> Result<()> {
        let now = Utc::now();
        let workspace_id = self
            .config
            .workspace_id
            .context("workspace id should be set")?;
        let new_time_entry = NewTimeEntry {
            workspace_id,
            created_with: "toggl-cli".to_string(),
            description,
            project_id: project_id.or_else(|| self.config.project_id),
            start: format!("{:?}", now),
            duration: -1 * now.timestamp(),
        };

        let stared_entry: TimeEntry = self
            .request(
                Method::POST,
                format!("workspaces/{}/time_entries", workspace_id),
            )?
            .json(&new_time_entry)
            .send()?
            .json()
            .context("Could not start a time entry")?;
        println!("Time entry started: {}", stared_entry);
        return Ok(());
    }

    pub fn get_default_workspace_id(&self) -> Result<u64> {
        return self
            .request(Method::GET, "me".to_string())?
            .send()?
            .json::<UserData>()
            .map(|data| data.default_workspace_id)
            .context("Could not get user data");
    }

    pub fn print_default_workspace_id(&self) -> Result<()> {
        let id = self.get_default_workspace_id()?;
        println!("Workspace id {}", id);
        return Ok(());
    }

    pub fn print_projects(&self) -> Result<()> {
        self.request(Method::GET, "me/projects".to_string())?
            .send()?
            .json::<Vec<Project>>()
            .context("Could not get projects")?
            .iter()
            .for_each(|project| println!("[{}] {}", project.id, project.name));
        return Ok(());
    }
}

#[derive(Debug, Deserialize)]
struct Project {
    id: u64,
    name: String,
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

        let start = parse_date_time(&self.start).unwrap();

        let still_running = "in progress".to_string();
        let stop = self
            .stop
            .as_ref()
            .map(|value| parse_date_time(&value).unwrap())
            .map(|value| format_time(&value))
            .unwrap_or(still_running);

        write!(f, "{} - {}", format_time(&start), stop)?;

        if let Some(day) = format_date(&start) {
            write!(f, " {}", day)?;
        }

        let duration = Duration::seconds(self.duration);
        if let Some(_) = &self.stop {
            write!(f, " ({})", format_duration(&duration))?;
        }
        return write!(f, "\t{}", description);
    }
}

fn parse_date_time(datetime: &str) -> Result<DateTime<Local>> {
    return DateTime::<Local>::from_str(datetime).context("Could not parse date time");
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

fn format_date(datetime: &DateTime<Local>) -> Option<String> {
    if datetime.date_naive() == Local::now().date_naive() {
        return None;
    }
    if datetime.date_naive() == Local::now().date_naive().pred_opt().unwrap() {
        return Some("yesterday".to_string());
    }
    return Some(datetime.format("%d %b").to_string());
}

fn format_time(datetime: &DateTime<Local>) -> String {
    return datetime.format("%H:%M").to_string();
}

fn default_if_empty<'a>(text: &'a String, default: &'a String) -> &'a String {
    if text.is_empty() {
        return default;
    }
    return text;
}
