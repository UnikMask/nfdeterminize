use std::{fs, ops::Range, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion};
use nfdeterminize::automaton::Automaton;
use nfdeterminize::transition_graphs::{get_buffer_and_stack_aut, get_two_stack_aut};

const NUM_BNS_BUFFERS: Range<usize> = 2..4;
const NUM_BNS_STACKS: Range<usize> = 2..8;
const NUM_TWO_STACK_STACK0: Range<usize> = 2..4;
const NUM_TWO_STACK_STACK1: Range<usize> = 2..6;
const NUM_GAP_BUFFERS: Range<usize> = 2..4;
const NUM_GAP_STACKS: Range<usize> = 2..6;
const AUTOMATONS_PATH: &str = "automatons/";

fn run_bns_benchmark(c: &mut Criterion) {
    for i in NUM_BNS_BUFFERS {
        for j in NUM_BNS_STACKS {
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
fn run_two_stack_benchmark(c: &mut Criterion) {
    for i in NUM_TWO_STACK_STACK0 {
        for j in NUM_TWO_STACK_STACK1 {
            let mut automaton = get_two_stack_aut(i, j);
            c.bench_function(&format!("determinize two-stack {i} {j}"), |b| {
                b.iter(|| automaton = automaton.determinized())
            });
            c.bench_function(&format!("minimize two-stack {i} {j}"), |b| {
                b.iter(|| automaton = automaton.minimized())
            });
        }
    }
}

fn run_gap_benchmarks(c: &mut Criterion) {
    for i in NUM_GAP_BUFFERS {
        for j in NUM_GAP_STACKS {
            let mut automaton = Automaton::from(
                &fs::read_to_string(format!("{}bns-{i}-{j}.nfa", AUTOMATONS_PATH).to_string())
                    .unwrap()
                    .to_string(),
            );
            c.bench_function(&format!("determinize file bns {i} {j}"), |b| {
                b.iter(|| automaton = automaton.determinized())
            });
            c.bench_function(&format!("minimize file bns {i} {j}"), |b| {
                b.iter(|| automaton = automaton.minimized())
            });
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.05).sample_size(25).measurement_time(Duration::new(5, 0));
    targets = run_bns_benchmark, run_two_stack_benchmark, run_gap_benchmarks
}
criterion_main!(benches);
