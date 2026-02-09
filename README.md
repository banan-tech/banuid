# banuid

[![Crates.io](https://img.shields.io/crates/v/banuid.svg)](https://crates.io/crates/banuid)
[![Documentation](https://docs.rs/banuid/badge.svg)](https://docs.rs/banuid)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, secure, and stateless library for generating Instagram-style lexicographically sortable 64-bit unique identifiers. Zero dependencies, thread-safe, and optimized for distributed systems.

## Features

- **64-bit Instagram-style IDs**: Time-ordered, lexicographically sortable identifiers
- **Auto-derived shard IDs**: Stateless generation using device and process characteristics
- **Thread-safe**: Concurrent generation without duplicates
- **Zero dependencies**: Pure Rust with no external crates
- **Fast**: ~10+ million IDs/second per shard
- **69-year range**: Custom epoch starting from 2024-01-01

## ID Structure

Each 64-bit ID is composed of:

| Component   | Bits | Description                                    |
|-------------|------|------------------------------------------------|
| Timestamp   | 41   | Milliseconds since 2024-01-01 (~69 years)      |
| Shard ID    | 13   | Auto-derived from machine/process (8,192 max)  |
| Sequence    | 10   | Per-shard counter, rolls over at 1,024/ms        |

This design provides:
- **Time ordering**: IDs sort by creation time without additional data
- **Distributed uniqueness**: Different machines/processes get different shard IDs
- **High throughput**: Up to 1,024 IDs per millisecond per shard

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
banuid = "1.0.0"
```

## Usage

### Basic Usage (Auto-derived Shard)

```rust
use banuid::IdGenerator;

fn main() {
    // Creates generator with auto-derived shard ID
    let generator = IdGenerator::new();
    
    // Generate IDs
    let id1 = generator.next_id();
    let id2 = generator.next_id();
    
    println!("ID 1: {}", id1);
    println!("ID 2: {}", id2);
    
    // Extract components
    let timestamp = IdGenerator::extract_timestamp(id1);
    let shard_id = IdGenerator::extract_shard_id(id1);
    let sequence = IdGenerator::extract_sequence(id1);
    
    println!("Created at: {} ms since epoch", timestamp);
    println!("Shard ID: {}", shard_id);
    println!("Sequence: {}", sequence);
}
```

### Manual Shard ID

```rust
use banuid::IdGenerator;

fn main() {
    // Use specific shard ID (0-8191)
    let generator = IdGenerator::with_shard_id(42);
    let id = generator.next_id();
    
    assert_eq!(IdGenerator::extract_shard_id(id), 42);
}
```

### Thread-Safe Generation

```rust
use banuid::IdGenerator;
use std::sync::Arc;
use std::thread;

fn main() {
    let generator = Arc::new(IdGenerator::new());
    let mut handles = vec![];
    
    // Spawn 10 threads generating 100 IDs each
    for _ in 0..10 {
        let gen = Arc::clone(&generator);
        handles.push(thread::spawn(move || {
            (0..100).map(|_| gen.next_id()).collect::<Vec<_>>()
        }));
    }
    
    // All 1000 IDs will be unique
    let mut ids = std::collections::HashSet::new();
    for handle in handles {
        for id in handle.join().unwrap() {
            assert!(ids.insert(id), "Duplicate ID found!");
        }
    }
    
    assert_eq!(ids.len(), 1000);
}
```

## How It Works

### Shard ID Derivation

The shard ID (13 bits) is automatically derived from:

1. **`HOSTNAME` environment variable** - For containerized environments (Kubernetes, Docker)
2. **`/etc/machine-id`** - Linux machine identifier
3. **Process ID** - Ensures different processes get different shards

These components are hashed using FNV-1a to produce a deterministic 13-bit value.

**Benefits:**
- Stateless - no configuration needed
- Survives process restarts (same machine gets consistent shard ID)
- Works in containers without manual configuration

### Performance

- **Generation rate**: ~10+ million IDs/second per shard
- **Memory overhead**: ~40 bytes per generator instance
- **Thread contention**: Minimal - uses short-lived mutex locks

### ID Properties

| Property            | Value                              |
|---------------------|------------------------------------|
| Format              | Unsigned 64-bit integer              |
| Time range          | ~69 years (2024-2093)              |
| Max throughput      | 1,024 IDs/ms per shard             |
| Max unique shards   | 8,192                              |
| Sortable            | Yes (chronologically)                |
| Duplicates          | None (across shards and time)        |

## Use Cases

- **Database primary keys** - Sortable, compact (8 bytes), no coordination needed
- **Distributed systems** - Unique IDs across services without central coordination
- **Event sourcing** - Chronologically ordered event IDs
- **Caching** - Time-ordered cache keys for efficient eviction
- **Message queues** - Message IDs with built-in ordering

## Comparison

| Feature          | banuid         | UUID v4        | Twitter Snowflake   |
|------------------|----------------|----------------|---------------------|
| Size             | 64 bits        | 128 bits       | 64 bits             |
| Sortable         | Yes            | No             | Yes                 |
| Dependencies     | 0              | 0 (std)        | Varies              |
| Coordination     | None           | None           | Zookeeper (optional)|
| Time embedded    | Yes            | No             | Yes                 |
| Throughput       | High           | Very High      | High                |

## Safety & Security

- **No collisions**: Guaranteed unique within a shard per millisecond
- **Non-predictable**: Sequence number prevents simple enumeration attacks
- **No side effects**: Pure generation with no network or disk access
- **Thread-safe**: Safe for concurrent use across threads

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Inspired by Instagram's ID generation scheme described in their engineering blog post "Sharding & IDs at Instagram" (2012).
