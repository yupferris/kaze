use super::internal_signal::*;
use super::mem::*;
use super::module::*;
use super::register::*;

use typed_arena::Arena;

use std::cell::RefCell;

// TODO: Move, doc
pub trait ModuleParent<'a> {
    // TODO: Update doc
    /// Creates a new [`Module`] called `name` in this `Context`.
    ///
    /// Conventionally, `name` should be `CamelCase`, though this is not enforced.
    ///
    /// # Panics
    ///
    /// Panics if a [`Module`] with the same `name` already exists in this `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let my_module = c.module("my_module", "MyModule");
    /// let another_mod = c.module("another_mod", "AnotherMod");
    /// ```
    ///
    /// The following example panics by creating a `Module` with the same `name` as a previously-created `Module` in the same `Context`:
    ///
    /// ```should_panic
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let _ = c.module("a", "A"); // Unique name, OK
    /// let _ = c.module("b", "B"); // Unique name, OK
    ///
    /// let _ = c.module("a", "A"); // Non-unique name, panic!
    /// ```
    fn module(&'a self, instance_name: impl Into<String>, name: impl Into<String>) -> &Module;
}

/// A top-level container/owner object for a [`Module`] graph.
///
/// A `Context` owns all parts of a module graph, and provides an API for creating [`Module`] objects.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// let m = c.module("m", "MyModule");
/// m.output("out", m.input("in", 1));
/// ```
#[must_use]
pub struct Context<'a> {
    pub(super) module_arena: Arena<Module<'a>>,
    pub(super) input_data_arena: Arena<InputData<'a>>,
    pub(super) input_arena: Arena<Input<'a>>,
    pub(super) output_data_arena: Arena<OutputData<'a>>,
    pub(super) output_arena: Arena<Output<'a>>,
    pub(super) signal_arena: Arena<InternalSignal<'a>>,
    pub(super) register_data_arena: Arena<RegisterData<'a>>,
    pub(super) register_arena: Arena<Register<'a>>,
    pub(super) mem_arena: Arena<Mem<'a>>,

    pub(super) modules: RefCell<Vec<&'a Module<'a>>>,
}

impl<'a> Context<'a> {
    /// Creates a new, empty `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    /// ```
    pub fn new() -> Context<'a> {
        Context {
            module_arena: Arena::new(),
            input_data_arena: Arena::new(),
            input_arena: Arena::new(),
            output_data_arena: Arena::new(),
            output_arena: Arena::new(),
            signal_arena: Arena::new(),
            register_data_arena: Arena::new(),
            register_arena: Arena::new(),
            mem_arena: Arena::new(),

            modules: RefCell::new(Vec::new()),
        }
    }
}

impl<'a> ModuleParent<'a> for Context<'a> {
    // TODO: Docs, error handling
    fn module(&'a self, instance_name: impl Into<String>, name: impl Into<String>) -> &Module {
        let instance_name = instance_name.into();
        let name = name.into();
        let module = self
            .module_arena
            .alloc(Module::new(self, None, instance_name, name));
        self.modules.borrow_mut().push(module);
        module
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_has_no_modules() {
        let c = Context::new();

        assert!(c.modules.borrow().is_empty());
    }
}
