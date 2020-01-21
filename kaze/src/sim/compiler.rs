use super::ir::*;
use super::module_context::*;

use crate::graph;

use typed_arena::Arena;

use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CompiledRegister<'a> {
    pub data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub(crate) struct Compiler<'graph, 'arena> {
    context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,

    pub regs: HashMap<
        (
            *const ModuleContext<'graph, 'arena>,
            *const graph::Signal<'graph>,
        ),
        CompiledRegister<'graph>,
    >,
    signal_exprs: HashMap<
        (
            *const ModuleContext<'graph, 'arena>,
            *const graph::Signal<'graph>,
        ),
        Expr,
    >,

    pub prop_assignments: Vec<Assignment>,

    local_count: u32,
}

impl<'graph, 'arena> Compiler<'graph, 'arena> {
    pub fn new(
        context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,
    ) -> Compiler<'graph, 'arena> {
        Compiler {
            context_arena,

            regs: HashMap::new(),
            signal_exprs: HashMap::new(),

            prop_assignments: Vec::new(),

            local_count: 0,
        }
    }

    pub fn gather_regs(
        &mut self,
        signal: &'graph graph::Signal<'graph>,
        context: &'arena ModuleContext<'graph, 'arena>,
    ) {
        match signal.data {
            graph::SignalData::Lit { .. } => (),

            graph::SignalData::Input { ref name, .. } => {
                if let Some((instance, parent)) = context.instance_and_parent {
                    self.gather_regs(instance.driven_inputs.borrow()[name], parent);
                }
            }

            graph::SignalData::Reg { data } => {
                let key = (context as *const _, signal as *const _);
                if self.regs.contains_key(&key) {
                    return;
                }
                let value_name = format!("__reg_{}_{}", data.name, self.regs.len());
                let next_name = format!("{}_next", value_name);
                self.regs.insert(
                    key,
                    CompiledRegister {
                        data,
                        value_name,
                        next_name,
                    },
                );
                self.gather_regs(data.next.borrow().unwrap(), context);
            }

            graph::SignalData::UnOp { source, .. } => {
                self.gather_regs(source, context);
            }
            graph::SignalData::BinOp { lhs, rhs, .. } => {
                self.gather_regs(lhs, context);
                self.gather_regs(rhs, context);
            }

            graph::SignalData::Bits { source, .. } => {
                self.gather_regs(source, context);
            }

            graph::SignalData::Repeat { source, .. } => {
                self.gather_regs(source, context);
            }
            graph::SignalData::Concat { lhs, rhs } => {
                self.gather_regs(lhs, context);
                self.gather_regs(rhs, context);
            }

            graph::SignalData::Mux {
                cond,
                when_true,
                when_false,
            } => {
                self.gather_regs(cond, context);
                self.gather_regs(when_true, context);
                self.gather_regs(when_false, context);
            }

            graph::SignalData::InstanceOutput { instance, ref name } => {
                let output = instance.instantiated_module.outputs.borrow()[name];
                let context = context.get_child(instance, self.context_arena);
                self.gather_regs(output, context);
            }
        }
    }

    pub fn compile_signal(
        &mut self,
        signal: &'graph graph::Signal<'graph>,
        context: &'arena ModuleContext<'graph, 'arena>,
    ) -> Expr {
        let key = (context as *const _, signal as *const _);
        if !self.signal_exprs.contains_key(&key) {
            let expr = match signal.data {
                graph::SignalData::Lit {
                    ref value,
                    bit_width,
                } => {
                    let value = match value {
                        graph::Value::Bool(value) => *value as u128,
                        graph::Value::U32(value) => *value as u128,
                        graph::Value::U64(value) => *value as u128,
                        graph::Value::U128(value) => *value,
                    };

                    let target_type = ValueType::from_bit_width(bit_width);
                    Expr::Value {
                        value: match target_type {
                            ValueType::Bool => Value::Bool(value != 0),
                            ValueType::U32 => Value::U32(value as _),
                            ValueType::U64 => Value::U64(value as _),
                            ValueType::U128 => Value::U128(value),
                        },
                    }
                }

                graph::SignalData::Input {
                    ref name,
                    bit_width,
                } => {
                    if let Some((instance, parent)) = context.instance_and_parent {
                        self.compile_signal(instance.driven_inputs.borrow()[name], parent)
                    } else {
                        let target_type = ValueType::from_bit_width(bit_width);
                        let expr = Expr::Ref {
                            name: name.clone(),
                            scope: RefScope::Member,
                        };
                        self.gen_mask(expr, bit_width, target_type)
                    }
                }

                graph::SignalData::Reg { .. } => Expr::Ref {
                    name: self.regs[&key].value_name.clone(),
                    scope: RefScope::Member,
                },

                graph::SignalData::UnOp { source, op } => {
                    let expr = self.compile_signal(source, context);
                    let expr = self.gen_temp(Expr::UnOp {
                        source: Box::new(expr),
                        op: match op {
                            graph::UnOp::Not => UnOp::Not,
                        },
                    });

                    let bit_width = source.bit_width();
                    let target_type = ValueType::from_bit_width(bit_width);
                    self.gen_mask(expr, bit_width, target_type)
                }
                graph::SignalData::BinOp { lhs, rhs, op, .. } => {
                    let source_type = ValueType::from_bit_width(lhs.bit_width());
                    let lhs = self.compile_signal(lhs, context);
                    let rhs = self.compile_signal(rhs, context);
                    let op_input_type = match (op, source_type) {
                        (graph::BinOp::Add, ValueType::Bool) => ValueType::U32,
                        _ => source_type,
                    };
                    let lhs = self.gen_cast(lhs, source_type, op_input_type);
                    let rhs = self.gen_cast(rhs, source_type, op_input_type);
                    let expr = self.gen_temp(Expr::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op: match op {
                            graph::BinOp::Add => BinOp::Add,
                            graph::BinOp::BitAnd => BinOp::BitAnd,
                            graph::BinOp::BitOr => BinOp::BitOr,
                            graph::BinOp::BitXor => BinOp::BitXor,
                            graph::BinOp::Equal => BinOp::Equal,
                            graph::BinOp::NotEqual => BinOp::NotEqual,
                            graph::BinOp::LessThan => BinOp::LessThan,
                            graph::BinOp::LessThanEqual => BinOp::LessThanEqual,
                            graph::BinOp::GreaterThan => BinOp::GreaterThan,
                            graph::BinOp::GreaterThanEqual => BinOp::GreaterThanEqual,
                        },
                    });
                    let op_output_type = match op {
                        graph::BinOp::Equal
                        | graph::BinOp::NotEqual
                        | graph::BinOp::LessThan
                        | graph::BinOp::LessThanEqual
                        | graph::BinOp::GreaterThan
                        | graph::BinOp::GreaterThanEqual => ValueType::Bool,
                        _ => op_input_type,
                    };
                    let target_bit_width = signal.bit_width();
                    let target_type = ValueType::from_bit_width(target_bit_width);
                    let expr = self.gen_cast(expr, op_output_type, target_type);
                    self.gen_mask(expr, target_bit_width, target_type)
                }

                graph::SignalData::Bits {
                    source, range_low, ..
                } => {
                    let expr = self.compile_signal(source, context);
                    let expr = self.gen_shift_right(expr, range_low);
                    let target_bit_width = signal.bit_width();
                    let target_type = ValueType::from_bit_width(target_bit_width);
                    let expr = self.gen_cast(
                        expr,
                        ValueType::from_bit_width(source.bit_width()),
                        target_type,
                    );
                    self.gen_mask(expr, target_bit_width, target_type)
                }

                graph::SignalData::Repeat { source, count } => {
                    let expr = self.compile_signal(source, context);
                    let mut expr = self.gen_cast(
                        expr,
                        ValueType::from_bit_width(source.bit_width()),
                        ValueType::from_bit_width(signal.bit_width()),
                    );

                    if count > 1 {
                        let source_expr = expr.clone();

                        for i in 1..count {
                            let rhs =
                                self.gen_shift_left(source_expr.clone(), i * source.bit_width());
                            expr = self.gen_temp(Expr::BinOp {
                                lhs: Box::new(expr),
                                rhs: Box::new(rhs),
                                op: BinOp::BitOr,
                            });
                        }
                    }

                    expr
                }
                graph::SignalData::Concat { lhs, rhs } => {
                    let lhs_type = ValueType::from_bit_width(lhs.bit_width());
                    let rhs_bit_width = rhs.bit_width();
                    let rhs_type = ValueType::from_bit_width(rhs_bit_width);
                    let lhs = self.compile_signal(lhs, context);
                    let rhs = self.compile_signal(rhs, context);
                    let target_type = ValueType::from_bit_width(signal.bit_width());
                    let lhs = self.gen_cast(lhs, lhs_type, target_type);
                    let rhs = self.gen_cast(rhs, rhs_type, target_type);
                    let lhs = self.gen_shift_left(lhs, rhs_bit_width);
                    self.gen_temp(Expr::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op: BinOp::BitOr,
                    })
                }

                graph::SignalData::Mux {
                    cond,
                    when_true,
                    when_false,
                } => {
                    let cond = self.compile_signal(cond, context);
                    let when_true = self.compile_signal(when_true, context);
                    let when_false = self.compile_signal(when_false, context);
                    self.gen_temp(Expr::Ternary {
                        cond: Box::new(cond),
                        when_true: Box::new(when_true),
                        when_false: Box::new(when_false),
                    })
                }

                graph::SignalData::InstanceOutput { instance, ref name } => {
                    let output = instance.instantiated_module.outputs.borrow()[name];
                    self.compile_signal(output, context.get_child(instance, self.context_arena))
                }
            };
            self.signal_exprs.insert(key.clone(), expr);
        }

        self.signal_exprs[&key].clone()
    }

    fn gen_temp(&mut self, expr: Expr) -> Expr {
        let target_name = format!("__temp_{}", self.local_count);
        self.local_count += 1;
        self.prop_assignments.push(Assignment {
            target_scope: TargetScope::Local,
            target_name: target_name.clone(),
            expr,
        });

        Expr::Ref {
            scope: RefScope::Local,
            name: target_name,
        }
    }

    fn gen_mask(&mut self, expr: Expr, bit_width: u32, target_type: ValueType) -> Expr {
        if bit_width == target_type.bit_width() {
            return expr;
        }

        let mask = (1u128 << bit_width) - 1;
        self.gen_temp(Expr::BinOp {
            lhs: Box::new(expr),
            rhs: Box::new(Expr::Value {
                value: match target_type {
                    ValueType::Bool => unreachable!(),
                    ValueType::U32 => Value::U32(mask as _),
                    ValueType::U64 => Value::U64(mask as _),
                    ValueType::U128 => Value::U128(mask),
                },
            }),
            op: BinOp::BitAnd,
        })
    }

    fn gen_shift_left(&mut self, expr: Expr, shift: u32) -> Expr {
        if shift == 0 {
            return expr;
        }

        self.gen_temp(Expr::BinOp {
            lhs: Box::new(expr),
            rhs: Box::new(Expr::Value {
                value: Value::U32(shift),
            }),
            op: BinOp::Shl,
        })
    }

    fn gen_shift_right(&mut self, expr: Expr, shift: u32) -> Expr {
        if shift == 0 {
            return expr;
        }

        self.gen_temp(Expr::BinOp {
            lhs: Box::new(expr),
            rhs: Box::new(Expr::Value {
                value: Value::U32(shift),
            }),
            op: BinOp::Shr,
        })
    }

    fn gen_cast(&mut self, expr: Expr, source_type: ValueType, target_type: ValueType) -> Expr {
        if source_type == target_type {
            return expr;
        }

        if target_type == ValueType::Bool {
            let expr = self.gen_mask(expr, 1, source_type);
            return self.gen_temp(Expr::BinOp {
                lhs: Box::new(expr),
                rhs: Box::new(Expr::Value {
                    value: match source_type {
                        ValueType::Bool => unreachable!(),
                        ValueType::U32 => Value::U32(0),
                        ValueType::U64 => Value::U64(0),
                        ValueType::U128 => Value::U128(0),
                    },
                }),
                op: BinOp::NotEqual,
            });
        }

        self.gen_temp(Expr::Cast {
            source: Box::new(expr),
            target_type,
        })
    }
}
