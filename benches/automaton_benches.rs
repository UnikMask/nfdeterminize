use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use nfdeterminize::transition_graphs::get_buffer_and_stack_aut;

fn determinization_bns_benchmark(c: &mut Criterion) {
    for i in 1..4 {
        for j in 1..8 {
            let mut automaton = get_buffer_and_stack_aut(i, j);
            c.bench_function(&format!("determinize bns {i} {j}"), |b| {
                b.iter(|| automaton = automaton.determinized())
            });
            c.bench_function(&format!("minimize bns {i} {j}"), |b| {
                b.iter(|| automaton = automaton.minimized())
            });
        }
    }
}
criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.05).sample_size(25).measurement_time(Duration::new(5, 0));
    targets = determinization_bns_benchmark
}
criterion_main!(benches);
