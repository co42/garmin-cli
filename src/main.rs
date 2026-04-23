mod commands;
mod config;
mod error;
mod garmin;
mod tracing;

use std::process::exit;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use commands::helpers::DateRangeArgs;
use commands::{
    activities::ActivityCommands, auth::AuthCommands, badges::BadgeCommands, calendar::CalendarCommands,
    coach::CoachCommands, courses::CourseCommands, devices::DeviceCommands, gear::GearCommands, health::HealthCommands,
    output::Output, profile::ProfileCommands, training::TrainingCommands, workouts::WorkoutCommands,
};
use error::Error;

#[derive(Parser)]
#[command(name = "garmin", about = "Garmin Connect CLI", version)]
struct Cli {
    /// Output as JSON instead of human-readable text
    #[arg(long, global = true)]
    json: bool,

    /// Filter output fields (comma-separated, JSON mode only)
    #[arg(long, global = true, value_delimiter = ',')]
    fields: Vec<String>,

    /// Log every Garmin API call (request + response) to stderr
    #[arg(long, short, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn output(&self) -> Output {
        Output::new(self.json, self.fields.clone())
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Activities
    Activities {
        #[command(subcommand)]
        command: ActivityCommands,
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
    /// Authentication (login, status, logout)
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Earned badges and achievements
    Badges {
        #[command(subcommand)]
        command: BadgeCommands,
    },
    /// Calendar (scheduled workouts, activities)
    Calendar {
        #[command(subcommand)]
        command: CalendarCommands,
    },
    /// Garmin Coach (adaptive training plan)
    Coach {
        #[command(subcommand)]
        command: CoachCommands,
    },
    /// Courses (saved routes)
    Courses {
        #[command(subcommand)]
        command: CourseCommands,
    },
    /// Devices
    Devices {
        #[command(subcommand)]
        command: DeviceCommands,
    },
    /// Gear (shoes, bikes, etc.)
    Gear {
        #[command(subcommand)]
        command: GearCommands,
    },
    /// Health metrics
    Health {
        #[command(subcommand)]
        command: HealthCommands,
    },
    /// User profile
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Personal records
    Records,
    /// Daily summary
    Summary {
        #[command(flatten)]
        range: DateRangeArgs,
    },
    /// Training status, readiness, and performance metrics
    Training {
        #[command(subcommand)]
        command: TrainingCommands,
    },
    /// Workouts (create, schedule, push to watch)
    Workouts {
        #[command(subcommand)]
        command: WorkoutCommands,
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let output = cli.output();

    if cli.verbose > 0 {
        tracing::init();
    }

    let result: std::result::Result<(), Error> = run(cli.command, &output).await;

    if let Err(err) = result {
        output.error_structured(&err);
        exit(1);
    }
}

async fn run(command: Commands, output: &Output) -> std::result::Result<(), Error> {
    match command {
        Commands::Auth { command } => commands::auth::run(command, output).await,
        Commands::Api { path, method, data } => commands::raw::run(&path, &method, data.as_deref(), output).await,
        Commands::Profile { command } => commands::profile::run(command, output).await,
        Commands::Summary { range } => commands::summary::run(range, output).await,
        Commands::Health { command } => commands::health::run(command, output).await,
        Commands::Training { command } => commands::training::run(command, output).await,
        Commands::Activities { command } => commands::activities::run(command, output).await,
        Commands::Workouts { command } => commands::workouts::run(command, output).await,
        Commands::Coach { command } => commands::coach::run(command, output).await,
        Commands::Courses { command } => commands::courses::run(command, output).await,
        Commands::Badges { command } => commands::badges::run(command, output).await,
        Commands::Gear { command } => commands::gear::run(command, output).await,
        Commands::Records => commands::records::run(output).await,
        Commands::Calendar { command } => commands::calendar::run(command, output).await,
        Commands::Devices { command } => commands::devices::run(command, output).await,
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
    }
}
