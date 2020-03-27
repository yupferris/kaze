use crate::graph;

use std::collections::HashMap;

pub struct InstanceDecls {
    pub input_names: HashMap<String, String>,
    pub output_names: HashMap<String, String>,
}

pub struct MemDecls<'a> {
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

pub struct RegisterDecls<'a> {
    pub(super) data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub struct ModuleDecls<'a> {
    pub instances: HashMap<&'a graph::Instance<'a>, InstanceDecls>,
    pub mems: HashMap<&'a graph::Mem<'a>, MemDecls<'a>>,
    pub regs: HashMap<&'a graph::Signal<'a>, RegisterDecls<'a>>,
}
