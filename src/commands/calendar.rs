use super::output::Output;
use crate::error::Result;
use crate::garmin::{CalendarItem, GarminClient};
use chrono::{Datelike, NaiveDate};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CalendarCommands {
    /// List scheduled workouts and activities
    List {
        /// Year (defaults to current)
        #[arg(long)]
        year: Option<u32>,
        /// Month (1-12, defaults to current)
        #[arg(long)]
        month: Option<u32>,
        /// Show next N weeks (spans months automatically)
        #[arg(long)]
        weeks: Option<u32>,
    },
    /// Remove a scheduled workout from the calendar
    Delete {
        /// Calendar entry ID
        id: u64,
    },
}

pub async fn run(command: CalendarCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        CalendarCommands::List { year, month, weeks } => list(&client, output, year, month, weeks).await,
        CalendarCommands::Delete { id } => delete(&client, output, id).await,
    }
}

async fn list(
    client: &GarminClient,
    output: &Output,
    year: Option<u32>,
    month: Option<u32>,
    weeks: Option<u32>,
) -> Result<()> {
    let now = chrono::Local::now();
    let year = year.unwrap_or(now.year() as u32);
    let month = month.unwrap_or(now.month());

    if let Some(w) = weeks {
        let start = NaiveDate::from_ymd_opt(year as i32, month, now.day()).unwrap_or(now.date_naive());
        let end = start + chrono::Duration::weeks(w as i64);

        let mut all_items: Vec<CalendarItem> = Vec::new();
        let mut cur_year = year;
        let mut cur_month = month;

        loop {
            all_items.extend(client.calendar_month(cur_year, cur_month).await?);
            let last_day = NaiveDate::from_ymd_opt(cur_year as i32, cur_month, 28).unwrap_or(now.date_naive());
            if last_day >= end {
                break;
            }

            // Next month
            (cur_year, cur_month) = if cur_month >= 12 {
                (cur_year + 1, 1)
            } else {
                (cur_year, cur_month + 1)
            };
        }

        // Filter to date range and deduplicate; adaptive workouts share IDs across days.
        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();
        let mut seen = std::collections::HashSet::new();
        let items: Vec<CalendarItem> = all_items
            .into_iter()
            .filter(|item| {
                if !seen.insert((item.id, item.title.clone())) {
                    return false;
                }
                item.date
                    .as_deref()
                    .is_some_and(|d| d >= start_str.as_str() && d <= end_str.as_str())
            })
            .collect();

        output.print_list(&items, &format!("Calendar {start_str} to {end_str}"));
    } else {
        let items = client.calendar_month(year, month).await?;
        output.print_list(&items, &format!("Calendar {year}-{month:02}"));
    }

    Ok(())
}

async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    client.delete_calendar_entry(id).await?;
    output.print_value(&serde_json::json!({
        "calendarEntryId": id,
        "deleted": true,
    }));
    Ok(())
}
