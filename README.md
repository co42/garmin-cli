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
garmin training scores [--date DATE] [--days N] [--from DATE --to DATE]
garmin training race-predictions
garmin training endurance-score [--date DATE] [--days N] [--from DATE --to DATE]
garmin training hill-score [--date DATE] [--days N] [--from DATE --to DATE]
garmin training fitness-age [--date DATE]
garmin training lactate-threshold
```

| Command | Key fields |
|---|---|
| `status` | training_status (PRODUCTIVE/DETRAINING/etc), vo2max, acute_load, chronic_load, acwr, load_balance |
| `readiness` | score (0–100), factor breakdowns (sleep, recovery, training, HRV, stress, sleep_history) |
| `scores` | VO2max daily history |
| `race-predictions` | 5K/10K/half/marathon predicted times and paces |
| `endurance-score` | score (0–10000), classification (Base→Elite) |
| `hill-score` | overall, strength, endurance components |
| `fitness-age` | fitness_age vs chronological_age, component breakdown |
| `lactate-threshold` | heart_rate (bpm), pace (min/km), speed (m/s) |

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
| `sleep` | sleep_score, total_sleep_seconds, deep/light/rem/awake seconds, start/end times | seconds |
| `sleep-scores` | overall_score, quality_score, duration_score, recovery_score | 0–100 |
| `stress` | avg_stress, max_stress, body_battery_high/low | 0–100 |
| `heart-rate` | resting_hr, max_hr, min_hr | bpm |
| `body-battery` | high, low, drain, charge | 0–100 |
| `hrv` | weekly_avg, last_night, balance_status | ms |
| `steps` | total_steps, distance_meters, goal | meters |
| `weight` | weight_kg, bmi, body_fat_percent, muscle_mass_kg, bone_mass_kg | kg |
| `hydration` | intake_ml, goal_ml | ml |
| `spo2` | avg_spo2, lowest_spo2 | % |
| `respiration` | avg_waking, avg_sleeping, highest, lowest | breaths/min |
| `intensity-minutes` | moderate_minutes, vigorous_minutes, weekly_goal | minutes |

### Profile

```bash
garmin profile show                    # Display name, profile info
garmin profile settings                # User settings (weight in kg, units, etc.)
```

### Daily Summary

```bash
garmin summary                         # Today
garmin summary --date 2025-03-01       # Specific date
garmin summary --days 7                # Last 7 days
```

### Courses

```bash
garmin courses list                    # Saved GPX routes
garmin courses get <ID>                # Course details (distance, elevation, coordinates)
```

### Badges

```bash
garmin badges list                     # Earned achievements
```

### Workouts

```bash
garmin workouts list [--limit 20]
garmin workouts get <ID>
garmin workouts create --file workout.json
garmin workouts schedule <ID> <DATE>
garmin workouts delete <ID>
garmin workouts template [--type interval|tempo|easy|long-run]
```

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

### Calendar

```bash
garmin calendar [--year 2026] [--month 3]
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
