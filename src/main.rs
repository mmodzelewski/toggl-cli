mod args;
mod config;
mod toggl_client;

use anyhow::{anyhow, Ok, Result};
use args::{Args, Command};
use clap::Parser;
use config::Config;
use toggl_client::TogglClient;

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Login { ref token }) = args.command {
        let mut config = Config::load()?;
        config.set_api_token(token);
        let client = TogglClient::new(config.clone())?;
        config.set_workspace_id(client.get_default_workspace_id()?);
        config.save()?;
        return Ok(());
    }

    let config = Config::load()?;
    if let config::Config {
        api_token: None,
        workspace_id: _,
        project_id: _,
    } = config
    {
        return Err(anyhow!("Missing API token. Use login command to set it"));
    }
    let client = TogglClient::new(config)?;

    match args.command {
        Some(command) => match command {
            Command::Start {
                description,
                project_id,
            } => client.start(description, project_id)?,
            Command::Stop => client.stop_current_entry()?,
            Command::Status => client.print_current_entry()?,
            Command::Recent => client.print_recent_entries()?,
            Command::Restart => client.restart()?,
            Command::Projects => client.print_projects()?,
            Command::Login { token: _ } => unreachable!(),
        },
        None => client.print_recent_entries()?,
    }

    return Ok(());
}
