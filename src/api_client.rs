use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::CONTENT_TYPE,
    Method,
};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.track.toggl.com/api/v9/";

pub struct ApiClient {
    client: Client,
    api_token: String,
}

impl ApiClient {
    pub fn new(api_token: Option<&str>) -> Result<ApiClient> {
        if let Some(api_token) = api_token {
            return Ok(ApiClient {
                client: Client::new(),
                api_token: api_token.to_string(),
            });
        }
        return Err(anyhow!("Missing API token. Use login command to set it"));
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let builder = self
            .client
            .request(method, format!("{}{}", BASE_URL, path))
            .basic_auth(&self.api_token, Some("api_token"))
            .header(CONTENT_TYPE, "application/json");

        return Ok(builder);
    }

    pub fn get_recent_entries(&self) -> Result<Vec<TimeEntryDto>> {
        return self
            .request(Method::GET, "me/time_entries")?
            .send()?
            .json()
            .context("Could not get time entries");
    }

    pub fn get_current_entry(&self) -> Result<Option<TimeEntryDto>> {
        return self
            .request(Method::GET, "me/time_entries/current")?
            .send()?
            .json()
            .context("Could not get time entry");
    }

    pub fn stop_current_entry(&self) -> Result<Option<TimeEntryDto>> {
        let running_time_entry = self.get_current_entry()?;
        if let Some(running_time_entry) = running_time_entry {
            let path = format!(
                "workspaces/{}/time_entries/{}/stop",
                running_time_entry.workspace_id, running_time_entry.id
            );
            let stopped_time_entry: TimeEntryDto = self
                .request(Method::PATCH, &path)?
                .send()?
                .json()
                .context("Could not stop the current time entry")?;
            return Ok(Some(stopped_time_entry));
        }
        return Ok(None);
    }

    pub fn restart(&self, time_entry: &TimeEntryDto) -> Result<TimeEntryDto> {
        let new_time_entry = NewTimeEntry::from_time_entry(time_entry)?;
        return self.start_time_entry(new_time_entry);
    }

    pub fn start(
        &self,
        workspace_id: u64,
        description: Option<String>,
        project_id: Option<u64>,
    ) -> Result<TimeEntryDto> {
        let now = Utc::now();
        let new_time_entry = NewTimeEntry {
            workspace_id,
            created_with: "toggl-cli".to_string(),
            description,
            project_id,
            start: format!("{:?}", now),
            duration: -1 * now.timestamp(),
        };

        return self.start_time_entry(new_time_entry);
    }

    fn start_time_entry(&self, new_time_entry: NewTimeEntry) -> Result<TimeEntryDto> {
        let path = format!("workspaces/{}/time_entries", &new_time_entry.workspace_id);
        let stared_entry: TimeEntryDto = self
            .request(Method::POST, &path)?
            .json(&new_time_entry)
            .send()?
            .json()
            .context("Could not start a time entry")?;
        return Ok(stared_entry);
    }

    pub fn get_default_workspace_id(&self) -> Result<u64> {
        return self
            .request(Method::GET, "me")?
            .send()?
            .json::<UserData>()
            .map(|data| data.default_workspace_id)
            .context("Could not get user data");
    }

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        return self
            .request(Method::GET, "me/projects")?
            .send()?
            .json::<Vec<Project>>()
            .context("Could not get projects");
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct UserData {
    default_workspace_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct TimeEntryDto {
    pub id: u64,
    pub workspace_id: u64,
    pub description: Option<String>,
    pub project_id: Option<u64>,
    pub start: String,
    pub stop: Option<String>,
    pub duration: i64,
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
    fn from_time_entry(time_entry: &TimeEntryDto) -> Result<NewTimeEntry> {
        let now = Utc::now();
        return Ok(NewTimeEntry {
            workspace_id: time_entry.workspace_id,
            created_with: "toggl-cli".to_string(),
            description: time_entry.description.to_owned(),
            project_id: time_entry.project_id,
            start: format!("{:?}", now),
            duration: -1 * now.timestamp(),
        });
    }
}
