# Plan: expose the coach's target event & projections

## Goal

Surface the data Garmin Connect already has about the user's current adaptive
training plan: the scheduled workout list, the training phases (BUILD / PEAK /
TAPER / RACE), the target race event (when / where / goal), and — most
importantly — the per-event race-time **projection history** that tells the
user whether they're on track for their goal.

Today `coach plan` only returns a thin header (name, dates, level, status).
Everything else is available via API but unused.

## Scope

Three user-visible changes under the existing `coach` subcommand:

1. **`coach plan`** — enhance: switch to the adaptive endpoint so the response
   also includes the task list and phase timeline. Render both.
2. **`coach plan list`** — new subcommand under `plan`: list all user plans
   (active + completed).
3. **`coach event [--days N | --from A --to B]`** — new: target event (when /
   where / goal) + projection history. Always returns the same shape; the
   flags control how much history is fetched (default 1 day = today).

No new top-level subcommands. No changes to `training race-predictions` —
that's the generic 5K/10K/HM/M predictor, a different question.

## Non-goals

- Creating, editing, or deleting training plans or events. Read-only.
- Searching Garmin's plan catalog (`/trainingplan-service/trainingplan/search`).
- Race-day weather. The web UI calls `/it-proxy/gcs/weather/...` which sits
  behind a cookie-session path; reachability from our OAuth2 flow is unknown
  and the HAR capture doesn't prove it works from the CLI. Ship projections
  first; revisit weather in a separate PR.
- Any UI for ATP-style (non-adaptive) plans. All fixtures use `FBT_ADAPTIVE`;
  if `list_coach_workouts` yields a non-adaptive plan ID the command should
  return `Error::NotFound("no active adaptive plan")`.

## Conventions to match

The codebase was just normalized; new code must follow the same patterns.

- **Naming**: every numeric field carries a unit suffix: `_seconds`, `_meters`,
  `_mps`, `_cm`, `_bpm`, `_watts`, `_kg`, `_ml`, `_percent`, `_joules`. No
  `*_in_secs`, `*_in_meters`, or `*_meters_per_second` (those are outliers in
  `course.rs` and `lactate.rs`; don't propagate them).
- **serde attributes**: struct-level `#[skip_serializing_none]` (from
  `serde_with`) on any struct with ≥ 2 `Option` fields — **never** per-field
  `skip_serializing_if`. `#[serde(default)]` only on non-`Option` fields that
  need a default (Option already defaults to None). Use
  `#[serde(rename_all(deserialize = "camelCase"))]` at the struct level;
  `#[serde(rename(deserialize = "apiKey"))]` only on fields whose API name
  doesn't match camelCase-to-snake (e.g. unit-suffixed renames).
- **Module shape**: one subdomain per file; a thin `mod.rs` with
  `mod foo; pub use foo::*;` lines. Follow `types/training/`, `types/health/`,
  `types/activity/` as the reference layout.
- **Human output**: `LABEL_WIDTH = 16`; `.bold()` title, `"\u{2500}".repeat(40)
  .dimmed()` separator, `.cyan()` for the primary metric on each line, section
  subtitles `.bold()` + 38-wide separator. Band ranges use `\u{2013}` (`–`)
  between values. Uses `fmt_hms` / `fmt_hm` / `fmt_pace_per_km` /
  `pace_from_speed` from `types/helpers.rs`.
- **Client**: all paths go through `self.get("/service-name/...")`. `build_url`
  prefixes `CONNECT_API` — `it-proxy` and `gc-api` are **not** special
  (checked: no path in the codebase uses either prefix). Independent fetches
  parallelized with `tokio::try_join!`.
- **Commands**: one subcommand per file under `commands/`; `clap::Subcommand`;
  `run(cmd, output)` dispatcher. `DateRangeArgs` for every windowed view.

## API surface

All paths are appended to `CONNECT_API = https://connectapi.garmin.com`.

### 1. Adaptive plan detail

`GET /trainingplan-service/trainingplan/fbt-adaptive/{planId}`

Returns the same shape as the non-adaptive endpoint **plus**:

```json
{
  "trainingPlanId": 45053764,
  "name": "Programme Semi 4:30",
  "durationInWeeks": 7,
  "avgWeeklyWorkouts": 5,
  "startDate": "2026-04-13T00:00:00.0",
  "endDate":   "2026-05-30T00:00:00.0",
  "trainingLevel":   {"levelKey": "Intermediate"},
  "trainingVersion": {"versionName": "Pace"},
  "trainingStatus":  {"statusKey": "Scheduled"},
  "supplementalSports": ["STRENGTH_TRAINING_BODYWEIGHT"],

  "taskList": [
    {
      "weekId": 2,
      "dayOfWeekId": 5,
      "workoutOrder": 1,
      "calendarDate": "2026-04-24",
      "taskWorkout": {
        "sportType": {"sportTypeKey": "running"},
        "workoutName": "Sprint",
        "workoutDescription": "2x3x0:15@2:45/km",
        "scheduledDate": "2026-04-24T05:00:00.0",
        "workoutUuid": "ad88bf33-…",
        "estimatedDurationInSecs": 2640,
        "estimatedDistanceInMeters": 7050,
        "trainingEffectLabel": "SPEED",
        "workoutPhrase": "ANAEROBIC_SPEED",
        "restDay": false,
        "adaptiveCoachingWorkoutStatus": "NOT_COMPLETE"
      }
    }
    /* Rest days have `taskWorkout.sportType: null`, no distance/duration,
       `restDay: true`. */
  ],

  "adaptivePlanPhases": [
    {"startDate": "2026-04-13", "endDate": "2026-04-29", "trainingPhase": "BUILD",             "currentPhase": true},
    {"startDate": "2026-04-30", "endDate": "2026-05-20", "trainingPhase": "PEAK",              "currentPhase": false},
    {"startDate": "2026-05-21", "endDate": "2026-05-29", "trainingPhase": "TAPER",             "currentPhase": false},
    {"startDate": "2026-05-30", "endDate": "2026-05-30", "trainingPhase": "TARGET_EVENT_DAY",  "currentPhase": false}
  ]
}
```

`planPhases` is the non-adaptive twin of `adaptivePlanPhases` and may or may
not be present; prefer `adaptivePlanPhases`, fall back to `planPhases`.

### 2. List of user plans

`GET /trainingplan-service/trainingplan/plans?limit=50`

```json
{
  "trainingPlanList": [
    {
      "trainingPlanId": 45053764,
      "trainingPlanCategory": "FBT_ADAPTIVE",
      "name": "Programme Semi 4:30",
      "durationInWeeks": 7,
      "startDate": "2026-04-13T00:00:00.0",
      "endDate":   "2026-05-30T00:00:00.0",
      "trainingLevel":   {"levelKey": "Intermediate"},
      "trainingVersion": {"versionName": "Pace"},
      "trainingStatus":  {"statusKey": "Scheduled"}    /* or "Completed" */
    }
  ]
}
```

### 3. Training-plan → event lookup

`GET /calendar-service/events?trainingPlanId={planId}`

Returns a JSON **array** of race events associated with the plan (usually one,
the target):

```json
[
  {"id": 27334883, "eventName": "Programme Semi 1h30",
   "eventCustomization": {"isPrimaryEvent": true, "trainingPlanId": 45053764}}
]
```

Pick the entry with `eventCustomization.isPrimaryEvent == true`. If none is
marked primary, take the first.

### 4. Event detail

`GET /calendar-service/event/{eventId}`

```json
{
  "id": 27334883,
  "eventName": "Programme Semi 1h30",
  "date": "2026-05-30",
  "eventType": "running",
  "eventTimeLocal": {"startTimeHhMm": "09:00", "timeZoneId": "Europe/Paris"},
  "location": "01120 Thil, France",
  "locationStartPoint": {"lat": 45.81357, "lon": 5.022083},
  "completionTarget": {"value": 21.1, "unit": "kilometer", "unitType": "distance"},
  "eventCustomization": {
    "customGoal": {"value": 5400.0, "unit": "second", "unitType": "time"},
    "isPrimaryEvent": true,
    "trainingPlanId": 45053764,
    "trainingPlanType": "FBT_ADAPTIVE",
    "projectedRaceTimeDurationSeconds": 5447,
    "predictedRaceTimeDurationSeconds": 5528,
    "projectedRaceSpeed": 3.873691940517716,
    "predictedRaceSpeed": 3.816931982633864,
    "enrollmentTime": "2026-04-13T03:42:42.430"
  }
}
```

Notes:
- `completionTarget.unit` can be `kilometer` or `mile`. `customGoal.unit` is
  always `second` for time goals. Don't assume — read the unit.
- `eventCustomization` may be absent for events the user didn't customize.

### 5. Event projection history ⭐

`GET /metrics-service/metrics/eventracetimeprojections/{eventId}/{startDate}/{endDate}`

Dates are `YYYY-MM-DD`. Returns an array of per-day snapshots (not every day
in the range has an entry — Garmin fills where it has data):

```json
[
  {
    "calendarDate": "2026-04-23",
    "sportingEventId": 27334883,
    "predictedRaceTime": 5528,
    "projectionRaceTime": 5447,
    "upperBoundProjectionRaceTime": 5513,
    "lowerBoundProjectionRaceTime": 5382,
    "eventRacePredictionsFeedbackPhrase": "IMPROVED_VO2MAX",
    "speedPrediction": 3.816931982633864,
    "speedProjection": 3.873691940517716,
    "upperBoundProjectionSpeed": 3.827317250136042,
    "lowerBoundProjectionSpeed": 3.9204756596060943
  }
]
```

Response order in the fixture is descending by date. Sort descending in the
CLI regardless of API order.

Field semantics (all durations in seconds, all speeds in m/s):
- `predictedRaceTime` — fitness-baseline race predictor (≈ what the generic
  predictor says).
- `projectionRaceTime` — plan-adjusted projection. **Usually faster** than
  prediction because it credits plan progress.
- `upperBoundProjectionRaceTime` / `lowerBoundProjectionRaceTime` — confidence
  band on the projection. Upper-bound is the slower time (higher seconds);
  lower-bound is the faster time. Display as `lower–upper`.
- `eventRacePredictionsFeedbackPhrase` — coaching focus enum. See mapping
  table below.

## Code changes

### New subdirectory: `src/garmin/types/coach/`

`coach.rs` is currently 154 lines; adding 9 new types pushes it past 500.
Split now, same pattern as `training/`, `health/`, `activity/`:

```
types/coach/
  mod.rs         — module wiring + shared helpers (humanize_phrase,
                    format_te, supplemental_label, phase_label,
                    feedback_phrase_label)
  workout.rs     — CoachWorkout (moved from current coach.rs)
  plan.rs        — CoachPlan + TrainingLevel/Version/StatusRef
                    + TrainingPlanSummary + TrainingPlanListResponse
  task.rs        — CoachTask + CoachTaskWorkout
  phase.rs       — TrainingPhase
  event.rs       — TargetEvent + EventTime + UnitValue +
                    EventCustomization + CoachEvent aggregate
  projection.rs  — EventProjection
```

`mod.rs`:
```rust
mod event;
mod phase;
mod plan;
mod projection;
mod task;
mod workout;

pub use event::*;
pub use phase::*;
pub use plan::*;
pub use projection::*;
pub use task::*;
pub use workout::*;
```

### `types/coach/workout.rs`

Moved verbatim from the existing `coach.rs` **with two fixes** to bring it
into line with the rest of the codebase:

```rust
// Renamed from estimated_duration_in_secs; rename(deserialize) preserves wire format.
#[serde(rename(deserialize = "estimatedDurationInSecs"))]
pub estimated_duration_seconds: Option<f64>,
// Renamed from estimated_distance_in_meters.
#[serde(rename(deserialize = "estimatedDistanceInMeters"))]
pub estimated_distance_meters: Option<f64>,
```

Update `impl HumanReadable for CoachWorkout` accordingly (2 references).

### `types/coach/plan.rs`

Extended `CoachPlan`:

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CoachPlan {
    #[serde(default)]
    pub training_plan_id: u64,
    #[serde(default = "unknown")]
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub duration_in_weeks: Option<u32>,
    pub training_level: Option<TrainingLevel>,
    pub avg_weekly_workouts: Option<u32>,
    pub training_version: Option<TrainingVersion>,
    pub training_status: Option<TrainingStatusRef>,

    // NEW: only populated by the fbt-adaptive endpoint.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub task_list: Vec<CoachTask>,

    // Prefer `adaptivePlanPhases`; fall back to `planPhases` via alias.
    #[serde(
        rename(deserialize = "adaptivePlanPhases"),
        alias = "planPhases",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub phases: Vec<TrainingPhase>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supplemental_sports: Vec<String>,
}
```

`skip_serializing_if` on `Vec` fields stays per-field — `skip_serializing_none`
only affects `Option`.

New `TrainingPlanSummary` + list wrapper:

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingPlanListResponse {
    #[serde(default)]
    pub training_plan_list: Vec<TrainingPlanSummary>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingPlanSummary {
    pub training_plan_id: u64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub duration_in_weeks: Option<u32>,
    pub training_plan_category: Option<String>,   // "FBT_ADAPTIVE" | ...
    pub training_level: Option<TrainingLevel>,
    pub training_version: Option<TrainingVersion>,
    pub training_status: Option<TrainingStatusRef>,
}

impl HumanReadable for TrainingPlanSummary { ... }  // see mockup
```

`impl HumanReadable for CoachPlan` extends with three new sections —
**Supplemental** (if non-empty), **Phases**, **Upcoming** (tasks). See
mockups.

### `types/coach/task.rs`

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CoachTask {
    pub calendar_date: String,                  // "2026-04-24"
    pub week_id: Option<u32>,
    pub day_of_week_id: Option<u32>,
    pub workout_order: Option<u32>,
    pub task_workout: CoachTaskWorkout,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CoachTaskWorkout {
    pub workout_uuid: Option<String>,
    pub workout_name: Option<String>,
    pub workout_description: Option<String>,
    // Nested {"sportTypeKey": "running"} — null for rest days.
    pub sport_type: Option<SportTypeRef>,
    #[serde(rename(deserialize = "estimatedDurationInSecs"))]
    pub estimated_duration_seconds: Option<f64>,
    #[serde(rename(deserialize = "estimatedDistanceInMeters"))]
    pub estimated_distance_meters: Option<f64>,
    pub training_effect_label: Option<String>,  // "SPEED" | "VO2MAX" | ...
    pub workout_phrase: Option<String>,         // "ANAEROBIC_SPEED" | ...
    #[serde(default)]
    pub rest_day: bool,
    pub adaptive_coaching_workout_status: Option<String>,
}
```

Reuse `SportTypeRef` from `types/workout.rs` (already nested `{sportTypeKey}`
shape; deserializes from `null` as `None` because it's `Option<_>`).

No per-task `HumanReadable` impl; rendering is done by `CoachPlan::print_human`
grouping tasks into an "Upcoming" section.

### `types/coach/phase.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TrainingPhase {
    pub start_date: String,
    pub end_date: String,
    pub training_phase: String,   // "BUILD" | "PEAK" | "TAPER" | "TARGET_EVENT_DAY"
    #[serde(default)]
    pub current_phase: bool,
}
```

### `types/coach/event.rs`

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TargetEvent {
    pub id: u64,
    pub event_name: String,
    pub date: String,                            // "2026-05-30"
    pub event_type: Option<String>,
    pub event_time_local: Option<EventTime>,
    pub location: Option<String>,
    // locationStartPoint.{lat,lon} — intentionally dropped from the view.
    // We have no consumer for coordinates now that weather is out of scope.
    pub completion_target: Option<UnitValue>,
    pub event_customization: Option<EventCustomization>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventTime {
    pub start_time_hh_mm: String,                // "09:00"
    pub time_zone_id: String,                    // "Europe/Paris"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitValue {
    pub value: f64,
    pub unit: String,                            // "kilometer" | "mile" | "second"
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventCustomization {
    pub custom_goal: Option<UnitValue>,
    #[serde(default)]
    pub is_primary_event: bool,
    pub training_plan_id: Option<u64>,
    pub training_plan_type: Option<String>,
    #[serde(rename(deserialize = "projectedRaceTimeDurationSeconds"))]
    pub projected_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "predictedRaceTimeDurationSeconds"))]
    pub predicted_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "projectedRaceSpeed"))]
    pub projected_race_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "predictedRaceSpeed"))]
    pub predicted_race_speed_mps: Option<f64>,
    pub enrollment_time: Option<String>,
}

/// Aggregate rendered by `coach event`. JSON shape is stable across modes:
/// `projections` has 1 entry for the today-snapshot, N for history.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct CoachEvent {
    pub event: TargetEvent,
    pub plan_id: Option<u64>,
    pub plan_name: Option<String>,
    pub projections: Vec<EventProjection>,       // sorted desc by date
}
```

`impl HumanReadable for CoachEvent` branches on `projections.len()`:
- **1** → today-snapshot layout (below).
- **≥ 2** → history table layout (below).
- **0** → snapshot layout with `(no projection yet)` dimmed.

### `types/coach/projection.rs`

All numeric fields get unit suffixes.

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EventProjection {
    pub calendar_date: String,
    pub sporting_event_id: u64,

    #[serde(rename(deserialize = "predictedRaceTime"))]
    pub predicted_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "projectionRaceTime"))]
    pub projection_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "upperBoundProjectionRaceTime"))]
    pub upper_bound_projection_race_time_seconds: Option<f64>,
    #[serde(rename(deserialize = "lowerBoundProjectionRaceTime"))]
    pub lower_bound_projection_race_time_seconds: Option<f64>,

    #[serde(rename(deserialize = "speedPrediction"))]
    pub speed_prediction_mps: Option<f64>,
    #[serde(rename(deserialize = "speedProjection"))]
    pub speed_projection_mps: Option<f64>,
    #[serde(rename(deserialize = "upperBoundProjectionSpeed"))]
    pub upper_bound_projection_speed_mps: Option<f64>,
    #[serde(rename(deserialize = "lowerBoundProjectionSpeed"))]
    pub lower_bound_projection_speed_mps: Option<f64>,

    pub event_race_predictions_feedback_phrase: Option<String>,
}
```

### `src/garmin/types/mod.rs`

Remove `pub mod coach;` + `pub use coach::*;`, replace with `pub mod coach;`
(unchanged) but the `coach.rs` file is gone and `coach/` takes its place. The
`pub use coach::*;` still works because `coach/mod.rs` re-exports everything.

### `src/garmin/client.rs` — coach section rewrite

```rust
// ── Coach ────────────────────────────────────────────────────────

pub async fn list_coach_workouts(&self) -> Result<Vec<CoachWorkout>> {
    self.get("/workout-service/fbt-adaptive").await
}

pub async fn coach_workout(&self, uuid: &str) -> Result<CoachWorkout> {
    let path = format!("/workout-service/fbt-adaptive/{uuid}");
    self.get(&path).await
}

/// Adaptive plan detail. Returns task list, phases, and supplemental sports
/// in addition to the fields the non-adaptive endpoint exposes. Falls back
/// to the non-adaptive endpoint on 404 so non-adaptive plans still render
/// (without tasks/phases).
pub async fn training_plan(&self, plan_id: u64) -> Result<CoachPlan> {
    let adaptive = format!("/trainingplan-service/trainingplan/fbt-adaptive/{plan_id}");
    match self.get::<CoachPlan>(&adaptive).await {
        Ok(plan) => Ok(plan),
        Err(Error::Http { status: 404, .. }) => {
            let fallback = format!("/trainingplan-service/trainingplan/{plan_id}");
            self.get(&fallback).await
        }
        Err(e) => Err(e),
    }
}

pub async fn list_training_plans(&self) -> Result<Vec<TrainingPlanSummary>> {
    let res: TrainingPlanListResponse =
        self.get("/trainingplan-service/trainingplan/plans?limit=50").await?;
    Ok(res.training_plan_list)
}

pub async fn plan_events(&self, plan_id: u64) -> Result<Vec<TargetEvent>> {
    let path = format!("/calendar-service/events?trainingPlanId={plan_id}");
    self.get(&path).await
}

pub async fn calendar_event(&self, event_id: u64) -> Result<TargetEvent> {
    let path = format!("/calendar-service/event/{event_id}");
    self.get(&path).await
}

pub async fn event_projections(
    &self, event_id: u64, start: NaiveDate, end: NaiveDate,
) -> Result<Vec<EventProjection>> {
    let (start, end) = (ymd(start), ymd(end));
    let path = format!(
        "/metrics-service/metrics/eventracetimeprojections/{event_id}/{start}/{end}"
    );
    self.get(&path).await
}
```

`ymd` already exists as a module-local helper in `client.rs`.

### `src/commands/coach.rs`

```rust
#[derive(Subcommand)]
pub enum CoachCommands {
    /// List coach workouts (including alternate variants)
    List,
    /// Get a specific coach workout by UUID
    Get { uuid: String },
    /// Show the active training plan (default) or manage plans
    Plan {
        #[command(subcommand)]
        cmd: Option<PlanCmd>,
    },
    /// Show the target event and projection history
    Event {
        #[command(flatten)]
        range: DateRangeArgs,
    },
}

#[derive(Subcommand)]
pub enum PlanCmd {
    /// List all training plans (active + completed)
    List,
}
```

Dispatch:

```rust
match command {
    CoachCommands::List           => list(&client, output).await,
    CoachCommands::Get { uuid }   => get(&client, output, &uuid).await,
    CoachCommands::Plan { cmd: None }              => plan_active(&client, output).await,
    CoachCommands::Plan { cmd: Some(PlanCmd::List) } => plan_list(&client, output).await,
    CoachCommands::Event { range } => event(&client, output, range).await,
}
```

`plan_active` = current `plan` body (now hits the adaptive endpoint via the
updated `training_plan` method).

`plan_list` = `list_training_plans` → `output.print_list(&plans, "Training
plans")`.

`event`:
1. `list_coach_workouts` → first `training_plan_id` (same as `plan_active`).
   On miss: `Error::NotFound("no active Coach training plan")`.
2. `plan_events(plan_id)` → pick `isPrimaryEvent == true`, else first. On
   empty: `Error::NotFound("no target event for active plan")`.
3. **Parallelize** `calendar_event(event.id)` with the projection fetch,
   using `tokio::try_join!`. Range resolution: `range.resolve(1)` gives
   `(today, today)` by default, which yields a 1-entry `projections` array
   for the snapshot path. Larger windows render the history table.
4. Sort projections descending by `calendar_date`.
5. Build `CoachEvent { event, plan_id, plan_name, projections }` and pass
   to `output.print`.

## Human output

`LABEL_WIDTH = 16` throughout. `colored::Colorize` usage mirrors existing
`CoachPlan::print_human` (title `.bold()`, separator `.dimmed()`, primary
value `.cyan()`, secondary detail plain or `.dimmed()`).

### `coach plan`

```
Programme Semi 4:30
────────────────────────────────────────
  ID:             45053764
  Level:          Intermediate
  Target:         Pace
  Range:          2026-04-13 → 2026-05-30 (7 weeks)
  Workouts/wk:    5
  Status:         Scheduled
  Supplemental:   Strength (bodyweight)

  Phases
    BUILD    2026-04-13 → 2026-04-29   ● current
    PEAK     2026-04-30 → 2026-05-20
    TAPER    2026-05-21 → 2026-05-29
    RACE     2026-05-30

  Upcoming (8 scheduled)
    Thu 2026-04-23   Rest
    Fri 2026-04-24   Running    Sprint                2x3x0:15@2:45/km   44m · 7.1 km
    Sat 2026-04-25   Running    Long Run              5:25/km            1h 26m · 16.1 km
    Sun 2026-04-26   Running    Base                  5:25/km            39m · 7.3 km
    Sun 2026-04-26   Strength   Total Body Circuit 1                     25m
    Mon 2026-04-27   Running    VO2 Max               5x3:00@4:00/km     43m · 8.7 km
    Tue 2026-04-28   Rest
    Wed 2026-04-29   Running    Anaerobic             2x4x0:40@3:35/km   53m · 9.8 km
```

- Phase rendering: map `TARGET_EVENT_DAY` → `RACE` (shorter, matches the
  mockup). Single-day phases print `date` only, not `date → date`.
- Supplemental labels: at least `STRENGTH_TRAINING_BODYWEIGHT → "Strength
  (bodyweight)"`. Others → title-case the raw value. Helper
  `supplemental_label` in `coach/mod.rs`.
- Task rendering: sort by `calendar_date` then `workout_order`. Date column
  `{Day} {ISO}` uses `chrono::NaiveDate::parse_from_str(...,"%Y-%m-%d")
  .format("%a %Y-%m-%d")` (depends on locale being en_*; acceptable for now).
- Duration uses `fmt_hm(seconds as u64)` (existing helper — returns `"Xh
  YYm"` or `"Ym"`).
- Distance uses `{:.1} km` from `estimated_distance_meters / 1000.0`.
- Sport label: `sport_type.as_ref().map(|s| s.sport_type_key)` → title-case
  mapping (`running` → `Running`, `strength_training` → `Strength`,
  otherwise title-case).
- Rest day → `Rest`, no further columns.

### `coach plan list`

```
Training plans
────────────────────────────────────────
  ●  Programme Semi 4:30        Scheduled    2026-04-13 → 2026-05-30    7 wk   Intermediate · Pace
  ✓  Semi-marathon en 1h30      Completed    2025-12-19 → 2026-03-01   10 wk   Intermediate · Pace

2 items
```

Uses `Output::print_list` with `title = "Training plans"`; glyphs from
`TrainingPlanSummary::print_human`:

- `●` for `Scheduled` / `Active` / unknown fallback
- `✓` for `Completed`
- `‖` for `Paused`
- `◦` for anything else

### `coach event` (no flags, today only)

```
Programme Semi 1h30                               target event
────────────────────────────────────────
  When:           Sat 2026-05-30  09:00  Europe/Paris   (in 37 days)
  Where:          01120 Thil, France
  Distance:       21.10 km
  Goal:           1:30:00   (4:16/km)
  Plan:           Programme Semi 4:30 (#45053764)

  Today (2026-04-23)
    Projection:   1:30:47   4:18/km    +47s vs goal
    Prediction:   1:32:08   4:22/km
    Band:         1:29:42 – 1:31:53
    Focus:        Improved VO2 Max (IMPROVED_VO2MAX)
```

- "(in N days)" from `event.date - today`. Negative → `(Xd ago)`; zero →
  `(today)`.
- Projection delta vs goal: `projection_race_time_seconds - goal_seconds`
  (goal_seconds comes from `custom_goal` if `unit == "second"`). Sign-prefixed
  (`+47s` / `-12s`). Skip the line if no goal or goal unit is not seconds.
- Pace from `speed_projection_mps` via existing `pace_from_speed(ms)`.
- Band: lower (faster) first, then `\u{2013}` (`–`), then upper (slower).
- Focus: `<label> (<raw_enum>)` with the raw enum `.dimmed()`.
- Title line: event name `.bold()`, label `"target event"` right-aligned in
  48 chars with `.dimmed()`. (Consistent with other titles — single `.bold()`
  line; the suffix is typographic sugar.)
- `Distance` respects `completion_target.unit` — `km` / `mile`.

### `coach event --days 14`

```
Programme Semi 1h30                               target event
────────────────────────────────────────
  When:           Sat 2026-05-30  (in 37 days)
  Distance:       21.10 km
  Goal:           1:30:00   (4:16/km)

  Projection history
    Date          Projection   Pace       Band               Focus
    2026-04-23    1:30:47      4:18/km    1:29:42–1:31:53    Improved VO2 Max (IMPROVED_VO2MAX)
    2026-04-22    1:30:43      4:18/km    1:29:36–1:31:51    Improved VO2 Max (IMPROVED_VO2MAX)
    2026-04-21    1:31:32      4:20/km    1:30:21–1:32:43    Improve long mileage (IMPROVE_LONG_TERM_MILEAGE_0)
    …
    2026-04-10    1:33:00      4:24/km    …                  Improve long mileage (IMPROVE_LONG_TERM_MILEAGE_0)

  Trend:          −2:13 over 13 days (projection improving)
```

- Rendered when `projections.len() >= 2`.
- Drop `When`'s `HH:MM timezone` bit (redundant in history mode — we're not
  looking at race-day logistics here) — keep just date + "(in N days)".
- Trend: filter to projections with a usable `projection_race_time_seconds`
  (`Option::is_some`); take oldest vs newest; print `oldest - newest` as
  signed `fmt_hms` with leading `−` (U+2212) for improvement. Span = "(newest
  - oldest).num_days()". Skip line entirely if < 2 usable points. Cue text:
  `projection improving` (positive diff), `regressing` (negative diff),
  `flat` (|diff| < 3s).
- Raw enum in parens `.dimmed()`.

## Feedback phrase mapping

In `coach/mod.rs` as `pub(super) fn feedback_phrase_label(code: &str) ->
String`. Known values from the fixture; extend as encountered. Unknown enums
fall back to title-casing the raw value with trailing digit suffixes stripped
(e.g. `IMPROVE_LONG_TERM_MILEAGE_0` → `Improve long mileage`; the raw enum is
kept visible via the dimmed suffix in output).

| Enum                             | Display                  |
|----------------------------------|--------------------------|
| `IMPROVED_VO2MAX`                | Improved VO2 Max         |
| `IMPROVE_LONG_TERM_MILEAGE_0`    | Improve long mileage     |
| `IMPROVE_LONG_TERM_MILEAGE_1`    | Improve long mileage     |
| `IMPROVE_LONG_TERM_MILEAGE_2`    | Improve long mileage     |

## Edge cases

- **No active plan**: `list_coach_workouts` returns no `training_plan_id` →
  `Error::NotFound("no active Coach training plan")`. Matches existing `plan`.
- **Plan with no primary event**: `plan_events` empty → `Error::NotFound("no
  target event for active plan")`.
- **Non-adaptive plan**: fallback to `/trainingplan/{id}` already baked into
  `training_plan`. `task_list` / `phases` / `supplemental_sports` all remain
  empty; rendering branches skip those sections cleanly.
- **No projections in range**: `projections.is_empty()` → render the event
  header + `(no projection yet)` `.dimmed()` in the snapshot position.
- **Goal in miles**: `completion_target.unit == "mile"` → display distance as
  miles, pace as `/mi`. Don't assume km. Minimum scaffolding: a
  `DistanceView` helper that takes `(meters, unit)` and returns the right
  string; reused across plan and event rendering. *Or* keep it inline with
  an `if unit == "mile"` branch per call site — two call sites in practice,
  acceptable.
- **Past event**: `(Xd ago)` instead of `(in X days)`. Historical projections
  still render.

## Acceptance checks

1. `cargo build` and `cargo clippy --all-targets -- -D warnings` clean.
2. `cargo test` still green (no new tests required; the one existing test is
   `garmin::auth::tests::test_oauth1_signature`).
3. `garmin coach plan` renders phases + at least one upcoming workout;
   exactly one phase marked `● current`.
4. `garmin coach plan list` renders both active + completed entries from the
   test account.
5. `garmin coach event` renders event header + today's projection (or `(no
   projection yet)`). Goal delta has correct sign.
6. `garmin coach event --days 14` renders the history table with ≥ 5 rows
   from the test account; trend line has correct sign.
7. `garmin --json coach event` and `garmin --json coach event --days 14` both
   round-trip through `serde_json` with every projection field populated
   (predicted + projection + both bounds + feedback phrase).
8. `garmin --json coach plan | jq '.task_list[0] | keys'` shows the renamed
   `estimated_duration_seconds` / `estimated_distance_meters`.
9. `garmin --json coach list | jq '.[0] | keys'` also shows the renamed
   suffixes on the existing `CoachWorkout` — this is the in-scope fix to the
   pre-existing inconsistency.

## Commits

One commit per numbered group; scope-scoped so each stands alone:

1. `refactor(coach): split coach.rs into a submodule per concept` — move-only
   commit, no behavior change. Includes the `estimated_duration_seconds` /
   `estimated_distance_meters` renames on `CoachWorkout` (small, in-scope
   fix; the rename touches the same file).
2. `feat(coach): adaptive plan detail with phases and task list` — add
   `CoachTask`, `CoachTaskWorkout`, `TrainingPhase`; switch `training_plan`
   to the adaptive endpoint with 404 fallback; extend `CoachPlan` and its
   `print_human`.
3. `feat(coach): coach plan list` — add `TrainingPlanSummary`,
   `list_training_plans` client method, `PlanCmd::List` dispatch.
4. `feat(coach): coach event with projection history` — add `TargetEvent`,
   `EventCustomization`, `EventProjection`, `CoachEvent` aggregate; add
   client methods `plan_events` / `calendar_event` / `event_projections`;
   wire `CoachCommands::Event` with `DateRangeArgs`; parallelize the two
   independent fetches.
5. `docs: coach plan/event in README + skill reference` — one sync commit
   covering `README.md` and `~/.claude/skills/garmin/references/garmin-cli.md`.

## Files to edit

### Added
- `src/garmin/types/coach/{mod,workout,plan,task,phase,event,projection}.rs`

### Deleted
- `src/garmin/types/coach.rs`

### Modified
- `src/garmin/client.rs` — update `training_plan` with 404 fallback; add
  `list_training_plans`, `plan_events`, `calendar_event`,
  `event_projections`.
- `src/commands/coach.rs` — extend `CoachCommands` with nested `Plan` and
  new `Event`; dispatch + handler functions.
- `README.md` — add coach subcommands to the table; no changes to units
  paragraph.
- `~/.claude/skills/garmin/references/garmin-cli.md` — same.

### Not modified (despite being touched in the review)
- `Cargo.toml` — all dependencies already present (`serde_with`, `chrono`,
  `tokio`, `colored`, `serde`, `serde_json`, `futures`).
- `morning-briefing.sh` — already removed from the skill.

## Open questions to close before coding

- The `coach plan` title-suffix `"target event"` on the event mockup is
  unusual styling. If it reads badly in practice, drop it; nothing else
  depends on that decoration.
- `Supplemental:   Strength (bodyweight)` — confirm the en dash handling in
  `print_human` renders as intended on narrow terminals.
