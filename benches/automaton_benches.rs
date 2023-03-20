use std::{ops::Range, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion};
use nfdeterminize::transition_graphs::get_buffer_and_stack_aut;

const NUM_BUFFERS: Range<usize> = 1..4;
const NUM_STACKS: Range<usize> = 1..8;

fn determinization_bns_benchmark(c: &mut Criterion) {
    for i in NUM_BUFFERS {
        for j in NUM_STACKS {
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
