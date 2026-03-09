use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn month(
    client: &GarminClient,
    _output: &Output,
    year: Option<u32>,
    month: Option<u32>,
) -> Result<()> {
    let now = chrono::Local::now();
    let y = year.unwrap_or(now.format("%Y").to_string().parse().unwrap());
    let m = month.unwrap_or(now.format("%-m").to_string().parse().unwrap());
    let path = format!("/calendar-service/year/{y}/month/{m}");
    let v: serde_json::Value = client.get_json(&path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}
