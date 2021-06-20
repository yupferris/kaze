use crate::graph;
use crate::graph::internal_signal;

pub fn validate_module_hierarchy<'a>(m: &'a graph::Module<'a>) {
    detect_undriven_registers_and_inputs(m, m);
    detect_mem_errors(m, m);
    detect_combinational_loops(m, m);
}

fn detect_undriven_registers_and_inputs<'a>(m: &graph::Module<'a>, root: &graph::Module<'a>) {
    for register in m.registers.borrow().iter() {
        match register.data {
            internal_signal::SignalData::Reg { ref data } => {
                if data.next.borrow().is_none() {
                    panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a register called \"{}\" which is not driven.", root.name, m.name, data.name);
                }
            }
            _ => unreachable!(),
        }
    }

    for module in m.modules.borrow().iter() {
        for (name, input) in module.inputs.borrow().iter() {
            if input.data.driven_value.borrow().is_none() {
                panic!("Cannot generate code for module \"{}\" because module \"{}\" contains an instance of module \"{}\" called \"{}\" whose input \"{}\" is not driven.", root.name, m.name, module.name, module.instance_name, name);
            }
        }

        detect_undriven_registers_and_inputs(module, root);
    }
}

fn detect_mem_errors<'a>(m: &graph::Module<'a>, root: &graph::Module<'a>) {
    for mem in m.mems.borrow().iter() {
        if mem.read_ports.borrow().is_empty() {
            panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a memory called \"{}\" which doesn't have any read ports.", root.name, m.name, mem.name);
        }

        if mem.initial_contents.borrow().is_none() && mem.write_port.borrow().is_none() {
            panic!("Cannot generate code for module \"{}\" because module \"{}\" contains a memory called \"{}\" which doesn't have initial contents or a write port specified. At least one of the two is required.", root.name, m.name, mem.name);
        }
    }

    for module in m.modules.borrow().iter() {
        detect_mem_errors(module, root);
    }
}

fn detect_combinational_loops<'a>(m: &graph::Module<'a>, root: &graph::Module<'a>) {
    for module in m.modules.borrow().iter() {
        for (_, output) in module.outputs.borrow().iter() {
            trace_signal(output.data.source, output.data.source, root);
        }

        detect_combinational_loops(module, root);
    }
}

fn trace_signal<'a>(
    signal: &'a internal_signal::InternalSignal<'a>,
    source_output: &'a internal_signal::InternalSignal<'a>,
    root: &graph::Module<'a>,
) {
    struct Frame<'a> {
        signal: &'a internal_signal::InternalSignal<'a>,
    }

    let mut frames = Vec::new();
    frames.push(Frame { signal });

    while let Some(frame) = frames.pop() {
        let signal = frame.signal;

        match signal.data {
            internal_signal::SignalData::Lit { .. } => (),

            internal_signal::SignalData::Input { data } => {
                if let Some(driven_value) = data.driven_value.borrow().clone() {
                    frames.push(Frame {
                        signal: driven_value,
                    });
                }
            }
            internal_signal::SignalData::Output { data } => {
                if data.source == source_output {
                    panic!("Cannot generate code for module \"{}\" because module \"{}\" contains an output called \"{}\" which forms a combinational loop with itself.", root.name, data.module.name, data.name);
                }
                frames.push(Frame {
                    signal: data.source,
                });
            }

            internal_signal::SignalData::Reg { .. } => (),

            internal_signal::SignalData::UnOp { ref source, .. } => {
                frames.push(Frame { signal: source });
            }
            internal_signal::SignalData::SimpleBinOp {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::AdditiveBinOp {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::ComparisonBinOp {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::ShiftBinOp {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }

            internal_signal::SignalData::Mul {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::MulSigned {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }

            internal_signal::SignalData::Bits { ref source, .. } => {
                frames.push(Frame { signal: source });
            }

            internal_signal::SignalData::Repeat { ref source, .. } => {
                frames.push(Frame { signal: source });
            }
            internal_signal::SignalData::Concat {
                ref lhs, ref rhs, ..
            } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }

            internal_signal::SignalData::Mux {
                ref cond,
                ref when_true,
                ref when_false,
                ..
            } => {
                frames.push(Frame { signal: cond });
                frames.push(Frame { signal: when_true });
                frames.push(Frame { signal: when_false });
            }

            internal_signal::SignalData::MemReadPortOutput { .. } => (),
        }
    }
}
