use super::instance_decls::*;
use super::ir::*;

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
        if !self.signal_exprs.contains_key(&signal) {
            let expr = match signal.data {
                graph::SignalData::Lit {
                    ref value,
                    bit_width,
                } => Expr::from_constant(value, bit_width),

                graph::SignalData::Input { ref name, .. } => Expr::Ref { name: name.clone() },

                graph::SignalData::Reg { .. } => Expr::Ref {
                    name: module_decls.regs[&signal].value_name.clone(),
                },

                graph::SignalData::UnOp { source, op } => {
                    let bit_width = source.bit_width();
                    let source = self.compile_signal(source, module_decls, a);
                    a.gen_temp(
                        Expr::UnOp {
                            source: Box::new(source),
                            op: match op {
                                graph::UnOp::Not => UnOp::Not,
                            },
                        },
                        bit_width,
                    )
                }
                graph::SignalData::SimpleBinOp { lhs, rhs, op } => {
                    let bit_width = lhs.bit_width();
                    let lhs = self.compile_signal(lhs, module_decls, a);
                    let rhs = self.compile_signal(rhs, module_decls, a);
                    a.gen_temp(
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
                    )
                }
                graph::SignalData::AdditiveBinOp { lhs, rhs, op } => {
                    let bit_width = lhs.bit_width();
                    let lhs = self.compile_signal(lhs, module_decls, a);
                    let rhs = self.compile_signal(rhs, module_decls, a);
                    a.gen_temp(
                        Expr::BinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: match op {
                                graph::AdditiveBinOp::Add => BinOp::Add,
                                graph::AdditiveBinOp::Sub => BinOp::Sub,
                            },
                        },
                        bit_width,
                    )
                }
                graph::SignalData::ComparisonBinOp { lhs, rhs, op } => {
                    let bit_width = signal.bit_width();
                    let mut lhs = self.compile_signal(lhs, module_decls, a);
                    let mut rhs = self.compile_signal(rhs, module_decls, a);
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
                    a.gen_temp(
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
                                | graph::ComparisonBinOp::GreaterThanSigned => BinOp::GreaterThan,
                                graph::ComparisonBinOp::GreaterThanEqual
                                | graph::ComparisonBinOp::GreaterThanEqualSigned => {
                                    BinOp::GreaterThanEqual
                                }
                            },
                        },
                        bit_width,
                    )
                }
                graph::SignalData::ShiftBinOp { lhs, rhs, op } => {
                    let bit_width = signal.bit_width();
                    let lhs = self.compile_signal(lhs, module_decls, a);
                    let rhs = self.compile_signal(rhs, module_decls, a);
                    a.gen_temp(Expr::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op: match op {
                            graph::ShiftBinOp::Shl => BinOp::Shl,
                            graph::ShiftBinOp::Shr => BinOp::Shr,
                            graph::ShiftBinOp::ShrArithmetic => BinOp::ShrArithmetic,
                        },
                    }, bit_width)
                }

                graph::SignalData::Bits {
                    source,
                    range_high,
                    range_low,
                } => {
                    let bit_width = signal.bit_width();
                    let source = self.compile_signal(source, module_decls, a);
                    a.gen_temp(
                        Expr::Bits {
                            source: Box::new(source),
                            range_high,
                            range_low,
                        },
                        bit_width,
                    )
                }

                graph::SignalData::Repeat { source, count } => {
                    let bit_width = signal.bit_width();
                    let source = self.compile_signal(source, module_decls, a);
                    a.gen_temp(Expr::Repeat {
                        source: Box::new(source),
                        count,
                    }, bit_width)
                }
                graph::SignalData::Concat { lhs, rhs } => {
                    let bit_width = signal.bit_width();
                    let lhs = self.compile_signal(lhs, module_decls, a);
                    let rhs = self.compile_signal(rhs, module_decls, a);
                    a.gen_temp(
                        Expr::Concat {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        bit_width,
                    )
                }

                graph::SignalData::Mux {
                    cond,
                    when_true,
                    when_false,
                } => {
                    let bit_width = when_true.bit_width();
                    let cond = self.compile_signal(cond, module_decls, a);
                    let when_true = self.compile_signal(when_true, module_decls, a);
                    let when_false = self.compile_signal(when_false, module_decls, a);
                    a.gen_temp(
                        Expr::Ternary {
                            cond: Box::new(cond),
                            when_true: Box::new(when_true),
                            when_false: Box::new(when_false),
                        },
                        bit_width,
                    )
                }

                graph::SignalData::InstanceOutput { instance, ref name } => {
                    let instance_decls = &module_decls.instances[&instance];
                    Expr::Ref {
                        name: instance_decls.output_names[name].clone(),
                    }
                }

                graph::SignalData::MemReadPortOutput { .. } => unimplemented!(),
            };
            self.signal_exprs.insert(signal, expr);
        }

        self.signal_exprs[&signal].clone()
    }
}
