use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn api(
    client: &GarminClient,
    output: &Output,
    path: &str,
    method: &str,
    data: Option<&str>,
) -> Result<()> {
    let method = method
        .parse::<reqwest::Method>()
        .map_err(|e| crate::error::Error::Api(format!("Invalid method: {e}")))?;

    let body = match data {
        Some(d) => Some(serde_json::from_str::<serde_json::Value>(d)?),
        None => None,
    };

    let result = client.request(method, path, body.as_ref()).await?;
    output.print_value(&result);
    Ok(())
}
