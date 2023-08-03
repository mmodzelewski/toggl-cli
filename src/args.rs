use clap::CommandFactory;
use clap::{Parser, Subcommand, ValueHint};
use clap_complete::generate;
use clap_complete::Shell;
use std::io;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Args {
    pub fn print_completions(shell: Shell) {
        let mut cmd = Args::command();
        let cmd: &mut clap::Command = &mut cmd;
        generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Generate shell completions")]
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },

    #[command(about = "Start a new time entry")]
    Start {
        #[arg(value_hint = ValueHint::Other)]
        description: Option<String>,
        #[arg(long, short, help = "Project id")]
        project_id: Option<u64>,
        #[arg(long, short, help = "Start time")]
        start: Option<String>,
        #[arg(long, short, help = "Running time")]
        time: Option<String>,
    },

    #[command(about = "Stop the current time entry")]
    Stop,

    #[command(about = "Print the current time entry")]
    Status,

    #[command(about = "Print recent time entries")]
    Recent,

    #[command(about = "Print time entries from a given day grouped by description")]
    Summary {
        #[arg(help = "Number of days before today")]
        days_before: Option<u8>,
    },

    #[command(about = "Restart the last time entry")]
    Restart,

    #[command(about = "Switch to the time entry before the current one")]
    Switch,

    #[command(about = "List all projects")]
    Projects,

    #[command(about = "Print the default workspace id")]
    DefaultWorkspaceId,

    #[command(about = "Set configuration options")]
    Set {
        #[arg(long, help = "Set config globally")]
        global: bool,

        #[arg(long, short, help = "Set default project id")]
        project_id: Option<u64>,

        #[arg(long, short, help = "Set default workspace id")]
        workspace_id: Option<u64>,
    },

    #[command(about = "Set api token")]
    Login {
        #[arg(value_hint = ValueHint::Other)]
        api_token: String,
    },
}
