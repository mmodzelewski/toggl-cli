# toggl-cli

toggl-cli is a command-line interface (CLI) tool for managing [Toggl Track](https://track.toggl.com/) through a terminal.

## Features

- Set an API key
- Set default workspace id and default project id globally and modify in directories
- Start a new time entry with a given description
- Stop currently running time entry
- Restart the latest time entry
- List recent time entries

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
toggl-cli --global set --api-token [API TOKEN]
```

The token will be saved in a config directory `~/.config/togglcli`.

### Start a new time entry
```sh
toggl-cli start "Time entry description" 
```

### Stop the current time entry
```sh
toggl-cli stop
```

### Restart the last time entry
```sh
toggl-cli restart
```

### Show recent time entries
```sh
toggl-cli recent
```

### Set default project
Global setting
```sh
toggl-cli --global --project-id [PROJECT ID]
```

Setting in a current directory
```sh
toggl-cli --project-id [PROJECT ID]
```

## Current limitations
- When starting a time entry it is not possible to specify tags.

## To do

[] Save config in toml files
[] Save API token in a keyring
[] Add autocompletion
[] Display project name when listing time entries

