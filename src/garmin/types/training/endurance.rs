use super::labels::classification_label;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::deser_string_or_int;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EnduranceScoreRaw {
    #[serde(default)]
    pub calendar_date: String,
    pub overall_score: Option<i64>,
    #[serde(alias = "classification")]
    pub classification_id: Option<i64>,
    /// API sometimes returns an integer phrase ID instead of a string.
    #[serde(default, deserialize_with = "deser_string_or_int")]
    pub feedback_phrase: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct EnduranceScore {
    pub date: String,
    pub score: Option<i64>,
    pub classification: Option<String>,
    pub feedback: Option<String>,
}

impl From<EnduranceScoreRaw> for EnduranceScore {
    fn from(r: EnduranceScoreRaw) -> Self {
        Self {
            date: r.calendar_date,
            score: r.overall_score,
            classification: r.classification_id.map(|c| classification_label(c).into()),
            feedback: r.feedback_phrase,
        }
    }
}

impl HumanReadable for EnduranceScore {
    fn print_human(&self) {
        println!("{}", self.date.bold());
        let score = self.score.map(|s| s.to_string()).unwrap_or_else(|| "\u{2013}".into());
        let class = self.classification.as_deref().unwrap_or("?");
        println!("  {:<LABEL_WIDTH$}{} ({})", "Score:", score.cyan(), class);
        if let Some(ref fb) = self.feedback {
            println!("  {:<LABEL_WIDTH$}{fb}", "Feedback:");
        }
    }
}
