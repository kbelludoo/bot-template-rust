use std::time::Duration;
use tracing::info;

pub struct SocketClientConfig {
    pub url: String,
    pub ping_interval: Duration,
    pub reconnect_base: Duration,
}

impl Default for SocketClientConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8080/ws".to_string(),
            ping_interval: Duration::from_secs(25),
            reconnect_base: Duration::from_millis(500),
        }
    }
}

pub struct SocketClient {
    pub config: SocketClientConfig,
}

impl SocketClient {
    pub fn new(config: SocketClientConfig) -> Self {
        Self { config }
    }

    pub fn url(&self) -> &str {
        &self.config.url
    }
}
