use std::env;

use super::output::Output;
use crate::error::Result;
use crate::garmin::auth::{self, Tokens};
use chrono::DateTime;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Log in to Garmin Connect (reads GARMIN_EMAIL / GARMIN_PASSWORD env vars, or prompts)
    Login,
    /// Show authentication status
    Status,
    /// Log out (delete stored tokens)
    Logout,
}

pub async fn run(command: AuthCommands, output: &Output) -> Result<()> {
    match command {
        AuthCommands::Login => login(output).await,
        AuthCommands::Status => status(output),
        AuthCommands::Logout => logout(output),
    }
}

async fn login(output: &Output) -> Result<()> {
    let email = env::var("GARMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .map_or_else(|| prompt("Email"), Ok)?;
    let pass = env::var("GARMIN_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .map_or_else(prompt_password, Ok)?;

    output.status("Logging in to Garmin Connect...");
    let tokens = auth::login(&email, &pass).await?;

    let expires = DateTime::from_timestamp(tokens.oauth2.expires_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "unknown".into());

    output.success(&format!("Logged in. Token expires {expires}"));
    Ok(())
}

fn prompt(label: &str) -> std::io::Result<String> {
    eprint!("{label}: ");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_password() -> std::io::Result<String> {
    rpassword::prompt_password("Password: ")
}

fn status(output: &Output) -> Result<()> {
    let tokens = Tokens::load()?;
    let expired = tokens.oauth2.is_expired();
    let expires = DateTime::from_timestamp(tokens.oauth2.expires_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "unknown".into());

    if output.is_json() {
        let status = serde_json::json!({
            "authenticated": true,
            "expired": expired,
            "expires_at": expires,
        });
        output.print_value(&status);
    } else if expired {
        output.status(&format!(
            "Token expired at {expires} (will auto-refresh on next request)"
        ));
    } else {
        output.success(&format!("Authenticated. Token expires {expires}"));
    }
    Ok(())
}

fn logout(output: &Output) -> Result<()> {
    Tokens::delete()?;
    output.success("Logged out. Tokens deleted");
    Ok(())
}
