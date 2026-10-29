use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionTier {
    pub level_req: i32,
    pub cost_gold: i64,
    pub target_name: String,
    pub action_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlessingConfig {
    pub max_blessings: i32,
    pub cost_per_bless: i64,
}

#[allow(dead_code)]
pub struct PromotionManager {
    tiers: Vec<PromotionTier>,
    bless_cfg: BlessingConfig,
    state: RwLock<(bool, i32)>, // (is_promoted, current_blessings)
}

impl PromotionManager {
    pub fn new(tiers: Vec<PromotionTier>, bless_cfg: BlessingConfig) -> Self {
        Self {
            tiers,
            bless_cfg,
            state: RwLock::new((false, 0)),
        }
    }

    pub fn check_eligible_promotion(
        &self,
        current_level: i32,
        current_gold: i64,
        current_title: &str,
    ) -> Option<PromotionTier> {
        for tier in &self.tiers {
            if current_level >= tier.level_req
                && current_gold >= tier.cost_gold
                && current_title != tier.target_name
            {
                return Some(tier.clone());
            }
        }
        None
    }

    pub fn missing_blessings(&self, active_bless_count: i32) -> i32 {
        if active_bless_count < self.bless_cfg.max_blessings {
            self.bless_cfg.max_blessings - active_bless_count
        } else {
            0
        }
    }

    pub fn can_afford_blessings(&self, active_bless_count: i32, gold: i64) -> bool {
        let missing = self.missing_blessings(active_bless_count);
        if missing <= 0 {
            return false;
        }
        let total_cost = (missing as i64) * self.bless_cfg.cost_per_bless;
        gold >= total_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotion_and_blessings() {
        let tiers = vec![PromotionTier {
            level_req: 20,
            cost_gold: 20000,
            target_name: "Elite Knight".to_string(),
            action_code: "buy_promotion".to_string(),
        }];
        let b_cfg = BlessingConfig {
            max_blessings: 5,
            cost_per_bless: 10000,
        };
        let pm = PromotionManager::new(tiers, b_cfg);

        // Nível insuficiente
        assert!(pm.check_eligible_promotion(19, 50000, "Knight").is_none());
        // Ouro insuficiente
        assert!(pm.check_eligible_promotion(20, 19999, "Knight").is_none());
        // Elegível
        let promo = pm.check_eligible_promotion(20, 20000, "Knight").unwrap();
        assert_eq!(promo.target_name, "Elite Knight");

        // Bênçãos
        assert_eq!(pm.missing_blessings(3), 2);
        assert!(!pm.can_afford_blessings(3, 19999));
        assert!(pm.can_afford_blessings(3, 20000));
        assert_eq!(pm.missing_blessings(5), 0);
        assert!(!pm.can_afford_blessings(5, 100000));
    }
}
