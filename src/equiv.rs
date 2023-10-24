//! Bounded equivalence checking on Aigs

use cat_solver::Solver;

use crate::{aig::Aig, gates::Gate, signal::Signal};

/**
 * Export a combinatorial Aig to a CNF formula
 */
fn to_clauses(aig: &Aig) -> Vec<Vec<Signal>> {
    use Gate::*;
    assert!(aig.is_comb());
    let mut ret = Vec::<Vec<Signal>>::new();
    for i in 0..aig.nb_nodes() {
        let n = aig.node(i);
        match aig.gate(i) {
            And(a, b) => {
                // 3 clauses, 6 literals
                ret.push(vec![a, !n]);
                ret.push(vec![b, !n]);
                ret.push(vec![!a, !b, n]);
            }
            Xor(a, b) => {
                // 4 clauses, 12 literals
                ret.push(vec![a, b, !n]);
                ret.push(vec![!a, !b, !n]);
                ret.push(vec![!a, b, n]);
                ret.push(vec![a, !b, n]);
            }
            And3(a, b, c) => {
                // 4 clauses, 10 literals
                ret.push(vec![a, !n]);
                ret.push(vec![b, !n]);
                ret.push(vec![c, !n]);
                ret.push(vec![!a, !b, !c, n]);
            }
            Xor3(a, b, c) => {
                // 8 clauses, 32 literals
                ret.push(vec![a, b, c, !n]);
                ret.push(vec![a, b, !c, n]);
                ret.push(vec![a, !b, c, n]);
                ret.push(vec![a, !b, !c, !n]);
                ret.push(vec![!a, b, c, n]);
                ret.push(vec![!a, b, !c, !n]);
                ret.push(vec![!a, !b, c, !n]);
                ret.push(vec![!a, !b, !c, n]);
            }
            Mux(s, a, b) => {
                // 4 clauses, 12 literals + 2 redundant clauses
                ret.push(vec![!s, !a, n]);
                ret.push(vec![!s, a, !n]);
                ret.push(vec![s, !b, n]);
                ret.push(vec![s, b, !n]);
                // Redundant but useful
                ret.push(vec![a, b, !n]);
                ret.push(vec![!a, !b, n]);
            }
            Maj(a, b, c) => {
                // 6 clauses, 18 literals
                ret.push(vec![!a, !b, n]);
                ret.push(vec![!b, !c, n]);
                ret.push(vec![!a, !b, n]);
                ret.push(vec![a, b, !n]);
                ret.push(vec![b, c, !n]);
                ret.push(vec![a, b, !n]);
            }
            Dff(_, _, _) => panic!("Combinatorial Aig expected"),
        }
    }
    ret
}

/**
 * Unroll a sequential Aig over a fixed number of steps
 */
fn unroll(aig: &Aig, nb_steps: usize) -> Aig {
    // TODO
    aig.clone()
}

/**
 * Perform equivalence checking on two combinatorial AIGs
 */
pub fn check_equivalence_comb(a: &Aig, b: &Aig) -> Result<(), Vec<bool>> {
    assert!(a.is_comb() && b.is_comb());
    let res = check_equivalence_bounded(a, b, 1);
    match res {
        Ok(()) => Ok(()),
        Err(v) => Err(v[0].clone()),
    }
}
/**
 * Perform bounded equivalence checking on two sequential AIGs
 */
pub fn check_equivalence_bounded(a: &Aig, b: &Aig, nb_steps: usize) -> Result<(), Vec<Vec<bool>>> {
    assert_eq!(a.nb_inputs(), b.nb_inputs());
    assert_eq!(a.nb_outputs(), b.nb_outputs());
    let mut solver = Solver::new();

    // TODO
    let res = solver.solve();
    match res {
        Some(sat) => {
            if sat {
                let mut assignment = Vec::<Vec<bool>>::new();
                /*
                for timestep_input_lits in t1.input_lits {
                    let mut timestep_assignment = Vec::<bool>::new();
                    for lit in timestep_input_lits {
                        let val = solver.value(lit).unwrap_or(false);
                        timestep_assignment.push(val);
                    }
                    assignment.push(timestep_assignment);
                }
                */
                Err(assignment)
            } else {
                Ok(())
            }
        }
        None => panic!("SAT solver didn't succeed"),
    }
}
