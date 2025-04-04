use anyhow::{Context, Result};
use keyring::Entry;

pub enum TokenUpdateResult {
    Deleted,
    Updated,
}

pub fn get() -> Result<Option<String>> {
    let entry = get_token_entry()?;
    return Ok(entry.get_password().ok());
}

pub fn update(api_token: &str) -> Result<TokenUpdateResult> {
    let entry = get_token_entry()?;
    if api_token.is_empty() {
        entry
            .delete_credential()
            .context("Could not delete api token")?;
        return Ok(TokenUpdateResult::Deleted);
    }
    entry
        .set_password(api_token)
        .context("Could not set api token")?;
    return Ok(TokenUpdateResult::Updated);
}

fn get_token_entry() -> Result<keyring::Entry> {
    return Entry::new("dev.modzelewski.toggl-cli", "api_token")
        .context("Could not create keyring entry");
}
