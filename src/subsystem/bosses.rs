use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBossEvent {
    pub id: String,
    pub name: String,
    pub spawn_time: DateTime<Utc>,
    pub min_level: i32,
    pub portal_id: String,
    pub is_open: bool,
    pub already_done: bool,
}

pub struct BossScheduler {
    bosses: RwLock<Vec<WorldBossEvent>>,
}

impl BossScheduler {
    pub fn new() -> Self {
        Self {
            bosses: RwLock::new(Vec::new()),
        }
    }

    pub fn add_boss(&self, boss: WorldBossEvent) {
        let mut list = self.bosses.write().unwrap();
        list.push(boss);
    }

    pub fn get_ready_bosses(&self, character_level: i32) -> Vec<WorldBossEvent> {
        let list = self.bosses.read().unwrap();
        let now = Utc::now();
        let mut ready = Vec::new();

        for b in list.iter() {
            if b.already_done || character_level < b.min_level {
                continue;
            }
            // Se estiver aberto ou dentro da janela de 10 minutos do spawn
            let time_diff = (now - b.spawn_time).num_minutes();
            if b.is_open || (now >= b.spawn_time && time_diff < 10) {
                ready.push(b.clone());
            }
        }
        ready
    }

    pub fn mark_completed(&self, boss_id: &str) {
        let mut list = self.bosses.write().unwrap();
        for b in list.iter_mut() {
            if b.id == boss_id {
                b.already_done = true;
                b.is_open = false;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boss_scheduler() {
        let scheduler = BossScheduler::new();
        let boss = WorldBossEvent {
            id: "ferumbras".to_string(),
            name: "Ferumbras".to_string(),
            spawn_time: Utc::now(),
            min_level: 100,
            portal_id: "portal_1".to_string(),
            is_open: true,
            already_done: false,
        };
        scheduler.add_boss(boss);

        let ready = scheduler.get_ready_bosses(150);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].name, "Ferumbras");

        scheduler.mark_completed("ferumbras");
        let after = scheduler.get_ready_bosses(150);
        assert_eq!(after.len(), 0);
    }
}
