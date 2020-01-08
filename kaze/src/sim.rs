//! Rust simulator code generation.

use crate::code_writer;
use crate::module;

use std::collections::HashMap;
use std::io::{Result, Write};
use std::rc::Rc;

#[derive(Clone, Eq, Hash, PartialEq)]
struct Stack<T: Clone + Eq + PartialEq> {
    head: Option<Rc<StackNode<T>>>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct StackNode<T: Clone + Eq + PartialEq> {
    datum: T,
    next: Option<Rc<StackNode<T>>>,
}

impl<T: Clone + Eq + PartialEq> Stack<T> {
    fn new() -> Stack<T> {
        Stack { head: None }
    }

    fn push(&self, datum: T) -> Stack<T> {
        Stack {
            head: Some(Rc::new(StackNode {
                datum,
                next: self.head.clone(),
            })),
        }
    }

    fn pop(&self) -> Option<(T, Stack<T>)> {
        self.head.as_ref().map(|node| {
            (
                node.datum.clone(),
                Stack {
                    head: node.next.clone(),
                },
            )
        })
    }
}

type InstanceStack<'a> = Stack<*const module::Instance<'a>>;

#[derive(Clone)]
struct RegNames {
    value_name: String,
    next_name: String,
}

struct Compiler<'a> {
    reg_names: HashMap<(InstanceStack<'a>, *const module::Signal<'a>), RegNames>,
    signal_exprs: HashMap<(InstanceStack<'a>, *const module::Signal<'a>), Expr>,

    prop_assignments: Vec<Assignment>,

    local_count: u32,
}

impl<'a> Compiler<'a> {
    fn new() -> Compiler<'a> {
        Compiler {
            reg_names: HashMap::new(),
            signal_exprs: HashMap::new(),

            prop_assignments: Vec::new(),

            local_count: 0,
        }
    }

    fn gather_regs(&mut self, signal: &'a module::Signal<'a>, instance_stack: &InstanceStack<'a>) {
        match signal.data {
            module::SignalData::Lit { .. } => (),

            module::SignalData::Input { ref name, .. } => {
                if let Some((instance, instance_stack_tail)) = instance_stack.pop() {
                    let instance = unsafe { &*instance };
                    // TODO: Report error if input isn't driven
                    //  Should we report errors for all undriven inputs here?
                    self.gather_regs(instance.driven_inputs.borrow()[name], &instance_stack_tail);
                }
            }

            module::SignalData::Reg { ref next, .. } => {
                let key = (instance_stack.clone(), signal as *const _);
                if self.reg_names.contains_key(&key) {
                    return;
                }
                let value_name = format!("__reg{}", self.reg_names.len());
                let next_name = format!("{}_next", value_name);
                self.reg_names.insert(
                    key,
                    RegNames {
                        value_name,
                        next_name,
                    },
                );
                // TODO: Proper error and test(s)
                self.gather_regs(
                    next.borrow().expect("Discovered undriven register(s)"),
                    instance_stack,
                );
            }

            module::SignalData::UnOp { source, .. } => {
                self.gather_regs(source, instance_stack);
            }
            module::SignalData::BinOp { lhs, rhs, .. } => {
                self.gather_regs(lhs, instance_stack);
                self.gather_regs(rhs, instance_stack);
            }

            module::SignalData::Bits { source, .. } => {
                self.gather_regs(source, instance_stack);
            }

            module::SignalData::Repeat { source, .. } => {
                self.gather_regs(source, instance_stack);
            }
            module::SignalData::Concat { lhs, rhs } => {
                self.gather_regs(lhs, instance_stack);
                self.gather_regs(rhs, instance_stack);
            }

            module::SignalData::Mux { a, b, sel } => {
                self.gather_regs(sel, instance_stack);
                self.gather_regs(b, instance_stack);
                self.gather_regs(a, instance_stack);
            }

            module::SignalData::InstanceOutput { instance, ref name } => {
                let output = instance.instantiated_module.outputs.borrow()[name];
                self.gather_regs(output, &instance_stack.push(instance));
            }
        }
    }

    fn compile_signal(
        &mut self,
        signal: &'a module::Signal<'a>,
        instance_stack: &InstanceStack<'a>,
    ) -> Expr {
        let key = (instance_stack.clone(), signal as *const _);
        if !self.signal_exprs.contains_key(&key) {
            let expr = match signal.data {
                module::SignalData::Lit {
                    ref value,
                    bit_width,
                } => {
                    let value = match value {
                        module::Value::Bool(value) => *value as u128,
                        module::Value::U32(value) => *value as u128,
                        module::Value::U64(value) => *value as u128,
                        module::Value::U128(value) => *value,
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

                module::SignalData::Input {
                    ref name,
                    bit_width,
                } => {
                    if let Some((instance, instance_stack_tail)) = instance_stack.pop() {
                        let instance = unsafe { &*instance };
                        self.compile_signal(
                            instance.driven_inputs.borrow()[name],
                            &instance_stack_tail,
                        )
                    } else {
                        let target_type = ValueType::from_bit_width(bit_width);
                        let expr = Expr::Ref {
                            name: name.clone(),
                            scope: RefScope::Member,
                        };
                        self.gen_mask(expr, bit_width, target_type)
                    }
                }

                module::SignalData::Reg { .. } => Expr::Ref {
                    name: self.reg_names[&key].value_name.clone(),
                    scope: RefScope::Member,
                },

                module::SignalData::UnOp { source, op } => {
                    let expr = self.compile_signal(source, instance_stack);
                    let expr = self.gen_temp(Expr::UnOp {
                        source: Box::new(expr),
                        op: match op {
                            module::UnOp::Not => UnOp::Not,
                        },
                    });

                    let bit_width = source.bit_width();
                    let target_type = ValueType::from_bit_width(bit_width);
                    self.gen_mask(expr, bit_width, target_type)
                }
                module::SignalData::BinOp { lhs, rhs, op, .. } => {
                    let source_type = ValueType::from_bit_width(lhs.bit_width());
                    let lhs = self.compile_signal(lhs, instance_stack);
                    let rhs = self.compile_signal(rhs, instance_stack);
                    let op_input_type = match (op, source_type) {
                        (module::BinOp::Add, ValueType::Bool) => ValueType::U32,
                        _ => source_type,
                    };
                    let lhs = self.gen_cast(lhs, source_type, op_input_type);
                    let rhs = self.gen_cast(rhs, source_type, op_input_type);
                    let expr = self.gen_temp(Expr::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op: match op {
                            module::BinOp::Add => BinOp::Add,
                            module::BinOp::BitAnd => BinOp::BitAnd,
                            module::BinOp::BitOr => BinOp::BitOr,
                            module::BinOp::BitXor => BinOp::BitXor,
                            module::BinOp::Equal => BinOp::Equal,
                            module::BinOp::NotEqual => BinOp::NotEqual,
                            module::BinOp::LessThan => BinOp::LessThan,
                            module::BinOp::LessThanEqual => BinOp::LessThanEqual,
                            module::BinOp::GreaterThan => BinOp::GreaterThan,
                            module::BinOp::GreaterThanEqual => BinOp::GreaterThanEqual,
                        },
                    });
                    let op_output_type = match op {
                        module::BinOp::Equal
                        | module::BinOp::NotEqual
                        | module::BinOp::LessThan
                        | module::BinOp::LessThanEqual
                        | module::BinOp::GreaterThan
                        | module::BinOp::GreaterThanEqual => ValueType::Bool,
                        _ => op_input_type,
                    };
                    let target_bit_width = signal.bit_width();
                    let target_type = ValueType::from_bit_width(target_bit_width);
                    let expr = self.gen_cast(expr, op_output_type, target_type);
                    self.gen_mask(expr, target_bit_width, target_type)
                }

                module::SignalData::Bits {
                    source, range_low, ..
                } => {
                    let expr = self.compile_signal(source, instance_stack);
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

                module::SignalData::Repeat { source, count } => {
                    let expr = self.compile_signal(source, instance_stack);
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
                module::SignalData::Concat { lhs, rhs } => {
                    let lhs_type = ValueType::from_bit_width(lhs.bit_width());
                    let rhs_bit_width = rhs.bit_width();
                    let rhs_type = ValueType::from_bit_width(rhs_bit_width);
                    let lhs = self.compile_signal(lhs, instance_stack);
                    let rhs = self.compile_signal(rhs, instance_stack);
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

                module::SignalData::Mux { a, b, sel } => {
                    let lhs = self.compile_signal(b, instance_stack);
                    let rhs = self.compile_signal(a, instance_stack);
                    let cond = self.compile_signal(sel, instance_stack);
                    self.gen_temp(Expr::Ternary {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        cond: Box::new(cond),
                    })
                }

                module::SignalData::InstanceOutput { instance, ref name } => {
                    let output = instance.instantiated_module.outputs.borrow()[name];
                    self.compile_signal(output, &instance_stack.push(instance))
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

struct Assignment {
    target_scope: TargetScope,
    target_name: String,
    expr: Expr,
}

enum TargetScope {
    Local,
    Member,
}

#[derive(Clone)]
enum Expr {
    BinOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinOp,
    },
    Cast {
        source: Box<Expr>,
        target_type: ValueType,
    },
    Ref {
        name: String,
        scope: RefScope,
    },
    Ternary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        cond: Box<Expr>,
    },
    UnOp {
        source: Box<Expr>,
        op: UnOp,
    },
    Value {
        value: Value,
    },
}

impl Expr {
    fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        match self {
            Expr::BinOp { lhs, rhs, op } => {
                lhs.write(w)?;
                match op {
                    BinOp::Add => {
                        w.append(".wrapping_add(")?;
                        rhs.write(w)?;
                        w.append(")")?;
                    }
                    _ => {
                        w.append(&format!(
                            " {} ",
                            match op {
                                BinOp::Add => unreachable!(),
                                BinOp::BitAnd => "&",
                                BinOp::BitOr => "|",
                                BinOp::BitXor => "^",
                                BinOp::Equal => "==",
                                BinOp::NotEqual => "!=",
                                BinOp::LessThan => "<",
                                BinOp::LessThanEqual => "<=",
                                BinOp::GreaterThan => ">",
                                BinOp::GreaterThanEqual => ">=",
                                BinOp::Shl => "<<",
                                BinOp::Shr => ">>",
                            }
                        ))?;
                        rhs.write(w)?;
                    }
                }
            }
            Expr::Cast {
                source,
                target_type,
            } => {
                source.write(w)?;
                w.append(&format!(" as {}", target_type.name()))?;
            }
            Expr::Ref { name, scope } => {
                if let RefScope::Member = scope {
                    w.append("self.")?;
                }
                w.append(name)?;
            }
            Expr::Ternary { lhs, rhs, cond } => {
                w.append("if ")?;
                cond.write(w)?;
                w.append(" { ")?;
                lhs.write(w)?;
                w.append(" } else { ")?;
                rhs.write(w)?;
                w.append(" }")?;
            }
            Expr::UnOp { source, op } => {
                w.append(match op {
                    UnOp::Not => "!",
                })?;
                source.write(w)?;
            }
            Expr::Value { value } => {
                w.append(&match value {
                    Value::Bool(value) => format!("{}", value),
                    Value::U32(value) => format!("0x{:x}u32", value),
                    Value::U64(value) => format!("0x{:x}u64", value),
                    Value::U128(value) => format!("0x{:x}u128", value),
                })?;
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
enum UnOp {
    Not,
}

#[derive(Clone)]
enum BinOp {
    Add,
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Shl,
    Shr,
}

#[derive(Clone)]
enum RefScope {
    Local,
    Member,
}

#[derive(Clone)]
enum Value {
    Bool(bool),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[derive(Clone, Copy, PartialEq)]
enum ValueType {
    Bool,
    U32,
    U64,
    U128,
}

impl ValueType {
    fn from_bit_width(bit_width: u32) -> ValueType {
        if bit_width == 1 {
            ValueType::Bool
        } else if bit_width <= 32 {
            ValueType::U32
        } else if bit_width <= 64 {
            ValueType::U64
        } else if bit_width <= 128 {
            ValueType::U128
        } else {
            unreachable!()
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ValueType::Bool => "bool",
            ValueType::U32 => "u32",
            ValueType::U64 => "u64",
            ValueType::U128 => "u128",
        }
    }

    fn bit_width(&self) -> u32 {
        match self {
            ValueType::Bool => 1,
            ValueType::U32 => 32,
            ValueType::U64 => 64,
            ValueType::U128 => 128,
        }
    }
}

pub fn generate<'a, W: Write>(m: &'a module::Module<'a>, w: &mut W) -> Result<()> {
    let mut c = Compiler::new();

    for (_, output) in m.outputs.borrow().iter() {
        c.gather_regs(&output, &InstanceStack::new());
    }

    for (name, output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(&output, &InstanceStack::new());
        c.prop_assignments.push(Assignment {
            target_scope: TargetScope::Member,
            target_name: name.clone(),
            expr,
        });
    }

    // TODO: Can we get rid of this clone?
    for ((instance_stack, reg), names) in c.reg_names.clone().iter() {
        let reg = unsafe { &**reg as &module::Signal };
        match reg.data {
            module::SignalData::Reg { ref next, .. } => {
                let expr = c.compile_signal(next.borrow().unwrap(), instance_stack);
                c.prop_assignments.push(Assignment {
                    target_scope: TargetScope::Member,
                    target_name: names.next_name.clone(),
                    expr,
                });
            }
            _ => unreachable!(),
        }
    }

    let mut w = code_writer::CodeWriter::new(w);

    w.append_line("#[allow(non_camel_case_types)]")?;
    w.append_line("#[derive(Default)]")?;
    w.append_line(&format!("pub struct {} {{", m.name))?;
    w.indent();

    let inputs = m.inputs.borrow();
    if inputs.len() > 0 {
        w.append_line("// Inputs")?;
        for (name, input) in inputs.iter() {
            w.append_line(&format!(
                "pub {}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(input.bit_width()).name(),
                input.bit_width()
            ))?;
        }
    }

    let outputs = m.outputs.borrow();
    if outputs.len() > 0 {
        w.append_line("// Outputs")?;
        for (name, output) in outputs.iter() {
            w.append_line(&format!(
                "pub {}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(output.bit_width()).name(),
                output.bit_width()
            ))?;
        }
    }

    if c.reg_names.len() > 0 {
        w.append_line("// Regs")?;
        for ((_, reg), names) in c.reg_names.iter() {
            let reg = unsafe { &**reg as &module::Signal };
            let type_name = ValueType::from_bit_width(reg.bit_width()).name();
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                names.value_name,
                type_name,
                reg.bit_width()
            ))?;
            w.append_line(&format!("{}: {},", names.next_name, type_name))?;
        }
    }

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    w.append_line(&format!("impl {} {{", m.name))?;
    w.indent();

    if c.reg_names.len() > 0 {
        w.append_line("pub fn reset(&mut self) {")?;
        w.indent();

        // TODO: Consider using assignments/exprs instead of generating statement strings
        for ((_, reg), names) in c.reg_names.iter() {
            let reg = unsafe { &**reg as &module::Signal };
            match reg.data {
                module::SignalData::Reg {
                    ref initial_value,
                    bit_width,
                    ..
                } => {
                    w.append_indent()?;
                    w.append(&format!("self.{} = ", names.value_name))?;
                    let type_name = ValueType::from_bit_width(bit_width).name();
                    w.append(&match initial_value {
                        module::Value::Bool(value) => {
                            if bit_width == 1 {
                                format!("{}", value)
                            } else {
                                format!("0x{:x}{}", if *value { 1 } else { 0 }, type_name)
                            }
                        }
                        module::Value::U32(value) => format!("0x{:x}{}", value, type_name),
                        module::Value::U64(value) => format!("0x{:x}{}", value, type_name),
                        module::Value::U128(value) => format!("0x{:x}{}", value, type_name),
                    })?;
                    w.append(";")?;
                    w.append_newline()?;
                }
                _ => unreachable!(),
            }
        }

        w.unindent()?;
        w.append_line("}")?;
        w.append_newline()?;

        w.append_line("pub fn posedge_clk(&mut self) {")?;
        w.indent();

        for names in c.reg_names.values() {
            w.append_line(&format!(
                "self.{} = self.{};",
                names.value_name, names.next_name
            ))?;
        }

        w.unindent()?;
        w.append_line("}")?;
        w.append_newline()?;
    }

    w.append_line("pub fn prop(&mut self) {")?;
    w.indent();

    for assignment in c.prop_assignments.iter() {
        w.append_indent()?;
        match assignment.target_scope {
            TargetScope::Local => {
                w.append("let ")?;
            }
            TargetScope::Member => {
                w.append("self.")?;
            }
        }
        w.append(&format!("{} = ", assignment.target_name))?;
        assignment.expr.write(&mut w)?;
        w.append(";")?;
        w.append_newline()?;
    }

    w.unindent()?;
    w.append_line("}")?;

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    Ok(())
}
