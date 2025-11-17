use std::{path::{Path, PathBuf}, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeSlot {
    start_time: String,
    end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BotConfig {
    max_won_bookings: u32,
    time_slots: Vec<TimeSlot>,
    paused: bool,
    interval_seconds: f64,
}

#[derive(Debug, Deserialize)]
struct UpdateQuery {
    max_won_bookings: Option<u32>,
    paused: Option<bool>,
    interval_seconds: Option<f64>,
    // JSON string like: [{"start_time":"09:00","end_time":"17:00"}]
    time_slots: Option<String>,
}

#[derive(Clone)]
struct AppState {
    config: Arc<RwLock<BotConfig>>,
    config_path: Arc<PathBuf>,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        error!(?status, error = %self, "request failed");
        (status, msg).into_response()
    }
}

async fn load_or_init_config(path: &Path) -> Result<BotConfig, AppError> {
    match tokio::fs::read_to_string(path).await {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Initialize with sensible defaults matching the provided example
            let default = BotConfig {
                max_won_bookings: 3,
                time_slots: vec![TimeSlot {
                    start_time: "09:00".to_string(),
                    end_time: "17:00".to_string(),
                }],
                paused: false,
                interval_seconds: 5.0,
            };
            save_config(path.to_path_buf(), &default).await?;
            Ok(default)
        }
        Err(e) => Err(AppError::Io(e)),
    }
}

async fn save_config(path: PathBuf, config: &BotConfig) -> Result<(), AppError> {
    let data = serde_json::to_string_pretty(config)?;
    tokio::fs::write(path, data).await?;
    Ok(())
}

async fn get_config(State(state): State<AppState>) -> Result<Json<BotConfig>, AppError> {
    info!("request: GET /config");
    let cfg = state.config.read().await.clone();
    let resp = Json(cfg);
    if let Ok(text) = serde_json::to_string(&resp.0) {
        info!(response = %text, "response: /config");
    }
    Ok(resp)
}

async fn update_config(
    State(state): State<AppState>,
    Query(q): Query<UpdateQuery>,
) -> Result<Json<BotConfig>, AppError> {
    info!(?q, "request: GET /update");
    let mut cfg = state.config.write().await;

    if let Some(v) = q.max_won_bookings { cfg.max_won_bookings = v; }
    if let Some(v) = q.paused { cfg.paused = v; }
    if let Some(v) = q.interval_seconds { cfg.interval_seconds = v; }

    if let Some(ts_json) = q.time_slots {
        let slots: Vec<TimeSlot> = serde_json::from_str(&ts_json)
            .map_err(|e| AppError::BadRequest(format!("invalid time_slots json: {e}")))?;
        cfg.time_slots = slots;
    }

    let path = (*state.config_path).clone();
    save_config(path, &cfg).await?;

    let resp = Json(cfg.clone());
    if let Ok(text) = serde_json::to_string(&resp.0) {
        info!(response = %text, "response: /update");
    }
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // init logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    // Resolve config path relative to current working directory
    let config_path = std::env::current_dir()?.join("bot_config.json");
    let cfg = load_or_init_config(&config_path).await?;

    let state = AppState {
        config: Arc::new(RwLock::new(cfg)),
        config_path: Arc::new(config_path),
    };

    let app = Router::new()
        .route("/config", get(get_config))
        .route("/update", get(update_config))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:8080";
    info!(addr, "starting server");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(())
}
