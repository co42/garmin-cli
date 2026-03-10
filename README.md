# garmin-cli

Garmin Connect CLI.

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

## Commands Reference

### Raw API (escape hatch)
```bash
garmin api /userprofile-service/usersummary
garmin api /some-endpoint --method POST --data '{"key": "value"}'
```

### Profile
```bash
garmin profile show
garmin profile settings
```

### Daily Summary
```bash
garmin summary                         # Today
garmin summary --date 2025-03-01       # Specific date
garmin summary --days 7                # Last 7 days
```

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

### Activities
```bash
garmin activities list [--limit 20] [--type trail_running] [--after DATE] [--before DATE]
garmin activities get <ID>
garmin activities details <ID>                  # Full metrics, polyline, time-series
garmin activities splits <ID>                   # Per-km lap data (pace, HR, elevation)
garmin activities hr-zones <ID>                 # HR time in zones
garmin activities compare <ID1> <ID2>           # Side-by-side comparison with deltas
garmin activities download <ID> [--format fit|gpx|tcx] [--output PATH]
garmin activities upload <FILE>
```

Activity summaries include a computed `pace_min_km` field (derived from distance and duration).

### Workouts
```bash
garmin workouts list [--limit 20]
garmin workouts get <ID>
garmin workouts create --file workout.json      # Push structured workout to Garmin
garmin workouts schedule <ID> <DATE>            # Schedule on calendar
garmin workouts delete <ID>
garmin workouts template [--type interval|tempo|easy|long-run]  # Print a workout JSON template
```

### Gear
```bash
garmin gear list                                # All gear (shoes, bikes, etc.)
garmin gear stats <UUID>                        # Usage statistics
garmin gear link <UUID> <ACTIVITY_ID>           # Link gear to activity
```

### Personal Records
```bash
garmin records
```

### Calendar
```bash
garmin calendar [--year 2026] [--month 3]       # Monthly view
```

### Devices
```bash
garmin devices list
garmin devices get <ID>
```

### Shell Completions
```bash
garmin completions bash > ~/.local/share/bash-completion/completions/garmin
garmin completions zsh > ~/.zfunc/_garmin
garmin completions fish > ~/.config/fish/completions/garmin.fish
```

## Output Formats

- Default: Human-readable (when TTY)
- `--json`: Force JSON output
- `--no-json`: Force human output
- `--compact`: Compact JSON (no pretty-printing)
- `--fields f1,f2`: Filter JSON output fields
- `-q, --quiet`: Suppress status messages

Pipes auto-detect: `garmin summary | jq .` outputs JSON automatically.

```bash
garmin summary --json --fields date,total_steps,resting_heart_rate
garmin health sleep --days 7 --json --fields date,sleep_score
```

### Structured Errors

With `--json`, errors are returned as JSON with machine-readable error codes and appropriate exit codes:

```json
{"error": "message", "code": "auth", "exit_code": 2}
```

Codes: `auth` (exit 2), `not_found` (exit 3), `rate_limit` (exit 4), `api` / `generic` (exit 1).

## Development

```bash
# Enable pre-commit hooks (fmt + clippy + test)
git config core.hooksPath .githooks
```
