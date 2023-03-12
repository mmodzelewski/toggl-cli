use std::{collections::HashMap, fs};

use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use reqwest::{
    blocking::{Client, Response},
    header::CONTENT_TYPE,
    Method,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args.command);

    let auth = read_env_variables()?;

    let client = Client::new();
    let json: Response = client
        .request(
            Method::GET,
            "https://api.track.toggl.com/api/v9/me".to_string(),
        )
        .basic_auth(auth.username, Some(auth.password))
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    println!("{:?}", json);

    return Ok(());
}

fn read_env_variables<'a>() -> Result<Auth> {
    let env_file = fs::read_to_string(".env").context(".env file should exist")?;
    let variables: HashMap<&str, &str> = env_file
        .lines()
        .filter_map(|line| line.split_once("="))
        .collect();

    return Ok(Auth {
        username: variables.get("USERNAME").context("Username must be provided")?.to_string(),
        password: variables.get("PASSWORD").context("Password must be provided")?.to_string(),
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
}
