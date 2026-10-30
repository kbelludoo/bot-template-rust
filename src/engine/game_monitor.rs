use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameMetricsSnapshot {
    pub gold: u64,
    pub exp: u64,
    pub stamina: u32,
    pub kills: u64,
    pub level: u32,
}

pub struct GameMonitor {
    account_id: String,
    last_gold: AtomicU64,
    last_exp: AtomicU64,
    last_stamina: AtomicU32,
    last_kills: AtomicU64,
    last_level: AtomicU32,
    last_activity_time: Mutex<Instant>,
    is_stalled: AtomicBool,
    stall_reason: Mutex<Option<String>>,
    stall_threshold_secs: u64,
}

impl GameMonitor {
    pub fn new(account_id: &str, stall_threshold_secs: u64) -> Self {
        Self {
            account_id: account_id.to_string(),
            last_gold: AtomicU64::new(0),
            last_exp: AtomicU64::new(0),
            last_stamina: AtomicU32::new(0),
            last_kills: AtomicU64::new(0),
            last_level: AtomicU32::new(0),
            last_activity_time: Mutex::new(Instant::now()),
            is_stalled: AtomicBool::new(false),
            stall_reason: Mutex::new(None),
            stall_threshold_secs,
        }
    }

    /// Avalia se algum número aumentou ou diminuiu.
    /// Retorna `true` se um stall for detectado (nenhum número variou além do tempo limite).
    pub fn record_metrics(&self, current: GameMetricsSnapshot) -> bool {
        let prev_gold = self.last_gold.load(Ordering::Relaxed);
        let prev_exp = self.last_exp.load(Ordering::Relaxed);
        let prev_stamina = self.last_stamina.load(Ordering::Relaxed);
        let prev_kills = self.last_kills.load(Ordering::Relaxed);
        let prev_level = self.last_level.load(Ordering::Relaxed);

        // Primeiro registro inicial
        if prev_gold == 0
            && prev_exp == 0
            && prev_stamina == 0
            && prev_kills == 0
            && prev_level == 0
        {
            self.last_gold.store(current.gold, Ordering::Relaxed);
            self.last_exp.store(current.exp, Ordering::Relaxed);
            self.last_stamina.store(current.stamina, Ordering::Relaxed);
            self.last_kills.store(current.kills, Ordering::Relaxed);
            self.last_level.store(current.level, Ordering::Relaxed);
            if let Ok(mut t) = self.last_activity_time.lock() {
                *t = Instant::now();
            }
            return false;
        }

        // Regra de ouro: se QUALQUER número aumentou OU diminuiu (ex: gastou gold, recuperou stamina, subiu exp)
        let gold_changed = current.gold != prev_gold;
        let exp_changed = current.exp != prev_exp;
        let stamina_changed = current.stamina != prev_stamina;
        let kills_changed = current.kills != prev_kills;
        let level_changed = current.level != prev_level;

        let has_activity =
            gold_changed || exp_changed || stamina_changed || kills_changed || level_changed;

        if has_activity {
            // Houve movimento real no jogo! Resetamos o monitor de inércia
            self.last_gold.store(current.gold, Ordering::Relaxed);
            self.last_exp.store(current.exp, Ordering::Relaxed);
            self.last_stamina.store(current.stamina, Ordering::Relaxed);
            self.last_kills.store(current.kills, Ordering::Relaxed);
            self.last_level.store(current.level, Ordering::Relaxed);
            if let Ok(mut t) = self.last_activity_time.lock() {
                *t = Instant::now();
            }

            if self.is_stalled.swap(false, Ordering::SeqCst) {
                info!(
                    "🟢 [GameMonitor] Bot [{}] retomou atividade! Métricas voltaram a oscilar.",
                    self.account_id
                );
                if let Ok(mut r) = self.stall_reason.lock() {
                    *r = None;
                }
            }
            return false;
        }

        // Se NENHUM número variou, verifica se excedeu o limiar de inércia
        let elapsed = if let Ok(t) = self.last_activity_time.lock() {
            t.elapsed().as_secs()
        } else {
            0
        };

        if elapsed >= self.stall_threshold_secs {
            let was_stalled = self.is_stalled.swap(true, Ordering::SeqCst);
            let reason = format!(
                "Forte indício de bot parado: nenhum número (Gold: {}, EXP: {}, Stamina: {}, Kills: {}, Lvl: {}) aumentou ou diminuiu há {}s",
                current.gold, current.exp, current.stamina, current.kills, current.level, elapsed
            );

            if !was_stalled {
                warn!(
                    "⚠️ [GameMonitor] [STALL DETECTADO] [{}] {}",
                    self.account_id, reason
                );
            }
            if let Ok(mut r) = self.stall_reason.lock() {
                *r = Some(reason);
            }
            return true;
        }

        false
    }

    pub fn is_stalled(&self) -> bool {
        self.is_stalled.load(Ordering::Relaxed)
    }

    pub fn stall_reason(&self) -> Option<String> {
        self.stall_reason.lock().ok().and_then(|r| r.clone())
    }

    pub fn force_activity(&self) {
        if let Ok(mut t) = self.last_activity_time.lock() {
            *t = Instant::now();
        }
        self.is_stalled.store(false, Ordering::SeqCst);
        if let Ok(mut r) = self.stall_reason.lock() {
            *r = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_game_monitor_activity_detection() {
        let monitor = GameMonitor::new("test-acc", 1); // 1 segundo para teste

        let initial = GameMetricsSnapshot {
            gold: 1000,
            exp: 5000,
            stamina: 100,
            kills: 50,
            level: 10,
        };

        // 1. Registro inicial
        assert!(!monitor.record_metrics(initial));
        assert!(!monitor.is_stalled());

        // 2. Dorme 1.2 segundos mantendo os números rigorosamente iguais
        sleep(Duration::from_millis(1200));
        let stalled = monitor.record_metrics(initial);
        assert!(
            stalled,
            "Deveria acusar forte indício de bot parado por números estáticos"
        );
        assert!(monitor.is_stalled());
        assert!(monitor.stall_reason().is_some());

        // 3. Qualquer número mudou (ex: stamina diminuiu de 100 para 99 ao caçar)
        let hunted = GameMetricsSnapshot {
            stamina: 99,
            ..initial
        };
        let still_stalled = monitor.record_metrics(hunted);
        assert!(
            !still_stalled,
            "Deveria desarmar o stall pois um número diminuiu"
        );
        assert!(!monitor.is_stalled());
        assert!(monitor.stall_reason().is_none());

        // 4. Qualquer número mudou (ex: gastou gold comprando poção de 1000 para 800)
        let bought = GameMetricsSnapshot {
            gold: 800,
            stamina: 99,
            ..initial
        };
        assert!(
            !monitor.record_metrics(bought),
            "Mudança de gold deve resetar inércia"
        );
    }
}
