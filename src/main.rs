mod args;
mod toggl_client;
mod config;


use anyhow::{anyhow, Ok, Result};
use args::{Args, Command};
use clap::Parser;
use toggl_client::TogglClient;

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Login { ref token }) = args.command {
        config::set_api_token(token)?;
        let config = config::get_config()?;
        let client = TogglClient::new(config)?;
        config::set_workspace_id(client.get_default_workspace_id()?)?;
        return Ok(());
    }

    let config = config::get_config()?;
    if let config::Config {
        api_token: None,
        workspace_id: _,
    } = config
    {
        return Err(anyhow!("Missing API token. Use login command to set it"));
    }
    let client = TogglClient::new(config)?;

    match args.command {
        Some(command) => match command {
            Command::Start { description } => client.start(description)?,
            Command::Stop => client.stop_current_entry()?,
            Command::Status => client.print_current_entry()?,
            Command::Recent => client.print_recent_entries()?,
            Command::Restart => client.restart()?,
            Command::Login { token: _ } => unreachable!(),
        },
        None => client.print_recent_entries()?,
    }

    return Ok(());
}


