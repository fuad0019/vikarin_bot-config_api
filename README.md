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

### Building Artifacts (Git Integration Flow)

Cloudflare’s Git build environment for this project does not include `cargo`. You build locally and commit the generated `build/` directory.

```powershell
# Ensure cargo bin path available
$env:Path += ";$HOME\.cargo\bin"

# (One-time) Install worker-build
cargo install worker-build

# Clean previous artifacts
Remove-Item -Recurse -Force build -ErrorAction SilentlyContinue

# Build release artifacts
worker-build --release

# Fix first-line non‑standard import if present
node scripts/patch-build.mjs   # or: pwsh -File scripts/patch-build.ps1

# Commit artifacts
git add build
git commit -m "build: update worker artifacts"
git push
```

After the push, Cloudflare Git integration deploys using `wrangler.toml` (`main = build/shim.js`).

## Deploy

Deployment options:
1. Git Integration (recommended): Push with updated `build/` artifacts → automatic deploy.
2. Manual: Run `wrangler deploy` locally after building (does not require committing artifacts).

Manual deploy:
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
- Generated `build/index.js` may contain a non‑standard first line `import source wasmModule ...`; patch scripts normalize it to `import wasmModule ...`.
- Git install of `worker-build` currently fails (template placeholders); use crates.io `cargo install worker-build`.
- Rebuild artifacts after any change to Rust source before pushing for Git‑based deploys.
