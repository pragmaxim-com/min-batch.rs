use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::stream::{self, Stream};
use min_batch::ext::MinBatchExt;
use tokio::runtime::Runtime;

async fn batch(stream: impl Stream<Item = i32>) {
    let _ = stream.min_batch(1000, |i| *i as usize);
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("min_batch");
    for &size in &[10, 100, 1000, 10_000, 100_000] {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |bencher, &size| {
                bencher.to_async(&rt).iter(|| batch(stream::iter(0..size)));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
