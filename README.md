# Vikarin Backend - Minimal Config API

A tiny Rust API (Axum) with two GET endpoints to read and update `bot_config.json` in-place.

- `GET /config` — returns the current config as JSON
- `GET /update` — updates fields via query params, persists to `bot_config.json`, and returns the updated JSON

The server reads `bot_config.json` from the current working directory. If missing, it creates one with these defaults:

```json
{
  "max_won_bookings": 3,
  "time_slots": [{ "start_time": "09:00", "end_time": "17:00" }],
  "paused": false,
  "interval_seconds": 5.0
}
```

## Build and Run

From the `vikarin-backend` folder:

```powershell
cargo run
```

This starts the server on `http://localhost:8080`.

## Examples (Windows PowerShell)

Use `curl.exe` to avoid PowerShell's alias behavior:

- Get config

```powershell
curl.exe http://localhost:8080/config
```

- Update simple fields

```powershell
# Pause the bot and set intervals
curl.exe "http://localhost:8080/update?paused=true&interval_seconds=10"

# Set max_won_bookings
curl.exe "http://localhost:8080/update?max_won_bookings=5"
```

- Replace `time_slots` (pass JSON as a single query param string):

```powershell
$slots = '[{"start_time":"08:00","end_time":"12:00"},{"start_time":"13:00","end_time":"18:00"}]'
curl.exe "http://localhost:8080/update?time_slots=$($slots)"
```

- Combine multiple updates

```powershell
$slots = '[{"start_time":"09:30","end_time":"16:30"}]'
curl.exe "http://localhost:8080/update?paused=false&max_won_bookings=4&interval_seconds=7.5&time_slots=$($slots)"
```

Notes:
- All query params are optional; only provided ones are updated.
- On success, each call returns the full, updated config JSON.

## Project

- `src/main.rs` — server and handlers
- `Cargo.toml` — dependencies
- `bot_config.json` — persisted config (already in your workspace)
