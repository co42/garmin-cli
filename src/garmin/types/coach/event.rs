use super::labels::feedback_phrase_label;
use super::projection::EventProjection;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{fmt_hms, fmt_pace_per_km, pace_from_speed};
use chrono::NaiveDate;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Flat view of a target event. The Garmin API wraps these fields in three
/// nested DTOs (`eventTimeLocal`, `completionTarget`, `eventCustomization`);
/// the custom Deserialize folds them into a single flat struct so JSON
/// consumers don't have to do it.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct TargetEvent {
    pub id: u64,
    pub name: String,
    pub event_type: Option<String>,
    pub date: String,
    pub start_time_local: Option<String>,
    pub timezone: Option<String>,
    pub location: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub distance_meters: Option<f64>,
    pub goal_seconds: Option<f64>,
    pub predicted_race_time_seconds: Option<f64>,
    pub projected_race_time_seconds: Option<f64>,
    pub predicted_race_speed_mps: Option<f64>,
    pub projected_race_speed_mps: Option<f64>,
    pub is_primary_event: Option<bool>,
    pub is_race: Option<bool>,
    pub is_training_event: Option<bool>,
    pub course_id: Option<u64>,
    pub url: Option<String>,
    pub registration_url: Option<String>,
    pub note: Option<String>,
    pub training_plan_id: Option<u64>,
    pub enrollment_time: Option<String>,
}

impl<'de> Deserialize<'de> for TargetEvent {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all(deserialize = "camelCase"))]
        struct Raw {
            id: u64,
            event_name: String,
            date: String,
            event_type: Option<String>,
            event_time_local: Option<EventTime>,
            location: Option<String>,
            location_start_point: Option<LocationPoint>,
            completion_target: Option<UnitValue>,
            event_customization: Option<EventCustomization>,
            #[serde(default)]
            race: Option<bool>,
            course_id: Option<u64>,
            url: Option<String>,
            registration_url: Option<String>,
            note: Option<String>,
        }
        let r = Raw::deserialize(d)?;
        let cust = r.event_customization;
        // Garmin returns "" for missing notes; collapse to None so JSON output stays clean.
        let note = r.note.filter(|s| !s.is_empty());
        Ok(TargetEvent {
            id: r.id,
            name: r.event_name,
            event_type: r.event_type,
            date: r.date,
            start_time_local: r.event_time_local.as_ref().map(|t| t.start_time_hh_mm.clone()),
            timezone: r.event_time_local.as_ref().map(|t| t.time_zone_id.clone()),
            location: r.location,
            latitude: r.location_start_point.as_ref().map(|p| p.lat),
            longitude: r.location_start_point.as_ref().map(|p| p.lon),
            distance_meters: r.completion_target.as_ref().and_then(unit_value_to_meters),
            goal_seconds: cust
                .as_ref()
                .and_then(|c| c.custom_goal.as_ref())
                .and_then(unit_value_to_seconds),
            predicted_race_time_seconds: cust.as_ref().and_then(|c| c.predicted_race_time_seconds),
            projected_race_time_seconds: cust.as_ref().and_then(|c| c.projected_race_time_seconds),
            predicted_race_speed_mps: cust.as_ref().and_then(|c| c.predicted_race_speed_mps),
            projected_race_speed_mps: cust.as_ref().and_then(|c| c.projected_race_speed_mps),
            is_primary_event: cust.as_ref().map(|c| c.is_primary_event),
            is_race: r.race,
            is_training_event: cust.as_ref().and_then(|c| c.is_training_event),
            course_id: r.course_id,
            url: r.url,
            registration_url: r.registration_url,
            note,
            training_plan_id: cust.as_ref().and_then(|c| c.training_plan_id),
            enrollment_time: cust.as_ref().and_then(|c| c.enrollment_time.clone()),
        })
    }
}

impl HumanReadable for TargetEvent {
    fn print_human(&self) {
        println!("{}  {}", self.date.bold(), self.name);
        let kind = self.event_type.as_deref().unwrap_or("event");
        let race_tag = if self.is_race == Some(true) {
            "  race".red().to_string()
        } else {
            String::new()
        };
        let primary_tag = if self.is_primary_event == Some(true) {
            "  primary".cyan().to_string()
        } else {
            String::new()
        };
        println!("  {:<LABEL_WIDTH$}{kind}{race_tag}{primary_tag}", "Type:");
        if let Some(m) = self.distance_meters {
            println!("  {:<LABEL_WIDTH$}{:.2} km", "Distance:", m / 1000.0);
        }
        let when = format_when(self, true);
        if !when.is_empty() {
            println!("  {:<LABEL_WIDTH$}{when}", "When:");
        }
        if let Some(ref loc) = self.location {
            println!("  {:<LABEL_WIDTH$}{loc}", "Where:");
        }
        if let Some(id) = self.course_id {
            println!("  {:<LABEL_WIDTH$}#{id}", "Course:");
        }
        if let (Some(goal_secs), Some(m)) = (self.goal_seconds, self.distance_meters) {
            println!("  {:<LABEL_WIDTH$}{}", "Goal:", format_goal(goal_secs, m));
        }
        if let Some(ref url) = self.url {
            println!("  {:<LABEL_WIDTH$}{}", "Link:", url.dimmed());
        }
        if let Some(ref note) = self.note {
            println!("  {:<LABEL_WIDTH$}{note}", "Note:");
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct EventTime {
    start_time_hh_mm: String,
    time_zone_id: String,
}

#[derive(Debug, Deserialize)]
struct LocationPoint {
    lat: f64,
    lon: f64,
}

/// Used for both `completionTarget` (distance, km/mi) and `customGoal` (time,
/// always seconds). The `unit` field disambiguates; don't assume.
#[derive(Debug, Deserialize)]
struct UnitValue {
    value: f64,
    unit: String,
}

fn unit_value_to_meters(u: &UnitValue) -> Option<f64> {
    match u.unit.as_str() {
        "kilometer" => Some(u.value * 1000.0),
        "meter" => Some(u.value),
        "mile" => Some(u.value * 1609.344),
        _ => None,
    }
}

fn unit_value_to_seconds(u: &UnitValue) -> Option<f64> {
    (u.unit == "second").then_some(u.value)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct EventCustomization {
    custom_goal: Option<UnitValue>,
    #[serde(default)]
    is_primary_event: bool,
    is_training_event: Option<bool>,
    training_plan_id: Option<u64>,
    #[serde(rename(deserialize = "projectedRaceTimeDurationSeconds"))]
    projected_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "predictedRaceTimeDurationSeconds"))]
    predicted_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "projectedRaceSpeed"))]
    projected_race_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "predictedRaceSpeed"))]
    predicted_race_speed_mps: Option<f64>,
    enrollment_time: Option<String>,
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
    print_today_block(today, ce.event.goal_seconds);
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
    println!("{}  {}", ce.event.name.bold(), "target event".dimmed());
    println!("{}", "\u{2500}".repeat(40).dimmed());

    let when = format_when(&ce.event, with_details);
    if !when.is_empty() {
        println!("  {:<LABEL_WIDTH$}{when}", "When:");
    }
    if with_details && let Some(ref loc) = ce.event.location {
        println!("  {:<LABEL_WIDTH$}{loc}", "Where:");
    }
    if let Some(m) = ce.event.distance_meters {
        println!("  {:<LABEL_WIDTH$}{:.2} km", "Distance:", m / 1000.0);
    }
    if let (Some(goal_secs), Some(m)) = (ce.event.goal_seconds, ce.event.distance_meters) {
        println!("  {:<LABEL_WIDTH$}{}", "Goal:", format_goal(goal_secs, m));
    }
    if with_details && (ce.plan_id.is_some() || ce.plan_name.is_some()) {
        let name = ce.plan_name.as_deref().unwrap_or("\u{2014}");
        let id_part = ce.plan_id.map(|id| format!(" (#{id})")).unwrap_or_default();
        println!("  {:<LABEL_WIDTH$}{name}{id_part}", "Plan:");
    }
}

fn print_today_block(proj: Option<&EventProjection>, goal_seconds: Option<f64>) {
    let today_label = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    println!("  {}", format!("Today ({today_label})").bold());
    let Some(p) = proj else {
        println!("    {}", "(no projection yet)".dimmed());
        return;
    };

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
        match (event.start_time_local.as_deref(), event.timezone.as_deref()) {
            (Some(start), Some(tz)) => format!("  {start}  {tz}"),
            (Some(start), None) => format!("  {start}"),
            _ => String::new(),
        }
    } else {
        String::new()
    };
    format!("{weekday} {iso}{time_part}   {rel}")
}

fn format_goal(goal_seconds: f64, distance_meters: f64) -> String {
    let hms = fmt_hms(goal_seconds);
    if distance_meters <= 0.0 {
        return hms;
    }
    let secs_per_km = goal_seconds / (distance_meters / 1000.0);
    let pace = format!("   ({}/km)", fmt_pace_per_km(secs_per_km).trim_end_matches(" /km"));
    format!("{hms}{pace}")
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}
