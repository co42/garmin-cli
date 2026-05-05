use super::helpers::DateRangeArgs;
use super::output::Output;
use crate::error::{Error, Result};
use crate::garmin::{CoachEvent, CoachWorkout, GarminClient, TargetEvent};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CoachCommands {
    /// List coach workouts (including alternate variants)
    List,
    /// Get a specific coach workout by UUID
    Get {
        /// Workout UUID
        uuid: String,
    },
    /// Show the active training plan (default) or manage plans
    Plan {
        #[command(subcommand)]
        cmd: Option<PlanCmd>,
    },
    /// Show the target event and projection history
    Event {
        /// Specific event ID. Defaults to the active plan's primary event.
        #[arg(long)]
        event_id: Option<u64>,
        #[command(flatten)]
        range: DateRangeArgs,
    },
}

#[derive(Subcommand)]
pub enum PlanCmd {
    /// List all training plans (active + completed)
    List,
}

pub async fn run(command: CoachCommands, output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    match command {
        CoachCommands::List => list(&client, output).await,
        CoachCommands::Get { uuid } => get(&client, output, &uuid).await,
        CoachCommands::Plan { cmd: None } => plan_active(&client, output).await,
        CoachCommands::Plan {
            cmd: Some(PlanCmd::List),
        } => plan_list(&client, output).await,
        CoachCommands::Event { event_id, range } => event(&client, output, event_id, range).await,
    }
}

async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let workouts = client.list_coach_workouts().await?;
    // JSON keeps every workout (including rest days); human view drops rest
    // since they carry no useful detail beyond their existence.
    let items: Vec<CoachWorkout> = if output.is_json() {
        workouts
    } else {
        workouts
            .into_iter()
            .filter(|w| !matches!(w.workout_phrase.as_deref(), Some("FORCED_REST" | "EASY_WEEK_LOAD_REST")))
            .collect()
    };
    output.print_list(&items, "Coach Workouts");
    Ok(())
}

async fn get(client: &GarminClient, output: &Output, uuid: &str) -> Result<()> {
    let workout = client.coach_workout(uuid).await?;
    output.print(&workout);
    Ok(())
}

async fn plan_active(client: &GarminClient, output: &Output) -> Result<()> {
    let plan_id = active_plan_id(client).await?;
    let plan = client.training_plan(plan_id).await?;
    output.print(&plan);
    Ok(())
}

async fn plan_list(client: &GarminClient, output: &Output) -> Result<()> {
    let plans = client.list_training_plans().await?;
    output.print_list(&plans, "Training plans");
    Ok(())
}

async fn event(
    client: &GarminClient,
    output: &Output,
    event_id_override: Option<u64>,
    range: DateRangeArgs,
) -> Result<()> {
    // Two modes:
    //   1. Caller passes `--event-id`: skip the plan lookup, fetch that event
    //      directly. The plan_id/plan_name fields stay None — the event may
    //      not be tied to a plan at all.
    //   2. No flag: original behavior — find the active plan's primary event.
    let (event_id, plan_id) = match event_id_override {
        Some(id) => (id, None),
        None => {
            let plan_id = active_plan_id(client).await?;
            let events = client.list_events(Some(plan_id), None, None).await?;
            let target =
                pick_primary(events).ok_or_else(|| Error::NotFound("no target event for active plan".into()))?;
            (target.id, Some(plan_id))
        }
    };

    let (start, end) = range.resolve(1)?;
    let (detail, mut projections) = tokio::try_join!(
        client.calendar_event(event_id),
        client.event_projections(event_id, start, end)
    )?;

    projections.sort_by(|a, b| b.calendar_date.cmp(&a.calendar_date));

    // Best-effort lookup of the plan name for the header. Don't fail the
    // command if the plan endpoint errors — the event itself rendered fine.
    let plan_name = match plan_id {
        Some(id) => client.training_plan(id).await.ok().map(|p| p.name),
        None => None,
    };

    let ce = CoachEvent {
        event: detail,
        plan_id,
        plan_name,
        projections,
    };
    output.print(&ce);
    Ok(())
}

/// Pick the event flagged `isPrimaryEvent`; fall back to the first entry.
fn pick_primary(events: Vec<TargetEvent>) -> Option<TargetEvent> {
    let mut primary: Option<TargetEvent> = None;
    let mut first: Option<TargetEvent> = None;
    for e in events {
        if e.is_primary_event == Some(true) {
            primary = Some(e);
            break;
        }
        if first.is_none() {
            first = Some(e);
        }
    }
    primary.or(first)
}

async fn active_plan_id(client: &GarminClient) -> Result<u64> {
    let workouts = client.list_coach_workouts().await?;
    workouts
        .iter()
        .find_map(|w| w.training_plan_id)
        .ok_or_else(|| Error::NotFound("no active Coach training plan".into()))
}
