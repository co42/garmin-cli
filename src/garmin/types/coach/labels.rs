/// Map `TARGET_EVENT_DAY` to a terser `RACE` for display; pass other phases
/// through unchanged (they're already short enums like `BUILD` / `PEAK`).
pub(super) fn phase_display_label(phase: &str) -> &str {
    match phase {
        "TARGET_EVENT_DAY" => "RACE",
        other => other,
    }
}

/// Map Garmin supplemental-sport enums to a friendly label. Falls back to
/// title-casing the raw value for anything we haven't mapped yet.
pub(super) fn supplemental_label(key: &str) -> String {
    match key {
        "STRENGTH_TRAINING_BODYWEIGHT" => "Strength (bodyweight)".into(),
        "STRENGTH_TRAINING" => "Strength".into(),
        "YOGA" => "Yoga".into(),
        other => title_case_enum(other),
    }
}

/// Map an `eventRacePredictionsFeedbackPhrase` value to a friendly label.
/// Known mappings from the fixture; unknowns fall back to title-casing and
/// stripping trailing digit suffixes (e.g. `IMPROVE_LONG_TERM_MILEAGE_0` →
/// `Improve long term mileage`). The raw enum is preserved separately in the
/// human output via a dimmed suffix, so callers get both.
pub(super) fn feedback_phrase_label(phrase: &str) -> String {
    match phrase {
        "IMPROVED_VO2MAX" => "Improved VO2 Max".into(),
        "IMPROVE_LONG_TERM_MILEAGE_0" | "IMPROVE_LONG_TERM_MILEAGE_1" | "IMPROVE_LONG_TERM_MILEAGE_2" => {
            "Improve long mileage".into()
        }
        other => title_case_enum(strip_trailing_digit(other)),
    }
}

/// Map a sport-type key to a human label. Known values from fixtures; falls
/// back to title-casing.
pub(super) fn sport_type_label(key: &str) -> String {
    match key {
        "running" => "Running".into(),
        "trail_running" => "Trail Run".into(),
        "cycling" => "Cycling".into(),
        "swimming" => "Swimming".into(),
        "strength_training" => "Strength".into(),
        other => title_case_enum(other),
    }
}

fn strip_trailing_digit(s: &str) -> &str {
    match s.rsplit_once('_') {
        Some((prefix, suffix)) if suffix.chars().all(|c| c.is_ascii_digit()) => prefix,
        _ => s,
    }
}

/// Title-case an enum-like key. Splits on `_`, lowercases each word, and
/// capitalises the first letter of the first word.
fn title_case_enum(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, word) in s.split('_').enumerate() {
        if word.is_empty() {
            continue;
        }
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            if i == 0 {
                out.extend(first.to_uppercase());
            } else {
                out.push(first.to_ascii_lowercase());
            }
            out.extend(chars.map(|c| c.to_ascii_lowercase()));
        }
    }
    out
}
