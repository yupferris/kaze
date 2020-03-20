use crate::graph;

use std::collections::HashMap;

pub struct InstanceDecls {
    pub input_names: HashMap<String, String>,
    pub output_names: HashMap<String, String>,
}

pub struct RegisterDecls<'a> {
    pub(super) data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub struct ModuleDecls<'graph> {
    pub instances: HashMap<&'graph graph::Instance<'graph>, InstanceDecls>,
    pub regs: HashMap<&'graph graph::Signal<'graph>, RegisterDecls<'graph>>,
}
