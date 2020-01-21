use super::context::*;
use super::instance::*;
use super::register::*;
use super::signal::*;
use super::value::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ptr;

/// A self-contained and potentially-reusable hardware design unit, created by the [`Context`]::[`module`] method.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// let m = c.module("my_module");
/// m.output("out", m.input("in", 1));
/// ```
///
/// [`Context`]: ./struct.Context.html
/// [`module`]: ./struct.Context.html#method.module
#[must_use]
pub struct Module<'a> {
    context: &'a Context<'a>,

    pub(crate) name: String,

    pub(crate) inputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
    pub(crate) outputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
    pub(crate) registers: RefCell<Vec<&'a Signal<'a>>>,
    pub(crate) instances: RefCell<Vec<&'a Instance<'a>>>,
}

impl<'a> Module<'a> {
    pub(super) fn new(context: &'a Context<'a>, name: String) -> Module<'a> {
        Module {
            context,

            name,

            inputs: RefCell::new(BTreeMap::new()),
            outputs: RefCell::new(BTreeMap::new()),
            registers: RefCell::new(Vec::new()),
            instances: RefCell::new(Vec::new()),
        }
    }

    /// Creates a [`Signal`] that represents the constant literal specified by `value` with `bit_width` bits.
    ///
    /// The bit width of the type provided by `value` doesn't need to match `bit_width`. but the value represented by `value` must fit into `bit_width` bits.
    ///
    /// # Panics
    ///
    /// Panics if `bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively, or if the specified `value` doesn't fit into `bit_width` bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let eight_bit_const = m.lit(0xffu32, 8);
    /// let one_bit_const = m.lit(0u32, 1);
    /// let twenty_seven_bit_const = m.lit(true, 27);
    /// ```
    ///
    /// [`MIN_SIGNAL_BIT_WIDTH`]: ./constant.MIN_SIGNAL_BIT_WIDTH.html
    /// [`MAX_SIGNAL_BIT_WIDTH`]: ./constant.MAX_SIGNAL_BIT_WIDTH.html
    /// [`Signal`]: ./struct.Signal.html
    pub fn lit<V: Into<Value>>(&'a self, value: V, bit_width: u32) -> &Signal<'a> {
        if bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a literal with {} bit(s). Signals must not be narrower than {} bit(s).",
                bit_width, MIN_SIGNAL_BIT_WIDTH
            );
        }
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a literal with {} bit(s). Signals must not be wider than {} bit(s).",
                bit_width, MAX_SIGNAL_BIT_WIDTH
            );
        }
        let value = value.into();
        let required_bits = value.required_bits();
        if required_bits > bit_width {
            let numeric_value = value.numeric_value();
            panic!("Cannot fit the specified value '{}' into the specified bit width '{}'. The value '{}' requires a bit width of at least {} bit(s).", numeric_value, bit_width, numeric_value, required_bits);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Lit { value, bit_width },
        })
    }

    /// Convenience method to create a [`Signal`] that represents a single `0` bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// // The following two signals are semantically equivalent:
    /// let low1 = m.low();
    /// let low2 = m.lit(false, 1);
    /// ```
    ///
    /// [`Signal`]: ./struct.Signal.html
    pub fn low(&'a self) -> &Signal<'a> {
        self.lit(false, 1)
    }

    /// Convenience method to create a [`Signal`] that represents a single `1` bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// // The following two signals are semantically equivalent:
    /// let high1 = m.high();
    /// let high2 = m.lit(true, 1);
    /// ```
    ///
    /// [`Signal`]: ./struct.Signal.html
    pub fn high(&'a self) -> &Signal<'a> {
        self.lit(true, 1)
    }

    /// Creates an input for this `Module` called `name` with `bit_width` bits, and returns a [`Signal`] that represents the value of this input.
    ///
    /// # Panics
    ///
    /// Panics if `bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let my_input = m.input("my_input", 80);
    /// ```
    ///
    /// [`MIN_SIGNAL_BIT_WIDTH`]: ./constant.MIN_SIGNAL_BIT_WIDTH.html
    /// [`MAX_SIGNAL_BIT_WIDTH`]: ./constant.MAX_SIGNAL_BIT_WIDTH.html
    /// [`Signal`]: ./struct.Signal.html
    pub fn input<S: Into<String>>(&'a self, name: S, bit_width: u32) -> &Signal<'a> {
        let name = name.into();
        // TODO: Error if name already exists in this context
        if bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create an input with {} bit(s). Signals must not be narrower than {} bit(s).",
                bit_width, MIN_SIGNAL_BIT_WIDTH
            );
        }
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create an input with {} bit(s). Signals must not be wider than {} bit(s).",
                bit_width, MAX_SIGNAL_BIT_WIDTH
            );
        }
        let input = self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Input {
                name: name.clone(),
                bit_width,
            },
        });
        self.inputs.borrow_mut().insert(name, input);
        input
    }

    /// Creates an output for this `Module` called `name` with the same number of bits as `source`, and drives this output with `source`.
    ///
    /// # Panics
    ///
    /// Panics of `source` doesn't belong to this `Module`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let some_signal = m.high();
    /// m.output("my_output", some_signal);
    /// ```
    pub fn output<S: Into<String>>(&'a self, name: S, source: &'a Signal<'a>) {
        if !ptr::eq(self, source.module) {
            panic!("Cannot output a signal from another module.");
        }
        // TODO: Error if name already exists in this context
        self.outputs.borrow_mut().insert(name.into(), source);
    }

    /// Creates a [`Register`] in this `Module` called `name` with `bit_width` bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let my_reg = m.reg("my_reg", 32);
    /// my_reg.default_value(0xfadebabeu32);
    /// my_reg.drive_next(!my_reg.value);
    /// m.output("my_output", my_reg.value);
    /// ```
    ///
    /// [`Register`]: ./struct.Register.html
    pub fn reg<S: Into<String>>(&'a self, name: S, bit_width: u32) -> &Register<'a> {
        // TODO: bit_width bounds checks
        // TODO: Error if name already exists in this context
        let data = self.context.register_data_arena.alloc(RegisterData {
            name: name.into(),
            initial_value: RefCell::new(None),
            bit_width,
            next: RefCell::new(None),
        });
        let value = self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Reg { data },
        });
        self.registers.borrow_mut().push(value);
        self.context.register_arena.alloc(Register { data, value })
    }

    pub fn mux(
        &'a self,
        cond: &'a Signal<'a>,
        when_true: &'a Signal<'a>,
        when_false: &'a Signal<'a>,
    ) -> &Signal<'a> {
        // TODO: This is an optimization to support kaze_sugar; if that doesn't go well, remove this
        if ptr::eq(when_true, when_false) {
            return when_true;
        }

        // TODO: Ensure when_true and when_false have the same bit widths
        // TODO: Ensure sel is 1 bit wide
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Mux {
                cond,
                when_true,
                when_false,
            },
        })
    }

    /// Creates an [`Instance`] called `instance_name` of the `Module` identified by `module_name` in this [`Context`] inside this `Module` definition.
    ///
    /// # Panics
    ///
    /// Panics if a `Module` identified by `module_name` doesn't exist in this [`Context`].
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// // Inner module (simple pass-through)
    /// let inner = c.module("inner");
    /// inner.output("o", inner.input("i", 32));
    ///
    /// // Outer module (wraps a single `inner` instance)
    /// let outer = c.module("outer");
    /// let inner_inst = outer.instance("inner_inst", "inner");
    /// inner_inst.drive_input("i", outer.input("i", 32));
    /// outer.output("o", inner_inst.output("o"));
    /// ```
    ///
    /// [`Context`]: ./struct.Context.html
    /// [`Instance`]: ./struct.Instance.html
    pub fn instance<S: Into<String>>(
        &'a self,
        instance_name: S,
        module_name: &str,
    ) -> &Instance<'a> {
        // TODO: Error if instance_name already exists in this context
        match self.context.modules.borrow().get(module_name) {
            Some(instantiated_module) => {
                let ret = self.context.instance_arena.alloc(Instance {
                    context: self.context,
                    module: self,

                    instantiated_module,
                    name: instance_name.into(),
                    driven_inputs: RefCell::new(BTreeMap::new()),
                });
                self.instances.borrow_mut().push(ret);
                ret
            }
            _ => panic!("Attempted to instantiate a module identified by \"{}\", but no such module exists in this context.", module_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Cannot create a literal with 0 bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn lit_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(false, 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a literal with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn lit_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(false, 129);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into the specified bit width '7'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_1() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(128u32, 7);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into the specified bit width '2'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_2() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(128u64, 2);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '1023' into the specified bit width '4'. The value '1023' requires a bit width of at least 10 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_3() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(1023u128, 4);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '65536' into the specified bit width '1'. The value '65536' requires a bit width of at least 17 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_4() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(65536u32, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create an input with 0 bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn input_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.input("i", 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create an input with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn input_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.input("i", 129);
    }

    #[test]
    #[should_panic(expected = "Cannot output a signal from another module.")]
    fn output_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");

        let m2 = c.module("b");
        let i = m2.high();

        // Panic
        m1.output("a", i);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to instantiate a module identified by \"nope\", but no such module exists in this context."
    )]
    fn instantiate_nonexistent_module_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.instance("lol", "nope");
    }
}
