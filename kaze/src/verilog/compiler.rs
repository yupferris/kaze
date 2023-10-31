use super::ir::*;

use crate::internal_signal;
use crate::state_elements::*;

use std::collections::HashMap;

pub(super) struct Compiler<'graph> {
    signal_exprs: HashMap<&'graph internal_signal::InternalSignal<'graph>, Expr>,
}

impl<'graph, 'context> Compiler<'graph> {
    pub fn new() -> Compiler<'graph> {
        Compiler {
            signal_exprs: HashMap::new(),
        }
    }

    pub fn compile_signal(
        &mut self,
        signal: &'graph internal_signal::InternalSignal<'graph>,
        state_elements: &'context StateElements<'graph>,
        a: &mut AssignmentContext,
    ) -> Expr {
        enum Frame<'graph> {
            Enter(&'graph internal_signal::InternalSignal<'graph>),
            Leave(&'graph internal_signal::InternalSignal<'graph>),
        }

        let mut frames = Vec::new();
        frames.push(Frame::Enter(signal));

        let mut results = Vec::new();

        while let Some(frame) = frames.pop() {
            if let Some(expr) = match frame {
                Frame::Enter(signal) => {
                    if let Some(expr) = self.signal_exprs.get(&signal) {
                        results.push(expr.clone());
                        continue;
                    }

                    match signal.data {
                        internal_signal::SignalData::Lit {
                            ref value,
                            bit_width,
                        } => Some(Expr::from_constant(value, bit_width)),

                        internal_signal::SignalData::Input { data } => {
                            if let Some(driven_value) = data.driven_value.borrow().clone() {
                                frames.push(Frame::Enter(driven_value));
                                None
                            } else {
                                Some(Expr::Ref {
                                    name: data.name.clone(),
                                })
                            }
                        }
                        internal_signal::SignalData::Output { data } => {
                            frames.push(Frame::Enter(data.source));
                            None
                        }

                        internal_signal::SignalData::Reg { .. } => Some(Expr::Ref {
                            name: state_elements.regs[&signal].value_name.clone(),
                        }),

                        internal_signal::SignalData::UnOp { source, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(source));
                            None
                        }
                        internal_signal::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }
                        internal_signal::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }
                        internal_signal::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }
                        internal_signal::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }

                        internal_signal::SignalData::Mul { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }
                        internal_signal::SignalData::MulSigned { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }

                        internal_signal::SignalData::Bits { source, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(source));
                            None
                        }

                        internal_signal::SignalData::Repeat { source, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(source));
                            None
                        }
                        internal_signal::SignalData::Concat { lhs, rhs, .. } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(lhs));
                            frames.push(Frame::Enter(rhs));
                            None
                        }

                        internal_signal::SignalData::Mux {
                            cond,
                            when_true,
                            when_false,
                            ..
                        } => {
                            frames.push(Frame::Leave(signal));
                            frames.push(Frame::Enter(cond));
                            frames.push(Frame::Enter(when_true));
                            frames.push(Frame::Enter(when_false));
                            None
                        }

                        internal_signal::SignalData::MemReadPortOutput {
                            mem,
                            address,
                            enable,
                        } => {
                            let mem = &state_elements.mems[&mem];
                            let read_signal_names = &mem.read_signal_names[&(address, enable)];
                            Some(Expr::Ref {
                                name: read_signal_names.value_name.clone(),
                            })
                        }
                    }
                }
                Frame::Leave(signal) => {
                    match signal.data {
                        internal_signal::SignalData::Lit { .. } => unreachable!(),

                        internal_signal::SignalData::Input { .. } => unreachable!(),
                        internal_signal::SignalData::Output { .. } => unreachable!(),

                        internal_signal::SignalData::Reg { .. } => unreachable!(),

                        internal_signal::SignalData::UnOp { op, bit_width, .. } => {
                            let source = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::UnOp {
                                    source: Box::new(source),
                                    op: match op {
                                        internal_signal::UnOp::Not => UnOp::Not,
                                    },
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }
                        internal_signal::SignalData::SimpleBinOp { op, bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        internal_signal::SimpleBinOp::BitAnd => BinOp::BitAnd,
                                        internal_signal::SimpleBinOp::BitOr => BinOp::BitOr,
                                        internal_signal::SimpleBinOp::BitXor => BinOp::BitXor,
                                    },
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }
                        internal_signal::SignalData::AdditiveBinOp { op, bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        internal_signal::AdditiveBinOp::Add => BinOp::Add,
                                        internal_signal::AdditiveBinOp::Sub => BinOp::Sub,
                                    },
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }
                        internal_signal::SignalData::ComparisonBinOp { op, .. } => {
                            let bit_width = signal.bit_width();
                            let mut lhs = results.pop().unwrap();
                            let mut rhs = results.pop().unwrap();
                            match op {
                                internal_signal::ComparisonBinOp::GreaterThanEqualSigned
                                | internal_signal::ComparisonBinOp::GreaterThanSigned
                                | internal_signal::ComparisonBinOp::LessThanEqualSigned
                                | internal_signal::ComparisonBinOp::LessThanSigned => {
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
                                        internal_signal::ComparisonBinOp::Equal => BinOp::Equal,
                                        internal_signal::ComparisonBinOp::NotEqual => BinOp::NotEqual,
                                        internal_signal::ComparisonBinOp::LessThan
                                        | internal_signal::ComparisonBinOp::LessThanSigned => BinOp::LessThan,
                                        internal_signal::ComparisonBinOp::LessThanEqual
                                        | internal_signal::ComparisonBinOp::LessThanEqualSigned => {
                                            BinOp::LessThanEqual
                                        }
                                        internal_signal::ComparisonBinOp::GreaterThan
                                        | internal_signal::ComparisonBinOp::GreaterThanSigned => {
                                            BinOp::GreaterThan
                                        }
                                        internal_signal::ComparisonBinOp::GreaterThanEqual
                                        | internal_signal::ComparisonBinOp::GreaterThanEqualSigned => {
                                            BinOp::GreaterThanEqual
                                        }
                                    },
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }
                        internal_signal::SignalData::ShiftBinOp { op, bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let lhs = match op {
                                internal_signal::ShiftBinOp::Shl
                                | internal_signal::ShiftBinOp::Shr => lhs,
                                internal_signal::ShiftBinOp::ShrArithmetic => Expr::Signed {
                                    source: Box::new(lhs),
                                },
                            };
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: match op {
                                        internal_signal::ShiftBinOp::Shl => BinOp::Shl,
                                        internal_signal::ShiftBinOp::Shr => BinOp::Shr,
                                        internal_signal::ShiftBinOp::ShrArithmetic => {
                                            BinOp::ShrArithmetic
                                        }
                                    },
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }

                        internal_signal::SignalData::Mul { bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::BinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                    op: BinOp::Mul,
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }
                        internal_signal::SignalData::MulSigned { bit_width, .. } => {
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
                                signal.module_instance_name_prefix(),
                            ))
                        }

                        internal_signal::SignalData::Bits {
                            source,
                            range_high,
                            range_low,
                        } => {
                            let source_bit_width = source.bit_width();
                            let bit_width = signal.bit_width();
                            let source = results.pop().unwrap();
                            // Verilog doesn't allow indexing scalars
                            Some(if source_bit_width == 1 {
                                source
                            } else {
                                a.gen_temp(
                                    Expr::Bits {
                                        source: Box::new(source),
                                        range_high,
                                        range_low,
                                    },
                                    bit_width,
                                    signal.module_instance_name_prefix(),
                                )
                            })
                        }

                        internal_signal::SignalData::Repeat {
                            count, bit_width, ..
                        } => {
                            let source = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::Repeat {
                                    source: Box::new(source),
                                    count,
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }
                        internal_signal::SignalData::Concat { bit_width, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some(a.gen_temp(
                                Expr::Concat {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                },
                                bit_width,
                                signal.module_instance_name_prefix(),
                            ))
                        }

                        internal_signal::SignalData::Mux { bit_width, .. } => {
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
                                signal.module_instance_name_prefix(),
                            ))
                        }

                        internal_signal::SignalData::MemReadPortOutput { .. } => unreachable!(),
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
