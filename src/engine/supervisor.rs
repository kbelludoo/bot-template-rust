use crate::config::AccountConfig;
use crate::engine::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::engine::stamina::{StaminaConfig, StaminaController};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct BotInstance {
    pub config: AccountConfig,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub stamina: Arc<StaminaController>,
    pub kills: AtomicU64,
    pub gold: AtomicU64,
    pub running: AtomicBool,
}

pub struct Supervisor {
    bots: Vec<Arc<BotInstance>>,
}

impl Supervisor {
    pub fn new(configs: Vec<AccountConfig>) -> Self {
        let mut bots = Vec::new();
        for cfg in configs {
            let cb_config = CircuitBreakerConfig::default();
            let cb = Arc::new(CircuitBreaker::new(&cfg.id, cb_config));
            let stam = Arc::new(StaminaController::new(StaminaConfig::default()));

            bots.push(Arc::new(BotInstance {
                config: cfg,
                circuit_breaker: cb,
                stamina: stam,
                kills: AtomicU64::new(0),
                gold: AtomicU64::new(0),
                running: AtomicBool::new(false),
            }));
        }
        Self { bots }
    }

    pub fn bots(&self) -> &[Arc<BotInstance>] {
        &self.bots
    }

    pub async fn start_all(&self) {
        for bot in &self.bots {
            let b = Arc::clone(bot);
            tokio::spawn(async move {
                Self::run_bot_loop(b).await;
            });
        }
    }

    async fn run_bot_loop(bot: Arc<BotInstance>) {
        bot.running.store(true, Ordering::SeqCst);
        info!("🚀 Bot [{}] iniciado com sucesso.", bot.config.name);

        while bot.running.load(Ordering::SeqCst) {
            // 1. Verificar Disjuntor Anti-Cheat
            if !bot.circuit_breaker.can_execute().await {
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            // 2. Verificar Fadiga / Stamina
            if !bot.stamina.should_hunt().await {
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            // 3. Execução de Caçada / Ações Idle
            if bot.config.settings.auto_hunt {
                bot.kills.fetch_add(1, Ordering::Relaxed);
                bot.gold.fetch_add(50, Ordering::Relaxed);
                bot.circuit_breaker.record_success().await;
            }

            sleep(Duration::from_millis(1500)).await;
        }

        info!("🛑 Bot [{}] finalizado.", bot.config.name);
    }
}
