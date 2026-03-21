use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client.get_json("/badge-service/badge/earned").await?;
    output.print_value(&v);
    Ok(())
}
