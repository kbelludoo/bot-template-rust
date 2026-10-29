use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaminaType {
    Decreasing,        // 42h -> 0h (Tibia, Huntera)
    IncreasingFatigue, // 0% -> 100% (Leveling, Protobuf)
}

#[derive(Debug, Clone)]
pub struct StaminaConfig {
    pub stamina_type: StaminaType,
    pub max_fatigue_percent: f64,    // e.g. 80.0%
    pub resume_fatigue_percent: f64, // e.g. 30.0%
    pub min_stamina_hours: f64,      // e.g. 14.0h
    pub resume_stamina_hours: f64,   // e.g. 38.0h
    pub min_rest_duration: Duration, // e.g. 5 minutes minimum hysteresis
}

impl Default for StaminaConfig {
    fn default() -> Self {
        Self {
            stamina_type: StaminaType::IncreasingFatigue,
            max_fatigue_percent: 80.0,
            resume_fatigue_percent: 30.0,
            min_stamina_hours: 14.0,
            resume_stamina_hours: 38.0,
            min_rest_duration: Duration::from_secs(5 * 60),
        }
    }
}

pub struct StaminaController {
    cfg: StaminaConfig,
    is_resting: RwLock<bool>,
    rest_start_time: RwLock<Option<Instant>>,
    fatigue_percent: RwLock<f64>,
    stamina_hours: RwLock<f64>,
}

impl StaminaController {
    pub fn new(cfg: StaminaConfig) -> Self {
        Self {
            cfg,
            is_resting: RwLock::new(false),
            rest_start_time: RwLock::new(None),
            fatigue_percent: RwLock::new(0.0),
            stamina_hours: RwLock::new(42.0),
        }
    }

    pub async fn update_fatigue(&self, fatigue_pct: f64) {
        *self.fatigue_percent.write().await = fatigue_pct;
    }

    pub async fn update_stamina_hours(&self, hours: f64) {
        *self.stamina_hours.write().await = hours;
    }

    pub async fn should_hunt(&self) -> bool {
        let resting = *self.is_resting.read().await;

        if resting {
            if let Some(start) = *self.rest_start_time.read().await {
                if start.elapsed() < self.cfg.min_rest_duration {
                    return false; // Still in mandatory hysteresis rest
                }
            }

            match self.cfg.stamina_type {
                StaminaType::IncreasingFatigue => {
                    let fatigue = *self.fatigue_percent.read().await;
                    if fatigue <= self.cfg.resume_fatigue_percent {
                        let mut r = self.is_resting.write().await;
                        *r = false;
                        *self.rest_start_time.write().await = None;
                        return true;
                    }
                    false
                }
                StaminaType::Decreasing => {
                    let hours = *self.stamina_hours.read().await;
                    if hours >= self.cfg.resume_stamina_hours {
                        let mut r = self.is_resting.write().await;
                        *r = false;
                        *self.rest_start_time.write().await = None;
                        return true;
                    }
                    false
                }
            }
        } else {
            // Currently hunting: check if we hit critical threshold
            match self.cfg.stamina_type {
                StaminaType::IncreasingFatigue => {
                    let fatigue = *self.fatigue_percent.read().await;
                    if fatigue >= self.cfg.max_fatigue_percent {
                        let mut r = self.is_resting.write().await;
                        *r = true;
                        *self.rest_start_time.write().await = Some(Instant::now());
                        return false;
                    }
                    true
                }
                StaminaType::Decreasing => {
                    let hours = *self.stamina_hours.read().await;
                    if hours <= self.cfg.min_stamina_hours {
                        let mut r = self.is_resting.write().await;
                        *r = true;
                        *self.rest_start_time.write().await = Some(Instant::now());
                        return false;
                    }
                    true
                }
            }
        }
    }
}
