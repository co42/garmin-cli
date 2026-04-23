pub(super) fn fitness_trend_label(code: i64) -> &'static str {
    // Verified by correlating `fitnessTrend` with `trainingStatusFeedbackPhrase`
    // across ~6 historical days: 1 ↔ UNPRODUCTIVE, 2 ↔ MAINTAINING/RECOVERY,
    // 3 ↔ PRODUCTIVE.
    match code {
        1 => "declining",
        2 => "stable",
        3 => "improving",
        _ => "unknown",
    }
}

pub(super) fn classification_label(code: i64) -> &'static str {
    match code {
        1 => "Base",
        2 => "Intermediate",
        3 => "Trained",
        4 => "Well-Trained",
        5 => "Expert",
        6 => "Superior",
        7 => "Elite",
        _ => "Unknown",
    }
}
