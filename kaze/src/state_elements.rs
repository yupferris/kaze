use crate::graph;
use crate::graph::internal_signal;

use std::collections::HashMap;

pub(super) struct Register<'a> {
    pub data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub(super) struct Mem<'a> {
    pub mem: &'a graph::Mem<'a>,
    pub mem_name: String,
    pub read_signal_names: HashMap<
        (
            &'a internal_signal::InternalSignal<'a>,
            &'a internal_signal::InternalSignal<'a>,
        ),
        ReadSignalNames,
    >,
    pub write_address_name: String,
    pub write_value_name: String,
    pub write_enable_name: String,
}

pub struct ReadSignalNames {
    pub address_name: String,
    pub enable_name: String,
    pub value_name: String,
}

// TODO: Move?
// TODO: Cover registers as well
#[derive(Clone, Copy)]
pub(super) enum IncludedPorts {
    All,
    ReachableFromTopLevelOutputs,
}

pub(super) struct StateElements<'a> {
    pub mems: HashMap<&'a graph::Mem<'a>, Mem<'a>>,
    pub regs: HashMap<&'a internal_signal::InternalSignal<'a>, Register<'a>>,
}

impl<'a> StateElements<'a> {
    pub fn new(
        m: &'a graph::Module<'a>,
        // TODO: Cover registers as well
        included_ports: IncludedPorts,
        signal_reference_counts: &mut HashMap<&'a internal_signal::InternalSignal<'a>, u32>,
    ) -> StateElements<'a> {
        let mut mems = HashMap::new();
        let mut regs = HashMap::new();

        visit_module(
            m,
            included_ports,
            &mut mems,
            &mut regs,
            signal_reference_counts,
        );

        StateElements { mems, regs }
    }
}

fn visit_module<'a>(
    m: &'a graph::Module<'a>,
    included_ports: IncludedPorts,
    mems: &mut HashMap<&'a graph::Mem<'a>, Mem<'a>>,
    regs: &mut HashMap<&'a internal_signal::InternalSignal<'a>, Register<'a>>,
    signal_reference_counts: &mut HashMap<&'a internal_signal::InternalSignal<'a>, u32>,
) {
    match included_ports {
        // TODO: Match all of these with traces
        // TODO: Test
        IncludedPorts::All => {
            for (_, &input) in m.inputs.borrow().iter() {
                visit_signal(input.value, mems, regs, signal_reference_counts);
            }
            for (_, &output) in m.outputs.borrow().iter() {
                visit_signal(output.data.source, mems, regs, signal_reference_counts);
            }
            for &register in m.registers.borrow().iter() {
                match register.data {
                    internal_signal::SignalData::Reg { ref data } => {
                        visit_signal(
                            data.next.borrow().unwrap(),
                            mems,
                            regs,
                            signal_reference_counts,
                        );
                    }
                    _ => unreachable!(),
                }
            }
            for &module in m.modules.borrow().iter() {
                visit_module(module, included_ports, mems, regs, signal_reference_counts);
            }
            // TODO: Cover all mems as well
        }
        IncludedPorts::ReachableFromTopLevelOutputs => {
            for (_, &output) in m.outputs.borrow().iter() {
                visit_signal(output.data.source, mems, regs, signal_reference_counts);
            }
        }
    }
}

// TODO: Move this to ctor and iterate over input module outputs there?
fn visit_signal<'a>(
    signal: &'a internal_signal::InternalSignal<'a>,
    mems: &mut HashMap<&'a graph::Mem<'a>, Mem<'a>>,
    regs: &mut HashMap<&'a internal_signal::InternalSignal<'a>, Register<'a>>,
    signal_reference_counts: &mut HashMap<&'a internal_signal::InternalSignal<'a>, u32>,
) {
    // TODO: Do we even need this with just the one member?
    struct Frame<'a> {
        signal: &'a internal_signal::InternalSignal<'a>,
    }

    let mut frames = Vec::new();
    frames.push(Frame { signal });

    while let Some(frame) = frames.pop() {
        let signal = frame.signal;

        let reference_count = signal_reference_counts.entry(signal).or_insert(0);
        *reference_count += 1;

        if *reference_count > 1 {
            continue;
        }

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
                frames.push(Frame {
                    signal: data.source,
                });
            }

            internal_signal::SignalData::Reg { data } => {
                let key = signal;
                let value_name = format!("__reg_{}_{}", data.name, regs.len());
                let next_name = format!("{}_next", value_name);
                regs.insert(
                    key,
                    Register {
                        data,
                        value_name,
                        next_name,
                    },
                );
                frames.push(Frame {
                    signal: data.next.borrow().unwrap(),
                });
            }

            internal_signal::SignalData::UnOp { source, .. } => {
                frames.push(Frame { signal: source });
            }
            internal_signal::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }

            internal_signal::SignalData::Mul { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }
            internal_signal::SignalData::MulSigned { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }

            internal_signal::SignalData::Bits { source, .. } => {
                frames.push(Frame { signal: source });
            }

            internal_signal::SignalData::Repeat { source, .. } => {
                frames.push(Frame { signal: source });
            }
            internal_signal::SignalData::Concat { lhs, rhs, .. } => {
                frames.push(Frame { signal: lhs });
                frames.push(Frame { signal: rhs });
            }

            internal_signal::SignalData::Mux {
                cond,
                when_true,
                when_false,
                ..
            } => {
                frames.push(Frame { signal: cond });
                frames.push(Frame { signal: when_true });
                frames.push(Frame { signal: when_false });
            }

            internal_signal::SignalData::MemReadPortOutput { mem, .. } => {
                let key = mem;
                let mem_name = format!("{}_{}", mem.name, mems.len());
                // TODO: It might actually be too conservative to trace all read ports,
                //  as we only know that the write port and _this_ read port are reachable
                //  at this point, but we have to keep some extra state to know whether or
                //  not we've hit each read port otherwise.
                let mut read_signal_names = HashMap::new();
                for (index, (address, enable)) in mem.read_ports.borrow().iter().enumerate() {
                    let name_prefix = format!("{}_read_port_{}_", mem_name, index);
                    read_signal_names.insert(
                        (*address, *enable),
                        ReadSignalNames {
                            address_name: format!("{}address", name_prefix),
                            enable_name: format!("{}enable", name_prefix),
                            value_name: format!("{}value", name_prefix),
                        },
                    );
                }
                let name_prefix = format!("{}_write_port_", mem_name);
                let write_address_name = format!("{}address", name_prefix);
                let write_value_name = format!("{}value", name_prefix);
                let write_enable_name = format!("{}enable", name_prefix);
                mems.insert(
                    key,
                    Mem {
                        mem,
                        mem_name,
                        write_address_name,
                        write_value_name,
                        write_enable_name,
                        read_signal_names,
                    },
                );
                for (address, enable) in mem.read_ports.borrow().iter() {
                    frames.push(Frame { signal: address });
                    frames.push(Frame { signal: enable });
                }
                if let Some((address, value, enable)) = *mem.write_port.borrow() {
                    frames.push(Frame { signal: address });
                    frames.push(Frame { signal: value });
                    frames.push(Frame { signal: enable });
                }
            }
        }
    }
}
