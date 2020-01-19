use std::ptr;

use crate::graph;

struct ModuleStackFrame<'graph, 'frame> {
    parent: Option<&'frame ModuleStackFrame<'graph, 'frame>>,
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

        detect_recursive_definitions(
            instantiated_module,
            &ModuleStackFrame {
                parent: Some(module_stack_frame),
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
                parent: Some(module_stack_frame),
                module: instantiated_module,
            },
            root,
        );
    }
}
