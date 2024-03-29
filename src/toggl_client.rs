use std::{collections::HashMap, fmt::Display};

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};

use crate::{
    api_client::{ApiClient, Project, TimeEntryDto},
    config::Config,
};

pub struct TogglClient {
    api_client: ApiClient,
    config: Config,
}

impl TogglClient {
    pub fn new(api_token: Option<String>, config: Config) -> Result<TogglClient> {
        return Ok(TogglClient {
            api_client: ApiClient::new(api_token.as_deref())?,
            config,
        });
    }

    pub fn print_recent_entries(&self) -> Result<()> {
        let time_entries = self.get_recent_entries()?;
        let today = Local::now().date_naive();
        let today_entries = time_entries
            .iter()
            .filter(|entry| entry.start.date_naive() == today)
            .collect::<Vec<_>>();

        if today_entries.len() > 0 {
            print!(" -- Today -- ");
            let total = today_entries
                .iter()
                .map(|entry| {
                    if entry.stop.is_some() {
                        entry.duration
                    } else {
                        Utc::now().timestamp() - entry.start.timestamp()
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
            .filter(|entry| entry.start.date_naive() != today)
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

    pub fn print_day_summary(&self, days_before: Option<u8>) -> Result<()> {
        let today = Local::now();
        let day = (today - Duration::days(days_before.unwrap_or(0) as i64)).date_naive();
        let time_entries = self.get_entries_from_day(day)?;

        if time_entries.len() > 0 {
            if day == today.date_naive() {
                print!(" -- Today -- ");
            } else {
                print!(" -- {} -- ", day);
            }
            let total = time_entries
                .iter()
                .map(|entry| {
                    if entry.stop.is_some() {
                        entry.duration
                    } else {
                        Utc::now().timestamp() - entry.start.timestamp()
                    }
                })
                .sum::<i64>();
            let duration = Duration::seconds(total);
            let hours = duration.num_hours();
            let minutes = duration.num_minutes() - hours * 60;
            println!("⌛{} hours {:02} minutes", hours, minutes);
        }

        let mut summed_entries = HashMap::new();
        time_entries.iter().for_each(|entry| {
            if entry.stop.is_none() {
                return;
            }
            let description = entry.description.clone().unwrap_or(String::from(""));
            let project = entry.project_name.clone().unwrap_or(String::from(""));
            let duration = entry.duration;
            summed_entries
                .entry(description)
                .and_modify(|v: &mut (String, i64)| (*v).1 += duration)
                .or_insert((project, duration));
        });

        for (key, (project, time)) in summed_entries {
            let duration = Duration::seconds(time);
            println!("{}\t[{}]\t{}", format_duration(&duration), project, key);
        }

        return Ok(());
    }

    fn get_recent_entries(&self) -> Result<Vec<TimeEntry>> {
        return self.api_client.get_recent_entries().and_then(|vec| {
            vec.into_iter()
                .map(|dto| TimeEntry::from_dto(&dto, &self.config))
                .collect::<Result<Vec<TimeEntry>>>()
        });
    }

    fn get_entries_from_day(&self, day: NaiveDate) -> Result<Vec<TimeEntry>> {
        return self.api_client.get_entries_from_day(day).and_then(|vec| {
            vec.into_iter()
                .map(|dto| TimeEntry::from_dto(&dto, &self.config))
                .collect::<Result<Vec<TimeEntry>>>()
        });
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
            .api_client
            .get_current_entry()?
            .map(|dto| TimeEntry::from_dto(&dto, &self.config))
            .transpose();
    }

    pub fn stop_current_entry(&self) -> Result<()> {
        if let Some(stopped_entry) = self.api_client.stop_current_entry()? {
            println!(
                "Stopped time entry: {}",
                TimeEntry::from_dto(&stopped_entry, &self.config)?
            );
        } else {
            println!("There are no active time entries");
        }

        return Ok(());
    }

    pub fn restart(&self) -> Result<()> {
        let recent_entries = self.api_client.get_recent_entries()?;
        let last_one = recent_entries.first();
        if let Some(last_one) = last_one {
            let started = self.api_client.restart(&last_one)?;
            println!(
                "Time entry started: {}",
                TimeEntry::from_dto(&started, &self.config)?
            );
        } else {
            println!("There are no recent entries");
        }
        return Ok(());
    }

    pub fn switch(&self) -> Result<()> {
        let recent_entries = self.api_client.get_recent_entries()?;
        let prev = recent_entries.iter().find(|entry| entry.stop.is_some());
        if let Some(prev) = prev {
            let started = self.api_client.restart(&prev)?;
            println!(
                "Time entry started: {}",
                TimeEntry::from_dto(&started, &self.config)?
            );
        } else {
            println!("There are no recent entries");
        }
        return Ok(());
    }

    pub fn start(
        &self,
        description: Option<String>,
        project_id: Option<u64>,
        start: Option<String>,
        time: Option<String>,
    ) -> Result<()> {
        let workspace_id = self
            .config
            .workspace_id
            .context("workspace id should be set")?;
        let started_entry = self.api_client.start(
            workspace_id,
            description,
            project_id.or_else(|| self.config.project_id),
            start,
            time,
        )?;
        println!(
            "Time entry started: {}",
            TimeEntry::from_dto(&started_entry, &self.config)?
        );
        return Ok(());
    }

    pub fn print_default_workspace_id(&self) -> Result<()> {
        let id = self.api_client.get_default_workspace_id()?;
        println!("Workspace id {}", id);
        return Ok(());
    }

    pub fn print_projects(&self) -> Result<()> {
        self.api_client
            .get_projects()?
            .iter()
            .for_each(|project| println!("[{}] {}", project.id, project.name));
        return Ok(());
    }
}

struct TimeEntry {
    _id: u64,
    _workspace_id: u64,
    description: Option<String>,
    _project_id: Option<u64>,
    project_name: Option<String>,
    start: DateTime<Local>,
    stop: Option<DateTime<Local>>,
    duration: i64,
}

impl TimeEntry {
    fn from_dto(dto: &TimeEntryDto, config: &Config) -> Result<TimeEntry> {
        return Ok(TimeEntry {
            _id: dto.id,
            _workspace_id: dto.workspace_id,
            description: dto.description.to_owned(),
            _project_id: dto.project_id,
            project_name: find_project_name(dto.project_id, &config.projects),
            start: dto.start.parse()?,
            stop: dto.stop.to_owned().map(|value| value.parse()).transpose()?,
            duration: dto.duration,
        });
    }
}

fn find_project_name(project_id: Option<u64>, projects: &Option<Vec<Project>>) -> Option<String> {
    project_id.and_then(|project_id| {
        projects.as_ref().and_then(|projects| {
            projects
                .iter()
                .find(|project| project.id == project_id)
                .map(|project| project.name.to_owned())
        })
    })
}

impl Display for TimeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let empty_description = "no description".to_string();
        let description = match &self.description {
            Some(desc) => default_if_empty(desc, &empty_description),
            None => &empty_description,
        };

        let still_running = "in progress".to_string();
        let stop = self
            .stop
            .as_ref()
            .map(|value| format_time(&value))
            .unwrap_or(still_running);

        write!(f, "{} - {}", format_time(&self.start), stop)?;

        if let Some(day) = format_date(&self.start) {
            write!(f, " {day}")?;
        }

        let duration = Duration::seconds(self.duration);
        if let Some(_) = &self.stop {
            write!(f, " ({})", format_duration(&duration))?;
        }
        if let Some(project_name) = &self.project_name {
            write!(f, "\t[{project_name}]")?;
        }
        return write!(f, "\t{description}");
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
