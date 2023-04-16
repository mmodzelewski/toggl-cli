mod api_client;
mod api_token;
mod args;
mod config;
mod dirs;
mod toggl_client;

use anyhow::{Ok, Result};
use args::{Args, Command};
use clap::Parser;

use config::{load_config, update_config, Config};
use toggl_client::TogglClient;

use crate::api_client::ApiClient;

fn main() -> Result<()> {
    let args = Args::parse();

    let api_token = api_token::get()?;
    let config = load_config()?;
    let client = TogglClient::new(api_token, config);

    match args.command {
        Some(command) => match command {
            Command::Completions { shell } => {
                Args::print_completions(shell);
            }
            Command::Start {
                description,
                project_id,
            } => client?.start(description, project_id)?,
            Command::Stop => client?.stop_current_entry()?,
            Command::Status => client?.print_current_entry()?,
            Command::Recent => client?.print_recent_entries()?,
            Command::Restart => client?.restart()?,
            Command::Projects => client?.print_projects()?,
            Command::DefaultWorkspaceId => client?.print_default_workspace_id()?,
            Command::Set {
                global,
                project_id,
                workspace_id,
            } => update_config(
                global,
                Config {
                    workspace_id,
                    project_id,
                    projects: None,
                },
            )?,
            Command::Login { api_token } => login(&api_token)?,
        },
        None => client?.print_recent_entries()?,
    }

    return Ok(());
}

fn login(api_token: &str) -> Result<()> {
    let result = api_token::update(&api_token)?;
    match result {
        api_token::TokenUpdateResult::Deleted => println!("Deleted API token"),
        api_token::TokenUpdateResult::Updated => {
            println!("Updated API token");
            let client = ApiClient::new(Some(&api_token))?;
            let workspace_id = client.get_default_workspace_id()?;
            let projects = client.get_projects()?;
            println!("Setting default workspace id to {}", &workspace_id);
            update_config(
                true,
                Config {
                    workspace_id: Some(workspace_id),
                    project_id: None,
                    projects: Some(projects),
                },
            )?;
        }
    }
    return Ok(());
}
