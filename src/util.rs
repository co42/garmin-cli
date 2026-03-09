use crate::client::GarminClient;
use crate::error::{Error, Result};
use crate::output::Output;

pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

pub fn parse_date(s: &str) -> std::result::Result<chrono::NaiveDate, Error> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| Error::Api(format!("Invalid date: {e}")))
}

/// Fetch JSON from a date-parameterized endpoint for one or more days.
/// Concurrent when days > 1.
pub async fn fetch_date_range(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
    path_fn: impl Fn(&str) -> String,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    let end = parse_date(&end_date)?;

    if days == 1 {
        let path = path_fn(&end_date);
        let v: serde_json::Value = client.get_json(&path).await?;
        output.print_value(&v);
    } else {
        let futs: Vec<_> = (0..days)
            .rev()
            .map(|i| {
                let d = end - chrono::Duration::days(i as i64);
                let path = path_fn(&d.format("%Y-%m-%d").to_string());
                async move { client.get_json::<serde_json::Value>(&path).await }
            })
            .collect();
        let results: Vec<serde_json::Value> = futures::future::join_all(futs)
            .await
            .into_iter()
            .collect::<Result<_>>()?;
        output.print_value(&serde_json::Value::Array(results));
    }
    Ok(())
}
