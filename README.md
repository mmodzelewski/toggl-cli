# toggl-cli

toggl-cli is a command-line interface (CLI) tool for managing [Toggl Track](https://track.toggl.com/) through a terminal.

## Features

- Set an API key
- Set default workspace id and default project id globally and modify in directories
- Start a new time entry with a given description
- Stop currently running time entry
- Restart the latest time entry
- Switch back to the previously ended time entry
- List recent time entries
- Print a summary for a given day

## Installation

### From source

Make sure you have Rust and Cargo installed. If not, follow the instructions on the [Rust website](https://www.rust-lang.org/tools/install) to set up Rust on your machine.

Clone the repository and build the project:

```sh
git clone https://github.com/mmodzelewski/toggl-cli.git
cd toggl-cli
cargo build --release
```

The compiled binary will be available in `./target/release/toggl-cli`.
You can move it to a directory in your `PATH` for easy access.

Alternatively, you can use the `cargo install` option to build the program directly from git 
and add it to cargo bin directory.
```sh
cargo install --git https://github.com/mmodzelewski/toggl-cli
```

## Usage

To use toggl-cli, you'll need to provide your Toggl Track API token. You can find it in [Profile settings](https://track.toggl.com/profile) on Toggl Track.

```sh
toggl-cli login [API TOKEN]
```

The token will be saved in a system's keyring.

### Options

```
Usage: toggl-cli [COMMAND]

Commands:
  completions           Generate shell completions
  start                 Start a new time entry
  stop                  Stop the current time entry
  status                Print the current time entry
  recent                Print recent time entries
  summary               Print time entries from a given day grouped by description
  restart               Restart the last time entry
  switch                Switch to the time entry before the current one
  projects              List all projects
  default-workspace-id  Print the default workspace id
  set                   Set configuration options
  login                 Set api token
  help                  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Set configuration options

It is possible to set configuration gloablly or in a specific directory.
The configuration in a directory takes precedence over the global configuration.

The tool will look for local configuration going up the directories until it gets to a home folder.

Global configuration
```sh
toggl-cli --global --project-id [PROJECT ID]
```

Configuration in a current directory
```sh
toggl-cli --project-id [PROJECT ID]
```

## Current limitations
- When starting a time entry it is not possible to specify tags.

