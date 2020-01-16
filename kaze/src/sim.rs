//! Rust simulator code generation.

mod compiler;
mod ir;
mod stack;

use compiler::*;
use ir::*;

use crate::code_writer;
use crate::graph;

use std::io::{Result, Write};

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
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
        let reg = unsafe { &**reg as &graph::Signal };
        match reg.data {
            graph::SignalData::Reg { ref next, .. } => {
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
            let reg = unsafe { &**reg as &graph::Signal };
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
            let reg = unsafe { &**reg as &graph::Signal };
            match reg.data {
                graph::SignalData::Reg {
                    ref initial_value,
                    bit_width,
                    ..
                } => {
                    if let Some(ref initial_value) = *initial_value.borrow() {
                        w.append_indent()?;
                        w.append(&format!("self.{} = ", names.value_name))?;
                        let type_name = ValueType::from_bit_width(bit_width).name();
                        w.append(&match initial_value {
                            graph::Value::Bool(value) => {
                                if bit_width == 1 {
                                    format!("{}", value)
                                } else {
                                    format!("0x{:x}{}", if *value { 1 } else { 0 }, type_name)
                                }
                            }
                            graph::Value::U32(value) => format!("0x{:x}{}", value, type_name),
                            graph::Value::U64(value) => format!("0x{:x}{}", value, type_name),
                            graph::Value::U128(value) => format!("0x{:x}{}", value, type_name),
                        })?;
                        w.append(";")?;
                        w.append_newline()?;
                    }
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
        assignment.write(&mut w)?;
    }

    w.unindent()?;
    w.append_line("}")?;

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    Ok(())
}
