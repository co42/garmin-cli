use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn parse_date(s: &str) -> std::result::Result<chrono::NaiveDate, crate::error::Error> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| crate::error::Error::Api(format!("Invalid date: {e}")))
}

fn print_value(v: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}

async fn fetch_multi(
    client: &GarminClient,
    _output: &Output,
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
        print_value(&v);
    } else {
        let mut results = Vec::new();
        for i in (0..days).rev() {
            let d = end - chrono::Duration::days(i as i64);
            let date_str = d.format("%Y-%m-%d").to_string();
            let path = path_fn(&date_str);
            let v: serde_json::Value = client.get_json(&path).await?;
            results.push(v);
        }
        print_value(&serde_json::Value::Array(results));
    }
    Ok(())
}

pub async fn status(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_multi(client, output, date, days, |d| {
        format!("/metrics-service/metrics/trainingstatus/aggregated/{d}")
    })
    .await
}

pub async fn readiness(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    fetch_multi(client, output, date, days, |d| {
        format!("/metrics-service/metrics/trainingreadiness/{d}")
    })
    .await
}

pub async fn race_predictions(client: &GarminClient, _output: &Output) -> Result<()> {
    let display_name = client.display_name().await?;
    let path = format!("/metrics-service/metrics/racepredictions/latest/{display_name}");
    let v: serde_json::Value = client.get_json(&path).await?;
    print_value(&v);
    Ok(())
}

pub async fn endurance_score(
    client: &GarminClient,
    _output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    if days == 1 {
        let path = format!("/metrics-service/metrics/endurancescore?calendarDate={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        print_value(&v);
    } else {
        let end = parse_date(&end_date)?;
        let start = end - chrono::Duration::days(days as i64 - 1);
        let start_str = start.format("%Y-%m-%d").to_string();
        let path = format!(
            "/metrics-service/metrics/endurancescore/stats?startDate={start_str}&endDate={end_date}&aggregation=daily"
        );
        let v: serde_json::Value = client.get_json(&path).await?;
        print_value(&v);
    }
    Ok(())
}

pub async fn hill_score(
    client: &GarminClient,
    _output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    let end_date = date.map(String::from).unwrap_or_else(today);
    let days = days.unwrap_or(1);
    if days == 1 {
        let path = format!("/metrics-service/metrics/hillscore?calendarDate={end_date}");
        let v: serde_json::Value = client.get_json(&path).await?;
        print_value(&v);
    } else {
        let end = parse_date(&end_date)?;
        let start = end - chrono::Duration::days(days as i64 - 1);
        let start_str = start.format("%Y-%m-%d").to_string();
        let path = format!(
            "/metrics-service/metrics/hillscore/stats?startDate={start_str}&endDate={end_date}&aggregation=daily"
        );
        let v: serde_json::Value = client.get_json(&path).await?;
        print_value(&v);
    }
    Ok(())
}

pub async fn fitness_age(
    client: &GarminClient,
    _output: &Output,
    date: Option<&str>,
) -> Result<()> {
    let date_str = date.map(String::from).unwrap_or_else(today);
    let path = format!("/fitnessage-service/fitnessage/{date_str}");
    let v: serde_json::Value = client.get_json(&path).await?;
    print_value(&v);
    Ok(())
}

pub async fn lactate_threshold(client: &GarminClient, _output: &Output) -> Result<()> {
    let v: serde_json::Value = client
        .get_json("/biometric-service/biometric/latestLactateThreshold")
        .await?;
    print_value(&v);
    Ok(())
}

pub async fn scores(
    client: &GarminClient,
    output: &Output,
    date: Option<&str>,
    days: Option<u32>,
) -> Result<()> {
    // maxmet needs a date range; for single day use a 7-day window to ensure data
    let end_date = date.map(String::from).unwrap_or_else(today);
    let end = parse_date(&end_date)?;
    let days = days.unwrap_or(7);
    let start = end - chrono::Duration::days(days as i64 - 1);
    let start_str = start.format("%Y-%m-%d").to_string();
    let path = format!("/metrics-service/metrics/maxmet/daily/{start_str}/{end_date}");
    let v: serde_json::Value = client.get_json(&path).await?;
    let _ = output;
    print_value(&v);
    Ok(())
}
