// use z3::ast::Bool;

use crate::syntax::*;
use crate::trace::*;
use itertools::Itertools;
use std::sync::Arc;

// pub fn learn<const N: usize>(sample: Sample<N>) -> Option<SyntaxTree> {
//     unimplemented!();
// }

// pub fn learn_size<const N: usize>(sample: Sample<N>, size: usize) -> Option<SyntaxTree> {
//     unimplemented!();
// }

#[derive(Debug, Clone)]
enum SkeletonTree {
    Zeroary,
    Unary(Arc<SkeletonTree>),
    Binary((Arc<SkeletonTree>, Arc<SkeletonTree>)),
}

// impl SkeletonTree {
//     fn depth(&self) -> u8 {
//         match self {
//             SkeletonTree::Zeroary => 1,
//             SkeletonTree::Unary(child) => child.depth() + 1,
//             SkeletonTree::Binary((left_child, right_child)) => {
//                 left_child.depth().max(right_child.depth()) + 1
//             }
//         }
//     }
// }

pub fn brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<Arc<SyntaxTree>> {
    (0..).into_iter().find_map(|size| {
        if log {
            println!("Searching formulae of size {}", size);
        }
        gen_skeleton_trees(size)
            .into_iter()
            .flat_map(|skeleton| gen_formulae(N, sample.time_lenght(), &skeleton))
            .find(|formula| sample.is_consistent(formula))
    })
}

pub fn par_brute_solve<const N: usize>(sample: &Sample<N>, log: bool) -> Option<Arc<SyntaxTree>> {
    use rayon::prelude::*;

    (0..).into_iter().find_map(|size| {
        if log {
            println!("Generating formulae of size {}", size);
        }
        let trees = gen_skeleton_trees(size)
            .into_iter()
            .flat_map(|skeleton| gen_formulae(N, sample.time_lenght(), &skeleton));
        if log {
            println!("Searching formulae of size {}", size);
        }
        trees
            .par_bridge()
            .find_any(|formula| sample.is_consistent(formula))
    })
}

// Should be possible to compute skeleton trees at compile time
fn gen_skeleton_trees(size: usize) -> Vec<Arc<SkeletonTree>> {
    let mut skeletons = vec![Arc::new(SkeletonTree::Zeroary)];
    if size > 0 {
        let smaller_skeletons = gen_skeleton_trees(size - 1);
        skeletons.extend(
            smaller_skeletons
                .iter()
                .map(|child| Arc::new(SkeletonTree::Unary(child.clone()))),
        );
        for left_size in 0..size {
            let left_smaller_skeletons = gen_skeleton_trees(left_size);
            let right_smaller_skeletons = gen_skeleton_trees(size - 1 - left_size);

            skeletons.extend(
                left_smaller_skeletons
                    .iter()
                    .cartesian_product(right_smaller_skeletons.iter())
                    .map(|(left_child, right_child)| {
                        Arc::new(SkeletonTree::Binary((
                            left_child.clone(),
                            right_child.clone(),
                        )))
                    }),
            );
        }
    }
    skeletons
}

fn gen_formulae(atomics: usize, time_lenght: u8, skeleton: &SkeletonTree) -> Vec<Arc<SyntaxTree>> {
    match skeleton {
        SkeletonTree::Zeroary => {
            let mut trees = (0..atomics)
                .map(|n| {
                    Arc::new(SyntaxTree::Zeroary {
                        op: ZeroaryOp::AtomicProp(n),
                    })
                })
                .collect::<Vec<Arc<SyntaxTree>>>();
            trees.push(Arc::new(SyntaxTree::Zeroary {
                op: ZeroaryOp::False,
            }));
            trees
        }
        SkeletonTree::Unary(child) => {
            let mut trees = Vec::new();
            let children = gen_formulae(atomics, time_lenght, child);

            for child in children {
                if check_globally(&child) {
                    trees.push(Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::Globally,
                        child: child.clone(),
                    }));
                }
                if check_finally(&child) {
                    trees.push(Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::Finally,
                        child: child.clone(),
                    }));
                }

                if check_not(&child) {
                    trees.push(Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::Not,
                        child: child.clone(),
                    }));
                }

                if check_next(&child) {
                    trees.push(Arc::new(SyntaxTree::Unary {
                        op: UnaryOp::Next,
                        child,
                    }));
                }

                // trees.extend(
                //     [
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Not,
                //             child: child.clone(),
                //         }),
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Next,
                //             child: child.clone(),
                //         }),
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Globally,
                //             child: child.clone(),
                //         }),
                //         Arc::new(SyntaxTree::Unary {
                //             op: UnaryOp::Finally,
                //             child: child.clone(),
                //         }),
                //     ].into_iter()
                // );
            }

            // trees.extend(children.clone().into_iter().map(|child| {
            //     Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Not,
            //         child,
            //     })
            // }));
            // trees.extend(children.clone().into_iter().map(|child| {
            //     Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Next,
            //         child,
            //     })
            // }));
            // trees.extend(children.iter().filter(|child| check_globally(child)).map(|child| {
            // // trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Globally,
            //         child: child.clone(),
            //     })
            // }));
            // trees.extend(children.iter().filter(|child| check_finally(child)).map(|child| {
            // // trees.extend(children.iter().map(|child| {
            //     Arc::new(SyntaxTree::Unary {
            //         op: UnaryOp::Finally,
            //         child: child.clone(),
            //     })
            // }));

            // for time in 0..time_lenght {
            //     trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //             op: UnaryOp::GloballyLeq(time),
            //             child: child.clone(),
            //         })
            //     }));
            //     trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //             op: UnaryOp::GloballyGneq(time),
            //             child: child.clone(),
            //         })
            //     }));
            //     trees.extend(children.iter().map(|child| {
            //         Arc::new(SyntaxTree::Unary {
            //             op: UnaryOp::FinallyLeq(time),
            //             child: child.clone(),
            //         })
            //     }));
            // }
            trees
        }
        SkeletonTree::Binary(child) => {
            let mut trees = Vec::new();
            let left_children = gen_formulae(atomics, time_lenght, &child.0);
            let right_children = gen_formulae(atomics, time_lenght, &child.1);
            let children = left_children
                .into_iter()
                .cartesian_product(right_children.into_iter());

            for (left_child, right_child) in children {
                if check_and(&left_child, &right_child) {
                    trees.push(Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::And,
                        left_child: left_child.clone(),
                        right_child: right_child.clone(),
                    }));
                }

                if check_or(&left_child, &right_child) {
                    trees.push(Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::Or,
                        left_child: left_child.clone(),
                        right_child: right_child.clone(),
                    }));
                }

                if check_implies(&left_child, &right_child) {
                    trees.push(Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::Implies,
                        left_child: left_child.clone(),
                        right_child: right_child.clone(),
                    }));
                }

                if check_until(&left_child, &right_child) {
                    trees.push(Arc::new(SyntaxTree::Binary {
                        op: BinaryOp::Until,
                        left_child,
                        right_child,
                    }));
                }
            }

            // // Optimization for symmetric operators: use ordering on syntax trees to cut down the possible trees
            // trees.extend(children.clone().filter_map(|(left_child, right_child)| {
            //     if check_and(&left_child, &right_child) {
            //         Some(
            //             Arc::new(SyntaxTree::Binary {
            //                 op: BinaryOp::And,
            //                 left_child,
            //                 right_child,
            //             })
            //         )
            //     } else if left_child > right_child {
            //         Some(
            //             Arc::new(SyntaxTree::Binary {
            //                 op: BinaryOp::Or,
            //                 left_child,
            //                 right_child,
            //             })
            //         )
            //     } else {
            //         None
            //     }
            // }));

            // trees.extend(children.clone().map(|(left_child, right_child)| {
            //     Arc::new(SyntaxTree::Binary {
            //         op: BinaryOp::Implies,
            //         left_child,
            //         right_child,
            //     })
            // }));
            // trees.extend(children.clone().map(|(left_child, right_child)| {
            //     Arc::new(SyntaxTree::Binary {
            //         op: BinaryOp::Until,
            //         left_child,
            //         right_child,
            //     })
            // }));
            // trees.extend(children.clone().map(|(left_child, right_child)| {
            //     Arc::new(SyntaxTree::Binary {
            //         op: BinaryOp::Release,
            //         left_child,
            //         right_child,
            //     })
            // }));
            // for time in 0..time_lenght {
            //     trees.extend(children.clone().map(|(left_child, right_child)| {
            //         Arc::new(SyntaxTree::Binary {
            //             op: BinaryOp::ReleaseGneq(time),
            //             left_child,
            //             right_child,
            //         })
            //     }));
            //     trees.extend(children.clone().map(|(left_child, right_child)| {
            //         Arc::new(SyntaxTree::Binary {
            //             op: BinaryOp::ReleaseLeq(time),
            //             left_child,
            //             right_child,
            //         })
            //     }));
            //     trees.extend(children.clone().map(|(left_child, right_child)| {
            //         Arc::new(SyntaxTree::Binary {
            //             op: BinaryOp::UntillLeq(time),
            //             left_child,
            //             right_child,
            //         })
            //     }));
            // }
            trees
        }
    }
}

fn check_not(child: &SyntaxTree) -> bool {
    match *child {
        // ¬¬φ ≡ φ
        SyntaxTree::Unary { op: UnaryOp::Not, .. }
        // ¬(φ -> ψ) ≡ φ ∧ ¬ψ
        | SyntaxTree::Binary { op: BinaryOp::Implies, .. } => false,
        _ => true,
    }
}

fn check_next(child: &SyntaxTree) -> bool {
    match *child {
        // ¬ X φ ≡ X ¬ φ
        SyntaxTree::Unary {
            op: UnaryOp::Next, ..
        } => false,
        _ => true,
    }
}

fn check_globally(child: &SyntaxTree) -> bool {
    match *child {
        // G G φ <=> G φ
        SyntaxTree::Unary { op: UnaryOp::Globally, .. }
        // ¬ F φ ≡ G ¬ φ
        | SyntaxTree::Unary { op: UnaryOp::Finally, .. } => false,
        _ => true,
    }
}

fn check_finally(child: &SyntaxTree) -> bool {
    match *child {
        // F F φ <=> F φ
        SyntaxTree::Unary {
            op: UnaryOp::Finally,
            ..
        } => false,
        _ => true,
    }
}

fn check_and(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    // Commutative law
    left_child < right_child
        && match (left_child, right_child) {
        // Domination law
        (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
        | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
        // Associative laws
        | (SyntaxTree::Binary { op: BinaryOp::And, .. }, ..)
        // De Morgan's laws
        | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
        // X (φ ∧ ψ) ≡ (X φ) ∧ (X ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. })
        // G (φ ∧ ψ)≡ (G φ) ∧ (G ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Globally, .. }, SyntaxTree::Unary { op: UnaryOp::Globally, .. }) => false,
        // (φ -> ψ_1) ∧ (φ -> ψ_2) ≡ φ -> (ψ_1 ∧ ψ_2)
        // (φ_1 -> ψ) ∧ (φ_2 -> ψ) ≡ (φ_1 ∨ φ_2) -> ψ
        (SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_2, right_child: r_2 }) if *l_1 == *l_2 || *r_1 == *r_2 => false,
        // Absorption laws
        (SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_1, right_child: r_1 }, right_child) if *(l_1.as_ref()) == *right_child || *(r_1.as_ref()) == *right_child => false,
        (left_child, SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_1, right_child: r_1 }) if *(l_1.as_ref()) == *left_child || *(r_1.as_ref()) == *left_child => false,
        // Distributive laws
        (SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::Or, left_child: l_2, right_child: r_2 }) if *l_1 == *l_2 || *l_1 == *r_2 || *r_1 == *l_2 || *r_1 == *r_2 => false,
        _ => true,
    }
}

fn check_or(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    // Commutative law
    left_child < right_child
        && match (left_child, right_child) {
        // Identity law
        (.., SyntaxTree::Zeroary { op: ZeroaryOp::False })
        | (SyntaxTree::Zeroary { op: ZeroaryOp::False }, ..)
        // Associative laws
        | (SyntaxTree::Binary { op: BinaryOp::Or, .. }, ..)
        // // De Morgan's laws
        // | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. })
        // ¬φ ∨ ψ ≡ φ -> ψ, subsumes De Morgan's laws
        | (SyntaxTree::Unary { op: UnaryOp::Not, .. }, ..)
        // X (φ ∨ ψ) ≡ (X φ) ∨ (X ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Next, .. }, SyntaxTree::Unary { op: UnaryOp::Next, .. })
        // F (φ ∨ ψ) ≡ (F φ) ∨ (F ψ)
        | (SyntaxTree::Unary { op: UnaryOp::Finally, .. }, SyntaxTree::Unary { op: UnaryOp::Finally, .. }) => false,
        // (φ -> ψ_1) ∨ (φ -> ψ_2) ≡ φ -> (ψ_1 ∨ ψ_2)
        // (φ_1 -> ψ) ∨ (φ_2 -> ψ) ≡ (φ_1 ∧ φ_2) -> ψ
        (SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::Implies, left_child: l_2, right_child: r_2 }) if *l_1 == *l_2 || *r_1 == *r_2 => false,
        // Absorption laws
        (SyntaxTree::Binary { op: BinaryOp::And, left_child: l_1, right_child: r_1 }, right_child) if *(l_1.as_ref()) == *right_child || *(r_1.as_ref()) == *right_child => false,
        (left_child, SyntaxTree::Binary { op: BinaryOp::And, left_child: l_1, right_child: r_1 }) if *(l_1.as_ref()) == *left_child || *(r_1.as_ref()) == *left_child => false,
        // Distributive laws
        (SyntaxTree::Binary { op: BinaryOp::And, left_child: l_1, right_child: r_1 }, SyntaxTree::Binary { op: BinaryOp::And, left_child: l_2, right_child: r_2 }) if *l_1 == *l_2 || *l_1 == *r_2 || *r_1 == *l_2 || *r_1 == *r_2 => false,
        _ => true,
    }
}

fn check_implies(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    match (left_child, right_child) {
        // Ex falso quodlibet (need to define True)
        // (SyntaxTree::Zeroary { op: ZeroaryOp::False, .. }, ..)
        // // φ -> ψ ≡ ¬ψ -> ¬φ
        // (SyntaxTree::Unary { op: UnaryOp::Not, .. }, SyntaxTree::Unary { op: UnaryOp::Not, .. }) => false,
        // ¬φ -> ψ ≡ ψ ∨ φ
        (
            SyntaxTree::Unary {
                op: UnaryOp::Not, ..
            },
            ..,
        ) => false,
        _ => true,
    }
}

fn check_until(left_child: &SyntaxTree, right_child: &SyntaxTree) -> bool {
    match (left_child, right_child) {
        // X (φ U ψ) ≡ (X φ) U (X ψ)
        (
            SyntaxTree::Unary {
                op: UnaryOp::Next, ..
            },
            SyntaxTree::Unary {
                op: UnaryOp::Next, ..
            },
        ) => false,
        _ => true,
    }
}

// fn solve_skeleton(skeleton: &SkeletonTree) {
//     use z3::*;

//     let mut cfg = Config::new();
//     cfg.set_model_generation(true);
//     let ctx = Context::new(&cfg);
//     let solver = Solver::new(&ctx);

//     solver.assert(&Bool::and(
//         &ctx,
//         &[
//             &Bool::new_const(&ctx, 0).not(),
//             &Bool::new_const(&ctx, 1).not(),
//         ],
//     ));
//     if let SatResult::Sat = solver.check() {
//         if let Some(model) = solver.get_model() {
//             if let Some(x_0) = model.eval(&Bool::new_const(&ctx, 0), false) {
//                 println!("{}", x_0.as_bool().expect("Boolean value"));
//             }
//             if let Some(x_1) = model.eval(&Bool::new_const(&ctx, 1), false) {
//                 println!("{}", x_1.as_bool().expect("Boolean value"));
//             }
//         }
//     }
// }