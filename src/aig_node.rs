use std::{cmp, fmt};

use crate::literal::Lit;
use crate::literal::Num;

/// Representation of an AIG node
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct AigNode {
    a: Lit,
    b: Lit,
    c: Lit,
}

/// Possible types for the AIG node
enum NodeType {
    Maj,
    Mux,
}

/// Represent the result of the normalization of a node.
/// Either a literal if it can be simplified, or a canonical representation of an AIG node
enum NormalizationResult {
    /// Directly a literal
    Literal(Lit),
    PosNode(AigNode),
    NegNode(AigNode),
}

/// Represent the different basic gates that can be represented
enum BasicGate {
    And(Lit, Lit),
    Xor(Lit, Lit),
    Mux(Lit, Lit, Lit),
    Maj(Lit, Lit, Lit),
}

fn sort_2_lits(lits: (Lit, Lit)) -> (Lit, Lit) {
    let (i1, i0) = lits;
    (cmp::max(i1, i0), cmp::min(i1, i0))
}

fn sort_3_lits(lits: (Lit, Lit, Lit)) -> (Lit, Lit, Lit) {
    let (mut i2, mut i1, mut i0) = lits;
    (i2, i1) = sort_2_lits((i2, i1));
    (i1, i0) = sort_2_lits((i1, i0));
    (i2, i1) = sort_2_lits((i2, i1));
    (i2, i1, i0)
}

impl AigNode {
    /// Return the input literals, with internal flags removed
    fn lits(&self) -> (Lit, Lit, Lit) {
        (
            self.a.without_flag(),
            self.b.without_flag(),
            self.c.without_flag(),
        )
    }

    fn node_type(&self) -> NodeType {
        if self.a.flag() {
            NodeType::Mux
        } else {
            NodeType::Maj
        }
    }

    fn maj(a: Lit, b: Lit, c: Lit) -> AigNode {
        AigNode { a: a, b: b, c: c }
    }

    fn mux(a: Lit, b: Lit, c: Lit) -> AigNode {
        AigNode {
            a: a.with_flag(),
            b: b,
            c: c,
        }
    }

    pub fn simulate<T: Num>(&self, a_val: T, b_val: T, c_val: T) -> T {
        // Convert boolean flags to full-width words
        let toggle_a: T = self.a.pol_to_word();
        let toggle_b: T = self.b.pol_to_word();
        let toggle_c: T = self.c.pol_to_word();
        let sel_mux: T = self.a.flag_to_word();
        let av: T = a_val ^ toggle_a;
        let bv: T = b_val ^ toggle_b;
        let cv: T = c_val ^ toggle_c;
        let mux: T = (av & bv) | (!av & cv);
        let maj: T = (av & bv) | (av & cv) | (bv & cv);
        (!sel_mux & maj) | (sel_mux & mux)
    }

    pub fn is_canonical(&self) -> bool {
        match self.node_type() {
            NodeType::Mux => self.is_canonical_mux(),
            NodeType::Maj => self.is_canonical_maj(),
        }
    }

    fn is_canonical_mux(&self) -> bool {
        let (a, b, c) = self.lits();
        if a.is_constant() || b.is_constant() || c.is_constant() {
            // No constant at all allowed on Mux
            return false;
        }
        if b == c {
            // Would be a constant
            return false;
        }
        if a.pol() {
            // No inversion on a
            return false;
        }
        if a.ind() == b.ind() || a.ind() == c.ind() {
            // No sharing between selector and other variables
            return false;
        }
        if c.pol() {
            // a and b not inverted
            // Valid for both Xor and Mux representation
            return false;
        }
        if b.ind() == c.ind() {
            // Xor
            if a > b {
                // Force the smallest one to be first
                return false;
            }
        }
        return true;
    }

    fn is_canonical_maj(&self) -> bool {
        let (a, b, c) = self.lits();
        if a.ind() <= b.ind() || b.ind() <= c.ind() {
            // Force strict ordering on the inputs, a > b > c
            return false;
        }
        if c == Lit::one() {
            // Only constant one on the last input, representing an And
            return false;
        }
        return true;
    }

    /// Normalize an and gate
    fn make_canonical_and(a: Lit, b: Lit) -> NormalizationResult {
        assert!(!a.flag() && !b.flag());
        let mn = cmp::min(a, b);
        let mx = cmp::max(a, b);
        if mn == Lit::zero() {
            return NormalizationResult::Literal(Lit::zero());
        }
        if mn == Lit::one() {
            return NormalizationResult::Literal(mx);
        }
        if mn == mx {
            return NormalizationResult::Literal(mn);
        }
        if mn == !mx {
            return NormalizationResult::Literal(Lit::zero());
        }
        NormalizationResult::PosNode(AigNode {
            a: mx,
            b: mn,
            c: Lit::zero(),
        })
    }

    /// Normalize a xor gate
    fn make_canonical_xor(a: Lit, b: Lit) -> NormalizationResult {
        assert!(!a.flag() && !b.flag());
        let mn = cmp::min(a, b);
        let mx = cmp::max(a, b);
        if mn == Lit::zero() {
            return NormalizationResult::Literal(mx);
        }
        if mn == Lit::one() {
            return NormalizationResult::Literal(!mx);
        }
        if mn == mx {
            return NormalizationResult::Literal(Lit::zero());
        }
        if mn == !mx {
            return NormalizationResult::Literal(Lit::one());
        }
        let pol = mn.pol() ^ mx.pol();
        // TODO: handling polarities
        NormalizationResult::PosNode(AigNode {
            a: mn.with_flag(),
            b: !mx,
            c: mx,
        })
    }

    // Normalize a mux gate
    fn make_canonical_mux(s: Lit, a: Lit, b: Lit) -> NormalizationResult {
        // TODO
        if s == Lit::zero() {
            return NormalizationResult::Literal(b);
        }
        if s == Lit::one() {
            return NormalizationResult::Literal(a);
        }
        let (i1, i0) = if s.pol() { (a, b) } else { (b, a) };
        NormalizationResult::PosNode(AigNode {
            a: s.with_flag(),
            b: a,
            c: b,
        })
    }

    // Normalize a maj gate
    fn make_canonical_maj(a: Lit, b: Lit, c: Lit) -> NormalizationResult {
        // Sort the inputs
        // Two inputs constant
        // One input constant
        let (i2, i1, i0) = sort_3_lits((a, b, c));
        NormalizationResult::PosNode(AigNode { a: a, b: b, c: c })
    }

    fn make_canonical(&self) -> NormalizationResult {
        // Normalize a majority

        NormalizationResult::PosNode(self.clone())
    }

    /// Return the basic gate corresponding to a node
    /// Canonization is not done here, so non-canonical And/Xor
    /// are not detected
    pub fn as_gate(&self) -> BasicGate {
        let (a, b, c) = self.lits();
        if self.a.flag() {
            // Mux/Xor
            if b == !c {
                BasicGate::Xor(a, c)
            } else {
                BasicGate::Mux(a, b, c)
            }
        } else if c == Lit::zero() {
            BasicGate::And(a, b)
        } else {
            BasicGate::Maj(a, b, c)
        }
    }
}

impl fmt::Display for AigNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BasicGate::*;
        let g = self.as_gate();
        match g {
            And(a, b) => {
                write!(f, "And({a}, {b})")
            }
            Xor(a, b) => {
                write!(f, "Xor({a}, {b})")
            }
            Mux(a, b, c) => {
                write!(f, "Mux({a}, {b}, {c})")
            }
            Maj(a, b, c) => {
                write!(f, "Maj({a}, {b}, {c})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_canonical() {
        let l0 = Lit::zero();
        let l1 = Lit::one();
        let i0 = Lit::from_var(0);
        let i1 = Lit::from_var(1);
        let i2 = Lit::from_var(2);

        // Canonical maj
        assert!(AigNode::maj(i2, i1, i0).is_canonical());
        assert!(AigNode::maj(!i2, i1, i0).is_canonical());
        assert!(AigNode::maj(i2, !i1, i0).is_canonical());
        assert!(AigNode::maj(i2, i1, !i0).is_canonical());
        assert!(AigNode::maj(i2, !i1, !i0).is_canonical());
        assert!(AigNode::maj(!i2, i1, !i0).is_canonical());
        assert!(AigNode::maj(!i2, !i1, i0).is_canonical());
        assert!(AigNode::maj(!i2, !i1, !i0).is_canonical());
        // Bad orders
        assert!(!AigNode::maj(i1, i2, i0).is_canonical());
        assert!(!AigNode::maj(i2, i0, i1).is_canonical());
        // Duplication
        assert!(!AigNode::maj(i1, i1, i0).is_canonical());
        assert!(!AigNode::maj(i1, i0, i0).is_canonical());
        assert!(!AigNode::maj(!i1, i1, i0).is_canonical());
        assert!(!AigNode::maj(i1, !i0, i0).is_canonical());

        // Canonical and
        assert!(AigNode::maj(i2, i1, l0).is_canonical());
        // Bad constants
        assert!(!AigNode::maj(i2, l1, l0).is_canonical());
        assert!(!AigNode::maj(i2, i1, l1).is_canonical());
        assert!(!AigNode::maj(i2, l1, l1).is_canonical());
        // Duplication
        assert!(!AigNode::maj(i2, i2, i1).is_canonical());
        assert!(!AigNode::maj(i2, !i2, i1).is_canonical());
        assert!(!AigNode::maj(i2, i1, !i2).is_canonical());

        // Canonical muxes
        assert!(AigNode::mux(i2, i1, i0).is_canonical());
        assert!(AigNode::mux(i1, i0, i2).is_canonical());
        assert!(AigNode::mux(i0, i2, i1).is_canonical());
        assert!(AigNode::mux(i2, i0, i1).is_canonical());
        assert!(AigNode::mux(i1, i2, i0).is_canonical());
        assert!(AigNode::mux(i0, i1, i2).is_canonical());
        assert!(AigNode::mux(i2, !i1, i0).is_canonical());
        assert!(AigNode::mux(i1, !i0, i2).is_canonical());
        assert!(AigNode::mux(i0, !i2, i1).is_canonical());
        assert!(AigNode::mux(i2, !i0, i1).is_canonical());
        assert!(AigNode::mux(i1, !i2, i0).is_canonical());
        assert!(AigNode::mux(i0, !i1, i2).is_canonical());
        // Bad complementation on selector
        assert!(!AigNode::mux(!i2, i1, i0).is_canonical());
        assert!(!AigNode::mux(!i1, i0, i2).is_canonical());
        assert!(!AigNode::mux(!i0, i2, i1).is_canonical());
        assert!(!AigNode::mux(!i2, i0, i1).is_canonical());
        assert!(!AigNode::mux(!i1, i2, i0).is_canonical());
        assert!(!AigNode::mux(!i0, i1, i2).is_canonical());
        // Bad complementation on selected
        assert!(!AigNode::mux(i2, i1, !i0).is_canonical());
        assert!(!AigNode::mux(i1, i0, !i2).is_canonical());
        assert!(!AigNode::mux(i0, i2, !i1).is_canonical());
        assert!(!AigNode::mux(i2, i0, !i1).is_canonical());
        assert!(!AigNode::mux(i1, i2, !i0).is_canonical());
        assert!(!AigNode::mux(i0, i1, !i2).is_canonical());
        assert!(!AigNode::mux(i2, !i1, !i0).is_canonical());
        assert!(!AigNode::mux(i1, !i0, !i2).is_canonical());
        assert!(!AigNode::mux(i0, !i2, !i1).is_canonical());
        assert!(!AigNode::mux(i2, !i0, !i1).is_canonical());
        assert!(!AigNode::mux(i1, !i2, !i0).is_canonical());
        assert!(!AigNode::mux(i0, !i1, !i2).is_canonical());
        // Bad constants
        assert!(!AigNode::mux(l1, i1, i0).is_canonical());
        assert!(!AigNode::mux(i2, l1, i0).is_canonical());
        assert!(!AigNode::mux(i2, i1, l1).is_canonical());
        assert!(!AigNode::mux(l0, i1, i0).is_canonical());
        assert!(!AigNode::mux(i2, l0, i0).is_canonical());
        assert!(!AigNode::mux(i2, i1, l0).is_canonical());

        // Canonical xor
        assert!(AigNode::mux(i0, !i1, i1).is_canonical());
        // Bad complementation
        assert!(!AigNode::mux(i0, i1, !i1).is_canonical());
        // Bad order
        assert!(!AigNode::mux(i1, !i0, i0).is_canonical());
    }

    #[test]
    fn test_format() {
        let l0 = Lit::zero();
        let i0 = Lit::from_var(0);
        let i1 = Lit::from_var(1);
        let i2 = Lit::from_var(2);
        assert_eq!(format!("{}", AigNode::maj(i2, i1, i0)), "Maj(v2, v1, v0)");
        assert_eq!(format!("{}", AigNode::maj(i2, i1, l0)), "And(v2, v1)");
        assert_eq!(format!("{}", AigNode::mux(i2, i1, i0)), "Mux(v2, v1, v0)");
        assert_eq!(format!("{}", AigNode::mux(i0, !i1, i1)), "Xor(v0, v1)");
    }
}
