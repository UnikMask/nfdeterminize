use std::{fs, ops::Range, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use nfdeterminize::automaton::{AlgorithmKind, Automaton};
use nfdeterminize::transition_graphs::{get_buffer_and_stack_aut, get_two_stack_aut};

const N_THREADS: usize = 12;

const AUT_KINDS: [AlgorithmKind; 2] = [
    AlgorithmKind::Sequential,
    AlgorithmKind::Multithreaded(N_THREADS),
];

const NUM_BNS_BUFFERS: Range<usize> = 2..4;
const NUM_BNS_STACKS: Range<usize> = 2..8;
const BNS_MT_INCREASE: (usize, usize) = (3, 7);
const BNS_MT_INCREASE_LO: (usize, usize) = (3, 4);
const NUM_TWO_STACK_STACK0: Range<usize> = 2..4;
const NUM_TWO_STACK_STACK1: Range<usize> = 2..6;
const NUM_GAP_BUFFERS: Range<usize> = 2..4;
const NUM_GAP_STACKS: Range<usize> = 2..6;
const AUTOMATONS_PATH: &str = "automatons/";

// Comparative benches
fn run_bns_benchmark(c: &mut Criterion) {
    for k in AUT_KINDS {
        for i in NUM_BNS_BUFFERS {
            for j in NUM_BNS_STACKS {
                let mut automaton = get_buffer_and_stack_aut(i, j);
                c.bench_function(&format!("determinize bns {i} {j} {k:?}"), |b| {
                    b.iter(|| automaton.determinized(k))
                });
                automaton = automaton.determinized(k);
                c.bench_function(&format!("minimize bns {i} {j} {k:?}"), |b| {
                    b.iter(|| automaton.minimized())
                });
            }
        }
    }
}
fn run_two_stack_benchmark(c: &mut Criterion) {
    for k in AUT_KINDS {
        for i in NUM_TWO_STACK_STACK0 {
            for j in NUM_TWO_STACK_STACK1 {
                let mut automaton = get_two_stack_aut(i, j);
                c.bench_function(&format!("determinize two-stack {i} {j} {k:?}"), |b| {
                    b.iter(|| automaton.determinized(k))
                });
                automaton = automaton.determinized(k);
                c.bench_function(&format!("minimize two-stack {i} {j} {k:?}"), |b| {
                    b.iter(|| automaton.minimized())
                });
            }
        }
    }
}
fn run_gap_benchmarks(c: &mut Criterion) {
    for k in AUT_KINDS {
        for i in NUM_GAP_BUFFERS {
            for j in NUM_GAP_STACKS {
                let mut automaton = Automaton::from(
                    &fs::read_to_string(format!("{}bns-{i}-{j}.nfa", AUTOMATONS_PATH).to_string())
                        .unwrap()
                        .to_string(),
                );
                c.bench_function(&format!("determinize file bns {i} {j} {k:?}"), |b| {
                    b.iter(|| automaton.determinized(k))
                });
                automaton = automaton.determinized(k);
                c.bench_function(&format!("minimize file bns {i} {j} {k:?}"), |b| {
                    b.iter(|| automaton.minimized())
                });
            }
        }
    }
}
fn run_mt_increase(c: &mut Criterion) {
    for k in 2..N_THREADS {
        let automaton = get_buffer_and_stack_aut(BNS_MT_INCREASE.0, BNS_MT_INCREASE.1);
        c.bench_with_input(
            BenchmarkId::new(&format!("determinize bns 3 7 mult_incr"), k),
            &k,
            |b, &s| {
                b.iter(|| automaton.determinized(AlgorithmKind::Multithreaded(s.clone())));
            },
        );
        let automaton = get_buffer_and_stack_aut(BNS_MT_INCREASE_LO.0, BNS_MT_INCREASE_LO.1);
        c.bench_with_input(
            BenchmarkId::new(&format!("determinize bns 3 4 mult_incr"), k),
            &k,
            |b, &s| {
                b.iter(|| automaton.determinized(AlgorithmKind::Multithreaded(s.clone())));
            },
        );
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.05).sample_size(25).measurement_time(Duration::new(5, 0));
    targets = run_bns_benchmark, run_two_stack_benchmark, run_gap_benchmarks, run_mt_increase
}
criterion_main!(benches);
