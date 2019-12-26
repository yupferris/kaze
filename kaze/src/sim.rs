//! Rust simulator code generation.

use crate::code_writer;
use crate::module;

use std::io::Write;

pub fn generate<W: Write>(m: &module::Module, w: &mut W) -> Result<(), code_writer::Error> {
    let mut w = code_writer::CodeWriter::new(w);

    w.append_line("#[allow(non_camel_case_types)]")?;
    w.append_line("#[derive(Default)]")?;
    w.append_line(&format!("pub struct {} {{", m.name))?;
    w.indent();

    let inputs = m.inputs();
    if inputs.len() > 0 {
        w.append_line("// Inputs")?;
        for (name, _) in inputs.iter() {
            // TODO: Properly determine output type
            w.append_line(&format!("pub {}: bool,", name))?;
        }
    }

    let outputs = m.outputs();
    if outputs.len() > 0 {
        w.append_line("// Outputs")?;
        for (name, _) in outputs.iter() {
            // TODO: Properly determine output type
            w.append_line(&format!("pub {}: bool,", name))?;
        }
    }

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    w.append_line(&format!("impl {} {{", m.name))?;
    w.indent();

    w.append_line("pub fn posedge_clk(&mut self) {")?;
    w.indent();

    // TODO

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    w.append_line("#[allow(unused_parens)]")?;
    w.append_line("pub fn prop(&mut self) {")?;
    w.indent();

    for (name, output) in m.outputs().iter() {
        w.append_indent()?;
        w.append(&format!("self.{} = ", name))?;
        gen_expr(&output.source, &mut w)?;
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

fn gen_expr<W: Write>(
    signal: &module::Signal,
    w: &mut code_writer::CodeWriter<W>,
) -> Result<(), code_writer::Error> {
    match signal.data {
        module::SignalData::Low => {
            w.append("false")?;
        }
        module::SignalData::High => {
            w.append("true")?;
        }

        module::SignalData::Input { ref name, .. } => {
            w.append(&format!("self.{}", name))?;
        }

        module::SignalData::UnOp { source, op } => {
            w.append(match op {
                module::UnOp::Not => "!",
            })?;
            gen_expr(source, w)?;
        }
        module::SignalData::BinOp { lhs, rhs, op } => {
            w.append("(")?;
            gen_expr(lhs, w)?;
            w.append(&format!(
                " {} ",
                match op {
                    module::BinOp::BitAnd => '&',
                    module::BinOp::BitOr => '|',
                }
            ))?;
            gen_expr(rhs, w)?;
            w.append(")")?;
        }

        module::SignalData::Mux { a, b, sel } => {
            w.append("if ")?;
            gen_expr(sel, w)?;
            w.append(" { ")?;
            gen_expr(b, w)?;
            w.append(" } else { ")?;
            gen_expr(a, w)?;
            w.append(" }")?;
        }
    }
    Ok(())
}
