use banuid;

fn main() {
    // Ergonomic API - no need to create instances
    let id1 = banuid::generate();
    let id2 = banuid::generate();
    
    println!("Generated IDs: {}, {}", id1, id2);
    
    // Parse ID components
    let timestamp = banuid::parse_timestamp(id1);
    let shard_id = banuid::parse_shard_id(id1);
    let sequence = banuid::parse_sequence(id1);
    
    println!("ID {} -> timestamp: {}, shard: {}, sequence: {}", id1, timestamp, shard_id, sequence);
    
    // Traditional API still works
    let generator = banuid::IdGenerator::with_shard_id(42);
    let id3 = generator.next_id(); // or generator.generate()
    println!("Traditional API: {}", id3);
    
    // New instance methods
    let id4 = generator.generate();
    println!("New instance method: {}", id4);
}
