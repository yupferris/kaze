use super::ir::*;
use super::state_elements::*;

use crate::graph;
use crate::module_context::*;

use typed_arena::Arena;

use std::collections::HashMap;

pub(super) struct Compiler<'graph, 'arena> {
    state_elements: &'arena StateElements<'graph, 'arena>,
    context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,

    signal_exprs: HashMap<
        (
            &'arena ModuleContext<'graph, 'arena>,
            &'graph graph::Signal<'graph>,
        ),
        Expr,
    >,
}

impl<'graph, 'arena> Compiler<'graph, 'arena> {
    pub fn new(
        state_elements: &'arena StateElements<'graph, 'arena>,
        context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,
    ) -> Compiler<'graph, 'arena> {
        Compiler {
            state_elements,
            context_arena,

            signal_exprs: HashMap::new(),
        }
    }

    pub fn compile_signal(
        &mut self,
        signal: &'graph graph::Signal<'graph>,
        context: &'arena ModuleContext<'graph, 'arena>,
        a: &mut AssignmentContext,
    ) -> Expr {
        enum Node<'graph, 'arena> {
            Enter {
                signal: &'graph graph::Signal<'graph>,
                context: &'arena ModuleContext<'graph, 'arena>,
            },
            Leave {
                signal: &'graph graph::Signal<'graph>,
            },
        }

        let mut nodes = Vec::new();
        nodes.push(Node::Enter { signal, context });

        let mut results = Vec::new();

        while let Some(node) = nodes.pop() {
            if let Some(expr) = match node {
                Node::Enter { signal, context } => {
                    let key = (context, signal);
                    if let Some(expr) = self.signal_exprs.get(&key) {
                        results.push(expr.clone());
                        continue;
                    }

                    match signal.data {
                        graph::SignalData::Lit {
                            ref value,
                            bit_width,
                        } => Some(Expr::from_constant(value, bit_width)),

                        graph::SignalData::Input {
                            ref name,
                            bit_width,
                        } => {
                            if let Some((instance, parent)) = context.instance_and_parent {
                                nodes.push(Node::Enter {
                                    signal: instance.driven_inputs.borrow()[name],
                                    context: parent,
                                });
                                None
                            } else {
                                let target_type = ValueType::from_bit_width(bit_width);
                                let expr = Expr::Ref {
                                    name: name.clone(),
                                    scope: Scope::Member,
                                };
                                Some(self.gen_mask(expr, bit_width, target_type, a))
                            }
                        }

                        graph::SignalData::Reg { .. } => Some(Expr::Ref {
                            name: self.state_elements.regs[&key].value_name.clone(),
                            scope: Scope::Member,
                        }),

                        graph::SignalData::UnOp { source, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: source,
                                context,
                            });
                            None
                        }
                        graph::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }
                        graph::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }
                        graph::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }
                        graph::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }

                        graph::SignalData::Mul { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }
                        graph::SignalData::MulSigned { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }

                        graph::SignalData::Bits { source, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: source,
                                context,
                            });
                            None
                        }

                        graph::SignalData::Repeat { source, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: source,
                                context,
                            });
                            None
                        }
                        graph::SignalData::Concat { lhs, rhs, .. } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: lhs,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: rhs,
                                context,
                            });
                            None
                        }

                        graph::SignalData::Mux {
                            cond,
                            when_true,
                            when_false,
                            ..
                        } => {
                            nodes.push(Node::Leave { signal });
                            nodes.push(Node::Enter {
                                signal: cond,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: when_true,
                                context,
                            });
                            nodes.push(Node::Enter {
                                signal: when_false,
                                context,
                            });
                            None
                        }

                        graph::SignalData::InstanceOutput {
                            instance, ref name, ..
                        } => {
                            let output = instance.instantiated_module.outputs.borrow()[name];
                            nodes.push(Node::Enter {
                                signal: output,
                                context: context.get_child(instance, self.context_arena),
                            });
                            None
                        }

                        graph::SignalData::MemReadPortOutput {
                            mem,
                            address,
                            enable,
                        } => {
                            let mem = &self.state_elements.mems[&(context, mem)];
                            let read_signal_names = &mem.read_signal_names[&(address, enable)];
                            Some(Expr::Ref {
                                name: read_signal_names.value_name.clone(),
                                scope: Scope::Member,
                            })
                        }
                    }
                }
                Node::Leave { signal } => match signal.data {
                    graph::SignalData::Lit { .. } => unreachable!(),

                    graph::SignalData::Input { .. } => unreachable!(),

                    graph::SignalData::Reg { .. } => unreachable!(),

                    graph::SignalData::UnOp { op, bit_width, .. } => {
                        let expr = results.pop().unwrap();
                        let expr = a.gen_temp(Expr::UnOp {
                            source: Box::new(expr),
                            op: match op {
                                graph::UnOp::Not => UnOp::Not,
                            },
                        });

                        let target_type = ValueType::from_bit_width(bit_width);
                        Some(self.gen_mask(expr, bit_width, target_type, a))
                    }
                    graph::SignalData::SimpleBinOp { op, .. } => {
                        let lhs = results.pop().unwrap();
                        let rhs = results.pop().unwrap();
                        Some(a.gen_temp(Expr::InfixBinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: match op {
                                graph::SimpleBinOp::BitAnd => InfixBinOp::BitAnd,
                                graph::SimpleBinOp::BitOr => InfixBinOp::BitOr,
                                graph::SimpleBinOp::BitXor => InfixBinOp::BitXor,
                            },
                        }))
                    }
                    graph::SignalData::AdditiveBinOp { lhs, op, .. } => {
                        let source_bit_width = lhs.bit_width();
                        let source_type = ValueType::from_bit_width(source_bit_width);
                        let lhs = results.pop().unwrap();
                        let rhs = results.pop().unwrap();
                        let op_input_type = match source_type {
                            ValueType::Bool => ValueType::U32,
                            _ => source_type,
                        };
                        let lhs = self.gen_cast(lhs, source_type, op_input_type, a);
                        let rhs = self.gen_cast(rhs, source_type, op_input_type, a);
                        let expr = a.gen_temp(Expr::UnaryMemberCall {
                            target: Box::new(lhs),
                            name: match op {
                                graph::AdditiveBinOp::Add => "wrapping_add".into(),
                                graph::AdditiveBinOp::Sub => "wrapping_sub".into(),
                            },
                            arg: Box::new(rhs),
                        });
                        let op_output_type = op_input_type;
                        let target_bit_width = signal.bit_width();
                        let target_type = ValueType::from_bit_width(target_bit_width);
                        let expr = self.gen_cast(expr, op_output_type, target_type, a);
                        Some(self.gen_mask(expr, target_bit_width, target_type, a))
                    }
                    graph::SignalData::ComparisonBinOp { lhs, op, .. } => {
                        let source_bit_width = lhs.bit_width();
                        let source_type = ValueType::from_bit_width(source_bit_width);
                        let mut lhs = results.pop().unwrap();
                        let mut rhs = results.pop().unwrap();
                        match op {
                            graph::ComparisonBinOp::GreaterThanEqualSigned
                            | graph::ComparisonBinOp::GreaterThanSigned
                            | graph::ComparisonBinOp::LessThanEqualSigned
                            | graph::ComparisonBinOp::LessThanSigned => {
                                let source_type_signed = source_type.to_signed();
                                lhs = self.gen_cast(lhs, source_type, source_type_signed, a);
                                rhs = self.gen_cast(rhs, source_type, source_type_signed, a);
                                lhs = self.gen_sign_extend_shifts(
                                    lhs,
                                    source_bit_width,
                                    source_type_signed,
                                    a,
                                );
                                rhs = self.gen_sign_extend_shifts(
                                    rhs,
                                    source_bit_width,
                                    source_type_signed,
                                    a,
                                );
                            }
                            _ => (),
                        }
                        Some(a.gen_temp(Expr::InfixBinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: match op {
                                graph::ComparisonBinOp::Equal => InfixBinOp::Equal,
                                graph::ComparisonBinOp::NotEqual => InfixBinOp::NotEqual,
                                graph::ComparisonBinOp::LessThan
                                | graph::ComparisonBinOp::LessThanSigned => InfixBinOp::LessThan,
                                graph::ComparisonBinOp::LessThanEqual
                                | graph::ComparisonBinOp::LessThanEqualSigned => {
                                    InfixBinOp::LessThanEqual
                                }
                                graph::ComparisonBinOp::GreaterThan
                                | graph::ComparisonBinOp::GreaterThanSigned => {
                                    InfixBinOp::GreaterThan
                                }
                                graph::ComparisonBinOp::GreaterThanEqual
                                | graph::ComparisonBinOp::GreaterThanEqualSigned => {
                                    InfixBinOp::GreaterThanEqual
                                }
                            },
                        }))
                    }
                    graph::SignalData::ShiftBinOp {
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
                        let lhs = self.gen_cast(lhs, lhs_source_type, lhs_op_input_type, a);
                        let lhs = match op {
                            graph::ShiftBinOp::Shl | graph::ShiftBinOp::Shr => lhs,
                            graph::ShiftBinOp::ShrArithmetic => {
                                let lhs_op_input_type_signed = lhs_op_input_type.to_signed();
                                let lhs = self.gen_cast(
                                    lhs,
                                    lhs_op_input_type,
                                    lhs_op_input_type_signed,
                                    a,
                                );
                                self.gen_sign_extend_shifts(
                                    lhs,
                                    lhs_source_bit_width,
                                    lhs_op_input_type_signed,
                                    a,
                                )
                            }
                        };
                        let rhs_op_input_type = match rhs_source_type {
                            ValueType::Bool => ValueType::U32,
                            _ => rhs_source_type,
                        };
                        let rhs = self.gen_cast(rhs, rhs_source_type, rhs_op_input_type, a);
                        let rhs = Expr::BinaryFunctionCall {
                            name: "std::cmp::min".into(),
                            lhs: Box::new(rhs),
                            rhs: Box::new(Expr::Constant {
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
                        };
                        let rhs = self.gen_cast(rhs, lhs_op_input_type, ValueType::U32, a);
                        let expr = Expr::UnaryMemberCall {
                            target: Box::new(lhs.clone()),
                            name: match op {
                                graph::ShiftBinOp::Shl => "checked_shl".into(),
                                graph::ShiftBinOp::Shr | graph::ShiftBinOp::ShrArithmetic => {
                                    "checked_shr".into()
                                }
                            },
                            arg: Box::new(rhs),
                        };
                        let expr = a.gen_temp(Expr::UnaryMemberCall {
                            target: Box::new(expr),
                            name: "unwrap_or".into(),
                            arg: Box::new(match op {
                                graph::ShiftBinOp::Shl | graph::ShiftBinOp::Shr => Expr::Constant {
                                    value: match lhs_op_input_type {
                                        ValueType::Bool
                                        | ValueType::I32
                                        | ValueType::I64
                                        | ValueType::I128 => unreachable!(),
                                        ValueType::U32 => Constant::U32(0),
                                        ValueType::U64 => Constant::U64(0),
                                        ValueType::U128 => Constant::U128(0),
                                    },
                                },
                                graph::ShiftBinOp::ShrArithmetic => Expr::InfixBinOp {
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(Expr::Constant {
                                        value: Constant::U32(lhs_op_input_type.bit_width() - 1),
                                    }),
                                    op: InfixBinOp::Shr,
                                },
                            }),
                        });
                        let op_output_type = lhs_op_input_type;
                        let expr = match op {
                            graph::ShiftBinOp::Shl | graph::ShiftBinOp::Shr => expr,
                            graph::ShiftBinOp::ShrArithmetic => {
                                let lhs_op_output_type_signed = op_output_type.to_signed();
                                self.gen_cast(expr, lhs_op_output_type_signed, op_output_type, a)
                            }
                        };
                        let target_bit_width = bit_width;
                        let target_type = ValueType::from_bit_width(target_bit_width);
                        let expr = self.gen_cast(expr, op_output_type, target_type, a);
                        Some(self.gen_mask(expr, target_bit_width, target_type, a))
                    }

                    graph::SignalData::Mul {
                        lhs,
                        rhs,
                        bit_width,
                    } => {
                        let lhs_type = ValueType::from_bit_width(lhs.bit_width());
                        let rhs_type = ValueType::from_bit_width(rhs.bit_width());
                        let lhs = results.pop().unwrap();
                        let rhs = results.pop().unwrap();
                        let target_type = ValueType::from_bit_width(bit_width);
                        let lhs = self.gen_cast(lhs, lhs_type, target_type, a);
                        let rhs = self.gen_cast(rhs, rhs_type, target_type, a);
                        Some(a.gen_temp(Expr::InfixBinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: InfixBinOp::Mul,
                        }))
                    }
                    graph::SignalData::MulSigned {
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
                        let lhs = self.gen_cast(lhs, lhs_type, target_type_signed, a);
                        let rhs = self.gen_cast(rhs, rhs_type, target_type_signed, a);
                        let lhs =
                            self.gen_sign_extend_shifts(lhs, lhs_bit_width, target_type_signed, a);
                        let rhs =
                            self.gen_sign_extend_shifts(rhs, rhs_bit_width, target_type_signed, a);
                        let expr = a.gen_temp(Expr::InfixBinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: InfixBinOp::Mul,
                        });
                        let expr = self.gen_cast(expr, target_type_signed, target_type, a);
                        Some(self.gen_mask(expr, target_bit_width, target_type, a))
                    }

                    graph::SignalData::Bits {
                        source, range_low, ..
                    } => {
                        let expr = results.pop().unwrap();
                        let expr = self.gen_shift_right(expr, range_low, a);
                        let target_bit_width = signal.bit_width();
                        let target_type = ValueType::from_bit_width(target_bit_width);
                        let expr = self.gen_cast(
                            expr,
                            ValueType::from_bit_width(source.bit_width()),
                            target_type,
                            a,
                        );
                        Some(self.gen_mask(expr, target_bit_width, target_type, a))
                    }

                    graph::SignalData::Repeat {
                        source,
                        count,
                        bit_width,
                    } => {
                        let expr = results.pop().unwrap();
                        let mut expr = self.gen_cast(
                            expr,
                            ValueType::from_bit_width(source.bit_width()),
                            ValueType::from_bit_width(bit_width),
                            a,
                        );

                        if count > 1 {
                            let source_expr = expr.clone();

                            for i in 1..count {
                                let rhs = self.gen_shift_left(
                                    source_expr.clone(),
                                    i * source.bit_width(),
                                    a,
                                );
                                expr = a.gen_temp(Expr::InfixBinOp {
                                    lhs: Box::new(expr),
                                    rhs: Box::new(rhs),
                                    op: InfixBinOp::BitOr,
                                });
                            }
                        }

                        Some(expr)
                    }
                    graph::SignalData::Concat {
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
                        let lhs = self.gen_cast(lhs, lhs_type, target_type, a);
                        let rhs = self.gen_cast(rhs, rhs_type, target_type, a);
                        let lhs = self.gen_shift_left(lhs, rhs_bit_width, a);
                        Some(a.gen_temp(Expr::InfixBinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: InfixBinOp::BitOr,
                        }))
                    }

                    graph::SignalData::Mux { .. } => {
                        let cond = results.pop().unwrap();
                        let when_true = results.pop().unwrap();
                        let when_false = results.pop().unwrap();
                        Some(a.gen_temp(Expr::Ternary {
                            cond: Box::new(cond),
                            when_true: Box::new(when_true),
                            when_false: Box::new(when_false),
                        }))
                    }

                    graph::SignalData::InstanceOutput { .. } => unreachable!(),

                    graph::SignalData::MemReadPortOutput { .. } => unreachable!(),
                },
            } {
                let key = (context, signal);
                self.signal_exprs.insert(key, expr.clone());
                results.push(expr);
            }
        }

        results.pop().unwrap()
    }

    fn gen_mask(
        &mut self,
        expr: Expr,
        bit_width: u32,
        target_type: ValueType,
        a: &mut AssignmentContext,
    ) -> Expr {
        if bit_width == target_type.bit_width() {
            return expr;
        }

        let mask = (1u128 << bit_width) - 1;
        a.gen_temp(Expr::InfixBinOp {
            lhs: Box::new(expr),
            rhs: Box::new(Expr::Constant {
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

    fn gen_shift_left(&mut self, expr: Expr, shift: u32, a: &mut AssignmentContext) -> Expr {
        if shift == 0 {
            return expr;
        }

        a.gen_temp(Expr::InfixBinOp {
            lhs: Box::new(expr),
            rhs: Box::new(Expr::Constant {
                value: Constant::U32(shift),
            }),
            op: InfixBinOp::Shl,
        })
    }

    fn gen_shift_right(&mut self, expr: Expr, shift: u32, a: &mut AssignmentContext) -> Expr {
        if shift == 0 {
            return expr;
        }

        a.gen_temp(Expr::InfixBinOp {
            lhs: Box::new(expr),
            rhs: Box::new(Expr::Constant {
                value: Constant::U32(shift),
            }),
            op: InfixBinOp::Shr,
        })
    }

    fn gen_cast(
        &mut self,
        expr: Expr,
        source_type: ValueType,
        target_type: ValueType,
        a: &mut AssignmentContext,
    ) -> Expr {
        if source_type == target_type {
            return expr;
        }

        if target_type == ValueType::Bool {
            let expr = self.gen_mask(expr, 1, source_type, a);
            return a.gen_temp(Expr::InfixBinOp {
                lhs: Box::new(expr),
                rhs: Box::new(Expr::Constant {
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

        a.gen_temp(Expr::Cast {
            source: Box::new(expr),
            target_type,
        })
    }

    fn gen_sign_extend_shifts(
        &mut self,
        expr: Expr,
        source_bit_width: u32,
        target_type: ValueType,
        a: &mut AssignmentContext,
    ) -> Expr {
        let shift = target_type.bit_width() - source_bit_width;
        let expr = self.gen_shift_left(expr, shift, a);
        self.gen_shift_right(expr, shift, a)
    }
}
