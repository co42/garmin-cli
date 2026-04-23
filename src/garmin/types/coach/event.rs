use super::labels::feedback_phrase_label;
use super::projection::EventProjection;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{fmt_hms, fmt_pace_per_km, pace_from_speed};
use chrono::NaiveDate;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TargetEvent {
    pub id: u64,
    pub event_name: String,
    pub date: String,
    pub event_type: Option<String>,
    pub event_time_local: Option<EventTime>,
    pub location: Option<String>,
    pub completion_target: Option<UnitValue>,
    pub event_customization: Option<EventCustomization>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventTime {
    pub start_time_hh_mm: String,
    pub time_zone_id: String,
}

/// Used for both `completionTarget` (distance, km/mi) and `customGoal` (time,
/// always seconds). The `unit` field disambiguates; don't assume.
#[derive(Debug, Serialize, Deserialize)]
pub struct UnitValue {
    pub value: f64,
    pub unit: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventCustomization {
    pub custom_goal: Option<UnitValue>,
    #[serde(default)]
    pub is_primary_event: bool,
    pub training_plan_id: Option<u64>,
    pub training_plan_type: Option<String>,
    #[serde(rename(deserialize = "projectedRaceTimeDurationSeconds"))]
    pub projected_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "predictedRaceTimeDurationSeconds"))]
    pub predicted_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "projectedRaceSpeed"))]
    pub projected_race_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "predictedRaceSpeed"))]
    pub predicted_race_speed_mps: Option<f64>,
    pub enrollment_time: Option<String>,
}

/// Aggregate rendered by `coach event`. JSON shape is stable across modes:
/// `projections` has 1 entry for the today-snapshot, N for history.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct CoachEvent {
    pub event: TargetEvent,
    pub plan_id: Option<u64>,
    pub plan_name: Option<String>,
    /// Sorted descending by `calendar_date` (newest first).
    pub projections: Vec<EventProjection>,
}

impl HumanReadable for CoachEvent {
    fn print_human(&self) {
        if self.projections.len() >= 2 {
            print_history(self);
        } else {
            print_snapshot(self);
        }
    }
}

fn print_snapshot(ce: &CoachEvent) {
    print_header(ce, true);
    println!();
    let today = ce.projections.first();
    print_today_block(today, ce.event.event_customization.as_ref());
}

fn print_history(ce: &CoachEvent) {
    print_header(ce, false);
    println!();
    println!("  {}", "Projection history".bold());
    println!("    Date          Projection   Pace       Band               Focus");
    for p in &ce.projections {
        print_history_row(p);
    }
    if let Some(trend) = compute_trend(&ce.projections) {
        println!();
        println!("  {:<LABEL_WIDTH$}{}", "Trend:", trend);
    }
}

fn print_header(ce: &CoachEvent, with_details: bool) {
    // Title line: event name + dimmed "target event" suffix.
    println!("{}  {}", ce.event.event_name.bold(), "target event".dimmed());
    println!("{}", "\u{2500}".repeat(40).dimmed());

    let when = format_when(&ce.event, with_details);
    if !when.is_empty() {
        println!("  {:<LABEL_WIDTH$}{when}", "When:");
    }
    if with_details && let Some(ref loc) = ce.event.location {
        println!("  {:<LABEL_WIDTH$}{loc}", "Where:");
    }
    if let Some(ref ct) = ce.event.completion_target {
        println!("  {:<LABEL_WIDTH$}{}", "Distance:", format_distance(ct));
    }
    let goal = ce
        .event
        .event_customization
        .as_ref()
        .and_then(|c| c.custom_goal.as_ref());
    if let (Some(goal), Some(ct)) = (goal, ce.event.completion_target.as_ref()) {
        println!("  {:<LABEL_WIDTH$}{}", "Goal:", format_goal(goal, ct));
    }
    if with_details && (ce.plan_id.is_some() || ce.plan_name.is_some()) {
        let name = ce.plan_name.as_deref().unwrap_or("\u{2014}");
        let id_part = ce.plan_id.map(|id| format!(" (#{id})")).unwrap_or_default();
        println!("  {:<LABEL_WIDTH$}{name}{id_part}", "Plan:");
    }
}

fn print_today_block(proj: Option<&EventProjection>, cust: Option<&EventCustomization>) {
    let today_label = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    println!("  {}", format!("Today ({today_label})").bold());
    let Some(p) = proj else {
        println!("    {}", "(no projection yet)".dimmed());
        return;
    };
    let goal_seconds = cust
        .and_then(|c| c.custom_goal.as_ref())
        .filter(|g| g.unit == "second")
        .map(|g| g.value);

    if let Some(secs) = p.projection_race_time_seconds {
        let pace = p
            .speed_projection_mps
            .filter(|&s| s > 0.0)
            .map(pace_from_speed)
            .unwrap_or_default();
        let delta = goal_seconds
            .map(|g| {
                let diff = secs - g;
                format!("    {:+}s vs goal", diff.round() as i64)
            })
            .unwrap_or_default();
        println!(
            "    {:<LABEL_WIDTH$}{}   {pace}{delta}",
            "Projection:",
            fmt_hms(secs).cyan()
        );
    }
    if let Some(secs) = p.predicted_race_time_seconds {
        let pace = p
            .speed_prediction_mps
            .filter(|&s| s > 0.0)
            .map(pace_from_speed)
            .unwrap_or_default();
        println!("    {:<LABEL_WIDTH$}{}   {pace}", "Prediction:", fmt_hms(secs));
    }
    if let (Some(lower), Some(upper)) = (
        p.lower_bound_projection_race_time_seconds,
        p.upper_bound_projection_race_time_seconds,
    ) {
        println!(
            "    {:<LABEL_WIDTH$}{} \u{2013} {}",
            "Band:",
            fmt_hms(lower),
            fmt_hms(upper)
        );
    }
    if let Some(ref phrase) = p.event_race_predictions_feedback_phrase {
        let label = feedback_phrase_label(phrase);
        println!(
            "    {:<LABEL_WIDTH$}{label} {}",
            "Focus:",
            format!("({phrase})").dimmed()
        );
    }
}

fn print_history_row(p: &EventProjection) {
    let proj = p
        .projection_race_time_seconds
        .map(fmt_hms)
        .unwrap_or_else(|| "\u{2014}".into());
    let pace = p
        .speed_projection_mps
        .filter(|&s| s > 0.0)
        .map(pace_from_speed)
        .unwrap_or_else(|| "\u{2014}".into());
    let band = match (
        p.lower_bound_projection_race_time_seconds,
        p.upper_bound_projection_race_time_seconds,
    ) {
        (Some(l), Some(u)) => format!("{}\u{2013}{}", fmt_hms(l), fmt_hms(u)),
        _ => "\u{2014}".into(),
    };
    let focus = p
        .event_race_predictions_feedback_phrase
        .as_deref()
        .map(|ph| format!("{} {}", feedback_phrase_label(ph), format!("({ph})").dimmed()))
        .unwrap_or_default();
    println!(
        "    {date:<14}{proj:<13}{pace:<11}{band:<19}{focus}",
        date = p.calendar_date,
    );
}

/// Sign-prefixed `fmt_hms` of (oldest − newest), plus a text cue. Returns
/// None when fewer than two usable projections are available.
fn compute_trend(projections: &[EventProjection]) -> Option<String> {
    let usable: Vec<&EventProjection> = projections
        .iter()
        .filter(|p| p.projection_race_time_seconds.is_some())
        .collect();
    if usable.len() < 2 {
        return None;
    }
    // `projections` is sorted descending; first = newest, last = oldest.
    let newest = usable.first()?;
    let oldest = usable.last()?;
    let newest_secs = newest.projection_race_time_seconds?;
    let oldest_secs = oldest.projection_race_time_seconds?;
    let diff = oldest_secs - newest_secs;
    let span_days = parse_date(&newest.calendar_date)?
        .signed_duration_since(parse_date(&oldest.calendar_date)?)
        .num_days();
    let cue = match diff {
        d if d.abs() < 3.0 => "flat",
        d if d > 0.0 => "projection improving",
        _ => "regressing",
    };
    let prefix = if diff > 0.0 { "\u{2212}" } else { "+" };
    Some(format!("{prefix}{} over {span_days} days ({cue})", fmt_hms(diff.abs())))
}

fn format_when(event: &TargetEvent, with_time: bool) -> String {
    let Some(date) = parse_date(&event.date) else {
        return event.date.clone();
    };
    let weekday = date.format("%a").to_string();
    let iso = date.format("%Y-%m-%d").to_string();
    let today = chrono::Local::now().date_naive();
    let delta = date.signed_duration_since(today).num_days();
    let rel = match delta {
        0 => "(today)".to_string(),
        d if d > 0 => format!("(in {d} days)"),
        d => format!("({}d ago)", -d),
    };
    let time_part = if with_time {
        event
            .event_time_local
            .as_ref()
            .map(|t| format!("  {}  {}", t.start_time_hh_mm, t.time_zone_id))
            .unwrap_or_default()
    } else {
        String::new()
    };
    format!("{weekday} {iso}{time_part}   {rel}")
}

fn format_distance(ct: &UnitValue) -> String {
    match ct.unit.as_str() {
        "kilometer" => format!("{:.2} km", ct.value),
        "mile" => format!("{:.2} mi", ct.value),
        other => format!("{} {other}", ct.value),
    }
}

fn format_goal(goal: &UnitValue, completion: &UnitValue) -> String {
    if goal.unit != "second" {
        return format!("{} {}", goal.value, goal.unit);
    }
    let hms = fmt_hms(goal.value);
    let pace = match completion.unit.as_str() {
        "kilometer" if completion.value > 0.0 => {
            let secs_per_km = goal.value / completion.value;
            format!("   ({}/km)", fmt_pace_per_km(secs_per_km).trim_end_matches(" /km"))
        }
        "mile" if completion.value > 0.0 => {
            let secs_per_mi = goal.value / completion.value;
            format!("   ({}/mi)", fmt_pace_per_km(secs_per_mi).trim_end_matches(" /km"))
        }
        _ => String::new(),
    };
    format!("{hms}{pace}")
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}
