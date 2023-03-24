#[cfg(test)]
mod tests {
    use crate::automaton::{AlgorithmKind, Automaton, AutomatonType};
    use serial_test::serial;

    impl Automaton {
        pub fn order_transitions(mut self) -> Self {
            self.table.sort();
            self.start.sort();
            self.end.sort();
            self
        }
    }

    const NUM_THREADS: usize = 4;
    const KINDS: [AlgorithmKind; 2] = [
        AlgorithmKind::Sequential,
        AlgorithmKind::Multithreaded(NUM_THREADS),
    ];

    #[test]
    #[serial]
    // Test the behaviour of determinization over an NFA that is already deterministic.
    fn test_determinization_redundant() {
        let redundant_nd = Automaton::new(
            AutomatonType::NonDet,
            1,
            2,
            vec![(0, 1, 0), (0, 2, 0)],
            vec![0],
            vec![0],
        );
        let redundant_d = Automaton::new(
            AutomatonType::Det,
            1,
            2,
            vec![(0, 1, 0), (0, 2, 0)],
            vec![0],
            vec![0],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(
                redundant_nd.determinized(*k).order_transitions(),
                redundant_d
            );
        });
    }

    #[test]
    #[serial]
    // Test the behaviour of determinization over a single state, no transition NFA.
    fn test_determinization_empty_lang() {
        let empty_lang_nd = Automaton::new(AutomatonType::NonDet, 1, 2, vec![], vec![0], vec![0]);
        let empty_lang_d = Automaton::new(
            AutomatonType::Det,
            2,
            2,
            vec![(0, 1, 1), (0, 2, 1), (1, 1, 1), (1, 2, 1)],
            vec![0],
            vec![0],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(
                empty_lang_nd.determinized(*k).order_transitions(),
                empty_lang_d
            );
        });
    }

    #[test]
    #[serial]
    // Test whether determinization gets rid of unreachable states.
    fn test_determinization_unreachable() {
        let unreachable_nd = Automaton::new(
            AutomatonType::NonDet,
            2,
            2,
            vec![(0, 1, 0), (0, 2, 0)],
            vec![0],
            vec![0],
        );
        let unreachable_d = Automaton::new(
            AutomatonType::Det,
            1,
            2,
            vec![(0, 1, 0), (0, 2, 0)],
            vec![0],
            vec![0],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(
                unreachable_nd.determinized(*k).order_transitions(),
                unreachable_d
            );
        });
    }

    #[test]
    #[serial]
    // Test whether determinization can successfully produce a sinkhole state from an empty set of states.
    fn test_determinization_sinkhole() {
        let sinkhole_nd = Automaton::new(
            AutomatonType::NonDet,
            3,
            2,
            vec![(0, 1, 1), (1, 1, 2)],
            vec![0],
            vec![2],
        );
        let sinkhole_d = Automaton::new(
            AutomatonType::Det,
            4,
            2,
            vec![
                (0, 1, 1),
                (0, 2, 2),
                (1, 1, 3),
                (1, 2, 2),
                (2, 1, 2),
                (2, 2, 2),
                (3, 1, 2),
                (3, 2, 2),
            ],
            vec![0],
            vec![3],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(sinkhole_nd.determinized(*k).order_transitions(), sinkhole_d);
        });
    }

    #[test]
    #[serial]
    // Test whether duplicate transitions in a non deterministic automata are lost after
    // determinization.
    fn test_determinization_duplicate_transitions() {
        let duplicate_transitions_nd = Automaton::new(
            AutomatonType::NonDet,
            2,
            2,
            vec![(0, 1, 1), (0, 1, 1), (0, 2, 1), (1, 1, 1), (1, 2, 1)],
            vec![0],
            vec![1],
        );
        let duplicate_transitions_d = Automaton::new(
            AutomatonType::Det,
            2,
            2,
            vec![(0, 1, 1), (0, 2, 1), (1, 1, 1), (1, 2, 1)],
            vec![0],
            vec![1],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(
                duplicate_transitions_nd
                    .determinized(*k)
                    .order_transitions(),
                duplicate_transitions_d
            );
        });
    }

    #[test]
    #[serial]
    // Test whether sets of states are detected and dealt as a single state in determinization.
    fn test_determinization_set_of_states() {
        let set_of_states_nd = Automaton::new(
            AutomatonType::NonDet,
            2,
            1,
            vec![(0, 1, 0), (0, 1, 1)],
            vec![0],
            vec![1],
        );
        let set_of_states_d = Automaton::new(
            AutomatonType::Det,
            2,
            1,
            vec![(0, 1, 1), (1, 1, 1)],
            vec![0],
            vec![1],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(
                set_of_states_nd.determinized(*k).order_transitions(),
                set_of_states_d
            );
        });
    }

    #[test]
    #[serial]
    // Test whether determinization identifies and deals with empty char transitions.
    fn test_determinization_empty_char() {
        let empty_char_nd = Automaton::new(
            AutomatonType::NonDet,
            4,
            2,
            vec![
                (0, 0, 1),
                (0, 1, 2),
                (1, 1, 3),
                (2, 2, 3),
                (3, 0, 3),
                (3, 1, 3),
                (3, 2, 3),
            ],
            vec![0],
            vec![3],
        );
        let empty_char_d = Automaton::new(
            AutomatonType::Det,
            4,
            2,
            vec![
                (0, 1, 1),
                (0, 2, 2),
                (1, 1, 3),
                (1, 2, 3),
                (2, 1, 2),
                (2, 2, 2),
                (3, 1, 3),
                (3, 2, 3),
            ],
            vec![0],
            vec![1, 3],
        );
        KINDS.iter().for_each(|k| {
            assert_eq!(
                empty_char_nd.determinized(*k).order_transitions(),
                empty_char_d
            );
        });
    }

    #[test]
    #[serial]
    // Test whether a machine minimizable into 2 partitions will be minimized as such.
    fn test_minimization_bipartite() {
        let bipartite_big = Automaton::new(
            AutomatonType::Det,
            3,
            2,
            vec![
                (0, 1, 1),
                (0, 2, 2),
                (1, 1, 1),
                (1, 2, 1),
                (2, 1, 2),
                (2, 2, 2),
            ],
            vec![0],
            vec![1, 2],
        );
        let bipartite_small = Automaton::new(
            AutomatonType::Det,
            2,
            2,
            vec![(0, 1, 1), (0, 2, 1), (1, 1, 1), (1, 2, 1)],
            vec![0],
            vec![1],
        );
        assert_eq!(
            bipartite_big.minimized().order_transitions(),
            bipartite_small
        );
    }

    #[test]
    #[serial]
    // Test whether minimization can separate sets partitions into smaller partitions.
    fn test_minimization_separation() {
        let sep_big = Automaton::new(
            AutomatonType::Det,
            6,
            2,
            vec![
                (0, 1, 3),
                (0, 2, 1),
                (1, 1, 2),
                (1, 2, 5),
                (2, 1, 2),
                (2, 2, 5),
                (3, 1, 0),
                (3, 2, 4),
                (4, 1, 2),
                (4, 2, 5),
                (5, 1, 5),
                (5, 2, 5),
            ],
            vec![0],
            vec![1, 2, 4],
        );

        let sep_small = Automaton::new(
            AutomatonType::Det,
            3,
            2,
            vec![
                (0, 1, 0),
                (0, 2, 2),
                (1, 1, 1),
                (1, 2, 1),
                (2, 1, 2),
                (2, 2, 1),
            ],
            vec![0],
            vec![2],
        );

        assert_eq!(sep_big.minimized().order_transitions(), sep_small);
    }

    #[test]
    #[serial]
    // Test whether unminimizable machines cannot be minimized (the size doesn't decrease).
    fn test_minimization_unminimizable() {
        let unmin_small = Automaton::new(
            AutomatonType::Det,
            4,
            2,
            vec![
                (0, 1, 1),
                (0, 2, 2),
                (1, 1, 2),
                (1, 2, 3),
                (2, 1, 2),
                (2, 2, 2),
                (3, 1, 1),
                (3, 2, 3),
            ],
            vec![0],
            vec![3],
        )
        .minimized()
        .order_transitions();
        assert_eq!(unmin_small.size, 4);
    }
}
