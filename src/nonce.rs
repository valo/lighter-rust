use crate::error::{LighterError, Result};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct NonceManager {
    last_nonce: AtomicU64,
}

impl NonceManager {
    pub fn new() -> Self {
        Self {
            last_nonce: AtomicU64::new(0),
        }
    }

    pub fn with_seed(next_nonce: u64) -> Self {
        let manager = Self::new();
        manager.synchronise(next_nonce);
        manager
    }

    pub fn generate(&self) -> Result<u64> {
        loop {
            let previous = self.last_nonce.load(Ordering::Acquire);
            let candidate = previous
                .checked_add(1)
                .ok_or_else(|| LighterError::Nonce("nonce overflow".to_string()))?;
            if self
                .last_nonce
                .compare_exchange(previous, candidate, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(candidate);
            }
        }
    }

    pub fn synchronise(&self, next_nonce: u64) {
        if next_nonce == 0 {
            return;
        }
        self.last_nonce
            .store(next_nonce.saturating_sub(1), Ordering::Release);
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
                nonce == previous + 1,
                "Nonce {} is not exactly previous + 1 {}",
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
    fn test_nonce_respects_time_progression() {
        let manager = NonceManager::new();
        let first = manager.generate().unwrap();
        let second = manager.generate().unwrap();
        assert_eq!(second, first + 1);
    }

    #[test]
    fn test_nonce_default_impl() {
        let manager1 = NonceManager::new();
        let manager2 = NonceManager::default();

        // Both should generate valid nonces
        assert!(manager1.generate().is_ok());
        assert!(manager2.generate().is_ok());
    }

    #[test]
    fn test_nonce_synchronise_uses_seed() {
        let manager = NonceManager::with_seed(1_234_567_890_000);
        let first = manager.generate().expect("nonce");
        assert_eq!(first, 1_234_567_890_000);
        let second = manager.generate().expect("nonce");
        assert_eq!(second, first + 1);
    }
}
