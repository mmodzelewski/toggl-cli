use anyhow::{Context, Result};
use keyring::Entry;

pub fn get() -> Result<Option<String>> {
    let entry = get_token_entry()?;
    return Ok(entry.get_password().ok());
}

pub fn update(api_token: &str) -> Result<()> {
    let entry = get_token_entry()?;
    if api_token.is_empty() {
        entry
            .delete_password()
            .context("Could not delete api token")?;
    } else {
        entry
            .set_password(api_token)
            .context("Could not set api token")?;
    }
    return Ok(());
}

fn get_token_entry() -> Result<keyring::Entry> {
    return Entry::new("dev.modzelewski.toggl-cli", "api_token")
        .context("Could not create keyring entry");
}
