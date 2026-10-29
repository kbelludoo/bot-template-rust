use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::sync::RwLock;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub struct RequestSigner {
    signing_key: RwLock<String>,
}

impl RequestSigner {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            signing_key: RwLock::new(key.into()),
        }
    }

    pub fn set_key(&self, key: &str) {
        if let Ok(mut k) = self.signing_key.write() {
            *k = key.to_string();
        }
    }

    pub fn compute_signature(
        &self,
        method: &str,
        path: &str,
        timestamp_ms: &str,
        body: &[u8],
    ) -> Option<String> {
        let key = self.signing_key.read().ok()?.clone();
        if key.is_empty() {
            return None;
        }

        let body_hash = hex::encode(Sha256::digest(body));
        let message = format!("{method}\n{path}\n{timestamp_ms}\n{body_hash}");

        let mut mac = HmacSha256::new_from_slice(key.as_bytes()).ok()?;
        mac.update(message.as_bytes());
        Some(hex::encode(mac.finalize().into_bytes()))
    }

    pub fn generate_idempotency_key() -> String {
        Uuid::new_v4().to_string()
    }

    /// DJB2 hash algorithm for partition sharding
    pub fn compute_djb2_hash(s: &str) -> u32 {
        let mut hash: u32 = 5381;
        for b in s.bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u32);
        }
        hash
    }
}
