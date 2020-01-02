//! Rust simulator code generation.

use crate::code_writer;
use crate::module;

use std::collections::HashMap;
use std::io::Write;

#[derive(Clone)]
struct RegNames {
    value_name: String,
    next_name: String,
}

struct Compiler<'a> {
    reg_names: HashMap<*const module::Signal<'a>, RegNames>,

    prop_assignments: Vec<Assignment>,

    local_count: u32,

    signal_exprs: HashMap<*const module::Signal<'a>, Expr>,
}

impl<'a> Compiler<'a> {
    fn new() -> Compiler<'a> {
        Compiler {
            reg_names: HashMap::new(),

            prop_assignments: Vec::new(),

            local_count: 0,

            signal_exprs: HashMap::new(),
        }
    }

    fn gather_regs(&mut self, signal: &'a module::Signal<'a>) {
        match signal.data {
            module::SignalData::Lit { .. } | module::SignalData::Input { .. } => (),

            module::SignalData::Reg { ref next, .. } => {
                if self.reg_names.contains_key(&(signal as *const _)) {
                    return;
                }
                let value_name = format!("__reg{}", self.reg_names.len());
                let next_name = format!("{}_next", value_name);
                self.reg_names.insert(
                    signal,
                    RegNames {
                        value_name,
                        next_name,
                    },
                );
                // TODO: Proper error and test(s)
                self.gather_regs(next.borrow().expect("Discovered undriven register(s)"));
            }

            module::SignalData::UnOp { source, .. } => {
                self.gather_regs(source);
            }
            module::SignalData::BinOp { lhs, rhs, .. } => {
                self.gather_regs(lhs);
                self.gather_regs(rhs);
            }

            module::SignalData::Bit { source, .. } => {
                self.gather_regs(source);
            }
            module::SignalData::Bits { source, .. } => {
                self.gather_regs(source);
            }

            module::SignalData::Repeat { source, .. } => {
                self.gather_regs(source);
            }
            module::SignalData::Concat { lhs, rhs } => {
                self.gather_regs(lhs);
                self.gather_regs(rhs);
            }

            module::SignalData::Mux { a, b, sel } => {
                self.gather_regs(sel);
                self.gather_regs(b);
                self.gather_regs(a);
            }
        }
    }

    fn compile_signal(&mut self, signal: &'a module::Signal<'a>) -> Expr {
        if !self.signal_exprs.contains_key(&(signal as *const _)) {
            let expr = match signal.data {
                module::SignalData::Lit {
                    ref value,
                    bit_width,
                } => {
                    let value = match value {
                        module::Value::Bool(value) => *value as u128,
                        module::Value::U8(value) => *value as u128,
                        module::Value::U16(value) => *value as u128,
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
                    let target_type = ValueType::from_bit_width(bit_width);
                    let expr = Expr::Ref {
                        name: name.clone(),
                        scope: RefScope::Member,
                    };
                    self.gen_mask(expr, bit_width, target_type)
                }

                module::SignalData::Reg { .. } => Expr::Ref {
                    name: self.reg_names[&(signal as *const _)].value_name.clone(),
                    scope: RefScope::Member,
                },

                module::SignalData::UnOp { source, op } => {
                    let expr = self.compile_signal(source);
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
                    let lhs = self.compile_signal(lhs);
                    let rhs = self.compile_signal(rhs);
                    self.gen_temp(Expr::BinOp {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op: match op {
                            module::BinOp::BitAnd => BinOp::BitAnd,
                            module::BinOp::BitOr => BinOp::BitOr,
                            module::BinOp::BitXor => BinOp::BitXor,
                            module::BinOp::Equal => BinOp::Equal,
                            module::BinOp::NotEqual => BinOp::NotEqual,
                            module::BinOp::LessThan => BinOp::LessThan,
                        },
                    })
                }

                module::SignalData::Bit { source, index } => {
                    let expr = self.compile_signal(source);
                    let expr = self.gen_shift_right(expr, index);
                    self.gen_cast(
                        expr,
                        ValueType::from_bit_width(source.bit_width()),
                        ValueType::Bool,
                    )
                }
                module::SignalData::Bits {
                    source, range_low, ..
                } => {
                    let expr = self.compile_signal(source);
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
                    let expr = self.compile_signal(source);
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
                    let lhs = self.compile_signal(lhs);
                    let rhs = self.compile_signal(rhs);
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
                    let lhs = self.compile_signal(b);
                    let rhs = self.compile_signal(a);
                    let cond = self.compile_signal(sel);
                    self.gen_temp(Expr::Ternary {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        cond: Box::new(cond),
                    })
                }
            };
            self.signal_exprs.insert(signal, expr);
        }

        self.signal_exprs[&(signal as *const _)].clone()
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
    fn write<W: Write>(
        &self,
        w: &mut code_writer::CodeWriter<W>,
    ) -> Result<(), code_writer::Error> {
        match self {
            Expr::BinOp { lhs, rhs, op } => {
                lhs.write(w)?;
                w.append(&format!(
                    " {} ",
                    match op {
                        BinOp::BitAnd => "&",
                        BinOp::BitOr => "|",
                        BinOp::BitXor => "^",
                        BinOp::Equal => "==",
                        BinOp::NotEqual => "!=",
                        BinOp::LessThan => "<",
                        BinOp::Shl => "<<",
                        BinOp::Shr => ">>",
                    }
                ))?;
                rhs.write(w)?;
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
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    LessThan,
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

pub fn generate<W: Write>(m: &module::Module, w: &mut W) -> Result<(), code_writer::Error> {
    let mut c = Compiler::new();

    for (_, output) in m.outputs().iter() {
        c.gather_regs(&output.source);
    }

    for (name, output) in m.outputs().iter() {
        let expr = c.compile_signal(&output.source);
        c.prop_assignments.push(Assignment {
            target_scope: TargetScope::Member,
            target_name: name.clone(),
            expr,
        });
    }

    // TODO: Can we get rid of this clone?
    for (reg, names) in c.reg_names.clone().iter() {
        let reg = unsafe { &**reg as &module::Signal };
        match reg.data {
            module::SignalData::Reg { ref next, .. } => {
                let expr = c.compile_signal(next.borrow().unwrap());
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

    let inputs = m.inputs();
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

    let outputs = m.outputs();
    if outputs.len() > 0 {
        w.append_line("// Outputs")?;
        for (name, output) in outputs.iter() {
            w.append_line(&format!(
                "pub {}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(output.source.bit_width()).name(),
                output.source.bit_width()
            ))?;
        }
    }

    if c.reg_names.len() > 0 {
        w.append_line("// Regs")?;
        for (reg, names) in c.reg_names.iter() {
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
        for (reg, names) in c.reg_names.iter() {
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
                        module::Value::U8(value) => format!("0x{:x}{}", value, type_name),
                        module::Value::U16(value) => format!("0x{:x}{}", value, type_name),
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

    w.append_line("#[allow(unused_parens)]")?;
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
