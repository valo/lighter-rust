use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{LighterError, Result};

#[derive(Debug)]
pub struct NonceManager {
    counter: AtomicU64,
    last_timestamp: AtomicU64,
}

impl NonceManager {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }
    
    pub fn generate(&self) -> Result<u64> {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| LighterError::Nonce(format!("Time error: {}", e)))?
            .as_millis() as u64;
        
        let last_timestamp = self.last_timestamp.load(Ordering::Acquire);
        
        if current_timestamp > last_timestamp {
            self.last_timestamp.store(current_timestamp, Ordering::Release);
            self.counter.store(0, Ordering::Release);
            Ok(current_timestamp * 1000)
        } else {
            let counter = self.counter.fetch_add(1, Ordering::AcqRel);
            if counter >= 999 {
                return Err(LighterError::Nonce(
                    "Too many nonces generated in the same millisecond".to_string()
                ));
            }
            Ok(last_timestamp * 1000 + counter + 1)
        }
    }
}

impl Default for NonceManager {
    fn default() -> Self {
        Self::new()
    }
}