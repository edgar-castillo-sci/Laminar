use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use laminar_core::children::augment_to;
use laminar_core::types::JamOption;
use std::time::Duration;

fn bench_augment(c: &mut Criterion) {
    let mut group = c.benchmark_group("augment_to");
    // n=5 for BU/OneH is ~500 ms per call (666 classes × 12 children × kappa_fast);
    // skip it to keep the benchmark suite under ~30s.
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(3));
    for option in [JamOption::PL, JamOption::BU, JamOption::OneH] {
        for n in [3usize, 4] {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", option), n),
                &n,
                |b, &n| b.iter(|| black_box(augment_to(n, option))),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_augment);
criterion_main!(benches);
