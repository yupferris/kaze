use super::ir::*;

use crate::graph::internal_signal;
use crate::state_elements::*;

use typed_arena::Arena;

use std::collections::HashMap;

// TODO: Can we merge the context and expr_arena lifetimes?
pub(super) struct Compiler<'graph, 'context, 'expr_arena> {
    state_elements: &'context StateElements<'graph>,
    signal_reference_counts:
        &'context HashMap<&'graph internal_signal::InternalSignal<'graph>, u32>,
    expr_arena: &'expr_arena Arena<Expr<'expr_arena>>,

    signal_exprs:
        HashMap<&'graph internal_signal::InternalSignal<'graph>, &'expr_arena Expr<'expr_arena>>,
}

impl<'graph, 'context, 'expr_arena> Compiler<'graph, 'context, 'expr_arena> {
    pub fn new(
        state_elements: &'context StateElements<'graph>,
        signal_reference_counts: &'context HashMap<
            &'graph internal_signal::InternalSignal<'graph>,
            u32,
        >,
        expr_arena: &'expr_arena Arena<Expr<'expr_arena>>,
    ) -> Compiler<'graph, 'context, 'expr_arena> {
        Compiler {
            state_elements,
            signal_reference_counts,
            expr_arena,

            signal_exprs: HashMap::new(),
        }
    }

    pub fn compile_signal(
        &mut self,
        signal: &'graph internal_signal::InternalSignal<'graph>,
        a: &mut AssignmentContext<'expr_arena>,
    ) -> &'expr_arena Expr<'expr_arena> {
        enum Frame<'graph> {
            Enter {
                signal: &'graph internal_signal::InternalSignal<'graph>,
            },
            Leave {
                signal: &'graph internal_signal::InternalSignal<'graph>,
            },
        }

        let mut frames = Vec::new();
        frames.push(Frame::Enter { signal });

        let mut results = Vec::new();

        while let Some(frame) = frames.pop() {
            if let Some((key, mut expr)) = match frame {
                Frame::Enter { signal } => {
                    let key = signal;
                    if let Some(expr) = self.signal_exprs.get(&key) {
                        results.push(*expr);
                        continue;
                    }

                    match signal.data {
                        internal_signal::SignalData::Lit {
                            ref value,
                            bit_width,
                        } => Some((key, Expr::from_constant(value, bit_width, &self.expr_arena))),

                        internal_signal::SignalData::Input { data } => {
                            if let Some(driven_value) = data.driven_value.borrow().clone() {
                                frames.push(Frame::Enter {
                                    signal: driven_value,
                                });
                                None
                            } else {
                                let bit_width = data.bit_width;
                                let target_type = ValueType::from_bit_width(bit_width);
                                let expr = self.expr_arena.alloc(Expr::Ref {
                                    name: data.name.clone(),
                                    scope: Scope::Member,
                                });
                                Some((key, self.gen_mask(expr, bit_width, target_type)))
                            }
                        }
                        internal_signal::SignalData::Output { data } => {
                            frames.push(Frame::Enter {
                                signal: data.source,
                            });
                            None
                        }

                        internal_signal::SignalData::Reg { .. } => Some((
                            key,
                            &*self.expr_arena.alloc(Expr::Ref {
                                name: self.state_elements.regs[&key].value_name.clone(),
                                scope: Scope::Member,
                            }),
                        )),

                        internal_signal::SignalData::UnOp { source, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: source });
                            None
                        }
                        internal_signal::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }
                        internal_signal::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }
                        internal_signal::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }
                        internal_signal::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }

                        internal_signal::SignalData::Mul { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }
                        internal_signal::SignalData::MulSigned { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }

                        internal_signal::SignalData::Bits { source, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: source });
                            None
                        }

                        internal_signal::SignalData::Repeat { source, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: source });
                            None
                        }
                        internal_signal::SignalData::Concat { lhs, rhs, .. } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: lhs });
                            frames.push(Frame::Enter { signal: rhs });
                            None
                        }

                        internal_signal::SignalData::Mux {
                            cond,
                            when_true,
                            when_false,
                            ..
                        } => {
                            frames.push(Frame::Leave { signal });
                            frames.push(Frame::Enter { signal: cond });
                            frames.push(Frame::Enter { signal: when_true });
                            frames.push(Frame::Enter { signal: when_false });
                            None
                        }

                        internal_signal::SignalData::MemReadPortOutput {
                            mem,
                            address,
                            enable,
                        } => {
                            let mem = &self.state_elements.mems[&mem];
                            let read_signal_names = &mem.read_signal_names[&(address, enable)];
                            Some((
                                key,
                                &*self.expr_arena.alloc(Expr::Ref {
                                    name: read_signal_names.value_name.clone(),
                                    scope: Scope::Member,
                                }),
                            ))
                        }
                    }
                }
                Frame::Leave { signal } => {
                    let key = signal;

                    match signal.data {
                        internal_signal::SignalData::Lit { .. } => unreachable!(),

                        internal_signal::SignalData::Input { .. } => unreachable!(),
                        internal_signal::SignalData::Output { .. } => unreachable!(),

                        internal_signal::SignalData::Reg { .. } => unreachable!(),

                        internal_signal::SignalData::UnOp { op, bit_width, .. } => {
                            let expr = results.pop().unwrap();
                            let expr = self.expr_arena.alloc(Expr::UnOp {
                                source: expr,
                                op: match op {
                                    internal_signal::UnOp::Not => UnOp::Not,
                                },
                            });

                            let target_type = ValueType::from_bit_width(bit_width);
                            Some((key, self.gen_mask(expr, bit_width, target_type)))
                        }
                        internal_signal::SignalData::SimpleBinOp { op, .. } => {
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            Some((
                                key,
                                &*self.expr_arena.alloc(Expr::InfixBinOp {
                                    lhs,
                                    rhs,
                                    op: match op {
                                        internal_signal::SimpleBinOp::BitAnd => InfixBinOp::BitAnd,
                                        internal_signal::SimpleBinOp::BitOr => InfixBinOp::BitOr,
                                        internal_signal::SimpleBinOp::BitXor => InfixBinOp::BitXor,
                                    },
                                }),
                            ))
                        }
                        internal_signal::SignalData::AdditiveBinOp { lhs, op, .. } => {
                            let source_bit_width = lhs.bit_width();
                            let source_type = ValueType::from_bit_width(source_bit_width);
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            let op_input_type = match source_type {
                                ValueType::Bool => ValueType::U32,
                                _ => source_type,
                            };
                            let lhs = self.gen_cast(lhs, source_type, op_input_type);
                            let rhs = self.gen_cast(rhs, source_type, op_input_type);
                            let expr = self.expr_arena.alloc(Expr::UnaryMemberCall {
                                target: lhs,
                                name: match op {
                                    internal_signal::AdditiveBinOp::Add => "wrapping_add".into(),
                                    internal_signal::AdditiveBinOp::Sub => "wrapping_sub".into(),
                                },
                                arg: rhs,
                            });
                            let op_output_type = op_input_type;
                            let target_bit_width = signal.bit_width();
                            let target_type = ValueType::from_bit_width(target_bit_width);
                            let expr = self.gen_cast(expr, op_output_type, target_type);
                            Some((key, self.gen_mask(expr, target_bit_width, target_type)))
                        }
                        internal_signal::SignalData::ComparisonBinOp { lhs, op, .. } => {
                            let source_bit_width = lhs.bit_width();
                            let source_type = ValueType::from_bit_width(source_bit_width);
                            let mut lhs = results.pop().unwrap();
                            let mut rhs = results.pop().unwrap();
                            match op {
                                internal_signal::ComparisonBinOp::GreaterThanEqualSigned
                                | internal_signal::ComparisonBinOp::GreaterThanSigned
                                | internal_signal::ComparisonBinOp::LessThanEqualSigned
                                | internal_signal::ComparisonBinOp::LessThanSigned => {
                                    let source_type_signed = source_type.to_signed();
                                    lhs = self.gen_cast(lhs, source_type, source_type_signed);
                                    rhs = self.gen_cast(rhs, source_type, source_type_signed);
                                    lhs = self.gen_sign_extend_shifts(
                                        lhs,
                                        source_bit_width,
                                        source_type_signed,
                                    );
                                    rhs = self.gen_sign_extend_shifts(
                                        rhs,
                                        source_bit_width,
                                        source_type_signed,
                                    );
                                }
                                _ => (),
                            }
                            Some((
                                key,
                                &*self.expr_arena.alloc(Expr::InfixBinOp {
                                    lhs,
                                    rhs,
                                    op: match op {
                                        internal_signal::ComparisonBinOp::Equal => InfixBinOp::Equal,
                                        internal_signal::ComparisonBinOp::NotEqual => InfixBinOp::NotEqual,
                                        internal_signal::ComparisonBinOp::LessThan
                                        | internal_signal::ComparisonBinOp::LessThanSigned => {
                                            InfixBinOp::LessThan
                                        }
                                        internal_signal::ComparisonBinOp::LessThanEqual
                                        | internal_signal::ComparisonBinOp::LessThanEqualSigned => {
                                            InfixBinOp::LessThanEqual
                                        }
                                        internal_signal::ComparisonBinOp::GreaterThan
                                        | internal_signal::ComparisonBinOp::GreaterThanSigned => {
                                            InfixBinOp::GreaterThan
                                        }
                                        internal_signal::ComparisonBinOp::GreaterThanEqual
                                        | internal_signal::ComparisonBinOp::GreaterThanEqualSigned => {
                                            InfixBinOp::GreaterThanEqual
                                        }
                                    },
                                }),
                            ))
                        }
                        internal_signal::SignalData::ShiftBinOp {
                            lhs,
                            rhs,
                            op,
                            bit_width,
                        } => {
                            let lhs_source_bit_width = lhs.bit_width();
                            let lhs_source_type = ValueType::from_bit_width(lhs_source_bit_width);
                            let rhs_source_bit_width = rhs.bit_width();
                            let rhs_source_type = ValueType::from_bit_width(rhs_source_bit_width);
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            let lhs_op_input_type = match lhs_source_type {
                                ValueType::Bool => ValueType::U32,
                                _ => lhs_source_type,
                            };
                            let lhs = self.gen_cast(lhs, lhs_source_type, lhs_op_input_type);
                            let lhs = match op {
                                internal_signal::ShiftBinOp::Shl
                                | internal_signal::ShiftBinOp::Shr => lhs,
                                internal_signal::ShiftBinOp::ShrArithmetic => {
                                    let lhs_op_input_type_signed = lhs_op_input_type.to_signed();
                                    let lhs = self.gen_cast(
                                        lhs,
                                        lhs_op_input_type,
                                        lhs_op_input_type_signed,
                                    );
                                    self.gen_sign_extend_shifts(
                                        lhs,
                                        lhs_source_bit_width,
                                        lhs_op_input_type_signed,
                                    )
                                }
                            };
                            let rhs_op_input_type = match rhs_source_type {
                                ValueType::Bool => ValueType::U32,
                                _ => rhs_source_type,
                            };
                            let rhs = self.gen_cast(rhs, rhs_source_type, rhs_op_input_type);
                            let rhs = self.expr_arena.alloc(Expr::BinaryFunctionCall {
                                name: "std::cmp::min".into(),
                                lhs: rhs,
                                rhs: self.expr_arena.alloc(Expr::Constant {
                                    value: match rhs_op_input_type {
                                        ValueType::Bool
                                        | ValueType::I32
                                        | ValueType::I64
                                        | ValueType::I128 => unreachable!(),
                                        ValueType::U32 => Constant::U32(std::u32::MAX),
                                        ValueType::U64 => Constant::U64(std::u32::MAX as _),
                                        ValueType::U128 => Constant::U128(std::u32::MAX as _),
                                    },
                                }),
                            });
                            let rhs = self.gen_cast(rhs, lhs_op_input_type, ValueType::U32);
                            let expr = self.expr_arena.alloc(Expr::UnaryMemberCall {
                                target: lhs,
                                name: match op {
                                    internal_signal::ShiftBinOp::Shl => "checked_shl".into(),
                                    internal_signal::ShiftBinOp::Shr
                                    | internal_signal::ShiftBinOp::ShrArithmetic => {
                                        "checked_shr".into()
                                    }
                                },
                                arg: rhs,
                            });
                            let expr = self.expr_arena.alloc(Expr::UnaryMemberCall {
                                target: expr,
                                name: "unwrap_or".into(),
                                arg: match op {
                                    internal_signal::ShiftBinOp::Shl
                                    | internal_signal::ShiftBinOp::Shr => {
                                        self.expr_arena.alloc(Expr::Constant {
                                            value: match lhs_op_input_type {
                                                ValueType::Bool
                                                | ValueType::I32
                                                | ValueType::I64
                                                | ValueType::I128 => unreachable!(),
                                                ValueType::U32 => Constant::U32(0),
                                                ValueType::U64 => Constant::U64(0),
                                                ValueType::U128 => Constant::U128(0),
                                            },
                                        })
                                    }
                                    internal_signal::ShiftBinOp::ShrArithmetic => {
                                        self.expr_arena.alloc(Expr::InfixBinOp {
                                            lhs,
                                            rhs: self.expr_arena.alloc(Expr::Constant {
                                                value: Constant::U32(
                                                    lhs_op_input_type.bit_width() - 1,
                                                ),
                                            }),
                                            op: InfixBinOp::Shr,
                                        })
                                    }
                                },
                            });
                            let op_output_type = lhs_op_input_type;
                            let expr = match op {
                                internal_signal::ShiftBinOp::Shl
                                | internal_signal::ShiftBinOp::Shr => expr,
                                internal_signal::ShiftBinOp::ShrArithmetic => {
                                    let lhs_op_output_type_signed = op_output_type.to_signed();
                                    self.gen_cast(expr, lhs_op_output_type_signed, op_output_type)
                                }
                            };
                            let target_bit_width = bit_width;
                            let target_type = ValueType::from_bit_width(target_bit_width);
                            let expr = self.gen_cast(expr, op_output_type, target_type);
                            Some((key, self.gen_mask(expr, target_bit_width, target_type)))
                        }

                        internal_signal::SignalData::Mul {
                            lhs,
                            rhs,
                            bit_width,
                        } => {
                            let lhs_type = ValueType::from_bit_width(lhs.bit_width());
                            let rhs_type = ValueType::from_bit_width(rhs.bit_width());
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            let target_type = ValueType::from_bit_width(bit_width);
                            let lhs = self.gen_cast(lhs, lhs_type, target_type);
                            let rhs = self.gen_cast(rhs, rhs_type, target_type);
                            Some((
                                key,
                                &*self.expr_arena.alloc(Expr::InfixBinOp {
                                    lhs,
                                    rhs,
                                    op: InfixBinOp::Mul,
                                }),
                            ))
                        }
                        internal_signal::SignalData::MulSigned {
                            lhs,
                            rhs,
                            bit_width,
                        } => {
                            let lhs_bit_width = lhs.bit_width();
                            let rhs_bit_width = rhs.bit_width();
                            let lhs_type = ValueType::from_bit_width(lhs_bit_width);
                            let rhs_type = ValueType::from_bit_width(rhs_bit_width);
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            let target_bit_width = bit_width;
                            let target_type = ValueType::from_bit_width(target_bit_width);
                            let target_type_signed = target_type.to_signed();
                            let lhs = self.gen_cast(lhs, lhs_type, target_type_signed);
                            let rhs = self.gen_cast(rhs, rhs_type, target_type_signed);
                            let lhs =
                                self.gen_sign_extend_shifts(lhs, lhs_bit_width, target_type_signed);
                            let rhs =
                                self.gen_sign_extend_shifts(rhs, rhs_bit_width, target_type_signed);
                            let expr = self.expr_arena.alloc(Expr::InfixBinOp {
                                lhs,
                                rhs,
                                op: InfixBinOp::Mul,
                            });
                            let expr = self.gen_cast(expr, target_type_signed, target_type);
                            Some((key, self.gen_mask(expr, target_bit_width, target_type)))
                        }

                        internal_signal::SignalData::Bits {
                            source, range_low, ..
                        } => {
                            let expr = results.pop().unwrap();
                            let expr = self.gen_shift_right(expr, range_low);
                            let target_bit_width = signal.bit_width();
                            let target_type = ValueType::from_bit_width(target_bit_width);
                            let expr = self.gen_cast(
                                expr,
                                ValueType::from_bit_width(source.bit_width()),
                                target_type,
                            );
                            Some((key, self.gen_mask(expr, target_bit_width, target_type)))
                        }

                        internal_signal::SignalData::Repeat {
                            source,
                            count,
                            bit_width,
                        } => {
                            let expr = results.pop().unwrap();
                            let mut expr = self.gen_cast(
                                expr,
                                ValueType::from_bit_width(source.bit_width()),
                                ValueType::from_bit_width(bit_width),
                            );

                            if count > 1 {
                                let source_expr = a.gen_temp(expr);

                                for i in 1..count {
                                    let rhs =
                                        self.gen_shift_left(source_expr, i * source.bit_width());
                                    expr = self.expr_arena.alloc(Expr::InfixBinOp {
                                        lhs: expr,
                                        rhs,
                                        op: InfixBinOp::BitOr,
                                    });
                                }
                            }

                            Some((key, expr))
                        }
                        internal_signal::SignalData::Concat {
                            lhs,
                            rhs,
                            bit_width,
                        } => {
                            let lhs_type = ValueType::from_bit_width(lhs.bit_width());
                            let rhs_bit_width = rhs.bit_width();
                            let rhs_type = ValueType::from_bit_width(rhs_bit_width);
                            let lhs = results.pop().unwrap();
                            let rhs = results.pop().unwrap();
                            let target_type = ValueType::from_bit_width(bit_width);
                            let lhs = self.gen_cast(lhs, lhs_type, target_type);
                            let rhs = self.gen_cast(rhs, rhs_type, target_type);
                            let lhs = self.gen_shift_left(lhs, rhs_bit_width);
                            Some((
                                key,
                                &*self.expr_arena.alloc(Expr::InfixBinOp {
                                    lhs,
                                    rhs,
                                    op: InfixBinOp::BitOr,
                                }),
                            ))
                        }

                        internal_signal::SignalData::Mux { .. } => {
                            let cond = results.pop().unwrap();
                            let when_true = results.pop().unwrap();
                            let when_false = results.pop().unwrap();
                            Some((
                                key,
                                &*self.expr_arena.alloc(Expr::Ternary {
                                    cond,
                                    when_true,
                                    when_false,
                                }),
                            ))
                        }

                        internal_signal::SignalData::MemReadPortOutput { .. } => unreachable!(),
                    }
                }
            } {
                // Generate a temp if this signal is referenced more than once
                if self.signal_reference_counts[&key] > 1 {
                    expr = a.gen_temp(expr);
                }
                self.signal_exprs.insert(key, expr);
                results.push(expr);
            }
        }

        results.pop().unwrap()
    }

    fn gen_mask(
        &mut self,
        expr: &'expr_arena Expr<'expr_arena>,
        bit_width: u32,
        target_type: ValueType,
    ) -> &'expr_arena Expr<'expr_arena> {
        if bit_width == target_type.bit_width() {
            return expr;
        }

        let mask = (1u128 << bit_width) - 1;
        self.expr_arena.alloc(Expr::InfixBinOp {
            lhs: expr,
            rhs: self.expr_arena.alloc(Expr::Constant {
                value: match target_type {
                    ValueType::Bool | ValueType::I32 | ValueType::I64 | ValueType::I128 => {
                        unreachable!()
                    }
                    ValueType::U32 => Constant::U32(mask as _),
                    ValueType::U64 => Constant::U64(mask as _),
                    ValueType::U128 => Constant::U128(mask),
                },
            }),
            op: InfixBinOp::BitAnd,
        })
    }

    fn gen_shift_left(
        &mut self,
        expr: &'expr_arena Expr<'expr_arena>,
        shift: u32,
    ) -> &'expr_arena Expr<'expr_arena> {
        if shift == 0 {
            return expr;
        }

        self.expr_arena.alloc(Expr::InfixBinOp {
            lhs: expr,
            rhs: self.expr_arena.alloc(Expr::Constant {
                value: Constant::U32(shift),
            }),
            op: InfixBinOp::Shl,
        })
    }

    fn gen_shift_right(
        &mut self,
        expr: &'expr_arena Expr<'expr_arena>,
        shift: u32,
    ) -> &'expr_arena Expr<'expr_arena> {
        if shift == 0 {
            return expr;
        }

        self.expr_arena.alloc(Expr::InfixBinOp {
            lhs: expr,
            rhs: self.expr_arena.alloc(Expr::Constant {
                value: Constant::U32(shift),
            }),
            op: InfixBinOp::Shr,
        })
    }

    fn gen_cast(
        &mut self,
        expr: &'expr_arena Expr<'expr_arena>,
        source_type: ValueType,
        target_type: ValueType,
    ) -> &'expr_arena Expr<'expr_arena> {
        if source_type == target_type {
            return expr;
        }

        if target_type == ValueType::Bool {
            let expr = self.gen_mask(expr, 1, source_type);
            return self.expr_arena.alloc(Expr::InfixBinOp {
                lhs: expr,
                rhs: self.expr_arena.alloc(Expr::Constant {
                    value: match source_type {
                        ValueType::Bool | ValueType::I32 | ValueType::I64 | ValueType::I128 => {
                            unreachable!()
                        }
                        ValueType::U32 => Constant::U32(0),
                        ValueType::U64 => Constant::U64(0),
                        ValueType::U128 => Constant::U128(0),
                    },
                }),
                op: InfixBinOp::NotEqual,
            });
        }

        self.expr_arena.alloc(Expr::Cast {
            source: expr,
            target_type,
        })
    }

    fn gen_sign_extend_shifts(
        &mut self,
        expr: &'expr_arena Expr,
        source_bit_width: u32,
        target_type: ValueType,
    ) -> &'expr_arena Expr<'expr_arena> {
        let shift = target_type.bit_width() - source_bit_width;
        let expr = self.gen_shift_left(expr, shift);
        self.gen_shift_right(expr, shift)
    }
}
