//! Rust simulator code generation.

use crate::code_writer;
use crate::module;

use std::collections::HashMap;
use std::io::Write;

struct RegNames {
    value_name: String,
    next_name: String,
}

struct Context<'a> {
    reg_names: HashMap<*const module::Signal<'a>, RegNames>,
}

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
    let mut c = Context {
        reg_names: HashMap::new(),
    };

    for (_, output) in m.outputs().iter() {
        gather_regs(&output.source, &mut c);
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
                    gen_value(initial_value, bit_width, &mut w)?;
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

    for (name, output) in m.outputs().iter() {
        w.append_indent()?;
        w.append(&format!("self.{} = ", name))?;
        gen_expr(&output.source, &c, &mut w)?;
        w.append(";")?;
        w.append_newline()?;
    }

    for (reg, names) in c.reg_names.iter() {
        let reg = unsafe { &**reg as &module::Signal };
        match reg.data {
            module::SignalData::Reg { ref next, .. } => {
                w.append_indent()?;
                w.append(&format!("self.{} = ", names.next_name))?;
                gen_expr(next.borrow().unwrap(), &c, &mut w)?;
                w.append(";")?;
                w.append_newline()?;
            }
            _ => unreachable!(),
        }
    }

    w.unindent()?;
    w.append_line("}")?;

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    Ok(())
}

fn gather_regs<'a>(signal: &'a module::Signal<'a>, c: &mut Context<'a>) {
    match signal.data {
        module::SignalData::Lit { .. } | module::SignalData::Input { .. } => (),

        module::SignalData::Reg { ref next, .. } => {
            if c.reg_names.contains_key(&(signal as *const _)) {
                return;
            }
            let value_name = format!("__reg{}", c.reg_names.len());
            let next_name = format!("{}_next", value_name);
            c.reg_names.insert(
                signal,
                RegNames {
                    value_name,
                    next_name,
                },
            );
            gather_regs(next.borrow().expect("Discovered undriven register(s)"), c);
            // TODO: Proper error and test(s)
        }

        module::SignalData::UnOp { source, .. } => {
            gather_regs(source, c);
        }
        module::SignalData::BinOp { lhs, rhs, .. } => {
            gather_regs(lhs, c);
            gather_regs(rhs, c);
        }

        module::SignalData::Bit { source, .. } => {
            gather_regs(source, c);
        }

        module::SignalData::Mux { a, b, sel } => {
            gather_regs(sel, c);
            gather_regs(b, c);
            gather_regs(a, c);
        }
    }
}

fn gen_expr<'a, W: Write>(
    signal: &'a module::Signal<'a>,
    c: &Context<'a>,
    w: &mut code_writer::CodeWriter<W>,
) -> Result<(), code_writer::Error> {
    match signal.data {
        module::SignalData::Lit {
            ref value,
            bit_width,
        } => {
            gen_value(value, bit_width, w)?;
        }

        module::SignalData::Input {
            ref name,
            bit_width,
        } => {
            let target_type = ValueType::from_bit_width(bit_width);
            if target_type.bit_width() > 1 && target_type.bit_width() != bit_width {
                w.append("(")?;
            }
            w.append(&format!("self.{}", name))?;
            if target_type.bit_width() > 1 && target_type.bit_width() != bit_width {
                w.append(&format!(" & 0x{:x}", (1u128 << bit_width) - 1))?;
                w.append(")")?;
            }
        }

        module::SignalData::Reg { .. } => {
            w.append(&format!(
                "self.{}",
                c.reg_names[&(signal as *const _)].value_name
            ))?;
        }

        module::SignalData::UnOp { source, op } => {
            let bit_width = source.bit_width();
            let target_type = ValueType::from_bit_width(bit_width);
            if target_type.bit_width() > 1 && target_type.bit_width() != bit_width {
                w.append("(")?;
            }
            w.append(match op {
                module::UnOp::Not => "!",
            })?;
            gen_expr(source, c, w)?;
            if target_type.bit_width() > 1 && target_type.bit_width() != bit_width {
                w.append(&format!(" & 0x{:x}", (1u128 << bit_width) - 1))?;
                w.append(")")?;
            }
        }
        module::SignalData::BinOp { lhs, rhs, op } => {
            w.append("(")?;
            gen_expr(lhs, c, w)?;
            w.append(&format!(
                " {} ",
                match op {
                    module::BinOp::BitAnd => '&',
                    module::BinOp::BitOr => '|',
                }
            ))?;
            gen_expr(rhs, c, w)?;
            w.append(")")?;
        }

        module::SignalData::Bit { source, index } => {
            let bit_width = source.bit_width();
            let source_type = ValueType::from_bit_width(bit_width);
            if source_type.bit_width() > 1 {
                w.append("((")?;
            }
            gen_expr(source, c, w)?;
            if source_type.bit_width() > 1 {
                w.append(&format!(" >> {}) & 1) == 1", index))?;
            }
        }

        module::SignalData::Mux { a, b, sel } => {
            w.append("if ")?;
            gen_expr(sel, c, w)?;
            w.append(" { ")?;
            gen_expr(b, c, w)?;
            w.append(" } else { ")?;
            gen_expr(a, c, w)?;
            w.append(" }")?;
        }
    }

    Ok(())
}

fn gen_value<W: Write>(
    value: &module::Value,
    bit_width: u32,
    w: &mut code_writer::CodeWriter<W>,
) -> Result<(), code_writer::Error> {
    let type_name = ValueType::from_bit_width(bit_width).name();
    w.append(&match value {
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
    })
}
