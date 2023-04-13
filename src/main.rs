mod api_client;
mod args;
mod config;
mod dirs;
mod toggl_client;

use anyhow::{Ok, Result};
use args::{Args, Command};
use clap::Parser;

use config::{load_config, update_config, Config};
use toggl_client::TogglClient;

fn main() -> Result<()> {
    let args = Args::parse();

    let config = load_config()?;
    let client = TogglClient::new(config)?;

    match args.command {
        Some(command) => match command {
            Command::Completions { shell } => {
                Args::print_completions(shell);
            }
            Command::Start {
                description,
                project_id,
            } => client.start(description, project_id)?,
            Command::Stop => client.stop_current_entry()?,
            Command::Status => client.print_current_entry()?,
            Command::Recent => client.print_recent_entries()?,
            Command::Restart => client.restart()?,
            Command::Projects => client.print_projects()?,
            Command::DefaultWorkspaceId => client.print_default_workspace_id()?,
            Command::Set {
                global,
                project_id,
                workspace_id,
                api_token,
            } => update_config(
                global,
                Config {
                    api_token,
                    workspace_id,
                    project_id,
                },
            )?,
        },
        None => client.print_recent_entries()?,
    }

    return Ok(());
}
