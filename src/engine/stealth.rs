use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct StealthConfig {
    pub enabled: bool,
    pub min_action_delay_ms: u64, // Delay entre cliques consecutivos (ex: 300ms)
    pub max_action_delay_ms: u64, // Delay máximo entre cliques consecutivos (ex: 600ms)
    pub jitter_variance: f64,     // Variação orgânica no timer (ex: 0.20 = ±20%)
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_action_delay_ms: 300,
            max_action_delay_ms: 600,
            jitter_variance: 0.20, // ±20% de variação natural no timer
        }
    }
}

pub struct StealthEngine {
    config: StealthConfig,
}

impl StealthEngine {
    pub fn new(config: StealthConfig) -> Self {
        Self { config }
    }

    /// Retorna um delay com jitter randômico orgânico (evita relógio mecânico do servidor sem pausar o farm)
    pub fn jitter_delay(&self, base: Duration) -> Duration {
        if !self.config.enabled || self.config.jitter_variance <= 0.0 {
            return base;
        }

        let mut rng = rand::thread_rng();
        let factor = 1.0 + rng.gen_range(-self.config.jitter_variance..=self.config.jitter_variance);
        let millis = (base.as_millis() as f64 * factor).max(50.0);
        Duration::from_millis(millis as u64)
    }

    /// Retorna o atraso humano orgânico entre cliques consecutivos (300ms a 600ms)
    pub fn human_action_delay(&self) -> Duration {
        if !self.config.enabled {
            return Duration::from_millis(50);
        }
        let mut rng = rand::thread_rng();
        let ms = rng.gen_range(self.config.min_action_delay_ms..=self.config.max_action_delay_ms);
        Duration::from_millis(ms)
    }

    /// Executa o pacing humano entre requisições em lote (anti-burst: evita 5 intents no mesmo ms)
    pub async fn pacing_step(&self) {
        let delay = self.human_action_delay();
        tokio::time::sleep(delay).await;
    }

    /// Injeta headers realistas de navegador desktop para evitar fingerprinting de datacenter
    pub fn apply_browser_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .header("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
            .header("Sec-Ch-Ua-Mobile", "?0")
            .header("Sec-Ch-Ua-Platform", "\"Windows\"")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Accept-Language", "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7")
    }
}
