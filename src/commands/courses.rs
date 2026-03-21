use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client.get_json("/course-service/course").await?;
    output.print_value(&v);
    Ok(())
}

pub async fn get(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/course-service/course/{id}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}
