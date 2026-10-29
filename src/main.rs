#![allow(dead_code)]
#![allow(unused_imports)]

mod auth;
mod config;
mod engine;
mod network;
mod protocol;
mod server;
mod subsystem;

use config::AccountConfig;
use engine::Supervisor;
use server::WebServer;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const DEFAULT_ACCOUNTS_EXAMPLE: &str = include_str!("../accounts.example.json");

fn format_number(n: u64) -> String {
    if n < 1000 {
        format!("{n}")
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{:.2}kk", n as f64 / 1_000_000.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    println!("================================================================================");
    println!("       ⚡ NATIVE RUST MULTI-BOT ENGINE (ZERO-GC & ALTA PERFORMANCE) ⚡         ");
    println!("================================================================================");

    // 1. Carregar contas do accounts.json
    let config_path = "accounts.json";
    let content = match std::fs::read_to_string(config_path) {
        Ok(data) => data,
        Err(_) => {
            warn!("Arquivo accounts.json não encontrado. Criando template padrão...");
            let _ = std::fs::write(config_path, DEFAULT_ACCOUNTS_EXAMPLE);
            DEFAULT_ACCOUNTS_EXAMPLE.to_string()
        }
    };

    let all_configs: Vec<AccountConfig> = match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            error!("Falha crítica ao analisar accounts.json: {}", e);
            std::process::exit(1);
        }
    };

    let target_acc = std::env::var("ACCOUNT").unwrap_or_default();
    let mut active_configs = Vec::new();

    for a in all_configs {
        if !a.enabled && a.id != target_acc {
            continue;
        }
        if !target_acc.is_empty() && a.id != target_acc {
            continue;
        }
        active_configs.push(a);
    }

    if active_configs.is_empty() {
        warn!("Nenhuma conta com 'enabled: true' encontrada em accounts.json.");
        info!("Edite accounts.json e habilite sua conta para iniciar o bot.");
    }

    // 2. Iniciar o Supervisor de Bots
    let supervisor = Arc::new(Supervisor::new(active_configs));
    supervisor.start_all().await;

    // 3. Iniciar o Servidor Web e Dashboard
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let web_server = WebServer::new(supervisor.clone(), port);
    tokio::spawn(async move {
        if let Err(e) = web_server.run().await {
            error!("Erro no servidor web: {}", e);
        }
    });

    // 4. Console Ticker no Terminal
    let sup_clone = supervisor.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(4)).await;
            let mut total_kills = 0;
            let mut total_gold = 0;
            let bots = sup_clone.bots();
            for b in bots {
                total_kills += b.kills.load(Ordering::Relaxed);
                total_gold += b.gold.load(Ordering::Relaxed);
            }
            info!(
                "📊 [TICKER] Contas ativas: {} | Total Kills: {} | Total Ouro: {}",
                bots.len(),
                format_number(total_kills),
                format_number(total_gold)
            );
        }
    });

    // 5. Graceful Shutdown
    signal::ctrl_c().await?;
    info!("🛑 Sinal de encerramento recebido. Desligando bots com segurança...");

    Ok(())
}
