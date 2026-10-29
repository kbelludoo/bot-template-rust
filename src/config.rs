use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub credentials: Credentials,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default, rename = "bindIP")]
    pub bind_ip: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_hunt_target", rename = "huntTarget")]
    pub hunt_target: String,
    #[serde(default, rename = "autoHunt")]
    pub auto_hunt: bool,
    #[serde(default = "default_true", rename = "autoSell")]
    pub auto_sell: bool,
    #[serde(default = "default_true", rename = "autoHeal")]
    pub auto_heal: bool,
    #[serde(default = "default_true", rename = "autoDaily")]
    pub auto_daily: bool,
    #[serde(default = "default_true", rename = "autoEquip")]
    pub auto_equip: bool,
    #[serde(default = "default_true", rename = "autoTreino")]
    pub auto_treino: bool,
    #[serde(default = "default_true", rename = "autoCatch")]
    pub auto_catch: bool,
    #[serde(default = "default_ball", rename = "ballType")]
    pub ball_type: String,
    #[serde(default = "default_heal_threshold", rename = "healThreshold")]
    pub heal_threshold: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hunt_target: default_hunt_target(),
            auto_hunt: false,
            auto_sell: true,
            auto_heal: true,
            auto_daily: true,
            auto_equip: true,
            auto_treino: true,
            auto_catch: true,
            ball_type: default_ball(),
            heal_threshold: default_heal_threshold(),
        }
    }
}

fn default_hunt_target() -> String {
    "auto".to_string()
}
fn default_true() -> bool {
    true
}
fn default_ball() -> String {
    "auto".to_string()
}
fn default_heal_threshold() -> f64 {
    0.5
}
