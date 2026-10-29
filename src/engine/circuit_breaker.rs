use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Tripped, anti-cheat protection active
    HalfOpen, // Testing recovery after cooldown
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "CLOSED (Operação Normal)"),
            CircuitState::Open => write!(f, "OPEN (Desarmado / Proteção Ativa)"),
            CircuitState::HalfOpen => write!(f, "HALF-OPEN (Testando Recuperação)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub max_failures: usize,
    pub base_cooldown: Duration,
    pub max_cooldown: Duration,
    pub backoff_multiplier: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_failures: 2,
            base_cooldown: Duration::from_secs(15 * 60),
            max_cooldown: Duration::from_secs(60 * 60),
            backoff_multiplier: 2.0,
        }
    }
}

pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failures: RwLock<usize>,
    consec_trips: RwLock<usize>,
    last_failure: RwLock<String>,
    opened_at: RwLock<Option<Instant>>,
    current_wait: RwLock<Duration>,
    generation: AtomicU64,
    recovering: RwLock<bool>,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let base = config.base_cooldown;
        Self {
            name: name.into(),
            config,
            state: RwLock::new(CircuitState::Closed),
            failures: RwLock::new(0),
            consec_trips: RwLock::new(0),
            last_failure: RwLock::new(String::new()),
            opened_at: RwLock::new(None),
            current_wait: RwLock::new(base),
            generation: AtomicU64::new(0),
            recovering: RwLock::new(false),
        }
    }

    pub async fn state(&self) -> CircuitState {
        let current_state = *self.state.read().await;
        if current_state == CircuitState::Open {
            if let Some(opened) = *self.opened_at.read().await {
                let wait = *self.current_wait.read().await;
                if opened.elapsed() >= wait {
                    let mut s = self.state.write().await;
                    if *s == CircuitState::Open {
                        *s = CircuitState::HalfOpen;
                        self.generation.fetch_add(1, Ordering::SeqCst);
                        info!("[{}] Cooldown expirou. Estado -> HALF-OPEN.", self.name);
                        return CircuitState::HalfOpen;
                    }
                }
            }
        }
        current_state
    }

    pub async fn can_execute(&self) -> bool {
        self.state().await == CircuitState::Closed
    }

    pub async fn record_success(&self) {
        let mut s = self.state.write().await;
        if *s == CircuitState::Closed || *s == CircuitState::HalfOpen {
            if *s == CircuitState::HalfOpen {
                info!("[{}] Recuperação bem-sucedida! Disjuntor rearmado -> CLOSED.", self.name);
                *s = CircuitState::Closed;
            }
            *self.failures.write().await = 0;
            *self.consec_trips.write().await = 0;
            *self.current_wait.write().await = self.config.base_cooldown;
            *self.recovering.write().await = false;
        }
    }

    pub async fn record_failure(&self, reason: &str, status_code: u16) {
        let is_suspicious = status_code == 429
            || status_code == 403
            || reason.contains("experiência rápida")
            || reason.contains("save malformado")
            || reason.contains("outraAba")
            || reason.contains("rate limit");

        let mut fail_count = self.failures.write().await;
        *fail_count += 1;
        *self.last_failure.write().await = reason.to_string();

        warn!(
            "[{}] Falha registrada: '{}' (Total: {}/{})",
            self.name, reason, *fail_count, self.config.max_failures
        );

        if is_suspicious || *fail_count >= self.config.max_failures {
            self.trip(reason).await;
        }
    }

    pub async fn trip(&self, reason: &str) {
        let mut s = self.state.write().await;
        *s = CircuitState::Open;
        *self.opened_at.write().await = Some(Instant::now());

        let mut trips = self.consec_trips.write().await;
        *trips += 1;

        let multiplier = self.config.backoff_multiplier.powi(*trips as i32 - 1);
        let calculated = self.config.base_cooldown.mul_f64(multiplier);
        let actual_wait = calculated.min(self.config.max_cooldown);

        *self.current_wait.write().await = actual_wait;
        self.generation.fetch_add(1, Ordering::SeqCst);

        warn!(
            "🚨 [{}] DISJUNTOR DESARMADO! Motivo: '{}'. Cooldown: {:?}.",
            self.name, reason, actual_wait
        );
    }
}
