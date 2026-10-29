use std::time::Duration;
use tokio::sync::RwLock;

pub struct AdaptiveSpeedTuner {
    base_delay: Duration,
    min_delay: Duration,
    max_delay: Duration,
    current_delay: RwLock<Duration>,
}

impl AdaptiveSpeedTuner {
    pub fn new(base: Duration, min: Duration, max: Duration) -> Self {
        Self {
            base_delay: base,
            min_delay: min,
            max_delay: max,
            current_delay: RwLock::new(base),
        }
    }

    pub async fn current_delay(&self) -> Duration {
        *self.current_delay.read().await
    }

    pub async fn record_success(&self) {
        let mut d = self.current_delay.write().await;
        // Gradually recover towards base delay
        if *d > self.base_delay {
            *d = d.saturating_sub(Duration::from_millis(50)).max(self.base_delay);
        }
    }

    pub async fn record_slowdown(&self) {
        let mut d = self.current_delay.write().await;
        // Increase delay on rate limit or high server load
        *d = (*d + Duration::from_millis(250)).min(self.max_delay);
    }
}

pub struct AntiWipeTracker {
    consecutive_deaths: RwLock<usize>,
    max_allowed_deaths: usize,
}

impl AntiWipeTracker {
    pub fn new(max_deaths: usize) -> Self {
        Self {
            consecutive_deaths: RwLock::new(0),
            max_allowed_deaths: max_deaths,
        }
    }

    pub async fn record_death(&self) -> bool {
        let mut d = self.consecutive_deaths.write().await;
        *d += 1;
        *d >= self.max_allowed_deaths
    }

    pub async fn record_kill(&self) {
        *self.consecutive_deaths.write().await = 0;
    }
}
