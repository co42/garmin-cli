use crate::error::{Error, Result};
use crate::garmin::auth::Tokens;
use chrono::NaiveDate;
use clap::Args;

pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| Error::Usage(format!("invalid date: {e}")))
}

pub(crate) fn require_auth() -> Result<Tokens> {
    Tokens::load()
}

/// Shared date-range selector. `--days` and `--from` are mutually exclusive.
/// Semantics:
/// - `--days N` → last N days ending today
/// - `--from A [--to B]` → explicit range (B defaults to today)
/// - nothing → `default_days` (or None for commands using `resolve_optional`)
#[derive(Args, Debug, Clone)]
pub struct DateRangeArgs {
    /// Number of days ending today (inclusive)
    #[arg(long, group = "date_selector")]
    pub days: Option<u32>,
    /// Start of date range (YYYY-MM-DD)
    #[arg(long, group = "date_selector")]
    pub from: Option<String>,
    /// End of date range (YYYY-MM-DD, defaults to today)
    #[arg(long, requires = "from")]
    pub to: Option<String>,
}

impl DateRangeArgs {
    pub fn is_empty(&self) -> bool {
        self.days.is_none() && self.from.is_none() && self.to.is_none()
    }

    /// Variant for commands where "no flags" means "no filter". Returns None
    /// when nothing was specified; otherwise resolves with `default_days = 1`.
    pub fn resolve_optional(self) -> Result<Option<(NaiveDate, NaiveDate)>> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.resolve(1).map(Some)
        }
    }

    /// Resolve the flags into a concrete `(start, end)` range.
    /// `default_days` is used when neither `--days` nor `--from` is given.
    pub fn resolve(self, default_days: u32) -> Result<(NaiveDate, NaiveDate)> {
        if let Some(from_str) = self.from {
            let start = parse_date(&from_str)?;
            let end = match self.to {
                Some(s) => parse_date(&s)?,
                None => today(),
            };
            if end < start {
                return Err(Error::Usage("--from must be before --to".into()));
            }
            Ok((start, end))
        } else {
            let days = self.days.unwrap_or(default_days);
            if days == 0 {
                return Err(Error::Usage("--days must be >= 1".into()));
            }
            let end = today();
            let start = end - chrono::Duration::days(days as i64 - 1);
            Ok((start, end))
        }
    }
}

/// `YYYY-MM-DD` for each day in `[start, end]`, inclusive.
pub fn range_dates(start: NaiveDate, end: NaiveDate) -> Vec<String> {
    let mut d = start;
    let mut out = Vec::new();
    while d <= end {
        out.push(d.format("%Y-%m-%d").to_string());
        d += chrono::Duration::days(1);
    }
    out
}

/// Max concurrent Garmin requests when fanning out per-day fetches. Low enough
/// to stay well under any rate-limit for multi-month windows.
const FETCH_RANGE_CONCURRENCY: usize = 8;

/// Fan out one fetch per day in `[start, end]`, bounded to a small concurrency
/// window, collecting in input order.
pub async fn fetch_range<T, Fut, Fetch>(start: NaiveDate, end: NaiveDate, fetch: Fetch) -> Result<Vec<T>>
where
    Fetch: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    use futures::StreamExt;
    futures::stream::iter(range_dates(start, end).into_iter().map(fetch))
        .buffered(FETCH_RANGE_CONCURRENCY)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}
