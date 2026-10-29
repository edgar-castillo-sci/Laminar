use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use laminar_core::children::{all_rows, children, fiber};
use laminar_core::types::{JamChain, JamOption, Token};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn gen_chains(k: usize, option: JamOption, n: usize) -> Vec<JamChain> {
    let rows = all_rows(option);
    let mut rng = StdRng::seed_from_u64(0xBEEF_u64.wrapping_add(k as u64));
    (0..n)
        .map(|_| {
            let chain_rows: Vec<[Token; 3]> = (0..k)
                .map(|_| rows[rng.gen_range(0..rows.len())].clone())
                .collect();
            JamChain::from_rows(chain_rows, option)
        })
        .collect()
}

fn bench_children(c: &mut Criterion) {
        let mut group = c.benchmark_group("children");
        group.sample_size(10);
        for option in [JamOption::PL, JamOption::BU, JamOption::OneH] {
        for k in [4usize, 6] {
            let chains = gen_chains(k, option, 20);
            group.bench_with_input(
                BenchmarkId::new("children", format!("{:?}/k{}", option, k)),
                &chains,
                |b, chains| {
                    b.iter(|| {
                        for chain in chains {
                            black_box(children(chain));
                        }
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new("fiber", format!("{:?}/k{}", option, k)),
                &chains,
                |b, chains| {
                    b.iter(|| {
                        for chain in chains {
                            black_box(fiber(chain));
                        }
                    })
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_children);
criterion_main!(benches);
