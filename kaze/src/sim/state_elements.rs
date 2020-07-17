use crate::graph;
use crate::module_context::*;

use typed_arena::Arena;

use std::collections::HashMap;

pub(super) struct Register<'a> {
    pub data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub(super) struct Mem<'a> {
    pub mem: &'a graph::Mem<'a>,
    pub mem_name: String,
    pub read_signal_names: HashMap<(&'a graph::Signal<'a>, &'a graph::Signal<'a>), ReadSignalNames>,
    pub write_address_name: String,
    pub write_value_name: String,
    pub write_enable_name: String,
}

pub struct ReadSignalNames {
    pub address_name: String,
    pub enable_name: String,
    pub value_name: String,
}

pub(super) struct StateElements<'graph, 'arena> {
    pub mems: HashMap<
        (
            &'arena ModuleContext<'graph, 'arena>,
            &'graph graph::Mem<'graph>,
        ),
        Mem<'graph>,
    >,
    pub regs: HashMap<
        (
            &'arena ModuleContext<'graph, 'arena>,
            &'graph graph::Signal<'graph>,
        ),
        Register<'graph>,
    >,
}

impl<'graph, 'arena> StateElements<'graph, 'arena> {
    pub fn new() -> StateElements<'graph, 'arena> {
        StateElements {
            mems: HashMap::new(),
            regs: HashMap::new(),
        }
    }

    pub fn gather(
        &mut self,
        signal: &'graph graph::Signal<'graph>,
        context: &'arena ModuleContext<'graph, 'arena>,
        context_arena: &'arena Arena<ModuleContext<'graph, 'arena>>,
    ) {
        match signal.data {
            graph::SignalData::Lit { .. } => (),

            graph::SignalData::Input { ref name, .. } => {
                if let Some((instance, parent)) = context.instance_and_parent {
                    self.gather(instance.driven_inputs.borrow()[name], parent, context_arena);
                }
            }

            graph::SignalData::Reg { data } => {
                let key = (context, signal);
                if self.regs.contains_key(&key) {
                    return;
                }
                let value_name = format!("__reg_{}_{}", data.name, self.regs.len());
                let next_name = format!("{}_next", value_name);
                self.regs.insert(
                    key,
                    Register {
                        data,
                        value_name,
                        next_name,
                    },
                );
                self.gather(data.next.borrow().unwrap(), context, context_arena);
            }

            graph::SignalData::UnOp { source, .. } => {
                self.gather(source, context, context_arena);
            }
            graph::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }
            graph::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }
            graph::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }
            graph::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }

            graph::SignalData::Mul { lhs, rhs } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }
            graph::SignalData::MulSigned { lhs, rhs } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }

            graph::SignalData::Bits { source, .. } => {
                self.gather(source, context, context_arena);
            }

            graph::SignalData::Repeat { source, .. } => {
                self.gather(source, context, context_arena);
            }
            graph::SignalData::Concat { lhs, rhs } => {
                self.gather(lhs, context, context_arena);
                self.gather(rhs, context, context_arena);
            }

            graph::SignalData::Mux {
                cond,
                when_true,
                when_false,
            } => {
                self.gather(cond, context, context_arena);
                self.gather(when_true, context, context_arena);
                self.gather(when_false, context, context_arena);
            }

            graph::SignalData::InstanceOutput { instance, ref name } => {
                let output = instance.instantiated_module.outputs.borrow()[name];
                let context = context.get_child(instance, context_arena);
                self.gather(output, context, context_arena);
            }

            graph::SignalData::MemReadPortOutput { mem, .. } => {
                let key = (context, mem);
                if self.mems.contains_key(&key) {
                    return;
                }
                let mem_name = format!("__mem_{}_{}", mem.name, self.mems.len());
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
                self.mems.insert(
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
                    self.gather(address, context, context_arena);
                    self.gather(enable, context, context_arena);
                }
                if let Some((address, value, enable)) = *mem.write_port.borrow() {
                    self.gather(address, context, context_arena);
                    self.gather(value, context, context_arena);
                    self.gather(enable, context, context_arena);
                }
            }
        }
    }
}
