use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, LABEL_WIDTH, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Badge {
    pub id: i64,
    pub name: String,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earned_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earned_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty_id: Option<i64>,
}

fn badge_from_json(v: &serde_json::Value) -> Badge {
    Badge {
        id: v["badgeId"].as_i64().unwrap_or(0),
        name: v["badgeName"].as_str().unwrap_or("Unknown").into(),
        key: v["badgeKey"].as_str().unwrap_or("").into(),
        earned_date: v["badgeEarnedDate"].as_str().map(Into::into),
        earned_count: v["badgeEarnedNumber"].as_i64(),
        points: v["badgePoints"].as_i64(),
        progress: v["badgeProgressValue"].as_f64(),
        target: v["badgeTargetValue"].as_f64(),
        category_id: v["badgeCategoryId"].as_i64(),
        difficulty_id: v["badgeDifficultyId"].as_i64(),
    }
}

impl HumanReadable for Badge {
    fn print_human(&self) {
        let count_str = self
            .earned_count
            .filter(|&c| c > 1)
            .map(|c| format!(" x{c}"))
            .unwrap_or_default();
        println!("{}{count_str}", self.name.bold());
        if let Some(ref d) = self.earned_date {
            let short = &d[..d.len().min(10)];
            println!("  {:<LABEL_WIDTH$}{short}", "Earned:");
        }
        if let Some(pts) = self.points {
            println!("  {:<LABEL_WIDTH$}{pts}", "Points:");
        }
        if let (Some(prog), Some(target)) = (self.progress, self.target)
            && target > 0.0
        {
            println!("  {:<LABEL_WIDTH$}{:.0} / {:.0}", "Progress:", prog, target);
        }
    }
}

pub async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client.get_json("/badge-service/badge/earned").await?;

    let badges: Vec<Badge> = v
        .as_array()
        .map(|arr| arr.iter().map(badge_from_json).collect())
        .unwrap_or_default();

    output.print_list(&badges, "Badges");
    Ok(())
}
