use core::fmt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::network::gates::Gate;
use crate::network::gates::Normalization;
use crate::network::signal::Signal;

/// Representation of a logic network as a gate-inverter-graph, used as the main representation for all logic manipulations
#[derive(Debug, Clone, Default)]
pub struct Aig {
    nb_inputs: usize,
    nodes: Vec<Gate>,
    outputs: Vec<Signal>,
}

impl Aig {
    /// Create a new Aig
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the number of primary inputs of the Aig
    pub fn nb_inputs(&self) -> usize {
        self.nb_inputs
    }

    /// Return the number of primary outputs of the Aig
    pub fn nb_outputs(&self) -> usize {
        self.outputs.len()
    }

    /// Return the number of nodes in the Aig
    pub fn nb_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get the input at index i
    pub fn input(&self, i: usize) -> Signal {
        assert!(i < self.nb_inputs());
        Signal::from_input(i as u32)
    }

    /// Get the output at index i
    pub fn output(&self, i: usize) -> Signal {
        assert!(i < self.nb_outputs());
        self.outputs[i]
    }

    /// Get the variable at index i
    pub fn node(&self, i: usize) -> Signal {
        Signal::from_var(i as u32)
    }

    /// Get the gate at index i
    pub fn gate(&self, i: usize) -> &Gate {
        &self.nodes[i]
    }

    /// Number of And2 gates
    pub fn nb_and(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::And(_, _)))
            .count()
    }

    /// Number of Xor2 gates
    pub fn nb_xor(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::Xor(_, _)))
            .count()
    }

    /// Number of And3 gates
    pub fn nb_and3(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::And3(_, _, _)))
            .count()
    }

    /// Number of Xor3 gates
    pub fn nb_xor3(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::Xor3(_, _, _)))
            .count()
    }

    /// Number of Mux gates
    pub fn nb_mux(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::Mux(_, _, _)))
            .count()
    }

    /// Number of Maj gates
    pub fn nb_maj(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::Maj(_, _, _)))
            .count()
    }

    /// Number of Dff gates
    pub fn nb_dff(&self) -> usize {
        self.nodes
            .iter()
            .filter(|g| matches!(g, Gate::Dff(_, _, _)))
            .count()
    }

    /// Add a new primary input
    pub fn add_input(&mut self) -> Signal {
        self.nb_inputs += 1;
        self.input(self.nb_inputs() - 1)
    }

    /// Add multiple primary inputs
    pub fn add_inputs(&mut self, nb: usize) {
        self.nb_inputs += nb;
    }

    /// Add a new primary output based on an existing literal
    pub fn add_output(&mut self, l: Signal) {
        self.outputs.push(l)
    }

    /// Create an And2 gate
    pub fn and(&mut self, a: Signal, b: Signal) -> Signal {
        self.add_gate(Gate::And(a, b))
    }

    /// Create an Or2 gate
    pub fn or(&mut self, a: Signal, b: Signal) -> Signal {
        !self.and(!a, !b)
    }

    /// Create a Xor2 gate
    pub fn xor(&mut self, a: Signal, b: Signal) -> Signal {
        self.add_gate(Gate::Xor(a, b))
    }

    /// Create an And3 gate
    pub fn and3(&mut self, a: Signal, b: Signal, c: Signal) -> Signal {
        self.add_gate(Gate::And3(a, b, c))
    }

    /// Create an Or3 gate
    pub fn or3(&mut self, a: Signal, b: Signal, c: Signal) -> Signal {
        !self.and3(!a, !b, !c)
    }

    /// Create a Xor3 gate
    pub fn xor3(&mut self, a: Signal, b: Signal, c: Signal) -> Signal {
        self.add_gate(Gate::Xor3(a, b, c))
    }

    /// Create a Mux gate
    pub fn mux(&mut self, s: Signal, a: Signal, b: Signal) -> Signal {
        self.add_gate(Gate::Mux(s, a, b))
    }

    /// Create a Maj gate
    pub fn maj(&mut self, a: Signal, b: Signal, c: Signal) -> Signal {
        self.add_gate(Gate::Maj(a, b, c))
    }

    /// Create an n-ary And as a tree
    pub fn and_n(&mut self, sigs: &Vec<Signal>) -> Signal {
        if sigs.is_empty() {
            Signal::one()
        } else if sigs.len() == 1 {
            sigs[0]
        } else {
            let mut next_sigs = Vec::new();
            for i in (0..sigs.len()).step_by(2) {
                if i + 1 < sigs.len() {
                    next_sigs.push(self.and(sigs[i], sigs[i + 1]));
                } else {
                    next_sigs.push(sigs[i]);
                }
            }
            self.and_n(&next_sigs)
        }
    }

    /// Create an n-ary Or as a tree
    pub fn or_n(&mut self, sigs: &Vec<Signal>) -> Signal {
        let ands = sigs.iter().cloned().map(|s| !s).collect();
        !self.and_n(&ands)
    }

    /// Create an n-ary Xor as a tree
    pub fn xor_n(&mut self, sigs: &Vec<Signal>) -> Signal {
        if sigs.is_empty() {
            Signal::zero()
        } else if sigs.len() == 1 {
            sigs[0]
        } else {
            let mut next_sigs = Vec::new();
            for i in (0..sigs.len()).step_by(2) {
                if i + 1 < sigs.len() {
                    next_sigs.push(self.xor(sigs[i], sigs[i + 1]));
                } else {
                    next_sigs.push(sigs[i]);
                }
            }
            self.xor_n(&next_sigs)
        }
    }

    /// Create a Dff gate (flip flop)
    pub fn dff(&mut self, data: Signal, enable: Signal, reset: Signal) -> Signal {
        self.add_gate(Gate::Dff(data, enable, reset))
    }

    /// Add a new gate, normalized
    pub fn add_gate(&mut self, gate: Gate) -> Signal {
        use Normalization::*;
        let g = gate.make_canonical();
        match g {
            Buf(l) => l,
            Node(g, inv) => self.add_raw_gate(g) ^ inv,
        }
    }

    /// Add a new gate, without normalization
    pub(crate) fn add_raw_gate(&mut self, gate: Gate) -> Signal {
        let l = Signal::from_var(self.nodes.len() as u32);
        self.nodes.push(gate);
        l
    }

    /// Return whether the Aig is purely combinatorial
    pub fn is_comb(&self) -> bool {
        self.nb_dff() == 0
    }

    /// Return whether the Aig is already topologically sorted (except for flip-flops)
    pub(crate) fn is_topo_sorted(&self) -> bool {
        for (i, g) in self.nodes.iter().enumerate() {
            let ind = i as u32;
            for v in g.comb_vars() {
                if v >= ind {
                    return false;
                }
            }
        }
        true
    }

    /// Remap nodes; there may be holes in the translation
    fn remap(&mut self, order: &[u32]) -> Box<[Signal]> {
        // Create the translation
        let mut translation = vec![Signal::zero(); self.nb_nodes()];
        for (new_i, old_i) in order.iter().enumerate() {
            translation[*old_i as usize] = Signal::from_var(new_i as u32);
        }

        // Remap the nodes
        let mut new_nodes = Vec::new();
        for o in order {
            let i = *o as usize;
            let g = self.gate(i);
            assert!(translation[i].is_var());
            assert_eq!(translation[i].var(), new_nodes.len() as u32);
            new_nodes.push(g.remap(translation.as_slice()));
        }
        self.nodes = new_nodes;

        // Remap the outputs
        self.remap_outputs(&translation);
        translation.into()
    }

    /// Remap outputs
    fn remap_outputs(&mut self, translation: &[Signal]) {
        let new_outputs = self.outputs.iter().map(|s| s.remap(translation)).collect();
        self.outputs = new_outputs;
    }

    /// Remove unused logic; this will invalidate all signals
    ///
    /// Returns the mapping of old variable indices to signals, if needed.
    /// Removed signals are mapped to zero.
    pub fn sweep(&mut self) -> Box<[Signal]> {
        // Mark unused logic
        let mut visited = vec![false; self.nb_nodes()];
        let mut to_visit = Vec::<u32>::new();
        for o in 0..self.nb_outputs() {
            let output = self.output(o);
            if output.is_var() {
                to_visit.push(output.var());
            }
        }
        while !to_visit.is_empty() {
            let node = to_visit.pop().unwrap() as usize;
            if visited[node] {
                continue;
            }
            visited[node] = true;
            to_visit.extend(self.gate(node).vars().iter());
        }

        // Now compute a mapping for all nodes that are reachable
        let mut order = Vec::new();
        for (i, v) in visited.iter().enumerate() {
            if *v {
                order.push(i as u32);
            }
        }
        self.remap(order.as_slice())
    }

    /// Remove duplicate logic and make all functions canonical; this will invalidate all signals
    ///
    /// Returns the mapping of old variable indices to signals, if needed.
    /// Removed signals are mapped to zero.
    pub fn dedup(&mut self) -> Vec<Signal> {
        // Replace each node, in turn, by a simplified version or an equivalent existing node
        // We need the network to be topologically sorted, so that the gate inputs are already replaced
        // Dff gates are an exception to the sorting, and are handled separately
        assert!(self.is_topo_sorted());
        let mut translation = (0..self.nb_nodes())
            .map(|i| Signal::from_var(i as u32))
            .collect::<Vec<Signal>>();

        /// Core function for deduplication
        fn dedup_node(g: &Gate, h: &mut HashMap<Gate, Signal>, nodes: &mut Vec<Gate>) -> Signal {
            let normalized = g.make_canonical();
            match normalized {
                Normalization::Buf(sig) => sig,
                Normalization::Node(g, inv) => {
                    let node_s = Signal::from_var(nodes.len() as u32);
                    match h.entry(g.clone()) {
                        Entry::Occupied(e) => e.get() ^ inv,
                        Entry::Vacant(e) => {
                            e.insert(node_s);
                            nodes.push(g);
                            node_s ^ inv
                        }
                    }
                }
            }
        }

        let mut hsh = HashMap::new();
        let mut new_nodes = Vec::new();

        // Dedup flip flops
        for i in 0..self.nb_nodes() {
            let g = self.gate(i);
            if let Gate::Dff(_, _, _) = g {
                translation[i] = dedup_node(g, &mut hsh, &mut new_nodes);
            }
        }

        // Remap and dedup combinatorial gates
        for i in 0..self.nb_nodes() {
            let g = self.gate(i).remap(translation.as_slice());
            if let Gate::Dff(_, _, _) = g {
                continue;
            }
            translation[i] = dedup_node(&g, &mut hsh, &mut new_nodes);
        }

        // Remap flip flops
        for i in 0..new_nodes.len() {
            if let Gate::Dff(_, _, _) = new_nodes[i] {
                new_nodes[i] = new_nodes[i].remap(translation.as_slice());
            }
        }

        self.nodes = new_nodes;
        self.remap_outputs(&translation);
        self.check();
        translation
    }

    /// Topologically sort the Aig; this will invalidate all signals
    ///
    /// Returns the mapping of old variable indices to signals, if needed.
    /// Removed signals are mapped to zero.
    pub(crate) fn topo_sort(&mut self) -> Box<[Signal]> {
        // Count the output dependencies of each gate
        let mut count_deps = vec![0u32; self.nb_nodes()];
        for g in self.nodes.iter() {
            for v in g.comb_vars() {
                count_deps[v as usize] += 1;
            }
        }

        // Compute the topological sort
        let mut rev_order: Vec<u32> = Vec::new();
        let mut visited = vec![false; self.nb_nodes()];
        // Start with gates with no dependencies
        let mut to_visit: Vec<u32> = (0..self.nb_nodes())
            .filter(|v| count_deps[*v] == 0)
            .map(|v| v as u32)
            .collect();
        while let Some(v) = to_visit.pop() {
            // Visit the gate and mark the gates with satisfied dependencies
            if visited[v as usize] {
                continue;
            }
            visited[v as usize] = true;
            rev_order.push(v);
            for d in self.gate(v as usize).comb_vars() {
                count_deps[d as usize] -= 1;
                if count_deps[d as usize] == 0 {
                    to_visit.push(d);
                }
            }
        }
        if rev_order.len() != self.nb_nodes() {
            panic!("Unable to find a valid topological sort: there must be a combinatorial loop");
        }
        rev_order.reverse();
        let order = rev_order;

        self.remap(order.as_slice())
    }

    /// Check consistency of the datastructure
    pub fn check(&self) {
        for i in 0..self.nb_nodes() {
            for v in self.gate(i).dependencies() {
                if v.is_input() {
                    assert!(v.input() < self.nb_inputs() as u32);
                }
                if v.is_var() {
                    assert!(v.var() < self.nb_nodes() as u32);
                }
            }
        }
        for i in 0..self.nb_outputs() {
            let v = self.output(i);
            if v.is_input() {
                assert!(v.input() < self.nb_inputs() as u32);
            }
            if v.is_var() {
                assert!(v.var() < self.nb_nodes() as u32);
            }
        }
    }
}

impl fmt::Display for Aig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Aig with {} inputs, {} outputs:",
            self.nb_inputs(),
            self.nb_outputs()
        )?;
        for i in 0..self.nb_nodes() {
            writeln!(f, "\t{} = {}", self.node(i), self.gate(i))?;
        }
        for i in 0..self.nb_outputs() {
            writeln!(f, "\to{} = {}", i, self.output(i))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Aig, Signal};

    #[test]
    fn test_basic() {
        let mut aig = Aig::default();
        let i0 = aig.add_input();
        let i1 = aig.add_input();
        let x = aig.xor(i0, i1);
        aig.add_output(x);

        // Basic properties
        assert_eq!(aig.nb_inputs(), 2);
        assert_eq!(aig.nb_outputs(), 1);
        assert_eq!(aig.nb_nodes(), 1);
        assert!(aig.is_comb());
        assert!(aig.is_topo_sorted());

        // Access
        assert_eq!(aig.input(0), i0);
        assert_eq!(aig.input(1), i1);
        assert_eq!(aig.output(0), x);
    }

    #[test]
    fn test_dff() {
        let mut aig = Aig::default();
        let i0 = aig.add_input();
        let i1 = aig.add_input();
        let i2 = aig.add_input();
        let c0 = Signal::zero();
        let c1 = Signal::one();
        // Useful Dff
        assert_eq!(aig.dff(i0, i1, i2), Signal::from_var(0));
        assert_eq!(aig.dff(i0, i1, c0), Signal::from_var(1));
        // Dff that reduces to 0
        assert_eq!(aig.dff(c0, i1, i2), c0);
        assert_eq!(aig.dff(i0, c0, i2), c0);
        assert_eq!(aig.dff(i0, i1, c1), c0);
        assert!(!aig.is_comb());
        assert!(aig.is_topo_sorted());
    }

    #[test]
    fn test_nary() {
        let mut aig = Aig::default();
        let i0 = aig.add_input();
        let i1 = aig.add_input();
        let i2 = aig.add_input();
        let i3 = aig.add_input();
        let i4 = aig.add_input();

        assert_eq!(aig.and_n(&Vec::new()), Signal::one());
        assert_eq!(aig.and_n(&vec![i0]), i0);
        aig.and_n(&vec![i0, i1]);
        aig.and_n(&vec![i0, i1, i2]);
        aig.and_n(&vec![i0, i1, i2, i3]);
        aig.and_n(&vec![i0, i1, i2, i3, i4]);

        assert_eq!(aig.or_n(&Vec::new()), Signal::zero());
        assert_eq!(aig.or_n(&vec![i0]), i0);
        aig.or_n(&vec![i0, i1]);
        aig.or_n(&vec![i0, i1, i2]);
        aig.or_n(&vec![i0, i1, i2, i3]);
        aig.or_n(&vec![i0, i1, i2, i3, i4]);

        assert_eq!(aig.xor_n(&Vec::new()), Signal::zero());
        assert_eq!(aig.xor_n(&vec![i0]), i0);
        aig.xor_n(&vec![i0, i1]);
        aig.xor_n(&vec![i0, i1, i2]);
        aig.xor_n(&vec![i0, i1, i2, i3]);
        aig.xor_n(&vec![i0, i1, i2, i3, i4]);
    }

    #[test]
    fn test_sweep() {
        let mut aig = Aig::default();
        let i0 = aig.add_input();
        let i1 = aig.add_input();
        let x0 = aig.and(i0, i1);
        let x1 = aig.or(i0, i1);
        let _ = aig.and(x0, i1);
        let x3 = aig.or(x1, i1);
        aig.add_output(x3);
        let t = aig.sweep();
        assert_eq!(t.len(), 4);
        assert_eq!(aig.nb_nodes(), 2);
        assert_eq!(aig.nb_outputs(), 1);
        assert_eq!(
            t,
            vec![
                Signal::zero(),
                Signal::from_var(0),
                Signal::zero(),
                Signal::from_var(1)
            ]
            .into()
        );
    }

    #[test]
    fn test_dedup() {
        let mut aig = Aig::default();
        let i0 = aig.add_input();
        let i1 = aig.add_input();
        let i2 = aig.add_input();
        let x0 = aig.and(i0, i1);
        let x0_s = aig.and(i0, i1);
        let x1 = aig.and(x0, i2);
        let x1_s = aig.and(x0_s, i2);
        aig.add_output(x1);
        aig.add_output(x1_s);
        aig.dedup();
        assert_eq!(aig.nb_and(), 2);
    }
}