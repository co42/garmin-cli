use super::output::Output;
use crate::error::{Error, Result};
use crate::garmin::GarminClient;

pub async fn run(path: &str, method: &str, data: Option<&str>, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    let method = method
        .parse::<reqwest::Method>()
        .map_err(|e| Error::Usage(format!("invalid HTTP method: {e}")))?;

    let body = match data {
        Some(d) => Some(serde_json::from_str::<serde_json::Value>(d)?),
        None => None,
    };

    let result = client.raw_request(method, path, body.as_ref()).await?;
    output.print_value(&result);
    Ok(())
}
