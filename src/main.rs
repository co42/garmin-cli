use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use garmin_cli::auth::Tokens;
use garmin_cli::commands;
use garmin_cli::error::Error;
use garmin_cli::{GarminClient, Output};

#[derive(Parser)]
#[command(name = "garmin", about = "Garmin Connect CLI", version)]
struct Cli {
    /// Force JSON output
    #[arg(long, global = true)]
    json: bool,

    /// Force human output (override TTY auto-detect)
    #[arg(long, global = true)]
    no_json: bool,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pretty: bool,

    /// Filter output fields (comma-separated, JSON mode only)
    #[arg(long, global = true, value_delimiter = ',')]
    fields: Vec<String>,

    /// Suppress status messages
    #[arg(long, short, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn output(&self) -> Output {
        let json = if self.json {
            Some(true)
        } else if self.no_json {
            Some(false)
        } else {
            None
        };
        Output::new(json, self.pretty, self.quiet, self.fields.clone())
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication (login, status, logout)
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Raw API call (escape hatch)
    Api {
        /// API path (e.g. /userprofile-service/usersummary)
        path: String,
        /// HTTP method
        #[arg(long, default_value = "GET")]
        method: String,
        /// Request body (JSON)
        #[arg(long)]
        data: Option<String>,
    },
    /// User profile
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Daily summary
    Summary {
        /// Date (YYYY-MM-DD), defaults to today
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        /// Number of days
        #[arg(long)]
        days: Option<u32>,
        /// Start of date range (YYYY-MM-DD)
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        /// End of date range (YYYY-MM-DD)
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Health metrics
    Health {
        #[command(subcommand)]
        command: HealthCommands,
    },
    /// Training status, readiness, and performance metrics
    Training {
        #[command(subcommand)]
        command: TrainingCommands,
    },
    /// Activities
    Activities {
        #[command(subcommand)]
        command: ActivityCommands,
    },
    /// Workouts (create, schedule, push to watch)
    Workouts {
        #[command(subcommand)]
        command: WorkoutCommands,
    },
    /// Courses (saved routes)
    Courses {
        #[command(subcommand)]
        command: CourseCommands,
    },
    /// Earned badges and achievements
    Badges {
        #[command(subcommand)]
        command: BadgeCommands,
    },
    /// Gear (shoes, bikes, etc.)
    Gear {
        #[command(subcommand)]
        command: GearCommands,
    },
    /// Personal records
    Records,
    /// Calendar (scheduled workouts, activities)
    Calendar {
        #[command(subcommand)]
        command: Option<CalendarCommands>,
        /// Year (defaults to current)
        #[arg(long, global = true)]
        year: Option<u32>,
        /// Month (1-12, defaults to current)
        #[arg(long, global = true)]
        month: Option<u32>,
        /// Show next N weeks (spans months automatically)
        #[arg(long, global = true)]
        weeks: Option<u32>,
    },
    /// Devices
    Devices {
        #[command(subcommand)]
        command: DeviceCommands,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: ShellChoice,
    },
}

#[derive(Clone, ValueEnum)]
enum ShellChoice {
    Bash,
    Zsh,
    Fish,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Log in to Garmin Connect (reads GARMIN_EMAIL / GARMIN_PASSWORD env vars, or prompts)
    Login,
    /// Show authentication status
    Status,
    /// Log out (delete stored tokens)
    Logout,
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// Show user profile
    Show,
    /// User settings (show or update)
    Settings {
        #[command(subcommand)]
        command: Option<SettingsCommands>,
    },
}

#[derive(Subcommand)]
enum SettingsCommands {
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

#[derive(Subcommand)]
enum HealthCommands {
    /// Sleep data
    Sleep {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        /// Start of date range (YYYY-MM-DD)
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        /// End of date range (YYYY-MM-DD)
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Sleep score trends
    SleepScores {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        /// Number of days (default 7)
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Stress levels
    Stress {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Heart rate
    HeartRate {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Body battery
    BodyBattery {
        #[arg(long)]
        date: Option<String>,
    },
    /// Heart rate variability
    Hrv {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Step count
    Steps {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Weight
    Weight {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Hydration
    Hydration {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Blood oxygen (SpO2)
    Spo2 {
        #[arg(long)]
        date: Option<String>,
    },
    /// Respiration rate
    Respiration {
        #[arg(long)]
        date: Option<String>,
    },
    /// Intensity minutes
    IntensityMinutes {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
}

#[derive(Subcommand)]
enum TrainingCommands {
    /// Training status
    Status {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Training readiness
    Readiness {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// VO2max history
    #[command(alias = "scores")]
    Vo2max {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Race predictions (5K, 10K, half, marathon)
    RacePredictions,
    /// Endurance score
    EnduranceScore {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Hill score
    HillScore {
        #[arg(long, group = "date_selector")]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long, group = "date_selector")]
        from: Option<String>,
        #[arg(long, requires = "from")]
        to: Option<String>,
    },
    /// Fitness age
    FitnessAge {
        #[arg(long)]
        date: Option<String>,
    },
    /// Lactate threshold (speed and HR)
    LactateThreshold,
    /// Heart rate zones (from most recent running activity)
    #[command(alias = "zones")]
    HrZones,
}

#[derive(Subcommand)]
enum ActivityCommands {
    /// List recent activities
    List {
        /// Max activities to return
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Start index for pagination
        #[arg(long, default_value = "0")]
        start: u32,
        /// Filter by activity type (e.g. running, trail_running, cycling)
        #[arg(long, short = 't')]
        r#type: Option<String>,
        /// Only show activities after this date (YYYY-MM-DD)
        #[arg(long)]
        after: Option<String>,
        /// Only show activities before this date (YYYY-MM-DD)
        #[arg(long)]
        before: Option<String>,
    },
    /// Get activity summary
    Get {
        /// Activity ID
        id: u64,
    },
    /// Get full activity details (metrics, polyline, time-series)
    Details {
        /// Activity ID
        id: u64,
    },
    /// Get per-km lap splits (pace, HR, elevation per lap)
    Splits {
        /// Activity ID
        id: u64,
    },
    /// Get HR time in zones for an activity
    HrZones {
        /// Activity ID
        id: u64,
    },
    /// Get weather conditions during an activity
    Weather {
        /// Activity ID
        id: u64,
    },
    /// Get raw laps for an activity
    Laps {
        /// Activity ID
        id: u64,
    },
    /// Get exercise sets (structured intervals)
    Exercises {
        /// Activity ID
        id: u64,
    },
    /// Get power time in zones
    PowerZones {
        /// Activity ID
        id: u64,
    },
    /// Download activity file
    Download {
        /// Activity ID
        id: u64,
        /// File format
        #[arg(long, default_value = "fit")]
        format: DownloadFormat,
        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Upload activity file
    Upload {
        /// Path to FIT/GPX/TCX file
        file: String,
    },
    /// Compare two activities side-by-side
    Compare {
        /// First activity ID
        id1: u64,
        /// Second activity ID
        id2: u64,
    },
}

#[derive(Clone, ValueEnum)]
enum DownloadFormat {
    Fit,
    Gpx,
    Tcx,
}

impl std::fmt::Display for DownloadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fit => write!(f, "fit"),
            Self::Gpx => write!(f, "gpx"),
            Self::Tcx => write!(f, "tcx"),
        }
    }
}

#[derive(Subcommand)]
enum CalendarCommands {
    /// Remove a scheduled workout from the calendar
    Delete {
        /// Calendar entry ID
        id: u64,
    },
}

#[derive(Subcommand)]
enum WorkoutCommands {
    /// List saved workouts
    List {
        /// Max workouts to return
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Start index for pagination
        #[arg(long, default_value = "0")]
        start: u32,
    },
    /// Get workout details
    Get {
        /// Workout ID
        id: u64,
    },
    /// Create workout from JSON file
    Create {
        /// Path to workout JSON file
        #[arg(long, short)]
        file: String,
    },
    /// Schedule workout on a date
    Schedule {
        /// Workout ID
        id: u64,
        /// Date (YYYY-MM-DD)
        date: String,
    },
    /// Update an existing workout from JSON file
    Update {
        /// Workout ID
        id: u64,
        /// Path to workout JSON file
        #[arg(long, short)]
        file: String,
    },
    /// Delete a workout
    Delete {
        /// Workout ID
        id: u64,
    },
    /// Generate a workout template
    Template {
        /// Template type
        #[arg(long, default_value = "interval")]
        r#type: TemplateType,
    },
}

#[derive(Clone, ValueEnum)]
enum TemplateType {
    Interval,
    Tempo,
    Easy,
    LongRun,
}

#[derive(Subcommand)]
enum GearCommands {
    /// List all gear
    List,
    /// Get gear usage statistics
    Stats {
        /// Gear UUID
        uuid: String,
    },
    /// Link gear to an activity
    Link {
        /// Gear UUID
        uuid: String,
        /// Activity ID
        activity_id: u64,
    },
}

#[derive(Subcommand)]
enum DeviceCommands {
    /// List registered devices
    List,
    /// Get device details
    Get {
        /// Device ID
        id: u64,
    },
}

#[derive(Subcommand)]
enum CourseCommands {
    /// List saved courses
    List,
    /// Get course details
    Get {
        /// Course ID
        id: u64,
    },
}

#[derive(Subcommand)]
enum BadgeCommands {
    /// List earned badges
    List,
}

fn require_auth() -> anyhow::Result<Tokens> {
    Tokens::load().map_err(|e| anyhow::anyhow!("{e}"))
}

/// Resolve `--from`/`--to` into `(date, days)` for use with existing fetch_date_range.
fn resolve_date_range(
    date: Option<String>,
    days: Option<u32>,
    from: Option<String>,
    to: Option<String>,
) -> std::result::Result<(Option<String>, Option<u32>), Error> {
    if let Some(from_str) = from {
        let from_d = garmin_cli::util::parse_date(&from_str)?;
        let to_str = to.unwrap_or_else(garmin_cli::util::today);
        let to_d = garmin_cli::util::parse_date(&to_str)?;
        let day_count = (to_d - from_d).num_days() + 1;
        if day_count < 1 {
            return Err(Error::Api("--from must be before --to".into()));
        }
        Ok((Some(to_str), Some(day_count as u32)))
    } else {
        Ok((date, days))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output = cli.output();

    let result: std::result::Result<(), Error> = run(cli.command, &output).await;

    if let Err(e) = result {
        output.error_structured(&e);
        std::process::exit(e.exit_code());
    }

    Ok(())
}

async fn run(command: Commands, output: &Output) -> std::result::Result<(), Error> {
    match command {
        // --- Auth (no client needed) ---
        Commands::Auth { command } => match command {
            AuthCommands::Login => commands::auth::login(output).await,
            AuthCommands::Status => commands::auth::status(output),
            AuthCommands::Logout => commands::auth::logout(output),
        },

        // --- Completions ---
        Commands::Completions { shell } => {
            let shell = match shell {
                ShellChoice::Bash => clap_complete::Shell::Bash,
                ShellChoice::Zsh => clap_complete::Shell::Zsh,
                ShellChoice::Fish => clap_complete::Shell::Fish,
            };
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "garmin", &mut std::io::stdout());
            Ok(())
        }

        // --- Everything else needs auth ---
        Commands::Api { path, method, data } => {
            let client = GarminClient::new(require_auth()?)?;
            commands::raw::api(&client, output, &path, &method, data.as_deref()).await
        }

        Commands::Profile { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                ProfileCommands::Show => commands::profile::show(&client, output).await,
                ProfileCommands::Settings { command } => match command {
                    None => commands::profile::settings(&client, output).await,
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
                        commands::profile::settings_set(
                            &client,
                            output,
                            commands::profile::SettingsSetArgs {
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

        Commands::Summary {
            date,
            days,
            from,
            to,
        } => {
            let client = GarminClient::new(require_auth()?)?;
            let (date, days) = resolve_date_range(date, days, from, to)?;
            commands::summary::summary(&client, output, date.as_deref(), days).await
        }

        Commands::Health { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                HealthCommands::Sleep {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::sleep(&client, output, date.as_deref(), days).await
                }
                HealthCommands::SleepScores {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::sleep_scores(&client, output, date.as_deref(), days).await
                }
                HealthCommands::Stress {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::stress(&client, output, date.as_deref(), days).await
                }
                HealthCommands::HeartRate {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::heart_rate(&client, output, date.as_deref(), days).await
                }
                HealthCommands::BodyBattery { date } => {
                    commands::health::body_battery(&client, output, date.as_deref()).await
                }
                HealthCommands::Hrv {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::hrv(&client, output, date.as_deref(), days).await
                }
                HealthCommands::Steps {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::steps(&client, output, date.as_deref(), days).await
                }
                HealthCommands::Weight {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::weight(&client, output, date.as_deref(), days).await
                }
                HealthCommands::Hydration {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::hydration(&client, output, date.as_deref(), days).await
                }
                HealthCommands::Spo2 { date } => {
                    commands::health::spo2(&client, output, date.as_deref()).await
                }
                HealthCommands::Respiration { date } => {
                    commands::health::respiration(&client, output, date.as_deref()).await
                }
                HealthCommands::IntensityMinutes {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::health::intensity_minutes(&client, output, date.as_deref(), days)
                        .await
                }
            }
        }

        Commands::Training { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                TrainingCommands::Status {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::training::status(&client, output, date.as_deref(), days).await
                }
                TrainingCommands::Readiness {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::training::readiness(&client, output, date.as_deref(), days).await
                }
                TrainingCommands::Vo2max {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::training::scores(&client, output, date.as_deref(), days).await
                }
                TrainingCommands::RacePredictions => {
                    commands::training::race_predictions(&client, output).await
                }
                TrainingCommands::EnduranceScore {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::training::endurance_score(&client, output, date.as_deref(), days)
                        .await
                }
                TrainingCommands::HillScore {
                    date,
                    days,
                    from,
                    to,
                } => {
                    let (date, days) = resolve_date_range(date, days, from, to)?;
                    commands::training::hill_score(&client, output, date.as_deref(), days).await
                }
                TrainingCommands::FitnessAge { date } => {
                    commands::training::fitness_age(&client, output, date.as_deref()).await
                }
                TrainingCommands::LactateThreshold => {
                    commands::training::lactate_threshold(&client, output).await
                }
                TrainingCommands::HrZones => commands::training::zones(&client, output).await,
            }
        }

        Commands::Activities { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                ActivityCommands::List {
                    limit,
                    start,
                    r#type,
                    after,
                    before,
                } => {
                    commands::activities::list(
                        &client,
                        output,
                        limit,
                        start,
                        r#type.as_deref(),
                        after.as_deref(),
                        before.as_deref(),
                    )
                    .await
                }
                ActivityCommands::Get { id } => {
                    commands::activities::get(&client, output, id).await
                }
                ActivityCommands::Details { id } => {
                    commands::activities::details(&client, output, id).await
                }
                ActivityCommands::Splits { id } => {
                    commands::activities::splits(&client, output, id).await
                }
                ActivityCommands::HrZones { id } => {
                    commands::activities::hr_zones(&client, output, id).await
                }
                ActivityCommands::Weather { id } => {
                    commands::activities::weather(&client, output, id).await
                }
                ActivityCommands::Laps { id } => {
                    commands::activities::laps(&client, output, id).await
                }
                ActivityCommands::Exercises { id } => {
                    commands::activities::exercises(&client, output, id).await
                }
                ActivityCommands::PowerZones { id } => {
                    commands::activities::power_zones(&client, output, id).await
                }
                ActivityCommands::Download {
                    id,
                    format,
                    output: out,
                } => {
                    commands::activities::download(&client, id, &format.to_string(), out.as_deref())
                        .await
                }
                ActivityCommands::Upload { file } => {
                    commands::activities::upload(&client, output, &file).await
                }
                ActivityCommands::Compare { id1, id2 } => {
                    commands::activities::compare(&client, output, id1, id2).await
                }
            }
        }

        Commands::Workouts { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                WorkoutCommands::List { limit, start } => {
                    commands::workouts::list(&client, output, limit, start).await
                }
                WorkoutCommands::Get { id } => commands::workouts::get(&client, output, id).await,
                WorkoutCommands::Create { file } => {
                    commands::workouts::create(&client, output, &file).await
                }
                WorkoutCommands::Schedule { id, date } => {
                    commands::workouts::schedule(&client, output, id, &date).await
                }
                WorkoutCommands::Update { id, file } => {
                    commands::workouts::update(&client, output, id, &file).await
                }
                WorkoutCommands::Delete { id } => {
                    commands::workouts::delete(&client, output, id).await
                }
                WorkoutCommands::Template { r#type } => {
                    let kind = match r#type {
                        TemplateType::Interval => "interval",
                        TemplateType::Tempo => "tempo",
                        TemplateType::Easy => "easy",
                        TemplateType::LongRun => "long_run",
                    };
                    commands::workouts::template(output, kind);
                    Ok(())
                }
            }
        }

        Commands::Courses { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                CourseCommands::List => commands::courses::list(&client, output).await,
                CourseCommands::Get { id } => commands::courses::get(&client, output, id).await,
            }
        }

        Commands::Badges { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                BadgeCommands::List => commands::badges::list(&client, output).await,
            }
        }

        Commands::Gear { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                GearCommands::List => commands::gear::list(&client, output).await,
                GearCommands::Stats { uuid } => commands::gear::stats(&client, output, &uuid).await,
                GearCommands::Link { uuid, activity_id } => {
                    commands::gear::link(&client, output, &uuid, activity_id).await
                }
            }
        }

        Commands::Records => {
            let client = GarminClient::new(require_auth()?)?;
            commands::records::list(&client, output).await
        }

        Commands::Calendar {
            command,
            year,
            month,
            weeks,
        } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                Some(CalendarCommands::Delete { id }) => {
                    commands::calendar::delete(&client, output, id).await
                }
                None => commands::calendar::list(&client, output, year, month, weeks).await,
            }
        }

        Commands::Devices { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                DeviceCommands::List => commands::devices::list(&client, output).await,
                DeviceCommands::Get { id } => commands::devices::get(&client, output, id).await,
            }
        }
    }
}
