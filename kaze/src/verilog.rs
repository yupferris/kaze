//! Verilog code generation.

mod compiler;
mod ir;

use compiler::*;
use ir::*;

use crate::code_writer;
use crate::graph;
use crate::state_elements::*;
use crate::validation::*;

use std::collections::HashMap;
use std::io::{Result, Write};

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
    validate_module_hierarchy(m);

    let mut signal_reference_counts = HashMap::new();
    let state_elements = StateElements::new(
        m,
        IncludedPorts::ReachableFromTopLevelOutputs,
        &mut signal_reference_counts,
    );

    let mut c = Compiler::new();

    let mut assignments = AssignmentContext::new();
    for (name, &output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(output.data.source, &state_elements, &mut assignments);
        assignments.push(Assignment {
            target_name: name.clone(),
            expr,
        });
    }

    let mut node_decls = Vec::new();

    for (mem, mem_decls) in state_elements.mems.iter() {
        for ((address, enable), read_signal_names) in mem_decls.read_signal_names.iter() {
            let expr = c.compile_signal(address, &state_elements, &mut assignments);
            node_decls.push(NodeDecl {
                net_type: NetType::Wire,
                name: read_signal_names.address_name.clone(),
                bit_width: address.bit_width(),
            });
            assignments.push(Assignment {
                target_name: read_signal_names.address_name.clone(),
                expr,
            });
            let expr = c.compile_signal(enable, &state_elements, &mut assignments);
            node_decls.push(NodeDecl {
                net_type: NetType::Wire,
                name: read_signal_names.enable_name.clone(),
                bit_width: enable.bit_width(),
            });
            assignments.push(Assignment {
                target_name: read_signal_names.enable_name.clone(),
                expr,
            });
            node_decls.push(NodeDecl {
                net_type: NetType::Reg,
                name: read_signal_names.value_name.clone(),
                bit_width: mem.element_bit_width,
            });
        }
        if let Some((address, value, enable)) = *mem.write_port.borrow() {
            let expr = c.compile_signal(address, &state_elements, &mut assignments);
            node_decls.push(NodeDecl {
                net_type: NetType::Wire,
                name: mem_decls.write_address_name.clone(),
                bit_width: address.bit_width(),
            });
            assignments.push(Assignment {
                target_name: mem_decls.write_address_name.clone(),
                expr,
            });
            let expr = c.compile_signal(value, &state_elements, &mut assignments);
            node_decls.push(NodeDecl {
                net_type: NetType::Wire,
                name: mem_decls.write_value_name.clone(),
                bit_width: value.bit_width(),
            });
            assignments.push(Assignment {
                target_name: mem_decls.write_value_name.clone(),
                expr,
            });
            let expr = c.compile_signal(enable, &state_elements, &mut assignments);
            node_decls.push(NodeDecl {
                net_type: NetType::Wire,
                name: mem_decls.write_enable_name.clone(),
                bit_width: enable.bit_width(),
            });
            assignments.push(Assignment {
                target_name: mem_decls.write_enable_name.clone(),
                expr,
            });
        }
    }

    for reg in state_elements.regs.values() {
        node_decls.push(NodeDecl {
            net_type: NetType::Reg,
            name: reg.value_name.clone(),
            bit_width: reg.data.bit_width,
        });
        node_decls.push(NodeDecl {
            net_type: NetType::Wire,
            name: reg.next_name.clone(),
            bit_width: reg.data.bit_width,
        });

        let expr = c.compile_signal(
            reg.data.next.borrow().unwrap(),
            &state_elements,
            &mut assignments,
        );
        assignments.push(Assignment {
            target_name: reg.next_name.clone(),
            expr,
        });
    }

    let mut w = code_writer::CodeWriter::new(w);

    w.append_line(&format!("module {}(", m.name))?;
    w.indent();

    // TODO: Make conditional based on the presence of (resetable) state elements
    w.append_line("input wire reset_n,")?;
    w.append_indent()?;
    w.append("input wire clk")?;
    if !m.inputs.borrow().is_empty() || !m.outputs.borrow().is_empty() {
        w.append(",")?;
        w.append_newline()?;
    }
    w.append_newline()?;
    let inputs = m.inputs.borrow();
    let num_inputs = inputs.len();
    for (i, (name, &input)) in inputs.iter().enumerate() {
        w.append_indent()?;
        w.append("input wire ")?;
        if input.data.bit_width > 1 {
            w.append(&format!("[{}:{}] ", input.data.bit_width - 1, 0))?;
        }
        w.append(name)?;
        if !m.outputs.borrow().is_empty() || i < num_inputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    let outputs = m.outputs.borrow();
    let num_outputs = outputs.len();
    for (i, (name, &output)) in outputs.iter().enumerate() {
        w.append_indent()?;
        w.append("output wire ")?;
        if output.data.bit_width > 1 {
            w.append(&format!("[{}:{}] ", output.data.bit_width - 1, 0))?;
        }
        w.append(name)?;
        if i < num_outputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    w.append_line(");")?;
    w.append_newline()?;

    if !node_decls.is_empty() {
        for node_decl in node_decls {
            node_decl.write(&mut w)?;
        }
        w.append_newline()?;
    }

    for (mem, mem_decls) in state_elements.mems.iter() {
        w.append_indent()?;
        w.append("reg ")?;
        if mem.element_bit_width > 1 {
            w.append(&format!("[{}:{}] ", mem.element_bit_width - 1, 0))?;
        }
        w.append(&format!(
            "{}[{}:{}];",
            mem_decls.mem_name,
            0,
            (1 << mem.address_bit_width) - 1
        ))?;
        w.append_newline()?;
        w.append_newline()?;
        if let Some(ref initial_contents) = *mem.initial_contents.borrow() {
            w.append_line("initial begin")?;
            w.indent();
            for (i, element) in initial_contents.iter().enumerate() {
                w.append_line(&format!(
                    "{}[{}] = {}'h{:x};",
                    mem_decls.mem_name,
                    i,
                    mem.element_bit_width,
                    element.numeric_value()
                ))?;
            }
            w.unindent();
            w.append_line("end")?;
            w.append_newline()?;
        }
        if !mem_decls.read_signal_names.is_empty() || mem.write_port.borrow().is_some() {
            w.append_line("always @(posedge clk) begin")?;
            w.indent();
        }
        for (_, read_signal_names) in mem_decls.read_signal_names.iter() {
            w.append_line(&format!("if ({}) begin", read_signal_names.enable_name))?;
            w.indent();
            w.append_line(&format!(
                "{} <= {}[{}];",
                read_signal_names.value_name, mem_decls.mem_name, read_signal_names.address_name
            ))?;
            w.unindent();
            w.append_line("end")?;
        }
        if mem.write_port.borrow().is_some() {
            w.append_line(&format!("if ({}) begin", mem_decls.write_enable_name))?;
            w.indent();
            w.append_line(&format!(
                "{}[{}] <= {};",
                mem_decls.mem_name, mem_decls.write_address_name, mem_decls.write_value_name
            ))?;
            w.unindent();
            w.append_line("end")?;
        }
        if !mem_decls.read_signal_names.is_empty() || mem.write_port.borrow().is_some() {
            w.unindent();
            w.append_line("end")?;
            w.append_newline()?;
        }
    }

    for reg in state_elements.regs.values() {
        w.append_indent()?;
        w.append("always @(posedge clk")?;
        if reg.data.initial_value.borrow().is_some() {
            w.append(", negedge reset_n")?;
        }
        w.append(") begin")?;
        w.append_newline()?;
        w.indent();
        if let Some(ref initial_value) = *reg.data.initial_value.borrow() {
            w.append_line("if (~reset_n) begin")?;
            w.indent();
            w.append_line(&format!(
                "{} <= {}'h{:x};",
                reg.value_name,
                reg.data.bit_width,
                initial_value.numeric_value()
            ))?;
            w.unindent();
            w.append_line("end")?;
            w.append_line("else begin")?;
            w.indent();
        }
        w.append_line(&format!("{} <= {};", reg.value_name, reg.next_name))?;
        if reg.data.initial_value.borrow().is_some() {
            w.unindent();
            w.append_line("end")?;
        }
        w.unindent();
        w.append_line("end")?;
        w.append_newline()?;
    }

    if !assignments.is_empty() {
        assignments.write(&mut w)?;
        w.append_newline()?;
    }

    w.unindent();
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
        expected = "Cannot generate code for module \"A\" because module \"A\" contains an instance of module \"B\" called \"b\" whose input \"i\" is not driven."
    )]
    fn undriven_instance_input_error() {
        let c = Context::new();

        let a = c.module("a", "A");
        let b = a.module("b", "B");
        let _ = b.input("i", 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error1() {
        let c = Context::new();

        let a = c.module("a", "A");
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

        let a = c.module("a", "A");
        let b = a.module("b", "B");
        let _ = b.reg("r", 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error1() {
        let c = Context::new();

        let a = c.module("a", "A");
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

        let a = c.module("a", "A");
        let b = a.module("b", "B");
        let _ = b.mem("m", 1, 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error1() {
        let c = Context::new();

        let a = c.module("a", "A");
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

        let a = c.module("a", "A");
        let b = a.module("b", "B");
        let m = b.mem("m", 1, 1);
        let _ = m.read_port(b.low(), b.low());

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"b\" because module \"a\" contains an output called \"o\" which forms a combinational loop with itself."
    )]
    fn combinational_loop_error() {
        let c = Context::new();

        let b = c.module("b", "b");
        let a = b.module("a", "a");
        let a_i = a.input("i", 1);
        let a_o = a.output("o", a_i);
        a_i.drive(a_o);

        // Panic
        generate(b, Vec::new()).unwrap();
    }
}
