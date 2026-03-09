use crate::auth::{self, Tokens};
use crate::error::Result;
use crate::output::Output;

pub async fn login(
    output: &Output,
    username: Option<String>,
    password: Option<String>,
) -> Result<()> {
    let email = username
        .or_else(|| std::env::var("GARMIN_EMAIL").ok())
        .ok_or_else(|| {
            crate::error::Error::Auth("Email required (--username or GARMIN_EMAIL)".into())
        })?;
    let pass = password
        .or_else(|| std::env::var("GARMIN_PASSWORD").ok())
        .ok_or_else(|| {
            crate::error::Error::Auth("Password required (--password or GARMIN_PASSWORD)".into())
        })?;

    output.status("Logging in to Garmin Connect...");
    let tokens = auth::login(&email, &pass).await?;

    let expires = chrono::DateTime::from_timestamp(tokens.oauth2.expires_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "unknown".into());

    output.success(&format!("Logged in. Token expires {expires}"));
    Ok(())
}

pub fn status(output: &Output) -> Result<()> {
    let tokens = Tokens::load()?;
    let expired = tokens.oauth2.is_expired();
    let expires = chrono::DateTime::from_timestamp(tokens.oauth2.expires_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "unknown".into());

    if output.is_json() {
        let status = serde_json::json!({
            "authenticated": true,
            "expired": expired,
            "expires_at": expires,
        });
        println!("{}", serde_json::to_string_pretty(&status).unwrap());
    } else if expired {
        output.status(&format!(
            "Token expired at {expires} (will auto-refresh on next request)"
        ));
    } else {
        output.success(&format!("Authenticated. Token expires {expires}"));
    }
    Ok(())
}

pub fn logout(output: &Output) -> Result<()> {
    Tokens::delete()?;
    output.success("Logged out. Tokens deleted.");
    Ok(())
}
