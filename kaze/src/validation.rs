use std::ptr;

use super::module_context::*;

use crate::graph;

use typed_arena::Arena;

struct ModuleStackFrame<'graph, 'frame> {
    parent: Option<(
        &'graph graph::Instance<'graph>,
        &'frame ModuleStackFrame<'graph, 'frame>,
    )>,
    module: &'graph graph::Module<'graph>,
}

pub fn validate_module_hierarchy<'graph>(m: &'graph graph::Module<'graph>) {
    detect_recursive_definitions(
        m,
        &ModuleStackFrame {
            parent: None,
            module: m,
        },
        m,
    );
    detect_undriven_registers(
        m,
        &ModuleStackFrame {
            parent: None,
            module: m,
        },
        m,
    );
    detect_mem_errors(
        m,
        &ModuleStackFrame {
            parent: None,
            module: m,
        },
        m,
    );
    let context_arena = Arena::new();
    let root_context = context_arena.alloc(ModuleContext::new());
    detect_combinational_loops(m, root_context, &context_arena, m);
}

fn detect_recursive_definitions<'graph, 'frame>(
    m: &graph::Module<'graph>,
    module_stack_frame: &ModuleStackFrame<'graph, 'frame>,
    root: &graph::Module<'graph>,
) {
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

            if let Some((_, parent)) = frame.parent {
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

        detect_recursive_definitions(
            instantiated_module,
            &ModuleStackFrame {
                parent: Some((instance, module_stack_frame)),
                module: instantiated_module,
            },
            root,
        );
    }
}

fn detect_undriven_registers<'graph, 'frame>(
    m: &graph::Module<'graph>,
    module_stack_frame: &ModuleStackFrame<'graph, 'frame>,
    root: &graph::Module<'graph>,
) {
    for register in m.registers.borrow().iter() {
        match register.data {
            graph::SignalData::Reg { ref data } => {
                if data.next.borrow().is_none() {
                    panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a register called \"{}\" which is not driven.", root.name, m.name, data.name);
                }
            }
            _ => unreachable!(),
        }
    }

    for instance in m.instances.borrow().iter() {
        let instantiated_module = instance.instantiated_module;

        detect_undriven_registers(
            instantiated_module,
            &ModuleStackFrame {
                parent: Some((instance, module_stack_frame)),
                module: instantiated_module,
            },
            root,
        );
    }
}

fn detect_mem_errors<'graph, 'frame>(
    m: &graph::Module<'graph>,
    module_stack_frame: &ModuleStackFrame<'graph, 'frame>,
    root: &graph::Module<'graph>,
) {
    for mem in m.mems.borrow().iter() {
        if mem.read_ports.borrow().is_empty() {
            panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a memory called \"{}\" which doesn't have any read ports.", root.name, m.name, mem.name);
        }

        if mem.initial_contents.borrow().is_none() && mem.write_port.borrow().is_none() {
            panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a memory called \"{}\" which doesn't have initial contents or a write port specified. At least one of the two is required.", root.name, m.name, mem.name);
        }
    }

    for instance in m.instances.borrow().iter() {
        let instantiated_module = instance.instantiated_module;

        detect_mem_errors(
            instantiated_module,
            &ModuleStackFrame {
                parent: Some((instance, module_stack_frame)),
                module: instantiated_module,
            },
            root,
        );
    }
}

fn detect_combinational_loops<'graph, 'arena>(
    m: &graph::Module<'graph>,
    context: &'arena ModuleContext<'graph, 'arena>,
    context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,
    root: &graph::Module<'graph>,
) {
    for instance in m.instances.borrow().iter() {
        let instantiated_module = instance.instantiated_module;

        let context = context.get_child(instance, context_arena);

        for (_, output) in instantiated_module.outputs.borrow().iter() {
            trace_signal(output, context, context_arena, (context, output), root);
        }

        detect_combinational_loops(instantiated_module, context, context_arena, root);
    }
}

fn trace_signal<'graph, 'arena>(
    signal: &graph::Signal<'graph>,
    context: &'arena ModuleContext<'graph, 'arena>,
    context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,
    source_output: (
        &'arena ModuleContext<'graph, 'arena>,
        &'graph graph::Signal<'graph>,
    ),
    root: &graph::Module<'graph>,
) {
    match signal.data {
        graph::SignalData::Lit { .. } => (),

        graph::SignalData::Input { ref name, .. } => {
            if let Some((instance, parent)) = context.instance_and_parent {
                trace_signal(
                    instance.driven_inputs.borrow()[name],
                    parent,
                    context_arena,
                    source_output,
                    root,
                );
            }
        }

        graph::SignalData::Reg { .. } => (),

        graph::SignalData::UnOp { ref source, .. } => {
            trace_signal(source, context, context_arena, source_output, root);
        }
        graph::SignalData::SimpleBinOp {
            ref lhs, ref rhs, ..
        } => {
            trace_signal(lhs, context, context_arena, source_output, root);
            trace_signal(rhs, context, context_arena, source_output, root);
        }
        graph::SignalData::AdditiveBinOp {
            ref lhs, ref rhs, ..
        } => {
            trace_signal(lhs, context, context_arena, source_output, root);
            trace_signal(rhs, context, context_arena, source_output, root);
        }
        graph::SignalData::ComparisonBinOp {
            ref lhs, ref rhs, ..
        } => {
            trace_signal(lhs, context, context_arena, source_output, root);
            trace_signal(rhs, context, context_arena, source_output, root);
        }
        graph::SignalData::ShiftBinOp {
            ref lhs, ref rhs, ..
        } => {
            trace_signal(lhs, context, context_arena, source_output, root);
            trace_signal(rhs, context, context_arena, source_output, root);
        }

        graph::SignalData::Bits { ref source, .. } => {
            trace_signal(source, context, context_arena, source_output, root);
        }

        graph::SignalData::Repeat { ref source, .. } => {
            trace_signal(source, context, context_arena, source_output, root);
        }
        graph::SignalData::Concat {
            ref lhs, ref rhs, ..
        } => {
            trace_signal(lhs, context, context_arena, source_output, root);
            trace_signal(rhs, context, context_arena, source_output, root);
        }

        graph::SignalData::Mux {
            ref cond,
            ref when_true,
            ref when_false,
        } => {
            trace_signal(cond, context, context_arena, source_output, root);
            trace_signal(when_true, context, context_arena, source_output, root);
            trace_signal(when_false, context, context_arena, source_output, root);
        }

        graph::SignalData::InstanceOutput { instance, ref name } => {
            let instantiated_module = instance.instantiated_module;
            let output = instantiated_module.outputs.borrow()[name];
            let context = context.get_child(instance, context_arena);
            if context == source_output.0 && output == source_output.1 {
                panic!("Cannot generate code for module \"{}\" because module \"{}\" contains an output called \"{}\" which forms a combinational loop with itself.", root.name, instantiated_module.name, name);
            }
            trace_signal(output, context, context_arena, source_output, root);
        }

        graph::SignalData::MemReadPortOutput { .. } => (),
    }
}
