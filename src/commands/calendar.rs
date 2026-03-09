use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;
use chrono::Datelike;

pub async fn month(
    client: &GarminClient,
    output: &Output,
    year: Option<u32>,
    month: Option<u32>,
) -> Result<()> {
    let now = chrono::Local::now();
    let y = year.unwrap_or(now.year() as u32);
    let m = month.unwrap_or(now.month());
    let path = format!("/calendar-service/year/{y}/month/{m}");
    let v: serde_json::Value = client.get_json(&path).await?;
    output.print_value(&v);
    Ok(())
}
