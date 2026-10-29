use chrono::{DateTime, Utc};
use std::sync::RwLock;

pub struct DailyManager {
    last_claim_date: RwLock<Option<String>>,
}

impl Default for DailyManager {
    fn default() -> Self {
        Self {
            last_claim_date: RwLock::new(None),
        }
    }
}

impl DailyManager {
    pub fn should_claim_daily(&self, now: DateTime<Utc>) -> bool {
        let today = now.format("%Y-%m-%d").to_string();
        let last = self.last_claim_date.read().unwrap();
        match &*last {
            Some(d) => d != &today,
            None => true,
        }
    }

    pub fn record_claim(&self, now: DateTime<Utc>) {
        let today = now.format("%Y-%m-%d").to_string();
        *self.last_claim_date.write().unwrap() = Some(today);
    }
}
