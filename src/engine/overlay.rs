use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// StateOverlay preserves unknown fields from game server state envelopes.
/// It merges known bot mutations over the original JSON payload, preventing
/// "malformed save" rejections from servers.
#[derive(Default)]
pub struct StateOverlay {
    raw_payload: RwLock<HashMap<String, Value>>,
}

impl StateOverlay {
    pub fn new() -> Self {
        Self {
            raw_payload: RwLock::new(HashMap::new()),
        }
    }

    pub async fn ingest(&self, json_bytes: &[u8]) -> Result<(), serde_json::Error> {
        if let Ok(Value::Object(map)) = serde_json::from_slice::<Value>(json_bytes) {
            let mut current = self.raw_payload.write().await;
            for (k, v) in map {
                current.insert(k, v);
            }
        }
        Ok(())
    }

    pub async fn set(&self, key: &str, val: Value) {
        self.raw_payload.write().await.insert(key.to_string(), val);
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        self.raw_payload.read().await.get(key).cloned()
    }

    pub async fn export_json(&self) -> String {
        let current = self.raw_payload.read().await;
        serde_json::to_string(&*current).unwrap_or_else(|_| "{}".to_string())
    }
}
