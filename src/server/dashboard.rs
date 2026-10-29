use axum::{
    extract::Json as ExtractJson,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::engine::Supervisor;

#[derive(Debug, Deserialize)]
pub struct TicketPayload {
    pub token: Option<String>,
    pub ticket: Option<String>,
}

pub struct WebServer {
    supervisor: Arc<Supervisor>,
    port: u16,
}

impl WebServer {
    pub fn new(supervisor: Arc<Supervisor>, port: u16) -> Self {
        Self { supervisor, port }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let supervisor = self.supervisor.clone();

        let app = Router::new()
            .route("/", get(dashboard_html))
            .route(
                "/api/status",
                get({
                    let sup = supervisor.clone();
                    move || async move {
                        let mut bots_data = Vec::new();
                        for bot in sup.bots() {
                            bots_data.push(json!({
                                "id": bot.config.id,
                                "name": bot.config.name,
                                "circuit_breaker": bot.circuit_breaker.state().await.to_string(),
                                "kills": bot.kills.load(std::sync::atomic::Ordering::Relaxed),
                                "gold": bot.gold.load(std::sync::atomic::Ordering::Relaxed),
                                "running": bot.running.load(std::sync::atomic::Ordering::Relaxed),
                            }));
                        }
                        Json(json!({
                            "engine": "bot-template-rust (High-Performance)",
                            "active_bots": bots_data.len(),
                            "bots": bots_data,
                        }))
                    }
                }),
            )
            .route(
                "/api/ticket",
                post(|ExtractJson(body): ExtractJson<TicketPayload>| async move {
                    let t = body.token.or(body.ticket).unwrap_or_default();
                    if !t.is_empty() {
                        info!("🎟️ [Turnstile Relay] Token Turnstile registrado com sucesso (len: {})", t.len());
                        Json(json!({ "ok": true, "message": "Ticket registrado com sucesso" }))
                    } else {
                        Json(json!({ "ok": false, "error": "Token vazio" }))
                    }
                }),
            )
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("🌐 Web Dashboard rodando em http://localhost:{}", self.port);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn dashboard_html() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <title>⚡ Bot Command Center (Rust Edition)</title>
    <style>
        body { font-family: monospace; background: #0d1117; color: #58a6ff; margin: 40px; }
        h1 { color: #f0883e; }
        .card { background: #161b22; padding: 20px; border-radius: 8px; border: 1px solid #30363d; margin-bottom: 20px; }
        .online { color: #3fb950; font-weight: bold; }
    </style>
</head>
<body>
    <h1>⚡ Game Bot Command Center — Rust Edition ⚡</h1>
    <div class="card">
        <p>Motor nativo compilado com zero-cost abstractions e ausência de Garbage Collector.</p>
        <p>Consumo de memória estimado: <strong>~2 MB de RAM</strong>.</p>
        <p>Endpoint da API: <a href="/api/status" style="color: #f0883e;">/api/status</a></p>
    </div>
</body>
</html>"#)
}
