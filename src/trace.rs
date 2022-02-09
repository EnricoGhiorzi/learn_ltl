use crate::syntax::*;
use serde::{Deserialize, Serialize};
use serde_with::*;

pub type Trace<const N: usize> = Vec<[bool; N]>;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Sample<const N: usize> {
    #[serde_as(as = "Vec<Vec<[_; N]>>")]
    pub positive_traces: Vec<Trace<N>>,
    #[serde_as(as = "Vec<Vec<[_; N]>>")]
    pub negative_traces: Vec<Trace<N>>,
}

impl<const N: usize> Sample<N> {
    pub fn is_consistent(&self, formula: &SyntaxTree) -> bool {
        use itertools::*;

        self.positive_traces
            .iter()
            .map(|trace| formula.eval(trace.as_slice()))
            .interleave(
                self.negative_traces
                    .iter()
                    .map(|trace| !formula.eval(trace.as_slice())),
            )
            .all(|val| val)
    }

    pub fn time_lenght(&self) -> Time {
        let positive_lenght = self
            .positive_traces
            .iter()
            .map(|trace| trace.len())
            .max()
            .unwrap_or(0);
        let negative_lenght = self
            .negative_traces
            .iter()
            .map(|trace| trace.len())
            .max()
            .unwrap_or(0);
        positive_lenght.max(negative_lenght) as Time
    }

    pub fn add_positive_trace(&mut self, trace: Trace<N>) -> Result<(), ()> {
        if !self.negative_traces.contains(&trace) {
            if !self.positive_traces.contains(&trace) {
                self.positive_traces.push(trace);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_negative_trace(&mut self, trace: Trace<N>) -> Result<(), ()> {
        if !self.positive_traces.contains(&trace) {
            if !self.negative_traces.contains(&trace) {
                self.negative_traces.push(trace);
            }
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod consistency {
    use std::sync::Arc;

    use super::*;

    const ATOM_0: SyntaxTree = SyntaxTree::Atom(0);

    const ATOM_1: SyntaxTree = SyntaxTree::Atom(1);

    #[test]
    fn and() {
        let sample = Sample {
            positive_traces: vec![vec![[true, true]]],
            negative_traces: vec![
                vec![[false, true]],
                vec![[true, false]],
                vec![[false, false]],
            ],
        };

        let formula = SyntaxTree::Binary {
            op: BinaryOp::And,
            children: Arc::new((ATOM_0, ATOM_1)),
        };

        assert!(sample.is_consistent(&formula));
    }
}
