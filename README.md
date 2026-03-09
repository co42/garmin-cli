# garmin-cli

Garmin Connect CLI.

## Install

```bash
cargo install --git https://github.com/co42/garmin-cli
```

## Auth

```bash
garmin auth login --username you@example.com --password hunter2

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
garmin health sleep [--date DATE] [--days N]
garmin health stress [--date DATE] [--days N]
garmin health heart-rate [--date DATE] [--days N]
garmin health body-battery [--date DATE]
garmin health hrv [--date DATE] [--days N]
garmin health steps [--date DATE] [--days N]
garmin health weight [--date DATE] [--days N]
garmin health hydration [--date DATE] [--days N]
garmin health spo2 [--date DATE]
garmin health respiration [--date DATE]
garmin health intensity-minutes [--date DATE] [--days N]
```

### Training
```bash
garmin training status [--date DATE] [--days N]
garmin training readiness [--date DATE] [--days N]
garmin training scores [--date DATE] [--days N]
```

### Activities
```bash
garmin activities list --limit 20
garmin activities get 12345678
garmin activities download 12345678 --format gpx --output run.gpx
garmin activities upload ./morning_run.fit
```

### Devices
```bash
garmin devices list
garmin devices get 12345678
```

## Output Formats

- Default: Human-readable (when TTY)
- `--json`: Force JSON output
- `--no-json`: Force human output
- `--fields f1,f2`: Filter JSON output fields
- `-q, --quiet`: Suppress status messages

Pipes auto-detect: `garmin summary | jq .` outputs JSON automatically.

```bash
garmin summary --json --fields date,total_steps,resting_heart_rate
garmin health sleep --days 7 --json --fields date,sleep_score
```

## Development

```bash
# Enable pre-commit hooks (fmt + clippy + test)
git config core.hooksPath .githooks
```
