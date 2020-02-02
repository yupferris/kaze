use super::context::*;
use super::module::*;
use super::signal::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ptr;

/// An instance of a [`Module`], created by the [`Module`]::[`instance`] method.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// // Inner module (simple pass-through)
/// let inner = c.module("Inner");
/// inner.output("o", inner.input("i", 32));
///
/// // Outer module (wraps a single `Inner` instance)
/// let outer = c.module("Outer");
/// let inner_inst = outer.instance("inner_inst", "Inner");
/// inner_inst.drive_input("i", outer.input("i", 32));
/// outer.output("o", inner_inst.output("o"));
/// ```
///
/// [`instance`]: ./struct.Module.html#method.instance
/// [`Module`]: ./struct.Module.html
#[must_use]
pub struct Instance<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) module: &'a Module<'a>,

    pub(crate) instantiated_module: &'a Module<'a>,
    pub(crate) name: String,
    pub(crate) driven_inputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
}

impl<'a> Instance<'a> {
    /// Drives the input of this [`Module`] `Instance` specified by `name` with the given [`Signal`].
    ///
    /// # Panics
    ///
    /// Panics if `i` is from a different [`Module`] than `self`, if `name` specifies an input that doesn't exist on this `Instance`'s [`Module`], if this input is already driven on this `Instance`, or if `i`'s bit width differs from that of the input.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let inner = c.module("Inner");
    /// inner.output("o", inner.input("i", 32));
    ///
    /// let outer = c.module("Outer");
    /// let inner_inst = outer.instance("inner_inst", "Inner");
    /// // Drive inner_inst's "i" input with a 32-bit literal
    /// inner_inst.drive_input("i", outer.lit(0xfadebabeu32, 32));
    /// ```
    ///
    /// [`Module`]: ./struct.Module.html
    /// [`Signal`]: ./struct.Signal.html
    pub fn drive_input<S: Into<String>>(&'a self, name: S, i: &'a Signal<'a>) {
        let name = name.into();
        let mut driven_inputs = self.driven_inputs.borrow_mut();
        if !ptr::eq(self.module, i.module) {
            panic!("Attempted to drive an instance input with a signal from a different module.");
        }
        if !self.instantiated_module.inputs.borrow().contains_key(&name) {
            panic!("Attempted to drive an input called \"{}\" on an instance of \"{}\", but no such input with this name exists on this module.", name, self.instantiated_module.name);
        }
        if driven_inputs.contains_key(&name) {
            panic!("Attempted to drive an input called \"{}\" on an instance of \"{}\", but this input is already driven for this instance.", name, self.instantiated_module.name);
        }
        let input_bit_width = self.instantiated_module.inputs.borrow()[&name].bit_width();
        if input_bit_width != i.bit_width() {
            panic!("Attempted to drive an input called \"{}\" on an instance of \"{}\", but this input and the provided signal have different bit widths ({} and {}, respectively).", name, self.instantiated_module.name, input_bit_width, i.bit_width());
        }
        driven_inputs.insert(name, i);
    }

    /// Creates a [`Signal`] that represents this `Instance`'s output called `name`.
    ///
    /// # Panics
    ///
    /// Panics if `name` specifies an output that doesn't exist on this `Instance`'s [`Module`].
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let inner = c.module("Inner");
    /// inner.output("o", inner.lit(true, 1));
    ///
    /// let outer = c.module("Outer");
    /// let inner_inst = outer.instance("inner_inst", "Inner");
    /// // Forward inner_inst's "o" output to a new output on outer with the same name
    /// outer.output("o", inner_inst.output("o"));
    /// ```
    ///
    /// [`Module`]: ./struct.Module.html
    /// [`Signal`]: ./struct.Signal.html
    pub fn output<S: Into<String>>(&'a self, name: S) -> &Signal<'a> {
        let name = name.into();
        if !self
            .instantiated_module
            .outputs
            .borrow()
            .contains_key(&name)
        {
            panic!("Attempted to create a signal for an output called \"{}\" on an instance of \"{}\", but no such output with this name exists on this module.", name, self.instantiated_module.name);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::InstanceOutput {
                instance: self,
                name,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Attempted to drive an instance input with a signal from a different module."
    )]
    fn drive_input_different_module_error() {
        let c = Context::new();

        let inner = c.module("Inner");
        let _ = inner.input("a", 1);

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let inner_inst = m2.instance("inner_inst", "Inner");

        // Panic
        inner_inst.drive_input("a", i1);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive an input called \"a\" on an instance of \"Inner\", but no such input with this name exists on this module."
    )]
    fn drive_input_nonexistent_input_error() {
        let c = Context::new();

        let _ = c.module("Inner");

        let m = c.module("A");
        let inner_inst = m.instance("inner_inst", "Inner");

        // Panic
        inner_inst.drive_input("a", m.input("i", 1));
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive an input called \"a\" on an instance of \"Inner\", but this input is already driven for this instance."
    )]
    fn drive_input_already_driven_error() {
        let c = Context::new();

        let inner = c.module("Inner");
        let _ = inner.input("a", 1);

        let m = c.module("A");
        let inner_inst = m.instance("inner_inst", "Inner");

        inner_inst.drive_input("a", m.input("i1", 1));

        // Panic
        inner_inst.drive_input("a", m.input("i2", 1));
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive an input called \"a\" on an instance of \"Inner\", but this input and the provided signal have different bit widths (1 and 32, respectively)."
    )]
    fn drive_input_incompatible_bit_widths_error() {
        let c = Context::new();

        let inner = c.module("Inner");
        let _ = inner.input("a", 1);

        let m = c.module("A");
        let inner_inst = m.instance("inner_inst", "Inner");

        // Panic
        inner_inst.drive_input("a", m.input("i1", 32));
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create a signal for an output called \"nope\" on an instance of \"Inner\", but no such output with this name exists on this module."
    )]
    fn output_nonexistent_output_error() {
        let c = Context::new();

        let _ = c.module("Inner");

        let m = c.module("A");
        let inner_inst = m.instance("inner_inst", "Inner");

        // Panic
        let _ = inner_inst.output("nope");
    }
}
