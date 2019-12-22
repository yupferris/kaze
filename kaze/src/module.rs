use typed_arena::Arena;

use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;
use std::ops::BitOr;
use std::ptr;

pub struct Context<'a> {
    module_arena: Arena<Module<'a>>,
    signal_arena: Arena<Signal<'a>>,
    output_arena: Arena<Output<'a>>,

    modules: RefCell<BTreeMap<String, &'a Module<'a>>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            module_arena: Arena::new(),
            signal_arena: Arena::new(),
            output_arena: Arena::new(),

            modules: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn module<S: Into<String>>(&'a self, name: S) -> &Module {
        let name = name.into();
        let mut modules = self.modules.borrow_mut();
        if modules.contains_key(&name) {
            panic!("A module with the name \"{}\" already exists in this context", name);
        }
        let module = self.module_arena.alloc(Module::new(self, name.clone()));
        modules.insert(name, module);
        module
    }
}

pub struct Module<'a> {
    context: &'a Context<'a>,

    pub name: String,

    inputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
    outputs: RefCell<BTreeMap<String, &'a Output<'a>>>,
}

impl<'a> Module<'a> {
    fn new(context: &'a Context<'a>, name: String) -> Module<'a> {
        Module {
            context,

            name,

            inputs: RefCell::new(BTreeMap::new()),
            outputs: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn inputs(&self) -> Ref<BTreeMap<String, &Signal<'a>>> {
        self.inputs.borrow()
    }

    pub fn outputs(&self) -> Ref<BTreeMap<String, &Output<'a>>> {
        self.outputs.borrow()
    }

    pub fn low(&'a self) -> &Signal<'a> {
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Low,
        })
    }

    pub fn high(&'a self) -> &Signal<'a> {
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::High,
        })
    }

    pub fn input<S: Into<String>>(&'a self, name: S, bit_width: u32) -> &Signal<'a> {
        let name = name.into();
        // TODO: Error if name already exists in this context
        let input = self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Input { name: name.clone(), bit_width },
        });
        self.inputs.borrow_mut().insert(name, input);
        input
    }

    pub fn output<S: Into<String>>(&'a self, name: S, source: &'a Signal<'a>) {
        if !ptr::eq(self, source.module) {
            panic!("Cannot output a signal from another module");
        }
        // TODO: Error if name already exists in this context
        let output = self.context.output_arena.alloc(Output {
            source,
        });
        self.outputs.borrow_mut().insert(name.into(), output);
    }
}

pub struct Signal<'a> {
    context: &'a Context<'a>,
    module: &'a Module<'a>,

    pub data: SignalData<'a>,
}

impl<'a> Signal<'a> {
    pub fn bit_width(&self) -> u32 {
        match &self.data {
            SignalData::Low | SignalData::High => 1,
            SignalData::Input { bit_width, .. } => *bit_width,
            SignalData::BinOp { lhs, .. } => lhs.bit_width(),
        }
    }
}

pub enum SignalData<'a> {
    // TODO: It probably make sense for these to alias to 1-bit literals instead to reduce cases
    Low,
    High,

    Input { name: String, bit_width: u32 },

    BinOp { lhs: &'a Signal<'a>, rhs: &'a Signal<'a>, op: BinOp }
}

impl<'a> BitOr for &'a Signal<'a> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!("Signals have different bit widths ({} and {}, respectively)", self.bit_width(), rhs.bit_width());
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp { lhs: self, rhs, op: BinOp::BitOr },
        })
    }
}

#[derive(Clone, Copy)]
pub enum BinOp {
    BitOr,
}

pub struct Output<'a> {
    pub source: &'a Signal<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "A module with the name \"a\" already exists in this context")]
    fn unique_module_names() {
        let c = Context::new();

        let _ = c.module("a"); // Unique name, OK
        let _ = c.module("b"); // Unique name, OK

        // Panic
        let _ = c.module("a");
    }

    #[test]
    #[should_panic(expected = "Cannot output a signal from another module")]
    fn output_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");

        let m2 = c.module("b");
        let i = m2.high();

        // Panic
        m1.output("a", i);
    }

    #[test]
    fn bitor() {
        let c = Context::new();

        let m = c.module("a");

        let lhs = m.low();
        let rhs = m.high();
        let _ = lhs | rhs;

        let lhs = m.input("a", 3);
        let rhs = m.input("b", 3);
        let _ = lhs | rhs;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules")]
    fn bitor_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1 | i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively)")]
    fn bitor_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 | i2;
    }
}
