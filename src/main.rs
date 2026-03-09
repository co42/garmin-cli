use clap::{Parser, Subcommand, ValueEnum};
use garmin_cli::auth::Tokens;
use garmin_cli::commands;
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
        Output::new(json, self.quiet, self.fields.clone())
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
        #[arg(long)]
        date: Option<String>,
        /// Number of days
        #[arg(long)]
        days: Option<u32>,
    },
    /// Health metrics
    Health {
        #[command(subcommand)]
        command: HealthCommands,
    },
    /// Training status and readiness
    Training {
        #[command(subcommand)]
        command: TrainingCommands,
    },
    /// Activities
    Activities {
        #[command(subcommand)]
        command: ActivityCommands,
    },
    /// Devices
    Devices {
        #[command(subcommand)]
        command: DeviceCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Log in to Garmin Connect
    Login {
        /// Garmin account email
        #[arg(long, env = "GARMIN_EMAIL")]
        username: Option<String>,
        /// Garmin account password
        #[arg(long, env = "GARMIN_PASSWORD")]
        password: Option<String>,
    },
    /// Show authentication status
    Status,
    /// Log out (delete stored tokens)
    Logout,
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// Show user profile
    Show,
    /// Show user settings
    Settings,
}

#[derive(Subcommand)]
enum HealthCommands {
    /// Sleep data
    Sleep {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Stress levels
    Stress {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Heart rate
    HeartRate {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Body battery
    BodyBattery {
        #[arg(long)]
        date: Option<String>,
    },
    /// Heart rate variability
    Hrv {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Step count
    Steps {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Weight
    Weight {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Hydration
    Hydration {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
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
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
}

#[derive(Subcommand)]
enum TrainingCommands {
    /// Training status
    Status {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Training readiness
    Readiness {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
    /// Training scores (endurance, VO2max, etc.)
    Scores {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
    },
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
    },
    /// Get activity details
    Get {
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
enum DeviceCommands {
    /// List registered devices
    List,
    /// Get device details
    Get {
        /// Device ID
        id: u64,
    },
}

fn require_auth() -> anyhow::Result<Tokens> {
    Tokens::load().map_err(|e| anyhow::anyhow!("{e}"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output = cli.output();

    let result = match cli.command {
        // --- Auth (no client needed) ---
        Commands::Auth { command } => match command {
            AuthCommands::Login { username, password } => {
                commands::auth::login(&output, username, password).await
            }
            AuthCommands::Status => commands::auth::status(&output),
            AuthCommands::Logout => commands::auth::logout(&output),
        },

        // --- Everything else needs auth ---
        Commands::Api { path, method, data } => {
            let client = GarminClient::new(require_auth()?)?;
            commands::raw::api(&client, &output, &path, &method, data.as_deref()).await
        }

        Commands::Profile { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                ProfileCommands::Show => commands::profile::show(&client, &output).await,
                ProfileCommands::Settings => commands::profile::settings(&client, &output).await,
            }
        }

        Commands::Summary { date, days } => {
            let client = GarminClient::new(require_auth()?)?;
            commands::summary::summary(&client, &output, date.as_deref(), days).await
        }

        Commands::Health { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                HealthCommands::Sleep { date, days } => {
                    commands::health::sleep(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::Stress { date, days } => {
                    commands::health::stress(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::HeartRate { date, days } => {
                    commands::health::heart_rate(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::BodyBattery { date } => {
                    commands::health::body_battery(&client, &output, date.as_deref()).await
                }
                HealthCommands::Hrv { date, days } => {
                    commands::health::hrv(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::Steps { date, days } => {
                    commands::health::steps(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::Weight { date, days } => {
                    commands::health::weight(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::Hydration { date, days } => {
                    commands::health::hydration(&client, &output, date.as_deref(), days).await
                }
                HealthCommands::Spo2 { date } => {
                    commands::health::spo2(&client, &output, date.as_deref()).await
                }
                HealthCommands::Respiration { date } => {
                    commands::health::respiration(&client, &output, date.as_deref()).await
                }
                HealthCommands::IntensityMinutes { date, days } => {
                    commands::health::intensity_minutes(&client, &output, date.as_deref(), days)
                        .await
                }
            }
        }

        Commands::Training { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                TrainingCommands::Status { date, days } => {
                    commands::training::status(&client, &output, date.as_deref(), days).await
                }
                TrainingCommands::Readiness { date, days } => {
                    commands::training::readiness(&client, &output, date.as_deref(), days).await
                }
                TrainingCommands::Scores { date, days } => {
                    commands::training::scores(&client, &output, date.as_deref(), days).await
                }
            }
        }

        Commands::Activities { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                ActivityCommands::List { limit, start } => {
                    commands::activities::list(&client, &output, limit, start).await
                }
                ActivityCommands::Get { id } => {
                    commands::activities::get(&client, &output, id).await
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
                    commands::activities::upload(&client, &output, &file).await
                }
            }
        }

        Commands::Devices { command } => {
            let client = GarminClient::new(require_auth()?)?;
            match command {
                DeviceCommands::List => commands::devices::list(&client, &output).await,
                DeviceCommands::Get { id } => commands::devices::get(&client, &output, id).await,
            }
        }
    };

    if let Err(e) = result {
        output.error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}
