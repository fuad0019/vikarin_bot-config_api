# Vikarin Config API — Cloudflare Workers (Rust)

[Live: vikarin-config-api.fuadh73.workers.dev](https://vikarin-config-api.fuadh73.workers.dev)

Cloudflare Worker (Rust `worker` crate) exposing two endpoints backed by Workers KV:
- GET `/config` — returns current config from KV (creates default if missing)
- GET `/update` — updates provided fields via query params and persists to KV

KV key used: `bot_config`

## Quick Start

Examples against the live deployment:

```powershell
# Get config
curl.exe https://vikarin-config-api.fuadh73.workers.dev/config

# Update paused and interval
curl.exe "https://vikarin-config-api.fuadh73.workers.dev/update?paused=true&interval_seconds=10"

# Replace time_slots (pass JSON in a single query param)
$slots = '[{"start_time":"08:00","end_time":"12:00"},{"start_time":"13:00","end_time":"18:00"}]'
curl.exe "https://vikarin-config-api.fuadh73.workers.dev/update?time_slots=$($slots)"
```

## Local Dev

Prereqs:
- Rust + `wasm32-unknown-unknown` target
- `wrangler` CLI (Cloudflare)
- `worker-build` helper for Rust Workers

```powershell
rustup target add wasm32-unknown-unknown
cargo install worker-build
npm i -g wrangler

wrangler dev
# Open http://127.0.0.1:8787/config
```

## Deploy

This repo is connected to Cloudflare’s Git integration. To deploy, push to the configured branch. You can also deploy manually:

```powershell
wrangler deploy
```

## KV Setup (for new environments)

If creating a fresh KV namespace:

```powershell
wrangler kv namespace create CONFIG_KV
# Copy the printed id into wrangler.toml under [[kv_namespaces]]
```

## Observability

`wrangler.toml` includes:

```
[observability]
enabled = false
```

Set `enabled = true` to forward logs/metrics in Cloudflare.

## Notes
- Workers KV is eventually consistent; for stronger consistency, consider a Durable Object to store the config.
