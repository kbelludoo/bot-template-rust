pub struct PotionManager {
    pub min_hp_ratio: f64,
}

impl Default for PotionManager {
    fn default() -> Self {
        Self { min_hp_ratio: 0.50 }
    }
}

impl PotionManager {
    pub fn should_drink_hp_potion(&self, current_hp: u32, max_hp: u32) -> bool {
        if max_hp == 0 {
            return false;
        }
        (current_hp as f64 / max_hp as f64) <= self.min_hp_ratio
    }
}
