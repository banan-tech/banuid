use banuid::IdGenerator;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_id_generation(c: &mut Criterion) {
    let generator = IdGenerator::with_shard_id(42);

    c.bench_function("id_generation", |b| {
        b.iter(|| {
            black_box(generator.next_id());
        });
    });
}

fn bench_id_creation(c: &mut Criterion) {
    c.bench_function("id_creation", |b| {
        b.iter(|| {
            black_box(IdGenerator::new());
        });
    });
}

fn bench_extract_operations(c: &mut Criterion) {
    let generator = IdGenerator::with_shard_id(123);
    let id = generator.next_id();

    c.bench_function("extract_timestamp", |b| {
        b.iter(|| {
            black_box(IdGenerator::extract_timestamp(black_box(id)));
        });
    });

    c.bench_function("extract_shard_id", |b| {
        b.iter(|| {
            black_box(IdGenerator::extract_shard_id(black_box(id)));
        });
    });

    c.bench_function("extract_sequence", |b| {
        b.iter(|| {
            black_box(IdGenerator::extract_sequence(black_box(id)));
        });
    });
}

fn bench_concurrent_generation(c: &mut Criterion) {
    let generator = std::sync::Arc::new(IdGenerator::with_shard_id(1));

    c.bench_function("concurrent_generation", |b| {
        b.iter(|| {
            let gen = std::sync::Arc::clone(&generator);
            let handle =
                std::thread::spawn(move || (0..1000).map(|_| gen.next_id()).collect::<Vec<u64>>());
            black_box(handle.join().unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_id_generation,
    bench_id_creation,
    bench_extract_operations,
    bench_concurrent_generation
);
criterion_main!(benches);
