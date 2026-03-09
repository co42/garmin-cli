use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let display_name = client.display_name().await?;
    let path = format!("/personalrecord-service/personalrecord/prs/{display_name}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}
