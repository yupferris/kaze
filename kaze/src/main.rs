mod code_writer;
mod module;

mod sim {
    use crate::code_writer;
    use crate::module;

    use std::io::Write;

    pub fn generate<W: Write>(m: &module::Module, w: W) -> Result<(), code_writer::Error> {
        let mut w = code_writer::CodeWriter::new(w);

        w.append_line(&format!("struct {} {{", m.name))?;
        w.indent();

        let inputs = m.inputs();
        if inputs.len() > 0 {
            w.append_line("// Inputs")?;
            for (name, _) in inputs.iter() {
                // TODO: Properly determine output type
                w.append_line(&format!("bool {},", name))?;
            }
        }

        let outputs = m.outputs();
        if outputs.len() > 0 {
            w.append_line("// Outputs")?;
            for (name, _) in outputs.iter() {
                // TODO: Properly determine output type
                w.append_line(&format!("bool {},", name))?;
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

    fn gen_expr<W: Write>(signal: &module::Signal, w: &mut code_writer::CodeWriter<W>) -> Result<(), code_writer::Error> {
        match signal.data {
            module::SignalData::Low => {
                w.append("false")?;
            }
            module::SignalData::High => {
                w.append("true")?;
            }

            module::SignalData::Input { ref name, .. } => {
                w.append(&name)?;
            }

            module::SignalData::BinOp { lhs, rhs, op } => {
                w.append("(")?;
                gen_expr(lhs, w)?;
                w.append(&format!(" {} ", match op {
                    module::BinOp::BitOr => '|',
                }))?;
                gen_expr(rhs, w)?;
                w.append(")")?;
            }
        }
        Ok(())
    }
}

mod verilog {
    use crate::code_writer;
    use crate::module;

    use std::io::Write;

    pub fn generate<W: Write>(m: &module::Module, w: W) -> Result<(), code_writer::Error> {
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
}

use module::*;

use std::io::stdout;

#[derive(Debug)]
enum Error {
    CodeWriter(code_writer::Error),
}

impl From<code_writer::Error> for Error {
    fn from(error: code_writer::Error) -> Error {
        Error::CodeWriter(error)
    }
}

fn main() -> Result<(), Error> {
    let c = Context::new();

    let m = test_mod(&c);

    println!("// Sim");
    sim::generate(&m, stdout())?;

    println!("// Verilog");
    verilog::generate(&m, stdout())?;

    Ok(())
}

fn test_mod<'a>(c: &'a Context<'a>) -> &'a Module<'a> {
    let m = c.module("test_module");

    m.output("cya", m.input("henlo", 1) | m.low() | m.low() | m.input("hiiii", 1));

    m
}
