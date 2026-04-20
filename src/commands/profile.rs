use crate::client::GarminClient;
use crate::error::{Error, Result};
use crate::output::{HumanReadable, LABEL_WIDTH, Output};
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
        println!("{}", "Profile".bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        println!("  {:<LABEL_WIDTH$}{}", "Name:", self.display_name.cyan());
        if let Some(ref u) = self.user_name {
            println!("  {:<LABEL_WIDTH$}{u}", "Username:");
        }
        if let Some(ref e) = self.email {
            println!("  {:<LABEL_WIDTH$}{e}", "Email:");
        }
        if let Some(ref l) = self.locale {
            println!("  {:<LABEL_WIDTH$}{l}", "Locale:");
        }
        if let Some(ref m) = self.measurement_system {
            println!("  {:<LABEL_WIDTH$}{m}", "Units:");
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
    // Biometrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handedness: Option<String>,

    // HR & training thresholds (from biometric service)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lactate_threshold_hr: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lactate_threshold_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_hr_auto_detected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max_running: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vo2max_cycling: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ftp: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ftp_auto_detected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub training_status_paused_date: Option<String>,

    // Preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_training_days: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_long_training_days: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wake_time: Option<String>,
}

fn secs_to_hhmm(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{h:02}:{m:02}")
}

fn json_str_array(v: &serde_json::Value) -> Option<Vec<String>> {
    v.as_array().map(|arr| {
        arr.iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect()
    })
}

fn settings_from_json(
    user_settings: &serde_json::Value,
    hr_zones: Option<&serde_json::Value>,
) -> ProfileSettings {
    let u = &user_settings["userData"];
    let s = &user_settings["userSleep"];

    // max_hr and resting_hr come from the biometric HR zones endpoint
    let (max_hr, resting_hr) = if let Some(zones) = hr_zones {
        let default = zones
            .as_array()
            .and_then(|arr| arr.iter().find(|z| z["sport"].as_str() == Some("DEFAULT")));
        (
            default.and_then(|z| z["maxHeartRateUsed"].as_i64()),
            default.and_then(|z| z["restingHeartRateUsed"].as_i64()),
        )
    } else {
        (None, None)
    };

    ProfileSettings {
        weight_kg: u["weight"].as_f64().map(|w| w / 1000.0),
        height_cm: u["height"].as_f64(),
        birth_date: u["birthDate"].as_str().map(Into::into),
        gender: u["gender"].as_str().map(Into::into),
        handedness: u["handedness"].as_str().map(Into::into),
        max_hr,
        resting_hr,
        lactate_threshold_hr: u["lactateThresholdHeartRate"].as_i64(),
        // Garmin API stores LT speed ~10x too low; correct like training command does
        lactate_threshold_speed: u["lactateThresholdSpeed"]
            .as_f64()
            .map(|s| if s > 0.0 && s < 1.0 { s * 10.0 } else { s }),
        threshold_hr_auto_detected: u["thresholdHeartRateAutoDetected"].as_bool(),
        vo2max_running: u["vo2MaxRunning"].as_f64(),
        vo2max_cycling: u["vo2MaxCycling"].as_f64(),
        ftp: u["functionalThresholdPower"].as_f64(),
        ftp_auto_detected: u["ftpAutoDetected"].as_bool(),
        training_status_paused_date: u["trainingStatusPausedDate"].as_str().map(Into::into),
        measurement_system: u["measurementSystem"].as_str().map(Into::into),
        time_format: u["timeFormat"].as_str().map(Into::into),
        available_training_days: json_str_array(&u["availableTrainingDays"]),
        preferred_long_training_days: json_str_array(&u["preferredLongTrainingDays"]),
        sleep_time: s["sleepTime"]
            .as_str()
            .map(Into::into)
            .or_else(|| s["sleepTime"].as_i64().map(secs_to_hhmm)),
        wake_time: s["wakeTime"]
            .as_str()
            .map(Into::into)
            .or_else(|| s["wakeTime"].as_i64().map(secs_to_hhmm)),
    }
}

impl HumanReadable for ProfileSettings {
    fn print_human(&self) {
        println!("{}", "Profile Settings".bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());

        // Biometrics
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

        // Training thresholds
        if let Some(hr) = self.max_hr {
            println!("  {:<LABEL_WIDTH$}{hr} bpm", "Max HR:");
        }
        if let Some(hr) = self.resting_hr {
            println!("  {:<LABEL_WIDTH$}{hr} bpm", "Resting HR:");
        }
        if let Some(hr) = self.lactate_threshold_hr {
            let auto = self
                .threshold_hr_auto_detected
                .map(|a| format!(" ({})", if a { "auto" } else { "manual" }))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{hr} bpm{auto}", "LT HR:");
        }
        if let Some(speed) = self.lactate_threshold_speed
            && speed > 0.0
        {
            let pace_secs = (1000.0 / speed) as u64;
            let min = pace_secs / 60;
            let sec = pace_secs % 60;
            println!("  {:<LABEL_WIDTH$}{min}:{sec:02} /km", "LT speed:");
        }
        if let Some(vo2) = self.vo2max_running {
            println!("  {:<LABEL_WIDTH$}{:.1}", "VO2max (run):", vo2);
        }
        if let Some(vo2) = self.vo2max_cycling {
            println!("  {:<LABEL_WIDTH$}{:.1}", "VO2max (bike):", vo2);
        }
        if let Some(ftp) = self.ftp {
            let auto = self
                .ftp_auto_detected
                .map(|a| format!(" ({})", if a { "auto" } else { "manual" }))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{:.0} W{auto}", "FTP:", ftp);
        }
        if let Some(ref d) = self.training_status_paused_date {
            println!("  {:<LABEL_WIDTH$}{d}", "Paused:");
        }

        // Preferences
        if let Some(ref m) = self.measurement_system {
            println!("  {:<LABEL_WIDTH$}{m}", "Units:");
        }
        if let Some(ref tf) = self.time_format {
            println!("  {:<LABEL_WIDTH$}{tf}", "Time format:");
        }
        if let Some(ref days) = self.available_training_days {
            let s: Vec<&str> = days.iter().map(|d| &d[..3]).collect();
            println!("  {:<LABEL_WIDTH$}{}", "Training days:", s.join(", "));
        }
        if let Some(ref days) = self.preferred_long_training_days {
            let s: Vec<&str> = days.iter().map(|d| &d[..3]).collect();
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

pub async fn settings(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client
        .get_json("/userprofile-service/userprofile/user-settings")
        .await?;
    let hr_zones: serde_json::Value = client.get_json("/biometric-service/heartRateZones").await?;
    let s = settings_from_json(&v, Some(&hr_zones));
    output.print(&s);
    Ok(())
}

// ── Settings Set ────────────────────────────────────────────────────

pub struct SettingsSetArgs {
    pub max_hr: Option<u16>,
    pub resting_hr: Option<u16>,
    pub weight: Option<f64>,
    pub height: Option<f64>,
    pub lactate_threshold_hr: Option<u16>,
    pub lactate_threshold_speed: Option<f64>,
    pub threshold_hr_auto_detected: Option<bool>,
    pub resting_hr_auto_update: Option<bool>,
    pub vo2max_running: Option<f64>,
    pub training_status_paused: bool,
    pub training_status_resumed: bool,
    pub sleep_time: Option<String>,
    pub wake_time: Option<String>,
}

impl SettingsSetArgs {
    fn has_any(&self) -> bool {
        self.max_hr.is_some()
            || self.resting_hr.is_some()
            || self.weight.is_some()
            || self.height.is_some()
            || self.lactate_threshold_hr.is_some()
            || self.lactate_threshold_speed.is_some()
            || self.threshold_hr_auto_detected.is_some()
            || self.resting_hr_auto_update.is_some()
            || self.vo2max_running.is_some()
            || self.training_status_paused
            || self.training_status_resumed
            || self.sleep_time.is_some()
            || self.wake_time.is_some()
    }
}

/// Convert HH:MM string to seconds since midnight (API format).
fn parse_time_to_seconds(time: &str) -> Result<i64> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::Api(format!(
            "Invalid time format '{time}', expected HH:MM"
        )));
    }
    let h: i64 = parts[0]
        .parse()
        .map_err(|_| Error::Api(format!("Invalid hour in '{time}'")))?;
    let m: i64 = parts[1]
        .parse()
        .map_err(|_| Error::Api(format!("Invalid minute in '{time}'")))?;
    if h >= 24 || m >= 60 {
        return Err(Error::Api(format!("Time out of range: '{time}'")));
    }
    Ok(h * 3600 + m * 60)
}

/// Tracks changes across user-settings and heartRateZones endpoints.
struct SettingsUpdate {
    current_settings: serde_json::Value,
    current_hr_zones: serde_json::Value, // full array from biometric-service
    user_data: serde_json::Map<String, serde_json::Value>,
    user_sleep: serde_json::Map<String, serde_json::Value>,
    hr_zone_data: serde_json::Map<String, serde_json::Value>,
    changes: Vec<Change>,
}

struct Change {
    label: String,
    section: &'static str, // "userData", "userSleep", or "heartRateZones"
    api_key: &'static str,
    old: serde_json::Value,
    new: serde_json::Value,
}

impl Change {
    fn fmt_human(&self) -> String {
        fn fmt_val(v: &serde_json::Value) -> String {
            match v {
                serde_json::Value::Null => "-".into(),
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            }
        }
        format!(
            "  {:<LABEL_WIDTH$}{} → {}",
            self.label,
            fmt_val(&self.old),
            fmt_val(&self.new)
        )
    }
}

impl SettingsUpdate {
    fn new(current_settings: serde_json::Value, current_hr_zones: serde_json::Value) -> Self {
        Self {
            current_settings,
            current_hr_zones,
            user_data: serde_json::Map::new(),
            user_sleep: serde_json::Map::new(),
            hr_zone_data: serde_json::Map::new(),
            changes: Vec::new(),
        }
    }

    /// Get the DEFAULT sport entry from the current heartRateZones array.
    fn default_hr_zone(&self) -> Option<&serde_json::Value> {
        self.current_hr_zones
            .as_array()
            .and_then(|arr| arr.iter().find(|z| z["sport"].as_str() == Some("DEFAULT")))
    }

    fn set_user_data(&mut self, label: &str, api_key: &'static str, new_val: serde_json::Value) {
        self.changes.push(Change {
            label: label.into(),
            section: "userData",
            api_key,
            old: self.current_settings["userData"][api_key].clone(),
            new: new_val.clone(),
        });
        self.user_data.insert(api_key.into(), new_val);
    }

    fn set_user_sleep(&mut self, label: &str, api_key: &'static str, new_val: serde_json::Value) {
        self.changes.push(Change {
            label: label.into(),
            section: "userSleep",
            api_key,
            old: self.current_settings["userSleep"][api_key].clone(),
            new: new_val.clone(),
        });
        self.user_sleep.insert(api_key.into(), new_val);
    }

    fn set_hr_zone(&mut self, label: &str, api_key: &'static str, new_val: serde_json::Value) {
        let old = self
            .default_hr_zone()
            .and_then(|z| z.get(api_key))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        self.changes.push(Change {
            label: label.into(),
            section: "heartRateZones",
            api_key,
            old,
            new: new_val.clone(),
        });
        self.hr_zone_data.insert(api_key.into(), new_val);
    }

    fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    fn has_settings_changes(&self) -> bool {
        !self.user_data.is_empty() || !self.user_sleep.is_empty()
    }

    fn has_hr_zone_changes(&self) -> bool {
        !self.hr_zone_data.is_empty()
    }

    fn build_settings_body(&self) -> serde_json::Value {
        let mut body = serde_json::Map::new();
        if !self.user_data.is_empty() {
            body.insert(
                "userData".into(),
                serde_json::Value::Object(self.user_data.clone()),
            );
        }
        if !self.user_sleep.is_empty() {
            body.insert(
                "userSleep".into(),
                serde_json::Value::Object(self.user_sleep.clone()),
            );
        }
        serde_json::Value::Object(body)
    }

    /// Build the heartRateZones PUT payload by merging changes into the DEFAULT entry.
    fn build_hr_zones_body(&self) -> serde_json::Value {
        let mut entry = self
            .default_hr_zone()
            .cloned()
            .unwrap_or_else(|| serde_json::json!({"sport": "DEFAULT"}));
        if let Some(obj) = entry.as_object_mut() {
            obj.insert("changeState".into(), serde_json::json!("CHANGED"));
            for (k, v) in &self.hr_zone_data {
                obj.insert(k.clone(), v.clone());
            }
        }
        serde_json::json!([entry])
    }

    fn print_planned(&self, output: &Output) {
        output.status("Changes to apply:");
        for c in &self.changes {
            eprintln!("{}", c.fmt_human());
        }
        eprintln!();
    }

    fn to_json(
        &self,
        updated_settings: &serde_json::Value,
        updated_hr_zones: &serde_json::Value,
    ) -> serde_json::Value {
        let default_zone = updated_hr_zones
            .as_array()
            .and_then(|arr| arr.iter().find(|z| z["sport"].as_str() == Some("DEFAULT")));
        let mut result = serde_json::Map::new();
        for c in &self.changes {
            let actual_new = if c.section == "heartRateZones" {
                default_zone
                    .and_then(|z| z.get(c.api_key))
                    .unwrap_or(&serde_json::Value::Null)
            } else {
                &updated_settings[c.section][c.api_key]
            };
            result.insert(
                c.api_key.to_string(),
                serde_json::json!({ "old": c.old, "new": actual_new }),
            );
        }
        serde_json::Value::Object(result)
    }
}

pub async fn settings_set(
    client: &GarminClient,
    output: &Output,
    args: SettingsSetArgs,
) -> Result<()> {
    if !args.has_any() {
        return Err(Error::Api(
            "No settings provided. Use --help to see available flags.".into(),
        ));
    }

    if args.training_status_paused && args.training_status_resumed {
        return Err(Error::Api(
            "Cannot both pause and resume training status.".into(),
        ));
    }

    let settings_path = "/userprofile-service/userprofile/user-settings";
    let hr_zones_path = "/biometric-service/heartRateZones";

    let current_settings: serde_json::Value = client.get_json(settings_path).await?;
    let current_hr_zones: serde_json::Value = client.get_json(hr_zones_path).await?;
    let mut update = SettingsUpdate::new(current_settings, current_hr_zones);

    // HR fields → biometric-service/heartRateZones (NOT user-settings)
    if let Some(v) = args.max_hr {
        update.set_hr_zone("Max HR (bpm)", "maxHeartRateUsed", serde_json::json!(v));
    }
    if let Some(v) = args.resting_hr {
        update.set_hr_zone(
            "Resting HR (bpm)",
            "restingHeartRateUsed",
            serde_json::json!(v),
        );
    }

    if let Some(v) = args.resting_hr_auto_update {
        update.set_hr_zone(
            "Resting HR auto-update",
            "restingHrAutoUpdateUsed",
            serde_json::json!(v),
        );
    }

    // Biometrics → user-settings
    if let Some(v) = args.weight {
        update.set_user_data(
            &format!("Weight ({v:.1} kg)"),
            "weight",
            serde_json::json!(v * 1000.0),
        );
    }
    if let Some(v) = args.height {
        update.set_user_data("Height (cm)", "height", serde_json::json!(v));
    }

    // Training thresholds → user-settings (override values)
    if let Some(v) = args.lactate_threshold_hr {
        update.set_user_data(
            "LT HR (bpm)",
            "lactateThresholdHeartRate",
            serde_json::json!(v),
        );
    }
    if let Some(v) = args.lactate_threshold_speed {
        // Garmin API stores LT speed ~10x too low; convert real m/s for storage
        let api_val = if v >= 1.0 { v / 10.0 } else { v };
        update.set_user_data(
            "LT speed (m/s)",
            "lactateThresholdSpeed",
            serde_json::json!(api_val),
        );
    }
    if let Some(v) = args.threshold_hr_auto_detected {
        update.set_user_data(
            "LT HR auto-detected",
            "thresholdHeartRateAutoDetected",
            serde_json::json!(v),
        );
    }
    if let Some(v) = args.vo2max_running {
        update.set_user_data("VO2max (running)", "vo2MaxRunning", serde_json::json!(v));
    }

    // Training status → user-settings
    if args.training_status_paused {
        let today = crate::util::today();
        update.set_user_data(
            "Training status",
            "trainingStatusPausedDate",
            serde_json::json!(today),
        );
    }
    if args.training_status_resumed {
        update.set_user_data(
            "Training status",
            "trainingStatusPausedDate",
            serde_json::Value::Null,
        );
    }

    // Sleep → user-settings
    if let Some(ref v) = args.sleep_time {
        update.set_user_sleep(
            &format!("Sleep time ({v})"),
            "sleepTime",
            serde_json::json!(parse_time_to_seconds(v)?),
        );
    }
    if let Some(ref v) = args.wake_time {
        update.set_user_sleep(
            &format!("Wake time ({v})"),
            "wakeTime",
            serde_json::json!(parse_time_to_seconds(v)?),
        );
    }

    if update.is_empty() {
        return Ok(());
    }

    if !output.is_json() {
        update.print_planned(output);
    }

    // PUT to user-settings if any userData/userSleep changes
    if update.has_settings_changes() {
        client
            .put(settings_path, &update.build_settings_body())
            .await?;
    }

    // PUT to biometric-service/heartRateZones if any HR zone changes
    if update.has_hr_zone_changes() {
        client
            .put(hr_zones_path, &update.build_hr_zones_body())
            .await?;
    }

    // Re-fetch both for verification and display
    let updated_settings: serde_json::Value = client.get_json(settings_path).await?;
    let updated_hr_zones: serde_json::Value = client.get_json(hr_zones_path).await?;

    if output.is_json() {
        output.print_value(&update.to_json(&updated_settings, &updated_hr_zones));
    } else {
        output.success("Settings updated.");
        let s = settings_from_json(&updated_settings, Some(&updated_hr_zones));
        s.print_human();
    }

    Ok(())
}
