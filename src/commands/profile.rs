use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Profile {
    pub display_name: String,
    pub user_name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub measurement_system: Option<String>,
}

impl HumanReadable for Profile {
    fn print_human(&self) {
        println!("{} {}", "Name:".bold(), self.display_name);
        if let Some(ref u) = self.user_name {
            println!("{} {}", "Username:".bold(), u);
        }
        if let Some(ref e) = self.email {
            println!("{} {}", "Email:".bold(), e);
        }
        if let Some(ref l) = self.locale {
            println!("{} {}", "Locale:".bold(), l);
        }
        if let Some(ref m) = self.measurement_system {
            println!("{} {}", "Units:".bold(), m);
        }
    }
}

pub async fn show(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client
        .get_json("/userprofile-service/socialProfile")
        .await?;
    let profile = Profile {
        display_name: v["userProfileFullName"]
            .as_str()
            .or(v["fullName"].as_str())
            .unwrap_or("")
            .into(),
        user_name: v["userName"].as_str().map(Into::into),
        email: None, // not in socialProfile
        locale: None,
        measurement_system: None,
    };
    output.print(&profile);
    Ok(())
}

// ── Settings ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProfileSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lactate_threshold_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max_running: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ftp_cycling: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_goal: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensity_minutes_goal: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wake_time: Option<String>,
}

fn settings_from_json(v: &serde_json::Value) -> ProfileSettings {
    let u = &v["userData"];
    let s = &v["userSleep"];
    ProfileSettings {
        weight_kg: u["weight"].as_f64().map(|w| w / 1000.0),
        height_cm: u["height"].as_f64(),
        birth_date: u["birthDate"].as_str().map(Into::into),
        gender: u["gender"].as_str().map(Into::into),
        activity_level: u["activityLevel"].as_str().map(Into::into),
        max_hr: u["maxHeartRate"].as_i64(),
        resting_hr: u["restingHeartRate"].as_i64(),
        lactate_threshold_hr: u["lactateThresholdHeartRate"].as_i64(),
        vo2max_running: u["vo2MaxRunning"].as_f64(),
        ftp_cycling: u["functionalThresholdPower"].as_f64(),
        step_goal: u["dailyStepGoal"].as_i64(),
        intensity_minutes_goal: u["intensityMinutesGoal"].as_i64(),
        sleep_time: s["sleepTime"].as_str().map(Into::into).or_else(|| {
            s["sleepTime"].as_i64().map(|t| {
                let h = t / 3600;
                let m = (t % 3600) / 60;
                format!("{h:02}:{m:02}")
            })
        }),
        wake_time: s["wakeTime"].as_str().map(Into::into).or_else(|| {
            s["wakeTime"].as_i64().map(|t| {
                let h = t / 3600;
                let m = (t % 3600) / 60;
                format!("{h:02}:{m:02}")
            })
        }),
    }
}

impl HumanReadable for ProfileSettings {
    fn print_human(&self) {
        println!("{}", "Profile Settings".bold());
        println!("{}", "\u{2500}".repeat(30));
        if let Some(w) = self.weight_kg {
            println!("  {:<22}{:.1} kg", "Weight:".dimmed(), w);
        }
        if let Some(h) = self.height_cm {
            println!("  {:<22}{:.0} cm", "Height:".dimmed(), h);
        }
        if let Some(ref bd) = self.birth_date {
            println!("  {:<22}{}", "Birth date:".dimmed(), bd);
        }
        if let Some(ref g) = self.gender {
            println!("  {:<22}{}", "Gender:".dimmed(), g);
        }
        if let Some(ref al) = self.activity_level {
            println!("  {:<22}{}", "Activity level:".dimmed(), al);
        }
        println!();
        if let Some(hr) = self.max_hr {
            println!("  {:<22}{} bpm", "Max HR:".dimmed(), hr);
        }
        if let Some(hr) = self.resting_hr {
            println!("  {:<22}{} bpm", "Resting HR:".dimmed(), hr);
        }
        if let Some(hr) = self.lactate_threshold_hr {
            println!("  {:<22}{} bpm", "LT HR:".dimmed(), hr);
        }
        if let Some(vo2) = self.vo2max_running {
            println!("  {:<22}{:.1}", "VO2max (running):".dimmed(), vo2);
        }
        if let Some(ftp) = self.ftp_cycling {
            println!("  {:<22}{:.0}W", "FTP (cycling):".dimmed(), ftp);
        }
        println!();
        if let Some(steps) = self.step_goal {
            println!("  {:<22}{}", "Step goal:".dimmed(), steps);
        }
        if let Some(im) = self.intensity_minutes_goal {
            println!("  {:<22}{} min/week", "Intensity goal:".dimmed(), im);
        }
        if let Some(ref st) = self.sleep_time {
            println!("  {:<22}{}", "Sleep time:".dimmed(), st);
        }
        if let Some(ref wt) = self.wake_time {
            println!("  {:<22}{}", "Wake time:".dimmed(), wt);
        }
        println!();
    }
}

pub async fn settings(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client
        .get_json("/userprofile-service/userprofile/user-settings")
        .await?;
    let s = settings_from_json(&v);
    output.print(&s);
    Ok(())
}
