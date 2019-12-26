//! Verilog code generation.

use crate::code_writer;
use crate::module;

use std::io::Write;

pub fn generate<W: Write>(m: &module::Module, w: &mut W) -> Result<(), code_writer::Error> {
    let mut w = code_writer::CodeWriter::new(w);

    w.append_line(&format!("module {}(", m.name))?;
    w.indent();

    w.append_line("input wire logic reset_n,")?;
    w.append_indent()?;
    w.append("input wire logic clk")?;
    if m.inputs().len() > 0 || m.outputs().len() > 0 {
        w.append(",")?;
        w.append_newline()?;
    }
    w.append_newline()?;
    let inputs = m.inputs();
    let num_inputs = inputs.len();
    for (i, (name, source)) in inputs.iter().enumerate() {
        w.append_indent()?;
        w.append("input wire logic ")?;
        if source.bit_width() > 1 {
            w.append(&format!("[{}:{}] ", source.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if m.outputs().len() > 0 || i < num_inputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    let outputs = m.outputs();
    let num_outputs = outputs.len();
    for (i, (name, output)) in outputs.iter().enumerate() {
        w.append_indent()?;
        w.append("output wire logic ")?;
        if output.source.bit_width() > 1 {
            w.append(&format!("[{}:{}]", output.source.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if i < num_outputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    w.append_line(");")?;
    w.append_newline()?;

    w.append_line("// TODO")?;
    // TODO: Node decls
    // TODO: Assignments

    w.append_newline()?;
    w.unindent()?;
    w.append_line("endmodule")?;
    w.append_newline()?;

    Ok(())
}
