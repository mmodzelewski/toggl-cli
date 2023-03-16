use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Start {
        description: Option<String>,
    },
    Stop,
    Status,
    Recent,
    Restart,
    Login {
        #[arg(long)]
        token: String,
    },
}
