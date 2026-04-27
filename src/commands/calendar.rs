use super::helpers::{DateRangeArgs, today};
use super::output::Output;
use crate::error::Result;
use crate::garmin::{CalendarItem, GarminClient, TargetEvent};
use chrono::{Datelike, NaiveDate};
use clap::Subcommand;

/// Default `limit` for `calendar events`. Mirrors the value Garmin's web UI uses
/// when listing upcoming events; enough headroom for any reasonable race calendar.
const DEFAULT_EVENTS_LIMIT: u32 = 20;

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
    /// List upcoming events (races, primary plan event, scheduled events)
    Events {
        #[command(flatten)]
        range: DateRangeArgs,
        /// Maximum number of events to return
        #[arg(long, default_value_t = DEFAULT_EVENTS_LIMIT)]
        limit: u32,
        /// Include past events (drops the default `startDate=today` filter)
        #[arg(long)]
        include_past: bool,
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
        CalendarCommands::Events {
            range,
            limit,
            include_past,
        } => events(&client, output, range, limit, include_past).await,
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

async fn events(
    client: &GarminClient,
    output: &Output,
    range: DateRangeArgs,
    limit: u32,
    include_past: bool,
) -> Result<()> {
    // Resolve the window. With no flags: upcoming from today, no end filter.
    // With flags: explicit window; we send `startDate` server-side and trim
    // `> end` client-side (the API only supports startDate).
    // `--include-past` drops the start filter entirely.
    let (start, end): (Option<NaiveDate>, Option<NaiveDate>) = match range.resolve_optional()? {
        Some((s, e)) => (Some(s), Some(e)),
        None => (Some(today()), None),
    };

    let api_start = if include_past { None } else { start };
    let mut items: Vec<TargetEvent> = client.list_events(None, api_start, Some(limit)).await?;

    if let Some(end_d) = end {
        let end_str = end_d.format("%Y-%m-%d").to_string();
        items.retain(|e| e.date.as_str() <= end_str.as_str());
    }

    let title = match (start, end) {
        (Some(s), Some(e)) => format!("Events {} to {}", ymd(s), ymd(e)),
        (Some(s), None) if !include_past => format!("Events from {}", ymd(s)),
        _ => "Events".to_string(),
    };
    output.print_list(&items, &title);
    Ok(())
}

fn ymd(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    client.delete_calendar_entry(id).await?;
    output.print_value(&serde_json::json!({
        "calendarEntryId": id,
        "deleted": true,
    }));
    Ok(())
}
