use super::helpers::deser_hhmm;
use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Wraps `/userprofile-service/socialProfile`.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SocialProfile {
    #[serde(default)]
    pub display_name: String,
    /// API returns both `userProfileFullName` and `fullName` with the same
    /// value; keeping both as `rename + alias` makes serde error on duplicate
    /// field. Use the `userProfile*` spelling exclusively.
    #[serde(default, rename(deserialize = "userProfileFullName"))]
    pub full_name: String,
    pub user_name: Option<String>,
    /// API: `userProfilePK` (all-caps "PK") — `rename_all = camelCase` would
    /// match `userProfilePk` (lower-k) and miss it.
    #[serde(alias = "profileId", rename(deserialize = "userProfilePK"))]
    pub user_profile_pk: Option<u64>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub user_level: Option<i64>,
    pub profile_visibility: Option<String>,
    pub primary_activity: Option<String>,
}

/// Friendly facade shown by the `profile show` command.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct Profile {
    pub display_name: String,
    pub user_name: Option<String>,
    pub user_id: Option<u64>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub primary_activity: Option<String>,
    pub user_level: Option<i64>,
    pub profile_visibility: Option<String>,
}

impl From<&SocialProfile> for Profile {
    fn from(p: &SocialProfile) -> Self {
        Self {
            display_name: p.full_name.clone(),
            user_name: p.user_name.clone(),
            user_id: p.user_profile_pk,
            location: p.location.clone(),
            bio: p.bio.clone(),
            primary_activity: p.primary_activity.clone(),
            user_level: p.user_level,
            profile_visibility: p.profile_visibility.clone(),
        }
    }
}

impl HumanReadable for Profile {
    fn print_human(&self) {
        println!("{}", "Profile".bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        println!("  {:<LABEL_WIDTH$}{}", "Name:", self.display_name.cyan());
        if let Some(ref u) = self.user_name {
            println!("  {:<LABEL_WIDTH$}{u}", "Username:");
        }
        if let Some(id) = self.user_id {
            println!("  {:<LABEL_WIDTH$}{id}", "User ID:");
        }
        if let Some(ref l) = self.location {
            println!("  {:<LABEL_WIDTH$}{l}", "Location:");
        }
        if let Some(ref b) = self.bio {
            println!("  {:<LABEL_WIDTH$}{b}", "Bio:");
        }
        if let Some(ref a) = self.primary_activity {
            println!("  {:<LABEL_WIDTH$}{a}", "Activity:");
        }
        if let Some(lvl) = self.user_level {
            println!("  {:<LABEL_WIDTH$}{lvl}", "Level:");
        }
        if let Some(ref v) = self.profile_visibility {
            println!("  {:<LABEL_WIDTH$}{v}", "Visibility:");
        }
    }
}

// ── User settings ────────────────────────────────────────────────────

/// `/userprofile-service/userprofile/user-settings` — two top-level sections.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserSettings {
    #[serde(default)]
    pub user_data: UserData,
    #[serde(default)]
    pub user_sleep: UserSleep,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserData {
    #[serde(
        rename(deserialize = "weight"),
        default,
        deserialize_with = "super::helpers::deser_g_to_kg"
    )]
    pub weight_kg: Option<f64>,
    #[serde(rename(deserialize = "height"))]
    pub height_cm: Option<f64>,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub handedness: Option<String>,
    #[serde(rename(deserialize = "lactateThresholdHeartRate"))]
    pub lactate_threshold_hr: Option<i64>,
    /// API stores ten times too low; `deser_lt_speed` corrects at the boundary
    /// so `lactate_threshold_speed_mps` is in m/s everywhere.
    #[serde(
        rename(deserialize = "lactateThresholdSpeed"),
        default,
        deserialize_with = "super::helpers::deser_lt_speed"
    )]
    pub lactate_threshold_speed_mps: Option<f64>,
    pub threshold_heart_rate_auto_detected: Option<bool>,
    /// API: `vO2MaxRunning` — irregular `vO2` casing that `rename_all` can't produce.
    #[serde(rename(deserialize = "vO2MaxRunning"))]
    pub vo2_max_running: Option<f64>,
    #[serde(rename(deserialize = "vO2MaxCycling"))]
    pub vo2_max_cycling: Option<f64>,
    pub functional_threshold_power: Option<f64>,
    pub ftp_auto_detected: Option<bool>,
    pub training_status_paused_date: Option<String>,
    pub measurement_system: Option<String>,
    pub time_format: Option<String>,
    pub available_training_days: Option<Vec<String>>,
    pub preferred_long_training_days: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserSleep {
    /// API returns HH:MM string OR seconds from midnight; normalized to HH:MM.
    #[serde(default, deserialize_with = "deser_hhmm")]
    pub sleep_time: Option<String>,
    #[serde(default, deserialize_with = "deser_hhmm")]
    pub wake_time: Option<String>,
}

// ── HR zones ─────────────────────────────────────────────────────────

/// `/biometric-service/heartRateZones` returns an array of zone entries per sport.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HrZoneEntry {
    pub sport: String,
    pub max_heart_rate_used: Option<i64>,
    pub resting_heart_rate_used: Option<i64>,
    pub resting_hr_auto_update_used: Option<bool>,
}

// ── Combined profile settings view ───────────────────────────────────

/// Display-only merged view of UserSettings + DEFAULT HrZoneEntry.
/// Built by the command layer since it needs two endpoints.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ProfileSettings {
    pub weight_kg: Option<f64>,
    pub height_cm: Option<f64>,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub handedness: Option<String>,
    pub max_hr_bpm: Option<i64>,
    pub resting_hr_bpm: Option<i64>,
    pub lactate_threshold_hr_bpm: Option<i64>,
    pub lactate_threshold_speed_mps: Option<f64>,
    pub threshold_hr_auto_detected: Option<bool>,
    pub vo2max_running: Option<f64>,
    pub vo2max_cycling: Option<f64>,
    pub ftp_watts: Option<f64>,
    pub ftp_auto_detected: Option<bool>,
    pub training_status_paused_date: Option<String>,
    pub measurement_system: Option<String>,
    pub time_format: Option<String>,
    pub available_training_days: Option<Vec<String>>,
    pub preferred_long_training_days: Option<Vec<String>>,
    pub sleep_time: Option<String>,
    pub wake_time: Option<String>,
}

impl ProfileSettings {
    pub fn from_parts(settings: &UserSettings, hr_zones: &[HrZoneEntry]) -> Self {
        let default_zone = hr_zones.iter().find(|z| z.sport == "DEFAULT");
        let u = &settings.user_data;
        let s = &settings.user_sleep;
        Self {
            weight_kg: u.weight_kg,
            height_cm: u.height_cm,
            birth_date: u.birth_date.clone(),
            gender: u.gender.clone(),
            handedness: u.handedness.clone(),
            max_hr_bpm: default_zone.and_then(|z| z.max_heart_rate_used),
            resting_hr_bpm: default_zone.and_then(|z| z.resting_heart_rate_used),
            lactate_threshold_hr_bpm: u.lactate_threshold_hr,
            lactate_threshold_speed_mps: u.lactate_threshold_speed_mps,
            threshold_hr_auto_detected: u.threshold_heart_rate_auto_detected,
            vo2max_running: u.vo2_max_running,
            vo2max_cycling: u.vo2_max_cycling,
            ftp_watts: u.functional_threshold_power,
            ftp_auto_detected: u.ftp_auto_detected,
            training_status_paused_date: u.training_status_paused_date.clone(),
            measurement_system: u.measurement_system.clone(),
            time_format: u.time_format.clone(),
            available_training_days: u.available_training_days.clone(),
            preferred_long_training_days: u.preferred_long_training_days.clone(),
            sleep_time: s.sleep_time.clone(),
            wake_time: s.wake_time.clone(),
        }
    }
}

impl HumanReadable for ProfileSettings {
    fn print_human(&self) {
        println!("{}", "Profile Settings".bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());

        if let Some(w) = self.weight_kg {
            println!("  {:<LABEL_WIDTH$}{:.1} kg", "Weight:", w);
        }
        if let Some(h) = self.height_cm {
            println!("  {:<LABEL_WIDTH$}{:.0} cm", "Height:", h);
        }
        if let Some(ref bd) = self.birth_date {
            println!("  {:<LABEL_WIDTH$}{bd}", "Birth date:");
        }
        if let Some(ref g) = self.gender {
            println!("  {:<LABEL_WIDTH$}{g}", "Gender:");
        }
        if let Some(ref h) = self.handedness {
            println!("  {:<LABEL_WIDTH$}{h}", "Handedness:");
        }

        if let Some(hr) = self.max_hr_bpm {
            println!("  {:<LABEL_WIDTH$}{hr} bpm", "Max HR:");
        }
        if let Some(hr) = self.resting_hr_bpm {
            println!("  {:<LABEL_WIDTH$}{hr} bpm", "Resting HR:");
        }
        if let Some(hr) = self.lactate_threshold_hr_bpm {
            let auto = self
                .threshold_hr_auto_detected
                .map(|a| format!(" ({})", if a { "auto" } else { "manual" }))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{hr} bpm{auto}", "LT HR:");
        }
        if let Some(speed) = self.lactate_threshold_speed_mps
            && speed > 0.0
        {
            println!(
                "  {:<LABEL_WIDTH$}{}",
                "LT speed:",
                super::helpers::pace_from_speed(speed)
            );
        }
        if let Some(vo2) = self.vo2max_running {
            println!("  {:<LABEL_WIDTH$}{:.1}", "VO2max (run):", vo2);
        }
        if let Some(vo2) = self.vo2max_cycling {
            println!("  {:<LABEL_WIDTH$}{:.1}", "VO2max (bike):", vo2);
        }
        if let Some(ftp) = self.ftp_watts {
            let auto = self
                .ftp_auto_detected
                .map(|a| format!(" ({})", if a { "auto" } else { "manual" }))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{:.0} W{auto}", "FTP:", ftp);
        }
        if let Some(ref d) = self.training_status_paused_date {
            println!("  {:<LABEL_WIDTH$}{d}", "Paused:");
        }

        if let Some(ref m) = self.measurement_system {
            println!("  {:<LABEL_WIDTH$}{m}", "Units:");
        }
        if let Some(ref tf) = self.time_format {
            println!("  {:<LABEL_WIDTH$}{tf}", "Time format:");
        }
        if let Some(ref days) = self.available_training_days {
            let s: Vec<&str> = days.iter().map(|d| &d[..3.min(d.len())]).collect();
            println!("  {:<LABEL_WIDTH$}{}", "Training days:", s.join(", "));
        }
        if let Some(ref days) = self.preferred_long_training_days {
            let s: Vec<&str> = days.iter().map(|d| &d[..3.min(d.len())]).collect();
            println!("  {:<LABEL_WIDTH$}{}", "Long run days:", s.join(", "));
        }
        if let Some(ref st) = self.sleep_time {
            println!("  {:<LABEL_WIDTH$}{st}", "Sleep time:");
        }
        if let Some(ref wt) = self.wake_time {
            println!("  {:<LABEL_WIDTH$}{wt}", "Wake time:");
        }
    }
}
