use crate::error::{LighterError, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
            self.last_timestamp
                .store(current_timestamp, Ordering::Release);
            self.counter.store(0, Ordering::Release);
            Ok(current_timestamp * 1000)
        } else {
            let counter = self.counter.fetch_add(1, Ordering::AcqRel);
            if counter >= 999 {
                return Err(LighterError::Nonce(
                    "Too many nonces generated in the same millisecond".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_nonce_uniqueness() {
        let manager = NonceManager::new();
        let mut nonces = HashSet::new();

        // Generate 100 nonces and ensure they're all unique
        for _ in 0..100 {
            let nonce = manager.generate().unwrap();
            assert!(nonces.insert(nonce), "Duplicate nonce generated: {}", nonce);
        }
    }

    #[test]
    fn test_nonce_increasing() {
        let manager = NonceManager::new();
        let mut previous = 0;

        // Generate nonces and ensure they're increasing
        for _ in 0..50 {
            let nonce = manager.generate().unwrap();
            assert!(
                nonce > previous,
                "Nonce {} is not greater than previous {}",
                nonce,
                previous
            );
            previous = nonce;
        }
    }

    #[test]
    fn test_nonce_concurrent_generation() {
        let manager = Arc::new(NonceManager::new());
        let mut handles = vec![];
        let nonces = Arc::new(std::sync::Mutex::new(HashSet::new()));

        // Spawn multiple threads generating nonces concurrently
        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let nonces_clone = Arc::clone(&nonces);

            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let nonce = manager_clone.generate().unwrap();
                    let mut nonces_guard = nonces_clone.lock().unwrap();
                    assert!(
                        nonces_guard.insert(nonce),
                        "Duplicate nonce in concurrent generation: {}",
                        nonce
                    );
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify we generated exactly 100 unique nonces
        let final_nonces = nonces.lock().unwrap();
        assert_eq!(final_nonces.len(), 100);
    }

    #[test]
    fn test_nonce_timestamp_format() {
        let manager = NonceManager::new();
        let nonce = manager.generate().unwrap();

        // Nonce should be a timestamp in microseconds (13+ digits)
        assert!(
            nonce > 1_000_000_000_000,
            "Nonce should be at least 13 digits"
        );

        // The nonce should be close to current time in microseconds
        let current_time_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            * 1000;

        // Allow for 1 second difference due to execution time
        assert!(nonce <= current_time_micros + 1_000_000);
        assert!(nonce >= current_time_micros - 1_000_000);
    }

    #[test]
    fn test_nonce_counter_reset() {
        let manager = NonceManager::new();

        // Generate a nonce
        let first_nonce = manager.generate().unwrap();

        // Sleep for a bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Generate another nonce - counter should reset with new timestamp
        let second_nonce = manager.generate().unwrap();

        // The second nonce should be based on a new timestamp
        assert!(second_nonce > first_nonce);

        // The difference should be at least 1 millisecond in microseconds (1000)
        assert!(second_nonce - first_nonce >= 1000);
    }

    #[test]
    fn test_nonce_rapid_generation() {
        let manager = NonceManager::new();
        let mut nonces = Vec::new();

        // Generate many nonces rapidly in the same millisecond
        for _ in 0..100 {
            match manager.generate() {
                Ok(nonce) => nonces.push(nonce),
                Err(e) => {
                    // If we hit the limit, that's expected behavior
                    if let LighterError::Nonce(msg) = e {
                        assert!(msg.contains("Too many nonces"));
                        break;
                    } else {
                        panic!("Unexpected error: {}", e);
                    }
                }
            }
        }

        // Verify all generated nonces are unique
        let unique_nonces: HashSet<_> = nonces.iter().collect();
        assert_eq!(unique_nonces.len(), nonces.len());
    }

    #[test]
    fn test_nonce_default_impl() {
        let manager1 = NonceManager::new();
        let manager2 = NonceManager::default();

        // Both should generate valid nonces
        assert!(manager1.generate().is_ok());
        assert!(manager2.generate().is_ok());
    }
}
