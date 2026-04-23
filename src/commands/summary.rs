use super::helpers::{DateRangeArgs, fetch_range};
use super::output::Output;
use crate::error::Result;
use crate::garmin::{DailySummary, GarminClient};

pub async fn run(range: DateRangeArgs, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    let (start, end) = range.resolve(1)?;

    let mut summaries: Vec<DailySummary> = fetch_range(start, end, |ds| {
        let client = &client;
        async move {
            let mut s = client.daily_summary(&ds).await?;
            // API sometimes returns empty `calendarDate`; prefer the requested date.
            s.calendar_date = ds;
            Ok(s)
        }
    })
    .await?;
    summaries.sort_by(|a, b| b.calendar_date.cmp(&a.calendar_date));
    output.print_list(&summaries, "Daily Summary");
    Ok(())
}
