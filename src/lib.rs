use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const CUSTOM_EPOCH: u64 = 1704067200000; // 2024-01-01 00:00:00 UTC
const SHARD_ID_BITS: u8 = 13;
const SEQUENCE_BITS: u8 = 10;

const MAX_SHARD_ID: u64 = (1 << SHARD_ID_BITS) - 1;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;

const SHARD_ID_SHIFT: u8 = SEQUENCE_BITS;
const TIMESTAMP_SHIFT: u8 = SHARD_ID_BITS + SEQUENCE_BITS;

struct GeneratorState {
    last_timestamp: u64,
    sequence: u64,
}

pub struct IdGenerator {
    shard_id: u16,
    state: Mutex<GeneratorState>,
}

impl IdGenerator {
    pub fn new() -> Self {
        let shard_id = derive_shard_id();
        IdGenerator {
            shard_id,
            state: Mutex::new(GeneratorState {
                last_timestamp: 0,
                sequence: 0,
            }),
        }
    }

    pub fn with_shard_id(shard_id: u16) -> Self {
        let shard_id = shard_id & (MAX_SHARD_ID as u16);
        IdGenerator {
            shard_id,
            state: Mutex::new(GeneratorState {
                last_timestamp: 0,
                sequence: 0,
            }),
        }
    }

    pub fn next_id(&self) -> u64 {
        loop {
            let mut state = self.state.lock().unwrap();
            let timestamp = current_timestamp();

            if timestamp == state.last_timestamp {
                if state.sequence >= MAX_SEQUENCE {
                    drop(state);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }
                state.sequence += 1;
                let sequence = state.sequence;
                return ((timestamp - CUSTOM_EPOCH) << TIMESTAMP_SHIFT)
                    | ((self.shard_id as u64) << SHARD_ID_SHIFT)
                    | sequence;
            } else {
                state.last_timestamp = timestamp;
                state.sequence = 0;
                return ((timestamp - CUSTOM_EPOCH) << TIMESTAMP_SHIFT)
                    | ((self.shard_id as u64) << SHARD_ID_SHIFT)
                    | 0;
            }
        }
    }

    pub fn extract_timestamp(id: u64) -> u64 {
        ((id >> TIMESTAMP_SHIFT) as u64) + CUSTOM_EPOCH
    }

    pub fn extract_shard_id(id: u64) -> u16 {
        ((id >> SHARD_ID_SHIFT) & MAX_SHARD_ID) as u16
    }

    pub fn extract_sequence(id: u64) -> u16 {
        (id & MAX_SEQUENCE) as u16
    }

    pub fn shard_id(&self) -> u16 {
        self.shard_id
    }
}

fn derive_shard_id() -> u16 {
    let mut hash: u64 = 14695981039346656037; // FNV offset basis
    const FNV_PRIME: u64 = 1099511628211;

    if let Ok(hostname) = std::env::var("HOSTNAME") {
        for byte in hostname.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
        for byte in machine_id.trim().bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    let pid = std::process::id();
    for byte in pid.to_string().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    (hash % (MAX_SHARD_ID + 1)) as u16
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let generator = IdGenerator::with_shard_id(42);
        let id1 = generator.next_id();
        let id2 = generator.next_id();

        assert_ne!(id1, id2, "IDs should be unique");
        assert!(id1 < id2 || id1 > id2, "IDs should be orderable");
    }

    #[test]
    fn test_extract_timestamp() {
        let generator = IdGenerator::with_shard_id(1);
        let id = generator.next_id();
        let extracted = IdGenerator::extract_timestamp(id);
        let now = current_timestamp();

        assert!(
            extracted <= now && extracted >= now - 1000,
            "Extracted timestamp should be recent"
        );
    }

    #[test]
    fn test_extract_shard_id() {
        let shard_id: u16 = 42;
        let generator = IdGenerator::with_shard_id(shard_id);
        let id = generator.next_id();
        let extracted = IdGenerator::extract_shard_id(id);

        assert_eq!(extracted, shard_id, "Shard ID should match");
    }

    #[test]
    fn test_shard_id_bounds() {
        let generator = IdGenerator::with_shard_id(8191); // Max 13-bit value
        assert_eq!(generator.shard_id(), 8191);

        let generator2 = IdGenerator::with_shard_id(10000); // Overflow
        assert_eq!(generator2.shard_id(), 10000 & (MAX_SHARD_ID as u16));
    }

    #[test]
    fn test_auto_derived_shard() {
        let generator = IdGenerator::new();
        let id = generator.next_id();
        let extracted_shard = IdGenerator::extract_shard_id(id);

        assert_eq!(extracted_shard, generator.shard_id());
        assert!(extracted_shard <= 8191);
    }

    #[test]
    fn test_concurrent_generation() {
        use std::sync::Arc;
        use std::thread;

        let generator = Arc::new(IdGenerator::with_shard_id(1));
        let mut handles = vec![];
        let mut ids = std::collections::HashSet::new();

        for _ in 0..10 {
            let gen = Arc::clone(&generator);
            handles.push(thread::spawn(move || {
                (0..100).map(|_| gen.next_id()).collect::<Vec<_>>()
            }));
        }

        for handle in handles {
            let thread_ids = handle.join().unwrap();
            for id in thread_ids {
                assert!(!ids.contains(&id), "Duplicate ID found: {}", id);
                ids.insert(id);
            }
        }

        assert_eq!(ids.len(), 1000, "Should have 1000 unique IDs");
    }
}
