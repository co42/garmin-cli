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
| Coordinates | degrees | WGS84 latitude/longitude |

## Commands Reference

### Activities

```bash
garmin activities list [--limit 20] [--type trail_running] [--after DATE] [--before DATE]
garmin activities get <ID>              # Typed activity summary
garmin activities details <ID>          # Full time-series (raw JSON)
garmin activities splits <ID>           # Per-km splits with pace, HR, power, elevation
garmin activities hr-zones <ID>         # HR time in zones
garmin activities laps <ID>             # Raw laps (auto/manual)
garmin activities weather <ID>          # Weather during activity (°C, km/h)
garmin activities power-zones <ID>      # Power time in zones
garmin activities exercises <ID>        # Exercise/interval sets (raw JSON)
garmin activities compare <ID1> <ID2>   # Side-by-side comparison with deltas
garmin activities download <ID> [--format fit|gpx|tcx] [--output PATH]
garmin activities upload <FILE>
```

Activity summaries include ~50 fields: basics (distance, duration, HR), training effect, training load, VO2max, running dynamics (cadence, stride length, ground contact time, vertical oscillation), elevation, fastest splits, power, and location.

### Training

```bash
garmin training status [--date DATE] [--days N] [--from DATE --to DATE]
garmin training readiness [--date DATE] [--days N] [--from DATE --to DATE]
garmin training vo2max [--date DATE] [--days N] [--from DATE --to DATE]
garmin training race-predictions [--from DATE --to DATE]
garmin training endurance-score [--date DATE] [--days N] [--from DATE --to DATE]
garmin training hill-score [--date DATE] [--days N] [--from DATE --to DATE]
garmin training fitness-age [--date DATE]
garmin training lactate-threshold
garmin training hr-zones
```

| Command | Key fields |
|---|---|
| `status` | status, fitness_trend (improving/stable/declining), vo2max, acute_load, chronic_load, min/max_training_load_chronic, acwr, load_balance, monthly_load targets |
| `readiness` | `{ date, morning, post_activity, latest }` — each with score (0–100) + factor breakdowns. `morning` = wake-up score, `post_activity` = after exercise (absent on rest days), `latest` = real-time score (matches watch display, absent if no update since morning/post-activity) |
| `vo2max` | VO2max daily history (alias: `scores`) |
| `race-predictions` | 5K/10K/half/marathon predicted times (formatted + seconds) and paces. Supports `--from`/`--to` for daily prediction history |
| `endurance-score` | score (0–10000), classification (Base→Elite) |
| `hill-score` | overall, strength, endurance components |
| `fitness-age` | date, fitness_age vs chronological_age, component breakdown |
| `lactate-threshold` | heart_rate (bpm), pace (min/km), speed (m/s) |
| `hr-zones` | HR zone boundaries in BPM (zone, min_bpm, max_bpm — max_bpm absent for last zone) from latest running activity (alias: `zones`) |

### Health

```bash
garmin health sleep [--date DATE] [--days N] [--from DATE --to DATE]
garmin health sleep-scores [--date DATE] [--days N] [--from DATE --to DATE]
garmin health stress [--date DATE] [--days N] [--from DATE --to DATE]
garmin health heart-rate [--date DATE] [--days N] [--from DATE --to DATE]
garmin health body-battery [--date DATE]
garmin health hrv [--date DATE] [--days N] [--from DATE --to DATE]
garmin health steps [--date DATE] [--days N] [--from DATE --to DATE]
garmin health weight [--date DATE] [--days N] [--from DATE --to DATE]
garmin health hydration [--date DATE] [--days N] [--from DATE --to DATE]
garmin health spo2 [--date DATE]
garmin health respiration [--date DATE]
garmin health intensity-minutes [--date DATE] [--days N] [--from DATE --to DATE]
```

`--from`/`--to` and `--date` are mutually exclusive. `--to` defaults to today.

| Command | Key fields | Units |
|---|---|---|
| `sleep` | sleep_score, sleep_score_qualifier, sleep_seconds, deep/light/rem/awake seconds, start/end times, sleep_need_seconds | seconds |
| `sleep-scores` | score | 0–100 |
| `stress` | avg_stress, max_stress | 0–100 |
| `heart-rate` | resting_hr, max_hr, min_hr | bpm |
| `body-battery` | body_battery_high, body_battery_low, body_battery_latest | 0–100 |
| `hrv` | last_night_avg, last_night_5min_high, weekly_average, status, baseline_balanced_low/upper | ms |
| `steps` | total_steps, step_goal, total_distance_meters | meters |
| `weight` | weight_kg, bmi, body_fat_percent, muscle_mass_kg, bone_mass_kg | kg |
| `hydration` | intake_ml, goal_ml | ml |
| `spo2` | avg_spo2, lowest_spo2 | % |
| `respiration` | avg_waking_br, avg_sleeping_br, highest_br, lowest_br | breaths/min |
| `intensity-minutes` | moderate, vigorous, total, weekly_goal | minutes |

### Profile

```bash
garmin profile show                    # Display name, profile info
garmin profile settings                # Biometrics, thresholds, training preferences
garmin profile settings set            # Update user settings (partial update)
  --max-hr <BPM>                       # Max heart rate (via biometric service)
  --resting-hr <BPM>                   # Resting heart rate (via biometric service)
  --weight <KG>                        # Weight (converted to grams for API)
  --height <CM>                        # Height
  --lactate-threshold-hr <BPM>         # Lactate threshold HR
  --lactate-threshold-speed <M/S>      # Lactate threshold speed
  --threshold-hr-auto-detected <BOOL>  # LT HR auto-detection on/off
  --resting-hr-auto-update <BOOL>     # Resting HR auto-update from device (via biometric service)
  --vo2max-running <VALUE>             # VO2max running (display-only, does not affect device)
  --training-status-paused             # Pause training status (sets date to today)
  --training-status-resumed            # Resume training status (clears paused date)
  --sleep-time <HH:MM>                # Sleep time
  --wake-time <HH:MM>                 # Wake time
```

| Command | Key fields |
|---|---|
| `settings` | weight_kg, height_cm, birth_date, gender, handedness, max_hr, resting_hr, lactate_threshold_hr, lactate_threshold_speed, threshold_hr_auto_detected, vo2max_running, vo2max_cycling, ftp, ftp_auto_detected, training_status_paused_date, measurement_system, time_format, available_training_days, preferred_long_training_days, sleep_time, wake_time |

`settings set` does a partial update — only provided flags are changed. Shows before/after values in human mode, returns `{"field": {"old": ..., "new": ...}}` in JSON mode. Max HR and resting HR are stored in the biometric service (heartRateZones endpoint), not in user-settings.

### Daily Summary

```bash
garmin summary                         # Today
garmin summary --date 2025-03-01       # Specific date
garmin summary --days 7                # Last 7 days
```

### Courses

```bash
garmin courses list                    # Saved GPX routes
garmin courses get <ID>                # Course details with full metadata and track points
```

| Command | Key fields |
|---|---|
| `list` | id, name, description, activity_type, distance_meters, elevation_gain/loss_meters, start_latitude, start_longitude, favorite, public, has_pace_band, has_power_guide, has_turn_detection_disabled, speed_meters_per_second, elapsed_seconds, elevation_source, start_note, finish_note, cutoff_duration, created_date, update_date |
| `get` | All list fields plus: start_point (latitude, longitude, elevation), bounding_box (lower_left, upper_right), include_laps, matched_to_segments, course_segments (sort_order, distance_meters, num_points), geo_points (latitude, longitude, elevation, distance) |

### Badges

```bash
garmin badges list                     # Earned achievements
```

### Workouts

```bash
garmin workouts list [--limit 20] [-v]
garmin workouts get <ID>               # Summary + full step structure
garmin workouts create --file workout.json
garmin workouts update <ID> --file workout.json
garmin workouts schedule <ID> <DATE>
garmin workouts delete <ID>
garmin workouts template [--type interval|tempo|easy|long-run]
```

`get` shows the full step structure in human mode (step types, targets, descriptions, repeat groups) and returns the raw API response in JSON mode. `-v`/`--verbose` on `list` shows step details inline (same as `get` but for each workout). Templates include all required API IDs and description fields — they can be used directly with `create`.

Pace targets use **m/s** (convert: `m/s = 1000 / sec_per_km`, e.g. 4:25/km → 3.774). Garmin Coach convention: `targetValueOne` = faster bound (higher m/s), `targetValueTwo` = slower bound (lower m/s). The watch normalizes regardless of order. HR targets use **BPM values**, not zone numbers — use `garmin training hr-zones` to get your boundaries.

### Coach (Garmin Coach / FBT Adaptive)

```bash
garmin coach list [--all] [-v]         # Adaptive workouts (REQUIRED only by default)
garmin coach get <UUID>                # Workout detail + step structure
garmin coach plan                      # Active training plan metadata
```

Coach workouts use UUIDs (not numeric IDs). `list` shows only REQUIRED priority workouts by default — use `--all` to include alternates. `-v`/`--verbose` shows step details inline. `get` reuses the same step display as `workouts get`.

Human output shows workout phrase (base, long run, anaerobic speed...), training effect (aerobic/anaerobic), and priority type.

### Calendar

```bash
garmin calendar [--year 2026] [--month 3]   # View a month
garmin calendar --weeks 4                    # Next N weeks (spans months)
garmin calendar delete <ID>                  # Remove a scheduled entry
```

Human output tags each item by source: `[coach <UUID>]` for Garmin Coach workouts, `[workout <ID>]` for user-created workouts, `[activity <ID>]` for completed activities, or `[type]` for other items (events, goals).

### Gear

```bash
garmin gear list                       # All gear (shoes, bikes, etc.)
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

## Output Formats

- Default: **Human-readable** (when TTY)
- `--json`: Force JSON output
- `--no-json`: Force human output
- `--pretty`: Pretty-print JSON (default is compact)
- `--fields f1,f2`: Filter JSON output fields
- `-q, --quiet`: Suppress status messages

Pipes auto-detect: `garmin summary | jq .` outputs JSON automatically.

```bash
garmin summary --json --fields date,total_steps,resting_heart_rate
garmin health sleep --days 7 --json --fields date,sleep_score
garmin activities list --limit 5 --json --fields id,name,pace_min_km,aerobic_training_effect
```

All JSON field names use **snake_case**. Both JSON and human output are rendered from the same typed structs — no data is lost between formats.

### Structured Errors

With `--json`, errors are returned as JSON:

```json
{"error": "message", "code": "auth", "exit_code": 2}
```

Codes: `auth` (exit 2), `not_found` (exit 3), `rate_limit` (exit 4), `api` / `generic` (exit 1).

## Development

```bash
# Enable pre-commit hooks (fmt + clippy + test)
git config core.hooksPath .githooks
```
