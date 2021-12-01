use learn_pltl_fast::*;
use std::fs::File;
use std::io::BufWriter;

use std::sync::Arc;

fn main() {
    // G(&(x0,->(!(x1),U(!(x1),&(x2,!(x1))))))
    // let not_x1 = Arc::new(SyntaxTree::Unary {
    //     op: UnaryOp::Not,
    //     child: Arc::new(SyntaxTree::Zeroary {
    //         op: ZeroaryOp::AtomicProp(1),
    //     }),
    // });
    // let formula = SyntaxTree::Unary {
    //     op: UnaryOp::Globally,
    //     child: Arc::new(SyntaxTree::Binary {
    //         op: BinaryOp::And,
    //         left_child: Arc::new(SyntaxTree::Zeroary {
    //             op: ZeroaryOp::AtomicProp(0),
    //         }),
    //         right_child: Arc::new(SyntaxTree::Binary {
    //             op: BinaryOp::Implies,
    //             left_child: not_x1.clone(),
    //             right_child: Arc::new(SyntaxTree::Binary {
    //                 op: BinaryOp::Until,
    //                 left_child: not_x1.clone(),
    //                 right_child: Arc::new(SyntaxTree::Binary {
    //                     op: BinaryOp::And,
    //                     left_child: Arc::new(SyntaxTree::Zeroary {
    //                         op: ZeroaryOp::AtomicProp(2),
    //                     }),
    //                     right_child: not_x1,
    //                 }),
    //             }),
    //         }),
    //     }),
    // };

    // let formula = SyntaxTree::Binary {
    //     op: BinaryOp::Or,
    //     left_child: Arc::new(SyntaxTree::Unary {
    //         op: UnaryOp::Globally,
    //         child: Arc::new(SyntaxTree::Unary {
    //             op: UnaryOp::Not,
    //             child: Arc::new(SyntaxTree::Zeroary {
    //                 op: ZeroaryOp::AtomicProp(0),
    //             }),
    //         }),
    //     }),
    //     right_child: Arc::new(SyntaxTree::Unary {
    //         op: UnaryOp::Finally,
    //         child: Arc::new(SyntaxTree::Binary {
    //             op: BinaryOp::And,
    //             left_child: Arc::new(SyntaxTree::Zeroary {
    //                 op: ZeroaryOp::AtomicProp(0),
    //             }),
    //             right_child: Arc::new(SyntaxTree::Unary {
    //                 op: UnaryOp::Finally,
    //                 child: Arc::new(SyntaxTree::Zeroary {
    //                     op: ZeroaryOp::AtomicProp(1),
    //                 }),
    //             }),
    //         }),
    //     }),
    // };
    // let formula = SyntaxTree::Binary {
    //     op: BinaryOp::Until,
    //     left_child: Arc::new(SyntaxTree::Zeroary {
    //         op: ZeroaryOp::AtomicProp(0)
    //     }),
    //     right_child: Arc::new(SyntaxTree::Zeroary {
    //         op: ZeroaryOp::AtomicProp(1)
    //     }),
    // };
    let formula = SyntaxTree::Binary {
        op: BinaryOp::Implies,
        children: Arc::new((
            SyntaxTree::Zeroary {
                op: ZeroaryOp::AtomicProp(0)
            },
            SyntaxTree::Zeroary {
                op: ZeroaryOp::AtomicProp(1)
            }
        )),
    };
    let sample = sample::<3>(&formula, 697, 1698 - 698, 10);
    assert!(sample.is_consistent(&formula));
    let name = format!("sample_{}.ron", formula);
    let file = File::create(name).expect("open sample file");
    let buf_writer = BufWriter::new(file);
    ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
}

fn sample<const N: usize>(
    formula: &SyntaxTree,
    positives: usize,
    negatives: usize,
    lenght: usize,
) -> Sample<N> {
    let mut positive_traces = Vec::new();
    let mut negative_traces = Vec::new();
    while positive_traces.len() < positives || negative_traces.len() < negatives {
        let trace = Vec::from_iter((0..lenght).map(|_| gen_bools()));
        let satisfaction = formula.eval(&trace);
        if satisfaction && positive_traces.len() < positives {
            positive_traces.push(trace);
        } else if !satisfaction && negative_traces.len() < negatives {
            negative_traces.push(trace);
        }
    }
    Sample {
        positive_traces,
        negative_traces,
    }
}

fn gen_bools<const N: usize>() -> [bool; N] {
    use rand::prelude::*;
    let mut values = [true; N];
    rand::thread_rng().fill(&mut values[..]);
    values
}
