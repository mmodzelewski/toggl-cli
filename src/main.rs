use std::{collections::HashMap, fs};

use clap::{Parser, Subcommand};
use reqwest::{blocking::Client, header::CONTENT_TYPE, Method};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();
    println!("{:?}", args.command);

    let env_file = fs::read_to_string(".env").expect(".env file should exist");
    let variables: HashMap<&str, &str> = env_file
        .lines()
        .filter_map(|line| line.split_once("="))
        .collect();

    let client = Client::new();
    let json: Value = client
        .request(
            Method::GET,
            "https://api.track.toggl.com/api/v9/me".to_string(),
        )
        .basic_auth(
            variables["USERNAME"],
            variables.get("PASSWORD")
        )
        .header(CONTENT_TYPE, "application/json")
        .send()?
        .json()?;

    println!("{:?}", json);

    return Ok(());
}

#[derive(Subcommand, Debug)]
enum Command {
    Start,
    Stop,
}
