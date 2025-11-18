use serde::{Deserialize, Serialize};
use worker::*;
use worker::kv::KvStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeSlot {
    start_time: String,
    end_time: String,
}

//Test

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BotConfig {
    max_won_bookings: u32,
    time_slots: Vec<TimeSlot>,
    paused: bool,
    interval_seconds: f64,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            max_won_bookings: 3,
            time_slots: vec![TimeSlot { start_time: "09:00".into(), end_time: "17:00".into() }],
            paused: false,
            interval_seconds: 5.0,
        }
    }
}

const KV_KEY: &str = "bot_config";

fn log_json<T: Serialize>(label: &str, value: &T) {
    if let Ok(text) = serde_json::to_string(value) {
        console_log!("{}: {}", label, text);
    }
}

fn add_cors_headers(resp: &mut Response) -> Result<()> {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
    headers.set("Access-Control-Max-Age", "86400")?;
    Ok(())
}

async fn load_or_init_config(kv: &KvStore) -> Result<BotConfig> {
    let existing = kv.get(KV_KEY).text().await?;
    if let Some(txt) = existing {
        let cfg: BotConfig = serde_json::from_str(&txt).map_err(|e| Error::RustError(e.to_string()))?;
        Ok(cfg)
    } else {
        let cfg = BotConfig::default();
        let data = serde_json::to_string(&cfg).map_err(|e| Error::RustError(e.to_string()))?;
        kv.put(KV_KEY, data)?.execute().await?;
        Ok(cfg)
    }
}

async fn save_config(kv: &KvStore, cfg: &BotConfig) -> Result<()> {
    let data = serde_json::to_string(cfg).map_err(|e| Error::RustError(e.to_string()))?;
    kv.put(KV_KEY, data)?.execute().await?;
    Ok(())
}

fn parse_query(req: &Request) -> Result<std::collections::HashMap<String, String>> {
    let url = req.url()?;
    let mut map = std::collections::HashMap::new();
    for (k, v) in url.query_pairs() {
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}

async fn handle_get_config(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.kv("CONFIG_KV")?;
    console_log!("request: GET /config Fuad1234");
    let cfg = load_or_init_config(&kv).await?;
    log_json("response /config", &cfg);
    let mut resp = Response::from_json(&cfg)?;
    add_cors_headers(&mut resp)?;
    Ok(resp)
}

async fn handle_update(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.kv("CONFIG_KV")?;
    let qmap = parse_query(&req)?;
    log_json("request /update query", &qmap);

    let mut cfg = load_or_init_config(&kv).await?;

    if let Some(v) = qmap.get("max_won_bookings") {
        if let Ok(n) = v.parse::<u32>() { cfg.max_won_bookings = n; }
    }
    if let Some(v) = qmap.get("paused") {
        if let Ok(b) = v.parse::<bool>() { cfg.paused = b; }
    }
    if let Some(v) = qmap.get("interval_seconds") {
        if let Ok(f) = v.parse::<f64>() { cfg.interval_seconds = f; }
    }
    if let Some(v) = qmap.get("time_slots") {
        let slots: Vec<TimeSlot> = match serde_json::from_str(v) {
            Ok(s) => s,
            Err(e) => return Response::error(format!("invalid time_slots json: {e}"), 400),
        };
        cfg.time_slots = slots;
    }

    save_config(&kv, &cfg).await?;
    log_json("response /update", &cfg);
    let mut resp = Response::from_json(&cfg)?;
    add_cors_headers(&mut resp)?;
    Ok(resp)
}

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    // Default to info logs if not provided
    if std::env::var("RUST_LOG").is_err() {
        console_log!("RUST_LOG not set; using default info level");
    }

    // Handle CORS preflight
    if req.method() == Method::Options {
        let mut resp = Response::ok("")?;
        add_cors_headers(&mut resp)?;
        return Ok(resp);
    }

    let router = Router::new();
    let mut resp = router
        .get_async("/config", handle_get_config)
        .get_async("/update", handle_update)
        .run(req, env)
        .await?;
    add_cors_headers(&mut resp)?;
    Ok(resp)
}
