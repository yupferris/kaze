use super::ir::*;
use super::module_decls::*;

use crate::graph;

use std::collections::HashMap;

pub struct Compiler<'graph> {
    signal_exprs: HashMap<&'graph graph::Signal<'graph>, Expr>,
}

impl<'graph> Compiler<'graph> {
    pub fn new() -> Compiler<'graph> {
        Compiler {
            signal_exprs: HashMap::new(),
        }
    }

    pub fn compile_signal(
        &mut self,
        signal: &'graph graph::Signal<'graph>,
        module_decls: &ModuleDecls<'graph>,
        a: &mut AssignmentContext,
    ) -> Expr {
        enum Node<'graph> {
            Enter(&'graph graph::Signal<'graph>),
            Leave(&'graph graph::Signal<'graph>),
        }

        let mut nodes = Vec::new();
        nodes.push(Node::Enter(signal));

        let mut results = Vec::new();

        while let Some(node) = nodes.pop() {
            if let Some(expr) = match node {
                Node::Enter(signal) => {
                    if let Some(expr) = self.signal_exprs.get(&signal) {
                        results.push(expr.clone());
                        continue;
                    }

                    match signal.data {
                        graph::SignalData::Lit {
                            ref value,
                            bit_width,
                        } => Some(Expr::from_constant(value, bit_width)),

                        graph::SignalData::Input { ref name, .. } => {
                            Some(Expr::Ref { name: name.clone() })
                        }

                        graph::SignalData::Reg { .. } => Some(Expr::Ref {
                            name: module_decls.regs[&signal].value_name.clone(),
                        }),

                        graph::SignalData::UnOp { source, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(source));
                            None
                        }
                        graph::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }
                        graph::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }
                        graph::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }
                        graph::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }

                        graph::SignalData::Mul { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }
                        graph::SignalData::MulSigned { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }

                        graph::SignalData::Bits { source, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(source));
                            None
                        }

                        graph::SignalData::Repeat { source, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(source));
                            None
                        }
                        graph::SignalData::Concat { lhs, rhs, .. } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(lhs));
                            nodes.push(Node::Enter(rhs));
                            None
                        }

                        graph::SignalData::Mux {
                            cond,
                            when_true,
                            when_false,
                            ..
                        } => {
                            nodes.push(Node::Leave(signal));
                            nodes.push(Node::Enter(cond));
                            nodes.push(Node::Enter(when_true));
                            nodes.push(Node::Enter(when_false));
                            None
                        }

                        graph::SignalData::InstanceOutput {
                            instance, ref name, ..
                        } => {
                            let instance_decls = &module_decls.instances[&instance];
                            Some(Expr::Ref {
                                name: instance_decls.output_names[name].clone(),
                            })
                        }

                        graph::SignalData::MemReadPortOutput {
                            mem,
                            address,
                            enable,
                        } => {
                            let mem = &module_decls.mems[&mem];
                            let read_signal_names = &mem.read_signal_names[&(address, enable)];
                            Some(Expr::Ref {
                                name: read_signal_names.value_name.clone(),
                            })
                        }
                    }
                }
                Node::Leave(signal) => {
                    match signal.data {
                        graph::SignalData::Lit { .. } => unreachable!(),

                        graph::SignalData::Input { .. } => unreachable!(),

                        graph::SignalData::Reg { .. } => unreachable!(),

                        graph::SignalData::UnOp { op, bit_width, .. } => {
                            let source = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::UnOp {
                                    source: Box::new(source),
                                    op: match op {
                                        graph::UnOp::Not => UnOp::Not,
                                    },
                                },
                                bit_width,
                            ))
                        }
                        graph::SignalData::SimpleBinOp { op, bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        graph::SimpleBinOp::BitAnd => BinOp::BitAnd,
                                        graph::SimpleBinOp::BitOr => BinOp::BitOr,
                                        graph::SimpleBinOp::BitXor => BinOp::BitXor,
                                    },
                                },
                                bit_width,
                            ))
                        }
                        graph::SignalData::AdditiveBinOp { op, bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        graph::AdditiveBinOp::Add => BinOp::Add,
                                        graph::AdditiveBinOp::Sub => BinOp::Sub,
                                    },
                                },
                                bit_width,
                            ))
                        }
                        graph::SignalData::ComparisonBinOp { op, .. } => {
                            let bit_width = signal.bit_width();
                            let mut lhs = results.pop().unwrap();
                            let mut rhs = results.pop().unwrap();
                            match op {
                                graph::ComparisonBinOp::GreaterThanEqualSigned
                                | graph::ComparisonBinOp::GreaterThanSigned
                                | graph::ComparisonBinOp::LessThanEqualSigned
                                | graph::ComparisonBinOp::LessThanSigned => {
                                    lhs = Expr::Signed {
                                        source: Box::new(lhs),
                                    };
                                    rhs = Expr::Signed {
                                        source: Box::new(rhs),
                                    };
                                }
                                _ => (),
                            }
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        graph::ComparisonBinOp::Equal => BinOp::Equal,
                                        graph::ComparisonBinOp::NotEqual => BinOp::NotEqual,
                                        graph::ComparisonBinOp::LessThan
                                        | graph::ComparisonBinOp::LessThanSigned => BinOp::LessThan,
                                        graph::ComparisonBinOp::LessThanEqual
                                        | graph::ComparisonBinOp::LessThanEqualSigned => {
                                            BinOp::LessThanEqual
                                        }
                                        graph::ComparisonBinOp::GreaterThan
                                        | graph::ComparisonBinOp::GreaterThanSigned => {
                                            BinOp::GreaterThan
                                        }
                                        graph::ComparisonBinOp::GreaterThanEqual
                                        | graph::ComparisonBinOp::GreaterThanEqualSigned => {
                                            BinOp::GreaterThanEqual
                                        }
                                    },
                                },
                                bit_width,
                            ))
                        }
                        graph::SignalData::ShiftBinOp { op, bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        graph::ShiftBinOp::Shl => BinOp::Shl,
                                        graph::ShiftBinOp::Shr => BinOp::Shr,
                                        graph::ShiftBinOp::ShrArithmetic => BinOp::ShrArithmetic,
                                    },
                                },
                                bit_width,
                            ))
                        }

                        graph::SignalData::Mul { bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: BinOp::Mul,
                                },
                                bit_width,
                            ))
                        }
                        graph::SignalData::MulSigned { bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            let lhs = Expr::Signed {
                                source: Box::new(lhs),
                            };
                            let rhs = Expr::Signed {
                                source: Box::new(rhs),
                            };
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: BinOp::Mul,
                                },
                                bit_width,
                            ))
                        }

                        graph::SignalData::Bits {
                            range_high,
                            range_low,
                            ..
                        } => {
                            let bit_width = signal.bit_width();
                            let source = results.pop().unwrap();
                            // Verilog doesn't allow indexing scalars
                            Some(if bit_width == 1 {
                                source
                            } else {
                                a.gen_temp(
                                    Expr::Bits {
                                        source: Box::new(source),
                                        range_high,
                                        range_low,
                                    },
                                    bit_width,
                                )
                            })
                        }

                        graph::SignalData::Repeat {
                            count, bit_width, ..
                        } => {
                            let source = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::Repeat {
                                    source: Box::new(source),
                                    count,
                                },
                                bit_width,
                            ))
                        }
                        graph::SignalData::Concat { bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::Concat {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                },
                                bit_width,
                            ))
                        }

                        graph::SignalData::Mux { bit_width, .. } => {
                            let cond = results.pop().unwrap();
                            let when_true = results.pop().unwrap();
                            let when_false = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::Ternary {
                                    cond: Box::new(cond),
                                    when_true: Box::new(when_true),
                                    when_false: Box::new(when_false),
                                },
                                bit_width,
                            ))
                        }

                        graph::SignalData::InstanceOutput { .. } => unreachable!(),

                        graph::SignalData::MemReadPortOutput { .. } => unreachable!(),
                    }
                }
            } {
                self.signal_exprs.insert(signal, expr.clone());
                results.push(expr);
            }
        }

        results.pop().unwrap()
    }
}
