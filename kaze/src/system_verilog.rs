//! SystemVerilog code generation.

use crate::code_writer;
use crate::graph;
use crate::validation::*;

use std::io::{Result, Write};

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
    validate_module_hierarchy(m);

    let mut w = code_writer::CodeWriter::new(w);

    w.append_line(&format!("module {}(", m.name))?;
    w.indent();

    w.append_line("input wire logic reset_n,")?;
    w.append_indent()?;
    w.append("input wire logic clk")?;
    if m.inputs.borrow().len() > 0 || m.outputs.borrow().len() > 0 {
        w.append(",")?;
        w.append_newline()?;
    }
    w.append_newline()?;
    let inputs = m.inputs.borrow();
    let num_inputs = inputs.len();
    for (i, (name, source)) in inputs.iter().enumerate() {
        w.append_indent()?;
        w.append("input wire logic ")?;
        if source.bit_width() > 1 {
            w.append(&format!("[{}:{}] ", source.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if m.outputs.borrow().len() > 0 || i < num_inputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    let outputs = m.outputs.borrow();
    let num_outputs = outputs.len();
    for (i, (name, output)) in outputs.iter().enumerate() {
        w.append_indent()?;
        w.append("output wire logic ")?;
        if output.bit_width() > 1 {
            w.append(&format!("[{}:{}]", output.bit_width() - 1, 0))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::*;

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because it has a recursive definition formed by an instance of itself called \"a\"."
    )]
    fn recursive_module_definition_error1() {
        let c = Context::new();

        let a = c.module("A");

        let _ = a.instance("a", "A");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because it has a recursive definition formed by an instance of itself called \"a\" in module \"B\"."
    )]
    fn recursive_module_definition_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");

        let _ = a.instance("b", "B");
        let _ = b.instance("a", "A");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains an instance of module \"B\" called \"b\" whose input \"i\" is not driven."
    )]
    fn undriven_instance_input_error() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.input("i", 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error1() {
        let c = Context::new();

        let a = c.module("A");
        let _ = a.reg("r", 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.reg("r", 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error1() {
        let c = Context::new();

        let a = c.module("A");
        let _ = a.mem("m", 1, 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.mem("m", 1, 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error1() {
        let c = Context::new();

        let a = c.module("A");
        let m = a.mem("m", 1, 1);
        let _ = m.read_port(a.low(), a.low());

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let m = b.mem("m", 1, 1);
        let _ = m.read_port(b.low(), b.low());

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"b\" because module \"a\" contains an output called \"o\" which forms a combinational loop with itself."
    )]
    fn combinational_loop_error() {
        let c = Context::new();

        let a = c.module("a");
        a.output("o", a.input("i", 1));

        let b = c.module("b");
        let a_inst = b.instance("a_inst", "a");
        let a_inst_o = a_inst.output("o");
        a_inst.drive_input("i", a_inst_o);

        // Panic
        generate(b, Vec::new()).unwrap();
    }
}
