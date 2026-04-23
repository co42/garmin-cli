# garmin-cli

Garmin Connect CLI. All data is returned through typed structs with consistent snake_case field names and metric units.

## Install

```bash
brew install co42/tap/garmin
```

Or from source:

```bash
cargo install --git https://github.com/co42/garmin-cli
```

## Auth

```bash
# Interactive (prompts for email and password)
garmin auth login

# Or via environment variables
export GARMIN_EMAIL="you@example.com"
export GARMIN_PASSWORD="hunter2"
garmin auth login

garmin auth status    # Check token expiry
garmin auth logout    # Delete stored tokens
```

Tokens are stored in `~/.config/garmin/tokens.json`. OAuth2 tokens auto-refresh transparently.

## Units

All output uses metric units. The Garmin API returns some values in imperial or raw formats — the CLI normalizes everything before output.

Numeric fields carry their unit as a suffix (`_seconds`, `_meters`, `_mps`, `_kg`, `_cm`, `_ms`, `_percent`, `_bpm`, `_joules`, `_watts`, `_ml`) so JSON output is self-describing. HR fields drop the suffix when context is unambiguous (`average_hr`, `max_hr` are always bpm).

| Measurement | Unit | Notes |
|---|---|---|
| Distance | meters (m) | Human output shows km when appropriate |
| Pace | MM:SS /km | Computed from distance and duration |
| Speed | m/s | Raw from API; human output may show km/h |
| Duration | seconds (s) | Human output shows MM:SS or HH:MM:SS |
| Temperature | °C | Converted from Fahrenheit (weather endpoint) |
| Wind speed | km/h | Converted from mph (weather endpoint) |
| Weight | kg | Converted from grams (Garmin stores in g) |
| Elevation | meters (m) | |
| Heart rate | bpm | |
| Cadence | steps/min (spm) | |
| Stride length | cm | |
| Ground contact time | ms | |
| Vertical oscillation | cm | |
| Power | watts (W) | Running power |
| Work | joules (J) | Activity detail: total_work_joules |
| Coordinates | degrees | WGS84 latitude/longitude |

## Date range flags

Every command that looks at a time window takes the same three flags, flattened onto its own arg list:

| Flag | Meaning |
|---|---|
| `--days N` | Last N days ending today (inclusive) |
| `--from YYYY-MM-DD [--to YYYY-MM-DD]` | Explicit range; `--to` defaults to today |
| *(no flag)* | Default window for that command (1 day for most, 7 for VO2max / sleep-scores / lactate-threshold) |

`--days` is mutually exclusive with `--from`/`--to`. `--to` requires `--from`. Output is always an array, one entry per day in the window.

`activities list` is the only exception — with no date flag it returns the most recent N regardless of date.

## Commands Reference

### Activities

```bash
garmin activities list [--limit 20] [--start N] [-t running] [--days N | --from DATE --to DATE]
garmin activities get <ID>              # Typed summary + merged details
garmin activities details <ID>          # Full time-series (raw JSON)
garmin activities splits <ID>           # Per-km splits
garmin activities hr-zones <ID>         # HR time in zones
garmin activities laps <ID>             # Raw laps (auto/manual)
garmin activities weather <ID>          # Weather during activity (°C, km/h)
garmin activities power-zones <ID>      # Power time in zones
garmin activities exercises <ID>        # Exercise/interval sets (raw JSON)
garmin activities download <ID> [--format fit|gpx|tcx] [-o PATH]
garmin activities upload <FILE>
```

`list` without a date flag returns the most recent activities ignoring date. Pass `--days`/`--from`/`--to` to restrict to a window (uses server-side `startDate`/`endDate`).

Activity summaries include ~50 fields: basics (distance, duration, HR), training effect, training load, VO2max, running dynamics (cadence, stride length, ground contact time, vertical oscillation), elevation, fastest splits, power, and location. `get` merges the summary with the full-detail endpoint.

### Training

```bash
garmin training status [--days N | --from DATE --to DATE]
garmin training readiness [--days N | --from DATE --to DATE]
garmin training vo2max [--days N | --from DATE --to DATE]             # alias: scores, default --days 7
garmin training race-predictions [--days N | --from DATE --to DATE]
garmin training endurance-score [--days N | --from DATE --to DATE]
garmin training hill-score [--days N | --from DATE --to DATE]
garmin training fitness-age [--days N | --from DATE --to DATE]
garmin training lactate-threshold [--days N | --from DATE --to DATE]  # default --days 7
garmin training hr-zones                                               # alias: zones
```

| Command | Key fields |
|---|---|
| `status` | status, fitness_trend (declining/stable/improving), vo2max, acute_load, chronic_load, min/max_training_load_chronic, acwr, load_balance_feedback, monthly_load targets |
| `readiness` | `{ date, morning, post_activity, latest }` — each with score (0–100) + factor breakdowns. `morning` = wake-up score, `post_activity` = after exercise (absent on rest days), `latest` = real-time score |
| `vo2max` | VO2max daily history |
| `race-predictions` | 5K/10K/half/marathon predicted times_seconds and paces |
| `endurance-score` | score (0–10000), classification (Base→Elite) |
| `hill-score` | overall, strength, endurance components |
| `fitness-age` | date, fitness_age vs chronological_age, component breakdown |
| `lactate-threshold` | heart_rate (bpm), pace (min/km), speed_mps. If the window is empty, falls back to the most recent prior value (up to 365 days back) |
| `hr-zones` | HR zone boundaries (zone, min_bpm, max_bpm — max_bpm absent for last zone) from latest running activity |

### Health

```bash
garmin health sleep [--days N | --from DATE --to DATE]
garmin health sleep-scores [--days N | --from DATE --to DATE]    # default --days 7
garmin health stress [--days N | --from DATE --to DATE]
garmin health heart-rate [--days N | --from DATE --to DATE]
garmin health body-battery [--days N | --from DATE --to DATE]
garmin health hrv [--days N | --from DATE --to DATE]
garmin health steps [--days N | --from DATE --to DATE]
garmin health weight [--days N | --from DATE --to DATE]
garmin health hydration [--days N | --from DATE --to DATE]
garmin health spo2 [--days N | --from DATE --to DATE]
garmin health respiration [--days N | --from DATE --to DATE]
garmin health intensity-minutes [--days N | --from DATE --to DATE]
```

| Command | Key fields | Units |
|---|---|---|
| `sleep` | calendar_date, sleep_scores.overall.value, sleep_time_seconds, deep/light/rem/awake_sleep_seconds, sleep_start/end_timestamp_local, sleep_need.actual | seconds |
| `sleep-scores` | calendar_date, value | 0–100 |
| `stress` | avg_stress_level, max_stress_level | 0–100 |
| `heart-rate` | resting_heart_rate, min_heart_rate, max_heart_rate, last_seven_days_avg_resting_heart_rate | bpm |
| `body-battery` | body_battery_high, body_battery_low, body_battery_latest, body_battery_reset_level, body_battery_reset_timestamp_ms | 0–100 |
| `hrv` | hrv_summary.{last_night_avg, last_night5_min_high, weekly_avg, status, baseline.balanced_low/upper} | ms |
| `steps` | total_steps, step_goal, total_distance_meters | meters |
| `weight` | weight_kg, bmi, body_fat_percent, muscle_mass_kg, bone_mass_kg, body_water_percent | kg |
| `hydration` | intake_ml, goal_ml | ml |
| `spo2` | average_spo2, lowest_spo2 | % |
| `respiration` | avg_waking_respiration_value, avg_sleep_respiration_value, highest/lowest_respiration_value | breaths/min |
| `intensity-minutes` | moderate_value, vigorous_value, weekly_goal | minutes |

### Profile

```bash
garmin profile show                    # Name, username, user_id, location, bio, primary_activity, user_level, profile_visibility
garmin profile settings                # Biometrics, thresholds, training preferences
garmin profile settings set            # Partial update (only the flags you pass are changed)
  --max-hr <BPM>                       # Max heart rate (via biometric service)
  --resting-hr <BPM>                   # Resting heart rate (via biometric service)
  --weight <KG>                        # Weight (converted to grams for API)
  --height <CM>                        # Height
  --lactate-threshold-hr <BPM>         # Lactate threshold HR
  --lactate-threshold-speed <M/S>      # Lactate threshold speed
  --threshold-hr-auto-detected <BOOL>  # LT HR auto-detection on/off
  --resting-hr-auto-update <BOOL>      # Resting HR auto-update from device (via biometric service)
  --vo2max-running <VALUE>             # VO2max running (display-only, does not affect device)
  --training-status-paused             # Pause training status (sets date to today)
  --training-status-resumed            # Resume training status (clears paused date)
  --sleep-time <HH:MM>                 # Sleep time
  --wake-time <HH:MM>                  # Wake time
```

| Command | Key fields |
|---|---|
| `settings` | weight_kg, height_cm, birth_date, gender, handedness, max_hr_bpm, resting_hr_bpm, lactate_threshold_hr_bpm, lactate_threshold_speed_mps, threshold_hr_auto_detected, vo2max_running, vo2max_cycling, ftp_watts, ftp_auto_detected, training_status_paused_date, measurement_system, time_format, available_training_days, preferred_long_training_days, sleep_time, wake_time |

`settings set` returns the re-fetched `ProfileSettings` view after the write. Max HR and resting HR are stored in the biometric service (`heartRateZones` endpoint), not in user-settings.

### Daily Summary

```bash
garmin summary                         # Today
garmin summary --days 7                # Last 7 days
garmin summary --from 2026-03-01 --to 2026-03-10
```

### Courses

```bash
garmin courses list                    # Saved GPX routes
garmin courses get <ID>                # Course details with full metadata and track points
```

### Badges

```bash
garmin badges list                     # Earned achievements
```

### Workouts

```bash
garmin workouts list [--limit 20] [--start N] [--steps]
garmin workouts get <ID>               # Summary + full step structure
garmin workouts create --file workout.json
garmin workouts update <ID> --file workout.json
garmin workouts schedule <ID> <DATE>
garmin workouts delete <ID>
garmin workouts template [--type interval|tempo|easy|long-run]
```

`--steps` on `list` fans out one detail fetch per workout so the printed rows include step structure. Templates include all required API IDs and description fields — they can be used directly with `create`.

Pace targets use **m/s** (convert: `m/s = 1000 / sec_per_km`, e.g. 4:25/km → 3.774). Garmin Coach convention: `targetValueOne` = faster bound (higher m/s), `targetValueTwo` = slower bound (lower m/s). HR targets use **BPM values**, not zone numbers — use `garmin training hr-zones` to get your boundaries.

### Coach (Garmin Coach / FBT Adaptive)

```bash
garmin coach list                                    # Adaptive workouts
garmin coach get <UUID>                              # Workout detail + step structure
garmin coach plan                                    # Active plan: phases, task list, supplemental sports
garmin coach plan list                               # All training plans (active + completed)
garmin coach event [--days N | --from DATE --to DATE]  # Target event + projection history (default: today)
```

Coach workouts use UUIDs (not numeric IDs). In human mode `list` filters out `FORCED_REST` and `EASY_WEEK_LOAD_REST` entries (they carry no useful detail); JSON mode keeps them.

`coach plan` hits the adaptive endpoint when available and falls back to the non-adaptive endpoint on 404 (phases and tasks will be empty in that case). `coach event` returns the same JSON shape regardless of `--days` — `projections` is an array of 1 for the default snapshot, N for history mode. Key fields on `event.event_customization`: `projected_race_time_seconds`, `predicted_race_time_seconds`, `projected_race_speed_mps`, `predicted_race_speed_mps`. Each entry in `projections`: `calendar_date`, `projection_race_time_seconds`, `predicted_race_time_seconds`, `upper/lower_bound_projection_race_time_seconds`, `speed_projection_mps`, `event_race_predictions_feedback_phrase`.

### Calendar

```bash
garmin calendar list [--year 2026] [--month 3]   # View a month
garmin calendar list --weeks 4                    # Next N weeks (spans months)
garmin calendar delete <ID>                       # Remove a scheduled entry (calendar entry ID, not workout ID)
```

Human output tags each item by source: `[coach <UUID>]` for Garmin Coach workouts, `[workout <ID>]` for user-created workouts, `[activity <ID>]` for completed activities, or `[type]` for other items.

### Gear

```bash
garmin gear list                       # All gear (shoes, bikes, etc.) — enriched with distance and activity count
garmin gear stats <UUID>               # Usage statistics
garmin gear link <UUID> <ACTIVITY_ID>  # Link gear to activity
```

### Personal Records

```bash
garmin records                         # PRs across all activities
```

### Devices

```bash
garmin devices list
garmin devices get <ID>
```

### Raw API (escape hatch)

```bash
garmin api /userprofile-service/usersummary
garmin api /some-endpoint --method POST --data '{"key": "value"}'
```

For any Garmin Connect API endpoint not covered by a dedicated command.

### Shell Completions

```bash
garmin completions bash > ~/.local/share/bash-completion/completions/garmin
garmin completions zsh > ~/.zfunc/_garmin
garmin completions fish > ~/.config/fish/completions/garmin.fish
```

## Output

Default output is human-readable. Pass `--json` to switch to pretty-printed JSON (no TTY auto-detect — pipes stay human unless you ask for JSON).

Global flags:

- `--json` — emit JSON instead of human output
- `--fields f1,f2` — filter top-level JSON keys (JSON mode only)
- `-v, --verbose` — log every Garmin API request/response to stderr

```bash
garmin summary --json --days 7 --fields calendar_date,total_steps,resting_heart_rate
garmin health sleep --days 7 --json --fields calendar_date,sleep_time_seconds,sleep_scores
garmin activities list --limit 5 --json --fields activity_id,activity_name,duration_seconds
```

All JSON field names use **snake_case**. Human and JSON output are rendered from the same typed structs.

### Structured errors

With `--json`, errors are emitted on stderr as:

```json
{"error": "...", "code": "..."}
```

Codes: `usage`, `not_found`, `rate_limit`, `auth`, `api`, `generic`. All errors exit with code `1`.

## Development

```bash
# Enable pre-commit hooks (fmt + clippy + test)
git config core.hooksPath .githooks
```
