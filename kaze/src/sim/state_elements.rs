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
        struct Node<'graph, 'arena> {
            signal: &'graph graph::Signal<'graph>,
            context: &'arena ModuleContext<'graph, 'arena>,
        }

        let mut nodes = Vec::new();
        nodes.push(Node { signal, context });

        while let Some(node) = nodes.pop() {
            let signal = node.signal;
            let context = node.context;

            match signal.data {
                graph::SignalData::Lit { .. } => (),

                graph::SignalData::Input { ref name, .. } => {
                    if let Some((instance, parent)) = context.instance_and_parent {
                        nodes.push(Node {
                            signal: instance.driven_inputs.borrow()[name],
                            context: parent,
                        });
                    }
                }

                graph::SignalData::Reg { data } => {
                    let key = (context, signal);
                    if self.regs.contains_key(&key) {
                        continue;
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
                    nodes.push(Node {
                        signal: data.next.borrow().unwrap(),
                        context,
                    });
                }

                graph::SignalData::UnOp { source, .. } => {
                    nodes.push(Node {
                        signal: source,
                        context,
                    });
                }
                graph::SignalData::SimpleBinOp { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }
                graph::SignalData::AdditiveBinOp { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }
                graph::SignalData::ComparisonBinOp { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }
                graph::SignalData::ShiftBinOp { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }

                graph::SignalData::Mul { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }
                graph::SignalData::MulSigned { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }

                graph::SignalData::Bits { source, .. } => {
                    nodes.push(Node {
                        signal: source,
                        context,
                    });
                }

                graph::SignalData::Repeat { source, .. } => {
                    nodes.push(Node {
                        signal: source,
                        context,
                    });
                }
                graph::SignalData::Concat { lhs, rhs, .. } => {
                    nodes.push(Node {
                        signal: lhs,
                        context,
                    });
                    nodes.push(Node {
                        signal: rhs,
                        context,
                    });
                }

                graph::SignalData::Mux {
                    cond,
                    when_true,
                    when_false,
                    ..
                } => {
                    nodes.push(Node {
                        signal: cond,
                        context,
                    });
                    nodes.push(Node {
                        signal: when_true,
                        context,
                    });
                    nodes.push(Node {
                        signal: when_false,
                        context,
                    });
                }

                graph::SignalData::InstanceOutput {
                    instance, ref name, ..
                } => {
                    let output = instance.instantiated_module.outputs.borrow()[name];
                    let context = context.get_child(instance, context_arena);
                    nodes.push(Node {
                        signal: output,
                        context,
                    });
                }

                graph::SignalData::MemReadPortOutput { mem, .. } => {
                    let key = (context, mem);
                    if self.mems.contains_key(&key) {
                        continue;
                    }
                    let mem_name = format!("{}_{}", mem.name, self.mems.len());
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
                        nodes.push(Node {
                            signal: address,
                            context,
                        });
                        nodes.push(Node {
                            signal: enable,
                            context,
                        });
                    }
                    if let Some((address, value, enable)) = *mem.write_port.borrow() {
                        nodes.push(Node {
                            signal: address,
                            context,
                        });
                        nodes.push(Node {
                            signal: value,
                            context,
                        });
                        nodes.push(Node {
                            signal: enable,
                            context,
                        });
                    }
                }
            }
        }
    }
}
