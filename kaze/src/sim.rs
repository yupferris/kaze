//! Rust simulator code generation.

mod compiler;
mod ir;
mod state_elements;

use compiler::*;
use ir::*;
use state_elements::*;

use typed_arena::Arena;

use crate::code_writer;
use crate::graph;
use crate::module_context::*;
use crate::validation::*;

use std::io::{Result, Write};

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
    validate_module_hierarchy(m);

    let context_arena = Arena::new();
    let root_context = context_arena.alloc(ModuleContext::new());

    let mut state_elements = StateElements::new();
    for (_, output) in m.outputs.borrow().iter() {
        state_elements.gather(&output, root_context, &context_arena);
    }

    let mut prop_context = AssignmentContext::new();
    let mut c = Compiler::new(&state_elements, &context_arena);
    for (name, output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(&output, root_context, &mut prop_context);
        prop_context.push(Assignment {
            target: Expr::Ref {
                name: name.clone(),
                scope: Scope::Member,
            },
            expr,
        });
    }
    for ((context, _), mem) in state_elements.mems.iter() {
        for ((address, enable), read_signal_names) in mem.read_signal_names.iter() {
            let address = c.compile_signal(address, context, &mut prop_context);
            prop_context.push(Assignment {
                target: Expr::Ref {
                    name: read_signal_names.address_name.clone(),
                    scope: Scope::Member,
                },
                expr: address,
            });
            let enable = c.compile_signal(enable, context, &mut prop_context);
            prop_context.push(Assignment {
                target: Expr::Ref {
                    name: read_signal_names.enable_name.clone(),
                    scope: Scope::Member,
                },
                expr: enable,
            });
        }
        if let Some((address, value, enable)) = *mem.mem.write_port.borrow() {
            let address = c.compile_signal(address, context, &mut prop_context);
            prop_context.push(Assignment {
                target: Expr::Ref {
                    name: mem.write_address_name.clone(),
                    scope: Scope::Member,
                },
                expr: address,
            });
            let value = c.compile_signal(value, context, &mut prop_context);
            prop_context.push(Assignment {
                target: Expr::Ref {
                    name: mem.write_value_name.clone(),
                    scope: Scope::Member,
                },
                expr: value,
            });
            let enable = c.compile_signal(enable, context, &mut prop_context);
            prop_context.push(Assignment {
                target: Expr::Ref {
                    name: mem.write_enable_name.clone(),
                    scope: Scope::Member,
                },
                expr: enable,
            });
        }
    }
    for ((context, _), reg) in state_elements.regs.iter() {
        let expr = c.compile_signal(reg.data.next.borrow().unwrap(), context, &mut prop_context);
        prop_context.push(Assignment {
            target: Expr::Ref {
                name: reg.next_name.clone(),
                scope: Scope::Member,
            },
            expr,
        });
    }

    let mut w = code_writer::CodeWriter::new(w);

    w.append_line(&format!("pub struct {} {{", m.name))?;
    w.indent();

    let inputs = m.inputs.borrow();
    if !inputs.is_empty() {
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
    if !outputs.is_empty() {
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

    if !state_elements.regs.is_empty() {
        w.append_newline()?;
        w.append_line("// Regs")?;
        for (_, reg) in state_elements.regs.iter() {
            let type_name = ValueType::from_bit_width(reg.data.bit_width).name();
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                reg.value_name, type_name, reg.data.bit_width
            ))?;
            w.append_line(&format!("{}: {},", reg.next_name, type_name))?;
        }
    }

    if !state_elements.mems.is_empty() {
        w.append_newline()?;
        w.append_line("// Mems")?;
        for (_, mem) in state_elements.mems.iter() {
            let address_type_name = ValueType::from_bit_width(mem.mem.address_bit_width).name();
            let element_type_name = ValueType::from_bit_width(mem.mem.element_bit_width).name();
            w.append_line(&format!(
                "{}: Box<[{}]>, // {} bit elements",
                mem.mem_name, element_type_name, mem.mem.element_bit_width
            ))?;
            for (_, read_signal_names) in mem.read_signal_names.iter() {
                w.append_line(&format!(
                    "{}: {},",
                    read_signal_names.address_name, address_type_name
                ))?;
                w.append_line(&format!(
                    "{}: {},",
                    read_signal_names.enable_name,
                    ValueType::Bool.name()
                ))?;
                w.append_line(&format!(
                    "{}: {},",
                    read_signal_names.value_name, element_type_name
                ))?;
            }
            if mem.mem.write_port.borrow().is_some() {
                w.append_line(&format!(
                    "{}: {},",
                    mem.write_address_name, address_type_name
                ))?;
                w.append_line(&format!("{}: {},", mem.write_value_name, element_type_name))?;
                w.append_line(&format!(
                    "{}: {},",
                    mem.write_enable_name,
                    ValueType::Bool.name()
                ))?;
            }
        }
    }

    w.unindent()?;
    w.append_line("}")?;
    w.append_newline()?;

    w.append_line(&format!("impl {} {{", m.name))?;
    w.indent();

    w.append_line(&format!("pub fn new() -> {} {{", m.name))?;
    w.indent();
    w.append_line(&format!("{} {{", m.name))?;
    w.indent();

    if !inputs.is_empty() {
        w.append_line("// Inputs")?;
        for (name, input) in inputs.iter() {
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(input.bit_width()).zero_str(),
                input.bit_width()
            ))?;
        }
    }

    if !outputs.is_empty() {
        w.append_line("// Outputs")?;
        for (name, output) in outputs.iter() {
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(output.bit_width()).zero_str(),
                output.bit_width()
            ))?;
        }
    }

    if !state_elements.regs.is_empty() {
        w.append_newline()?;
        w.append_line("// Regs")?;
        for (_, reg) in state_elements.regs.iter() {
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                reg.value_name,
                ValueType::from_bit_width(reg.data.bit_width).zero_str(),
                reg.data.bit_width
            ))?;
            w.append_line(&format!(
                "{}: {},",
                reg.next_name,
                ValueType::from_bit_width(reg.data.bit_width).zero_str()
            ))?;
        }
    }

    if !state_elements.mems.is_empty() {
        w.append_newline()?;
        w.append_line("// Mems")?;
        for (_, mem) in state_elements.mems.iter() {
            let address_type = ValueType::from_bit_width(mem.mem.address_bit_width);
            let element_type = ValueType::from_bit_width(mem.mem.element_bit_width);
            if let Some(ref initial_contents) = *mem.mem.initial_contents.borrow() {
                w.append_line(&format!("{}: vec![", mem.mem_name))?;
                w.indent();
                for element in initial_contents.iter() {
                    w.append_line(&match *element {
                        graph::Constant::Bool(value) => format!("{},", value),
                        graph::Constant::U32(value) => format!("0x{:x},", value),
                        graph::Constant::U64(value) => format!("0x{:x},", value),
                        graph::Constant::U128(value) => format!("0x{:x},", value),
                    })?;
                }
                w.unindent()?;
                w.append_line("].into_boxed_slice(),")?;
            } else {
                w.append_line(&format!(
                    "{}: vec![{}; {}].into_boxed_slice(),",
                    mem.mem_name,
                    element_type.zero_str(),
                    1 << mem.mem.address_bit_width
                ))?;
            }
            for (_, read_signal_names) in mem.read_signal_names.iter() {
                w.append_line(&format!(
                    "{}: {},",
                    read_signal_names.address_name,
                    address_type.zero_str()
                ))?;
                w.append_line(&format!(
                    "{}: {},",
                    read_signal_names.enable_name,
                    ValueType::Bool.zero_str()
                ))?;
                w.append_line(&format!(
                    "{}: {},",
                    read_signal_names.value_name,
                    element_type.zero_str()
                ))?;
            }
            if mem.mem.write_port.borrow().is_some() {
                w.append_line(&format!(
                    "{}: {},",
                    mem.write_address_name,
                    address_type.zero_str()
                ))?;
                w.append_line(&format!(
                    "{}: {},",
                    mem.write_value_name,
                    element_type.zero_str()
                ))?;
                w.append_line(&format!(
                    "{}: {},",
                    mem.write_enable_name,
                    ValueType::Bool.zero_str()
                ))?;
            }
        }
    }

    w.unindent()?;
    w.append_line("}")?;
    w.unindent()?;
    w.append_line("}")?;

    let mut reset_context = AssignmentContext::new();
    let mut posedge_clk_context = AssignmentContext::new();

    for (_, reg) in state_elements.regs.iter() {
        let target = Expr::Ref {
            name: reg.value_name.clone(),
            scope: Scope::Member,
        };

        if let Some(ref initial_value) = *reg.data.initial_value.borrow() {
            reset_context.push(Assignment {
                target: target.clone(),
                expr: Expr::from_constant(initial_value, reg.data.bit_width),
            });
        }

        posedge_clk_context.push(Assignment {
            target,
            expr: Expr::Ref {
                name: reg.next_name.clone(),
                scope: Scope::Member,
            },
        });
    }

    for (_, mem) in state_elements.mems.iter() {
        for (_, read_signal_names) in mem.read_signal_names.iter() {
            let address = Expr::Ref {
                name: read_signal_names.address_name.clone(),
                scope: Scope::Member,
            };
            let enable = Expr::Ref {
                name: read_signal_names.enable_name.clone(),
                scope: Scope::Member,
            };
            let value = Expr::Ref {
                name: read_signal_names.value_name.clone(),
                scope: Scope::Member,
            };
            let element = Expr::ArrayIndex {
                target: Box::new(Expr::Ref {
                    name: mem.mem_name.clone(),
                    scope: Scope::Member,
                }),
                index: Box::new(address),
            };
            // TODO: Conditional assign statement instead of always writing ternary
            posedge_clk_context.push(Assignment {
                target: value.clone(),
                expr: Expr::Ternary {
                    cond: Box::new(enable),
                    when_true: Box::new(element),
                    when_false: Box::new(value),
                },
            });
        }
        if mem.mem.write_port.borrow().is_some() {
            let address = Expr::Ref {
                name: mem.write_address_name.clone(),
                scope: Scope::Member,
            };
            let value = Expr::Ref {
                name: mem.write_value_name.clone(),
                scope: Scope::Member,
            };
            let enable = Expr::Ref {
                name: mem.write_enable_name.clone(),
                scope: Scope::Member,
            };
            let element = Expr::ArrayIndex {
                target: Box::new(Expr::Ref {
                    name: mem.mem_name.clone(),
                    scope: Scope::Member,
                }),
                index: Box::new(address),
            };
            // TODO: Conditional assign statement instead of always writing ternary
            posedge_clk_context.push(Assignment {
                target: element.clone(),
                expr: Expr::Ternary {
                    cond: Box::new(enable),
                    when_true: Box::new(value),
                    when_false: Box::new(element),
                },
            });
        }
    }

    if !reset_context.is_empty() {
        w.append_newline()?;
        w.append_line("pub fn reset(&mut self) {")?;
        w.indent();

        reset_context.write(&mut w)?;

        w.unindent()?;
        w.append_line("}")?;
    }

    if !posedge_clk_context.is_empty() {
        w.append_newline()?;
        w.append_line("pub fn posedge_clk(&mut self) {")?;
        w.indent();

        posedge_clk_context.write(&mut w)?;

        w.unindent()?;
        w.append_line("}")?;
    }

    w.append_newline()?;
    w.append_line("pub fn prop(&mut self) {")?;
    w.indent();

    prop_context.write(&mut w)?;

    w.unindent()?;
    w.append_line("}")?;

    w.unindent()?;
    w.append_line("}")?;
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
