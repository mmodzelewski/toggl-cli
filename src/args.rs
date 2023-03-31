use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub global: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Start {
        description: Option<String>,
        #[arg(long, short)]
        project_id: Option<u64>,
    },
    Stop,
    Status,
    Recent,
    Restart,
    Projects,
    DefaultWorkspaceId,
    Set {
        #[arg(long, short)]
        project_id: Option<u64>,
        #[arg(long, short)]
        workspace_id: Option<u64>,
        #[arg(long, short)]
        api_token: Option<String>,
    },
}
