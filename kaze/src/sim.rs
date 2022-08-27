//! Rust simulator code generation.

mod compiler;
mod ir;

use compiler::*;
use ir::*;

use typed_arena::Arena;

use crate::code_writer;
use crate::graph;
use crate::runtime::tracing::*;
use crate::state_elements::*;
use crate::validation::*;

use std::collections::HashMap;
use std::io::{Result, Write};

#[derive(Default)]
pub struct GenerationOptions {
    pub override_module_name: Option<String>,
    pub tracing: bool,
}

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(
    m: &'a graph::Module<'a>,
    options: GenerationOptions,
    w: W,
) -> Result<()> {
    validate_module_hierarchy(m);

    // TODO: Consider exposing as a codegen option (and testing both variants)
    let included_ports = if options.tracing {
        IncludedPorts::All
    } else {
        IncludedPorts::ReachableFromTopLevelOutputs
    };

    let mut signal_reference_counts = HashMap::new();
    let state_elements = StateElements::new(m, included_ports, &mut signal_reference_counts);

    struct TraceSignal {
        name: String,
        member_name: String,
        value_name: String,
        bit_width: u32,
        type_: TraceValueType,
    }
    let mut trace_signals: HashMap<&'a graph::Module<'a>, Vec<TraceSignal>> = HashMap::new();
    let mut num_trace_signals = 0;
    let mut add_trace_signal = |module, name, value_name, bit_width| {
        if options.tracing {
            let member_name = format!("__trace_signal_id_{}_{}", name, num_trace_signals);
            let module_trace_signals = trace_signals.entry(module).or_insert(Vec::new());
            module_trace_signals.push(TraceSignal {
                name,
                member_name,
                value_name,
                bit_width,
                type_: TraceValueType::from_bit_width(bit_width),
            });
            num_trace_signals += 1;
        }
    };

    let expr_arena = Arena::new();
    let mut prop_context = AssignmentContext::new(&expr_arena);
    let mut c = Compiler::new(&state_elements, &signal_reference_counts, &expr_arena);
    for (name, input) in m.inputs.borrow().iter() {
        add_trace_signal(m, name.clone(), name.clone(), input.data.bit_width);
    }
    for (name, output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(output.data.source, &mut prop_context);
        prop_context.push(Assignment {
            target: expr_arena.alloc(Expr::Ref {
                name: name.clone(),
                scope: Scope::Member,
            }),
            expr,
        });

        add_trace_signal(m, name.clone(), name.clone(), output.data.bit_width);
    }
    struct InnerField {
        name: String,
        bit_width: u32,
    }
    let mut inner_fields = Vec::new();
    if options.tracing {
        fn visit_module<'graph, 'context, 'expr_arena>(
            module: &'graph graph::Module<'graph>,
            c: &mut Compiler<'graph, 'context, 'expr_arena>,
            inner_fields: &mut Vec<InnerField>,
            prop_context: &mut AssignmentContext<'expr_arena>,
            expr_arena: &'expr_arena Arena<Expr>,
            add_trace_signal: &mut impl FnMut(&'graph graph::Module<'graph>, String, String, u32),
        ) -> Result<()> {
            // TODO: Identify and fix duplicate signals in traces
            for (name, &input) in module.inputs.borrow().iter() {
                // TODO: De-dupe inner field allocs
                let field_name = format!("__inner_{}_{}", name, inner_fields.len());
                inner_fields.push(InnerField {
                    name: field_name.clone(),
                    bit_width: input.data.bit_width,
                });
                let expr =
                    c.compile_signal(input.data.driven_value.borrow().unwrap(), prop_context);
                prop_context.push(Assignment {
                    target: expr_arena.alloc(Expr::Ref {
                        name: field_name.clone(),
                        scope: Scope::Member,
                    }),
                    expr,
                });

                add_trace_signal(module, name.clone(), field_name, input.data.bit_width);
            }
            for (name, &output) in module.outputs.borrow().iter() {
                // TODO: De-dupe inner field allocs
                let field_name = format!("__inner_{}_{}", name, inner_fields.len());
                inner_fields.push(InnerField {
                    name: field_name.clone(),
                    bit_width: output.data.bit_width,
                });
                let expr = c.compile_signal(output.data.source, prop_context);
                prop_context.push(Assignment {
                    target: expr_arena.alloc(Expr::Ref {
                        name: field_name.clone(),
                        scope: Scope::Member,
                    }),
                    expr,
                });

                add_trace_signal(module, name.clone(), field_name, output.data.bit_width);
            }
            for child in module.modules.borrow().iter() {
                visit_module(
                    child,
                    c,
                    inner_fields,
                    prop_context,
                    expr_arena,
                    add_trace_signal,
                )?;
            }

            Ok(())
        }
        for child in m.modules.borrow().iter() {
            visit_module(
                child,
                &mut c,
                &mut inner_fields,
                &mut prop_context,
                &expr_arena,
                &mut add_trace_signal,
            )?;
        }
    }
    for (graph_mem, mem) in state_elements.mems.iter() {
        for ((address, enable), read_signal_names) in mem.read_signal_names.iter() {
            let address = c.compile_signal(address, &mut prop_context);
            prop_context.push(Assignment {
                target: expr_arena.alloc(Expr::Ref {
                    name: read_signal_names.address_name.clone(),
                    scope: Scope::Member,
                }),
                expr: address,
            });
            let enable = c.compile_signal(enable, &mut prop_context);
            prop_context.push(Assignment {
                target: expr_arena.alloc(Expr::Ref {
                    name: read_signal_names.enable_name.clone(),
                    scope: Scope::Member,
                }),
                expr: enable,
            });

            add_trace_signal(
                graph_mem.module,
                read_signal_names.address_name.clone(),
                read_signal_names.address_name.clone(),
                graph_mem.address_bit_width,
            );
            add_trace_signal(
                graph_mem.module,
                read_signal_names.enable_name.clone(),
                read_signal_names.enable_name.clone(),
                1,
            );
        }
        if let Some((address, value, enable)) = *graph_mem.write_port.borrow() {
            let address = c.compile_signal(address, &mut prop_context);
            prop_context.push(Assignment {
                target: expr_arena.alloc(Expr::Ref {
                    name: mem.write_address_name.clone(),
                    scope: Scope::Member,
                }),
                expr: address,
            });
            let value = c.compile_signal(value, &mut prop_context);
            prop_context.push(Assignment {
                target: expr_arena.alloc(Expr::Ref {
                    name: mem.write_value_name.clone(),
                    scope: Scope::Member,
                }),
                expr: value,
            });
            let enable = c.compile_signal(enable, &mut prop_context);
            prop_context.push(Assignment {
                target: expr_arena.alloc(Expr::Ref {
                    name: mem.write_enable_name.clone(),
                    scope: Scope::Member,
                }),
                expr: enable,
            });

            add_trace_signal(
                graph_mem.module,
                mem.write_address_name.clone(),
                mem.write_address_name.clone(),
                graph_mem.address_bit_width,
            );
            add_trace_signal(
                graph_mem.module,
                mem.write_value_name.clone(),
                mem.write_value_name.clone(),
                graph_mem.element_bit_width,
            );
            add_trace_signal(
                graph_mem.module,
                mem.write_enable_name.clone(),
                mem.write_enable_name.clone(),
                1,
            );
        }
    }
    for (_, reg) in state_elements.regs.iter() {
        let signal = reg.data.next.borrow().unwrap();
        let expr = c.compile_signal(signal, &mut prop_context);
        prop_context.push(Assignment {
            target: expr_arena.alloc(Expr::Ref {
                name: reg.next_name.clone(),
                scope: Scope::Member,
            }),
            expr,
        });

        add_trace_signal(
            signal.module,
            reg.data.name.clone(),
            reg.value_name.clone(),
            signal.bit_width(),
        );
    }

    let mut w = code_writer::CodeWriter::new(w);

    let module_name = options
        .override_module_name
        .unwrap_or_else(|| m.name.clone());

    w.append_indent()?;
    w.append(&format!("pub struct {}", module_name))?;
    if options.tracing {
        w.append("<T: kaze::runtime::tracing::Trace>")?;
    }
    w.append("{")?;
    w.append_newline()?;
    w.indent();

    let inputs = m.inputs.borrow();
    if !inputs.is_empty() {
        w.append_line("// Inputs")?;
        for (name, input) in inputs.iter() {
            w.append_line(&format!(
                "pub {}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(input.data.bit_width).name(),
                input.data.bit_width
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
                ValueType::from_bit_width(output.data.bit_width).name(),
                output.data.bit_width
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

    if !inner_fields.is_empty() {
        w.append_newline()?;
        w.append_line("// Inner")?;
        for field in &inner_fields {
            let type_name = ValueType::from_bit_width(field.bit_width).name();
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                field.name, type_name, field.bit_width
            ))?;
        }
    }

    if options.tracing {
        w.append_newline()?;
        w.append_line("__trace: T,")?;
        for module_trace_signals in trace_signals.values() {
            for trace_signal in module_trace_signals.iter() {
                w.append_line(&format!("{}: T::SignalId,", trace_signal.member_name))?;
            }
        }
    }

    w.unindent();
    w.append_line("}")?;
    w.append_newline()?;

    w.append_line("#[allow(unused_parens)]")?;
    w.append_line("#[automatically_derived]")?;
    w.append_indent()?;
    w.append("impl")?;
    if options.tracing {
        w.append("<T: kaze::runtime::tracing::Trace>")?;
    }
    w.append(&format!(" {}", module_name))?;
    if options.tracing {
        w.append("<T>")?;
    }
    w.append(" {")?;
    w.append_newline()?;
    w.indent();

    w.append_indent()?;
    w.append("pub fn new(")?;
    if options.tracing {
        w.append(&format!(
            "mut trace: T) -> std::io::Result<{}<T>> {{",
            module_name
        ))?;
    } else {
        w.append(&format!(") -> {} {{", module_name))?;
    }
    w.append_newline()?;
    w.indent();

    if options.tracing {
        fn visit_module<'a, W: Write>(
            module: &'a graph::Module<'a>,
            trace_signals: &HashMap<&'a graph::Module<'a>, Vec<TraceSignal>>,
            w: &mut code_writer::CodeWriter<W>,
        ) -> Result<()> {
            w.append_line(&format!(
                "trace.push_module(\"{}\")?;",
                module.instance_name
            ))?;

            if let Some(module_trace_signals) = trace_signals.get(&module) {
                for trace_signal in module_trace_signals.iter() {
                    w.append_line(&format!("let {} = trace.add_signal(\"{}\", {}, kaze::runtime::tracing::TraceValueType::{})?;", trace_signal.member_name, trace_signal.name, trace_signal.bit_width, match trace_signal.type_ {
                        TraceValueType::Bool => "Bool",
                        TraceValueType::U32 => "U32",
                        TraceValueType::U64 => "U64",
                        TraceValueType::U128 => "U128",
                    }))?;
                }
            }

            for child in module.modules.borrow().iter() {
                visit_module(child, trace_signals, w)?;
            }

            w.append_line("trace.pop_module()?;")?;

            Ok(())
        }
        visit_module(m, &trace_signals, &mut w)?;
        w.append_newline()?;
    }

    w.append_indent()?;
    if options.tracing {
        w.append("Ok(")?;
    }
    w.append(&format!("{} {{", module_name))?;
    w.append_newline()?;
    w.indent();

    if !inputs.is_empty() {
        w.append_line("// Inputs")?;
        for (name, input) in inputs.iter() {
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(input.data.bit_width).zero_str(),
                input.data.bit_width
            ))?;
        }
    }

    if !outputs.is_empty() {
        w.append_line("// Outputs")?;
        for (name, output) in outputs.iter() {
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                name,
                ValueType::from_bit_width(output.data.bit_width).zero_str(),
                output.data.bit_width
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
                w.unindent();
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

    if !inner_fields.is_empty() {
        w.append_newline()?;
        for field in &inner_fields {
            w.append_line(&format!(
                "{}: {}, // {} bit(s)",
                field.name,
                ValueType::from_bit_width(field.bit_width).zero_str(),
                field.bit_width
            ))?;
        }
    }

    if options.tracing {
        w.append_newline()?;
        w.append_line("__trace: trace,")?;
        for module_trace_signals in trace_signals.values() {
            for trace_signal in module_trace_signals.iter() {
                w.append_line(&format!("{},", trace_signal.member_name))?;
            }
        }
    }

    w.unindent();
    w.append_indent()?;
    w.append("}")?;
    if options.tracing {
        w.append(")")?;
    }
    w.append_newline()?;
    w.unindent();
    w.append_line("}")?;

    let mut reset_context = AssignmentContext::new(&expr_arena);
    let mut posedge_clk_context = AssignmentContext::new(&expr_arena);

    for (_, reg) in state_elements.regs.iter() {
        let target = expr_arena.alloc(Expr::Ref {
            name: reg.value_name.clone(),
            scope: Scope::Member,
        });

        if let Some(ref initial_value) = *reg.data.initial_value.borrow() {
            reset_context.push(Assignment {
                target,
                expr: Expr::from_constant(initial_value, reg.data.bit_width, &expr_arena),
            });
        }

        posedge_clk_context.push(Assignment {
            target,
            expr: expr_arena.alloc(Expr::Ref {
                name: reg.next_name.clone(),
                scope: Scope::Member,
            }),
        });
    }

    for (_, mem) in state_elements.mems.iter() {
        for (_, read_signal_names) in mem.read_signal_names.iter() {
            let address = expr_arena.alloc(Expr::Ref {
                name: read_signal_names.address_name.clone(),
                scope: Scope::Member,
            });
            let enable = expr_arena.alloc(Expr::Ref {
                name: read_signal_names.enable_name.clone(),
                scope: Scope::Member,
            });
            let value = expr_arena.alloc(Expr::Ref {
                name: read_signal_names.value_name.clone(),
                scope: Scope::Member,
            });
            let element = expr_arena.alloc(Expr::ArrayIndex {
                target: expr_arena.alloc(Expr::Ref {
                    name: mem.mem_name.clone(),
                    scope: Scope::Member,
                }),
                index: address,
            });
            // TODO: Conditional assign statement instead of always writing ternary
            posedge_clk_context.push(Assignment {
                target: value,
                expr: expr_arena.alloc(Expr::Ternary {
                    cond: enable,
                    when_true: element,
                    when_false: value,
                }),
            });
        }
        if mem.mem.write_port.borrow().is_some() {
            let address = expr_arena.alloc(Expr::Ref {
                name: mem.write_address_name.clone(),
                scope: Scope::Member,
            });
            let value = expr_arena.alloc(Expr::Ref {
                name: mem.write_value_name.clone(),
                scope: Scope::Member,
            });
            let enable = expr_arena.alloc(Expr::Ref {
                name: mem.write_enable_name.clone(),
                scope: Scope::Member,
            });
            let element = expr_arena.alloc(Expr::ArrayIndex {
                target: expr_arena.alloc(Expr::Ref {
                    name: mem.mem_name.clone(),
                    scope: Scope::Member,
                }),
                index: address,
            });
            // TODO: Conditional assign statement instead of always writing ternary
            posedge_clk_context.push(Assignment {
                target: element,
                expr: expr_arena.alloc(Expr::Ternary {
                    cond: enable,
                    when_true: value,
                    when_false: element,
                }),
            });
        }
    }

    if !reset_context.is_empty() {
        w.append_newline()?;
        w.append_line("pub fn reset(&mut self) {")?;
        w.indent();

        reset_context.write(&mut w)?;

        w.unindent();
        w.append_line("}")?;
    }

    if !posedge_clk_context.is_empty() {
        w.append_newline()?;
        w.append_line("pub fn posedge_clk(&mut self) {")?;
        w.indent();

        posedge_clk_context.write(&mut w)?;

        w.unindent();
        w.append_line("}")?;
    }

    w.append_newline()?;
    w.append_line("pub fn prop(&mut self) {")?;
    w.indent();

    prop_context.write(&mut w)?;

    w.unindent();
    w.append_line("}")?;

    if options.tracing {
        w.append_newline()?;
        w.append_line("pub fn update_trace(&mut self, time_stamp: u64) -> std::io::Result<()> {")?;
        w.indent();

        w.append_line("self.__trace.update_time_stamp(time_stamp)?;")?;
        w.append_newline()?;

        for module_trace_signals in trace_signals.values() {
            for trace_signal in module_trace_signals.iter() {
                w.append_line(&format!("self.__trace.update_signal(&self.{}, kaze::runtime::tracing::TraceValue::{}(self.{}))?;", trace_signal.member_name, match trace_signal.type_ {
                    TraceValueType::Bool => "Bool",
                    TraceValueType::U32 => "U32",
                    TraceValueType::U64 => "U64",
                    TraceValueType::U128 => "U128",
                }, trace_signal.value_name))?;
            }
        }
        w.append_newline()?;

        w.append_line("Ok(())")?;

        w.unindent();
        w.append_line("}")?;
    }

    w.unindent();
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
        expected = "Cannot generate code for module \"A\" because module \"A\" contains an instance of module \"B\" called \"b\" whose input \"i\" is not driven."
    )]
    fn undriven_instance_input_error() {
        let c = Context::new();

        let a = c.module("a", "A");
        let b = a.module("b", "B");
        let _ = b.input("i", 1);

        // Panic
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(a, GenerationOptions::default(), Vec::new()).unwrap();
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
        generate(b, GenerationOptions::default(), Vec::new()).unwrap();
    }
}
