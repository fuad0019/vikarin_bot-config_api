# Vikarin Config API — Cloudflare Workers (Rust)

This is a Cloudflare Workers port of the API using the `worker` crate (wasm) and Workers KV for persistence (no filesystem).

Endpoints:
- GET `/config` — returns current config from KV (creates default if missing)
- GET `/update` — updates any provided fields via query params and persists to KV

KV key used: `bot_config`

## Prereqs
- Rust + `wasm32-unknown-unknown` target
- `wrangler` CLI (Cloudflare)
- `worker-build` helper for Rust Workers

```powershell
rustup target add wasm32-unknown-unknown
cargo install worker-build
npm i -g wrangler
```

## Setup KV
From this `workers` folder:

```powershell
# Create a KV namespace and capture the id
wrangler kv namespace create CONFIG_KV

# Edit wrangler.toml and set the produced `id` for CONFIG_KV
# Example snippet:
# [[kv_namespaces]]
# binding = "CONFIG_KV"
# id = "<YOUR_ID>"
```

## Dev
```powershell
cd workers
wrangler dev
# Open http://127.0.0.1:8787/config
```

## Publish
```powershell
cd workers
wrangler publish
```

## Examples (PowerShell)
```powershell
# Get config
curl.exe http://127.0.0.1:8787/config

# Update paused and interval
curl.exe "http://127.0.0.1:8787/update?paused=true&interval_seconds=10"

# Replace time_slots
$slots = '[{"start_time":"08:00","end_time":"12:00"},{"start_time":"13:00","end_time":"18:00"}]'
curl.exe "http://127.0.0.1:8787/update?time_slots=$($slots)"
```

## Notes
- Workers KV is eventually consistent; for stronger consistency, consider a Durable Object to store the config.
- The existing Axum server uses the local filesystem (`bot_config.json`). Workers cannot use a filesystem, so this port uses KV.
