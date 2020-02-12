use super::module_context::*;

use crate::graph;

use typed_arena::Arena;

use std::collections::HashMap;

#[derive(Clone)]
pub(super) struct Register<'a> {
    pub data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub(super) struct StateElements<'graph, 'arena> {
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
        }
    }
}
