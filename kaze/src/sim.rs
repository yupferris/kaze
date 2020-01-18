//! Rust simulator code generation.

mod compiler;
mod ir;

use compiler::*;
use ir::*;

use typed_arena::Arena;

use crate::code_writer;
use crate::graph;

use std::io::{Result, Write};
use std::ptr;

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
    validate_module_hierarchy(
        m,
        &ModuleStackFrame {
            parent: None,
            module: m,
        },
        m,
    );

    let context_arena = Arena::new();
    let root_context = context_arena.alloc(ModuleContext::new(None));
    let mut c = Compiler::new(&context_arena);

    for (_, output) in m.outputs.borrow().iter() {
        c.gather_regs(&output, root_context);
    }

    for (name, output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(&output, root_context);
        c.prop_assignments.push(Assignment {
            target_scope: TargetScope::Member,
            target_name: name.clone(),
            expr,
        });
    }

    // TODO: Can we get rid of this clone?
    for ((context, reg), names) in c.reg_names.clone().iter() {
        let context = unsafe { &**context as &ModuleContext };
        let reg = unsafe { &**reg as &graph::Signal };
        match reg.data {
            graph::SignalData::Reg { ref next, .. } => {
                let expr = c.compile_signal(next.borrow().unwrap(), context);
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

struct ModuleStackFrame<'graph, 'frame> {
    parent: Option<&'frame ModuleStackFrame<'graph, 'frame>>,
    module: &'graph graph::Module<'graph>,
}

fn validate_module_hierarchy<'graph, 'frame>(
    m: &graph::Module<'graph>,
    module_stack_frame: &ModuleStackFrame<'graph, 'frame>,
    root: &graph::Module<'graph>,
) {
    for register in m.registers.borrow().iter() {
        match register.data {
            graph::SignalData::Reg {
                ref name, ref next, ..
            } => {
                if next.borrow().is_none() {
                    panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a register called \"{}\" which is not driven.", root.name, m.name, name);
                }
            }
            _ => unreachable!(),
        }
    }

    for instance in m.instances.borrow().iter() {
        let instantiated_module = instance.instantiated_module;

        if ptr::eq(instantiated_module, m) {
            panic!("Cannot generate code for module \"{}\" because it has a recursive definition formed by an instance of itself called \"{}\".", m.name, instance.name);
        }

        let mut frame = module_stack_frame;
        loop {
            if ptr::eq(instantiated_module, frame.module) {
                panic!("Cannot generate code for module \"{}\" because it has a recursive definition formed by an instance of itself called \"{}\" in module \"{}\".", root.name, instance.name, m.name);
            }

            if let Some(parent) = frame.parent {
                frame = parent;
            } else {
                break;
            }
        }

        for input_name in instantiated_module.inputs.borrow().keys() {
            if !instance.driven_inputs.borrow().contains_key(input_name) {
                panic!("Cannot generate code for module \"{}\" because module \"{}\" contains an instance of module \"{}\" called \"{}\" whose input \"{}\" is not driven.", root.name, m.name, instantiated_module.name, instance.name, input_name);
            }
        }

        validate_module_hierarchy(
            instantiated_module,
            &ModuleStackFrame {
                parent: Some(module_stack_frame),
                module: instantiated_module,
            },
            root,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::*;

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"a\" because it has a recursive definition formed by an instance of itself called \"a1\"."
    )]
    fn recursive_module_definition_error1() {
        let c = Context::new();

        let a = c.module("a");

        let _ = a.instance("a", "a1");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"a\" because it has a recursive definition formed by an instance of itself called \"a1\" in module \"b\"."
    )]
    fn recursive_module_definition_error2() {
        let c = Context::new();

        let a = c.module("a");
        let b = c.module("b");

        let _ = a.instance("b", "b1");
        let _ = b.instance("a", "a1");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"a\" because module \"a\" contains an instance of module \"b\" called \"b1\" whose input \"i\" is not driven."
    )]
    fn undriven_instance_input_error() {
        let c = Context::new();

        let a = c.module("a");
        let b = c.module("b");
        let _ = b.input("i", 1);

        let _ = a.instance("b", "b1");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"a\" because module \"a\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error1() {
        let c = Context::new();

        let a = c.module("a");
        let _ = a.reg("r", 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"a\" because module \"b\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error2() {
        let c = Context::new();

        let a = c.module("a");
        let b = c.module("b");
        let _ = b.reg("r", 1);

        let _ = a.instance("b", "b1");

        // Panic
        generate(a, Vec::new()).unwrap();
    }
}
