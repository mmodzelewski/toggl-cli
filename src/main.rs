use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args.command);
}

#[derive(Subcommand, Debug)]
enum Command {
    Start,
    Stop,
}
