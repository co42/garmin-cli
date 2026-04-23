//! Shared helpers for type modules: serde defaults, format functions, and
//! custom deserializers. Keep domain-specific helpers (exercise naming,
//! record label translation, etc.) in the owning module.

use serde::Deserialize;

// ─── serde `default = "..."` helpers ─────────────────────────────────

pub fn untitled() -> String {
    "Untitled".into()
}

pub fn unknown() -> String {
    "Unknown".into()
}

pub fn unknown_key() -> String {
    "unknown".into()
}

// ─── Duration / time formatters ──────────────────────────────────────

/// Format seconds as "H:MM:SS" (if hours > 0) or "M:SS".
pub fn fmt_hms(secs: f64) -> String {
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// Format integer seconds as "Xh YYm" or "Ym" — sleep-style.
pub fn fmt_hm(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m")
    }
}

/// Unix milliseconds → local "HH:MM".
pub fn fmt_local_time(ts_ms: Option<i64>) -> Option<String> {
    ts_ms.and_then(|ms| chrono::DateTime::from_timestamp(ms / 1000, 0).map(|dt| dt.format("%H:%M").to_string()))
}

// ─── Distance / pace formatters ──────────────────────────────────────

/// Optional meters → "X.XX km" or em-dash.
pub fn fmt_dist(m: Option<f64>) -> String {
    m.map(|v| format!("{:.2} km", v / 1000.0))
        .unwrap_or_else(|| "\u{2014}".into())
}

/// Format pace as "M:SS /km" from seconds-per-kilometer.
pub fn fmt_pace_per_km(secs_per_km: f64) -> String {
    let total = secs_per_km.round() as u64;
    format!("{}:{:02} /km", total / 60, total % 60)
}

/// Format pace as "M:SS /km" from (distance, duration). Returns None if either
/// is invalid.
pub fn compute_pace(distance_meters: Option<f64>, duration_seconds: f64) -> Option<String> {
    let d = distance_meters?;
    if d <= 0.0 || duration_seconds <= 0.0 {
        return None;
    }
    Some(fmt_pace_per_km(duration_seconds / (d / 1000.0)))
}

/// Format pace as "M:SS /km" from speed (m/s). Used for LT speed, workout
/// pace targets.
pub fn pace_from_speed(speed_ms: f64) -> String {
    fmt_pace_per_km(1000.0 / speed_ms)
}

/// API returns lactate-threshold speed exactly 10× too low (e.g. 0.388 for a
/// 3.88 m/s threshold). Always multiply — applied at the deserialize boundary
/// via `deser_lt_speed`.
pub fn correct_lt_speed(s: f64) -> f64 {
    s * 10.0
}

// ─── Custom deserializers ────────────────────────────────────────────

/// "2026-03-28 09:56:14.0" → "2026-03-28T09:56:14".
pub fn deser_norm_ts<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let s = String::deserialize(d)?;
    Ok(s.replacen(' ', "T", 1).trim_end_matches(".0").to_string())
}

/// Extract `typeKey` from a nested `{"typeKey": "..."}` object.
pub fn deser_type_key<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    struct TypeRef {
        #[serde(rename(deserialize = "typeKey"))]
        type_key: String,
    }
    Ok(TypeRef::deserialize(d)?.type_key)
}

/// Coerce null to 0.
pub fn deser_nullable_u64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    Ok(Option::<u64>::deserialize(d)?.unwrap_or(0))
}

/// HH:MM string OR seconds-from-midnight integer → "HH:MM".
pub fn deser_hhmm<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Str(String),
        Int(i64),
    }
    let v: Option<Raw> = Option::deserialize(d)?;
    Ok(v.map(|r| match r {
        Raw::Str(s) => s,
        Raw::Int(secs) => {
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            format!("{h:02}:{m:02}")
        }
    }))
}

/// Accept either a string or an integer, coerce the integer to its decimal
/// string representation.
pub fn deser_string_or_int<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Str(String),
        Int(i64),
    }
    let v: Option<Raw> = Option::deserialize(d)?;
    Ok(v.map(|r| match r {
        Raw::Str(s) => s,
        Raw::Int(n) => n.to_string(),
    }))
}

/// Fahrenheit → Celsius, rounded to 0.1.
pub fn deser_f_to_c<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    let v: Option<f64> = Option::deserialize(d)?;
    Ok(v.map(|f| ((f - 32.0) * 5.0 / 9.0 * 10.0).round() / 10.0))
}

/// mph → km/h, rounded to 0.1.
pub fn deser_mph_to_kmh<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    let v: Option<f64> = Option::deserialize(d)?;
    Ok(v.map(|m| (m * 1.60934 * 10.0).round() / 10.0))
}

/// Grams → kilograms. Garmin stores body masses in grams; kg is the consumer unit.
pub fn deser_g_to_kg<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    Ok(Option::<f64>::deserialize(d)?.map(|g| g / 1000.0))
}

/// Centimetres → metres. Used by `CalendarItem.distance`.
pub fn deser_cm_to_m<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    Ok(Option::<f64>::deserialize(d)?.map(|cm| cm / 100.0))
}

/// Milliseconds → seconds (as f64). Used by `CalendarItem.duration`.
pub fn deser_ms_to_s<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    Ok(Option::<f64>::deserialize(d)?.map(|ms| ms / 1000.0))
}

/// Applies `correct_lt_speed` (×10) at the deserialize boundary.
pub fn deser_lt_speed<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    Ok(Option::<f64>::deserialize(d)?.map(correct_lt_speed))
}

/// Extract `desc` from `{"desc": "..."}`.
pub fn deser_nested_desc<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Dto {
        desc: Option<String>,
    }
    Ok(Option::<Dto>::deserialize(d)?.and_then(|v| v.desc))
}

/// Extract `name` from `{"name": "..."}`.
pub fn deser_nested_name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Dto {
        name: Option<String>,
    }
    Ok(Option::<Dto>::deserialize(d)?.and_then(|v| v.name))
}
