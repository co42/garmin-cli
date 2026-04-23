use super::output::Output;
use crate::error::{Error, Result};
use crate::garmin::{GarminClient, HrZoneEntry, Profile, ProfileSettings};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Show user profile
    Show,
    /// User settings (show or update)
    Settings {
        #[command(subcommand)]
        command: Option<SettingsCommands>,
    },
}

#[derive(Subcommand)]
pub enum SettingsCommands {
    /// Update user settings
    Set {
        /// Max heart rate (bpm)
        #[arg(long)]
        max_hr: Option<u16>,
        /// Resting heart rate (bpm)
        #[arg(long)]
        resting_hr: Option<u16>,
        /// Weight (kg, converted to grams for the API)
        #[arg(long)]
        weight: Option<f64>,
        /// Height (cm)
        #[arg(long)]
        height: Option<f64>,
        /// Lactate threshold heart rate (bpm)
        #[arg(long)]
        lactate_threshold_hr: Option<u16>,
        /// Lactate threshold speed (m/s)
        #[arg(long)]
        lactate_threshold_speed: Option<f64>,
        /// Whether lactate threshold HR is auto-detected
        #[arg(long)]
        threshold_hr_auto_detected: Option<bool>,
        /// Whether resting HR auto-updates from device
        #[arg(long)]
        resting_hr_auto_update: Option<bool>,
        /// VO2max running
        #[arg(long)]
        vo2max_running: Option<f64>,
        /// Pause training status (sets date to today)
        #[arg(long)]
        training_status_paused: bool,
        /// Resume training status (clears paused date)
        #[arg(long)]
        training_status_resumed: bool,
        /// Sleep time (HH:MM)
        #[arg(long)]
        sleep_time: Option<String>,
        /// Wake time (HH:MM)
        #[arg(long)]
        wake_time: Option<String>,
    },
}

pub async fn run(command: ProfileCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        ProfileCommands::Show => show(&client, output).await,
        ProfileCommands::Settings { command } => match command {
            None => settings(&client, output).await,
            Some(SettingsCommands::Set {
                max_hr,
                resting_hr,
                weight,
                height,
                lactate_threshold_hr,
                lactate_threshold_speed,
                threshold_hr_auto_detected,
                resting_hr_auto_update,
                vo2max_running,
                training_status_paused,
                training_status_resumed,
                sleep_time,
                wake_time,
            }) => {
                settings_set(
                    &client,
                    output,
                    SettingsSetArgs {
                        max_hr,
                        resting_hr,
                        weight,
                        height,
                        lactate_threshold_hr,
                        lactate_threshold_speed,
                        threshold_hr_auto_detected,
                        resting_hr_auto_update,
                        vo2max_running,
                        training_status_paused,
                        training_status_resumed,
                        sleep_time,
                        wake_time,
                    },
                )
                .await
            }
        },
    }
}

async fn show(client: &GarminClient, output: &Output) -> Result<()> {
    let p = client.social_profile().await?;
    let profile = Profile::from(&p);
    output.print(&profile);
    Ok(())
}

async fn settings(client: &GarminClient, output: &Output) -> Result<()> {
    let (settings, hr_zones) = tokio::try_join!(client.user_settings(), client.hr_zones())?;
    let view = ProfileSettings::from_parts(&settings, &hr_zones);
    output.print(&view);
    Ok(())
}

struct SettingsSetArgs {
    max_hr: Option<u16>,
    resting_hr: Option<u16>,
    weight: Option<f64>,
    height: Option<f64>,
    lactate_threshold_hr: Option<u16>,
    lactate_threshold_speed: Option<f64>,
    threshold_hr_auto_detected: Option<bool>,
    resting_hr_auto_update: Option<bool>,
    vo2max_running: Option<f64>,
    training_status_paused: bool,
    training_status_resumed: bool,
    sleep_time: Option<String>,
    wake_time: Option<String>,
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

fn parse_time_to_seconds(time: &str) -> Result<i64> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::Usage(format!("invalid time '{time}', expected HH:MM")));
    }
    let h: i64 = parts[0]
        .parse()
        .map_err(|_| Error::Usage(format!("invalid hour in '{time}'")))?;
    let m: i64 = parts[1]
        .parse()
        .map_err(|_| Error::Usage(format!("invalid minute in '{time}'")))?;
    if h >= 24 || m >= 60 {
        return Err(Error::Usage(format!("time out of range: '{time}'")));
    }
    Ok(h * 3600 + m * 60)
}

async fn settings_set(client: &GarminClient, output: &Output, args: SettingsSetArgs) -> Result<()> {
    if !args.has_any() {
        return Err(Error::Usage(
            "no settings provided. Use --help to see available flags.".into(),
        ));
    }
    if args.training_status_paused && args.training_status_resumed {
        return Err(Error::Usage("cannot both pause and resume training status".into()));
    }

    // Fetch the current HR zones to extract the DEFAULT-sport entry which we
    // then merge changes into.
    let current_hr_zones = client.hr_zones().await?;

    let mut user_data = serde_json::Map::new();
    let mut user_sleep = serde_json::Map::new();
    let mut hr_zone_changes = serde_json::Map::new();

    if let Some(v) = args.max_hr {
        hr_zone_changes.insert("maxHeartRateUsed".into(), serde_json::json!(v));
    }
    if let Some(v) = args.resting_hr {
        hr_zone_changes.insert("restingHeartRateUsed".into(), serde_json::json!(v));
    }
    if let Some(v) = args.resting_hr_auto_update {
        hr_zone_changes.insert("restingHrAutoUpdateUsed".into(), serde_json::json!(v));
    }
    if let Some(v) = args.weight {
        user_data.insert("weight".into(), serde_json::json!(v * 1000.0));
    }
    if let Some(v) = args.height {
        user_data.insert("height".into(), serde_json::json!(v));
    }
    if let Some(v) = args.lactate_threshold_hr {
        user_data.insert("lactateThresholdHeartRate".into(), serde_json::json!(v));
    }
    if let Some(v) = args.lactate_threshold_speed {
        // API expects the value ten times too low (see helpers::correct_lt_speed).
        user_data.insert("lactateThresholdSpeed".into(), serde_json::json!(v / 10.0));
    }
    if let Some(v) = args.threshold_hr_auto_detected {
        user_data.insert("thresholdHeartRateAutoDetected".into(), serde_json::json!(v));
    }
    if let Some(v) = args.vo2max_running {
        user_data.insert("vo2MaxRunning".into(), serde_json::json!(v));
    }
    if args.training_status_paused {
        let today = super::helpers::today().format("%Y-%m-%d").to_string();
        user_data.insert("trainingStatusPausedDate".into(), serde_json::json!(today));
    }
    if args.training_status_resumed {
        user_data.insert("trainingStatusPausedDate".into(), serde_json::Value::Null);
    }
    if let Some(ref v) = args.sleep_time {
        user_sleep.insert("sleepTime".into(), serde_json::json!(parse_time_to_seconds(v)?));
    }
    if let Some(ref v) = args.wake_time {
        user_sleep.insert("wakeTime".into(), serde_json::json!(parse_time_to_seconds(v)?));
    }

    let has_settings = !user_data.is_empty() || !user_sleep.is_empty();
    let has_hr = !hr_zone_changes.is_empty();

    if !output.is_json() && (has_settings || has_hr) {
        output.status("Applying changes...");
    }

    if has_settings {
        let mut body = serde_json::Map::new();
        if !user_data.is_empty() {
            body.insert("userData".into(), serde_json::Value::Object(user_data));
        }
        if !user_sleep.is_empty() {
            body.insert("userSleep".into(), serde_json::Value::Object(user_sleep));
        }
        client.update_user_settings(&serde_json::Value::Object(body)).await?;
    }

    if has_hr {
        // Merge changes into the DEFAULT-sport HR zone entry.
        let default_zone = serde_json::to_value(current_hr_zones.iter().find(|z| z.sport == "DEFAULT").unwrap_or(
            &HrZoneEntry {
                sport: "DEFAULT".into(),
                max_heart_rate_used: None,
                resting_heart_rate_used: None,
                resting_hr_auto_update_used: None,
            },
        ))?;
        let mut entry = default_zone;
        if let serde_json::Value::Object(ref mut obj) = entry {
            obj.insert("changeState".into(), serde_json::json!("CHANGED"));
            for (k, v) in hr_zone_changes {
                obj.insert(k, v);
            }
        }
        client.update_hr_zones(&serde_json::json!([entry])).await?;
    }

    // Re-fetch and display.
    let (updated, updated_hr) = tokio::try_join!(client.user_settings(), client.hr_zones())?;
    let view = ProfileSettings::from_parts(&updated, &updated_hr);
    output.success("Settings updated.");
    output.print(&view);
    Ok(())
}
