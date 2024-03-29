use serde::Deserialize;
use std::{fmt, sync::Arc};

/// The type representing time instants.
pub type Time = u8;

/// The type of indexes of propositional variables.
pub type Idx = u8;

/// A formula represented via its syntax tree.
/// This is a recursive data structure, so it requires the use of smart pointers.
/// We use `Arc` to make it compatible with parallel computations.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum SyntaxTree {
    Atom(Idx),
    Not(Arc<SyntaxTree>),
    Next(Arc<SyntaxTree>),
    Globally(Arc<SyntaxTree>),
    Finally(Arc<SyntaxTree>),
    And(Arc<SyntaxTree>, Arc<SyntaxTree>),
    Or(Arc<SyntaxTree>, Arc<SyntaxTree>),
    Implies(Arc<SyntaxTree>, Arc<SyntaxTree>),
    Until(Arc<SyntaxTree>, Arc<SyntaxTree>),
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxTree::Atom(var) => write!(f, "x{}", var),
            SyntaxTree::Not(branch) => write!(f, "¬({})", branch),
            SyntaxTree::Next(branch) => write!(f, "X({})", branch),
            SyntaxTree::Globally(branch) => write!(f, "G({})", branch),
            SyntaxTree::Finally(branch) => write!(f, "F({})", branch),
            SyntaxTree::And(left_branch, right_branch) => {
                write!(f, "({})∧({})", left_branch, right_branch)
            }
            SyntaxTree::Or(left_branch, right_branch) => {
                write!(f, "({})∨({})", left_branch, right_branch)
            }
            SyntaxTree::Implies(left_branch, right_branch) => {
                write!(f, "({})→({})", left_branch, right_branch)
            }
            SyntaxTree::Until(left_branch, right_branch) => {
                write!(f, "({})U({})", left_branch, right_branch)
            }
        }
    }
}

impl SyntaxTree {
    pub fn print_w_named_vars(&self, vars: &[String]) -> String {
        match self {
            SyntaxTree::Atom(var) => vars[*var as usize].clone(),
            SyntaxTree::Not(branch) => format!("¬({})", branch.print_w_named_vars(vars)),
            SyntaxTree::Next(branch) => format!("X({})", branch.print_w_named_vars(vars)),
            SyntaxTree::Globally(branch) => format!("G({})", branch.print_w_named_vars(vars)),
            SyntaxTree::Finally(branch) => format!("F({})", branch.print_w_named_vars(vars)),
            SyntaxTree::And(left_branch, right_branch) => {
                format!(
                    "({})∧({})",
                    left_branch.print_w_named_vars(vars),
                    right_branch.print_w_named_vars(vars)
                )
            }
            SyntaxTree::Or(left_branch, right_branch) => {
                format!(
                    "({})∨({})",
                    left_branch.print_w_named_vars(vars),
                    right_branch.print_w_named_vars(vars)
                )
            }
            SyntaxTree::Implies(left_branch, right_branch) => {
                format!(
                    "({})→({})",
                    left_branch.print_w_named_vars(vars),
                    right_branch.print_w_named_vars(vars)
                )
            }
            SyntaxTree::Until(left_branch, right_branch) => {
                format!(
                    "({})U({})",
                    left_branch.print_w_named_vars(vars),
                    right_branch.print_w_named_vars(vars)
                )
            }
        }
    }

    /// Returns the highest propositional variable index appearing in the formula, plus 1.
    /// Used to count how many variables are needed to interpret the formula.
    pub fn vars(&self) -> Idx {
        match self {
            SyntaxTree::Atom(n) => *n + 1,
            SyntaxTree::Not(branch)
            | SyntaxTree::Next(branch)
            | SyntaxTree::Globally(branch)
            | SyntaxTree::Finally(branch) => branch.as_ref().vars(),
            SyntaxTree::And(left_branch, right_branch)
            | SyntaxTree::Or(left_branch, right_branch)
            | SyntaxTree::Implies(left_branch, right_branch)
            | SyntaxTree::Until(left_branch, right_branch) => {
                left_branch.vars().max(right_branch.vars())
            }
        }
    }

    /// Evaluate a formula on a trace.
    pub fn eval<const N: usize>(&self, trace: &[[bool; N]]) -> bool {
        self.eval_at_time(trace, 0)
    }

    /// Evaluate a formula on a trace.
    pub fn eval_at_time<const N: usize>(&self, trace: &[[bool; N]], time: usize) -> bool {
        assert!(time < trace.len());

        match self {
            SyntaxTree::Atom(var) => trace[time][*var as usize],
            SyntaxTree::Not(branch) => !branch.eval_at_time(trace, time),
            SyntaxTree::Next(branch) => {
                time + 1 < trace.len() && branch.eval_at_time(trace, time + 1)
            }
            // Globally and Finally are interpreted by reverse temporal order because interpreting on shorter traces is generally faster.
            SyntaxTree::Globally(branch) => (time..trace.len())
                .rev()
                .all(|t| branch.eval_at_time(trace, t)),
            SyntaxTree::Finally(branch) => (time..trace.len())
                .rev()
                .any(|t| branch.eval_at_time(trace, t)),
            SyntaxTree::And(left_branch, right_branch) => {
                left_branch.eval_at_time(trace, time) && right_branch.eval_at_time(trace, time)
            }
            SyntaxTree::Or(left_branch, right_branch) => {
                left_branch.eval_at_time(trace, time) || right_branch.eval_at_time(trace, time)
            }
            SyntaxTree::Implies(left_branch, right_branch) => {
                !left_branch.eval_at_time(trace, time) || right_branch.eval_at_time(trace, time)
            }
            SyntaxTree::Until(left_branch, right_branch) => {
                for t in time..trace.len() {
                    if right_branch.eval_at_time(trace, t) {
                        return true;
                    } else if !left_branch.eval_at_time(trace, t) {
                        return false;
                    }
                }
                // (Strong) Until is not satisfied if its right-hand-side argument never becomes true.
                false
            }
        }
    }
}

#[cfg(test)]
mod eval {
    use super::*;

    const ATOM_0: SyntaxTree = SyntaxTree::Atom(0);

    const ATOM_1: SyntaxTree = SyntaxTree::Atom(1);

    #[test]
    fn atomic_prop() {
        let trace = [[true]];
        assert!(ATOM_0.eval(&trace));

        let trace = [[false]];
        assert!(!ATOM_0.eval(&trace));

        // let trace: [[bool; 1]; 0] = [];
        // assert!(!ATOM_0.eval(&trace));
    }

    #[test]
    fn not() {
        let formula = SyntaxTree::Not(Arc::new(ATOM_0));

        let trace = [[false]];
        assert!(formula.eval(&trace));

        let trace = [[true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn next() {
        let formula = SyntaxTree::Next(Arc::new(ATOM_0));

        let trace = [[false], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn globally() {
        let formula = SyntaxTree::Globally(Arc::new(ATOM_0));

        let trace = [[true], [true], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false], [true]];
        assert!(!formula.eval(&trace));

        // // Not even Globally can be true at moment 0 on an empty trace
        // let trace: [[bool; 1]; 0] = [];
        // assert!(!formula.eval(&trace));
    }

    #[test]
    fn finally() {
        let formula = SyntaxTree::Finally(Arc::new(ATOM_0));

        let trace = [[false], [false], [true]];
        assert!(formula.eval(&trace));

        let trace = [[false], [true], [false]];
        assert!(formula.eval(&trace));

        let trace = [[false], [false], [false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn and() {
        let formula = SyntaxTree::And(Arc::new(ATOM_0), Arc::new(ATOM_1));

        let trace = [[true, true]];
        assert!(formula.eval(&trace));

        let trace = [[true, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn or() {
        let formula = SyntaxTree::Or(Arc::new(ATOM_0), Arc::new(ATOM_1));

        let trace = [[true, false]];
        assert!(formula.eval(&trace));

        let trace = [[false, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn until() {
        let formula = SyntaxTree::Until(Arc::new(ATOM_0), Arc::new(ATOM_1));

        let trace = [[true, false], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [true, false], [false, false]];
        assert!(!formula.eval(&trace));

        // Until is not satisfied if its right-hand-side argument never becomes true.
        let trace = [[true, false], [true, false], [true, false]];
        assert!(!formula.eval(&trace));

        // let trace: [[bool; 2]; 0] = [];
        // assert!(!formula.eval(&trace));
    }
}
