use super::constant::*;
use super::context::*;
use super::instance::*;
use super::mem::*;
use super::register::*;
use super::signal::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ptr;
use std::collections::btree_map::Entry;

/// A self-contained and potentially-reusable hardware design unit, created by the [`Context::module`] method.
///
/// Once a `Module` is specified, it can be [instantiated](Self::instance) in another `Module` to form a hierarchy, or it can be used to generate [Rust simulator code](crate::sim::generate) or a [Verilog module](crate::verilog::generate).
///
/// All `Module`s in kaze have an implicit reset and clock. These are only visible in generated code. It's assumed that all kaze modules operate in the same clock domain.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// let m = c.module("MyModule");
/// m.output("out", m.input("in", 1));
/// ```
// TODO: Validation error if a module has no inputs/outputs
#[must_use]
pub struct Module<'a> {
    context: &'a Context<'a>,

    pub(crate) name: String,

    pub(crate) inputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
    pub(crate) outputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
    pub(crate) registers: RefCell<BTreeMap<String, &'a Signal<'a>>>,
    pub(crate) instances: RefCell<BTreeMap<String, &'a Instance<'a>>>,
    pub(crate) mems: RefCell<BTreeMap<String, &'a Mem<'a>>>,
}

impl<'a> Module<'a> {
    pub(super) fn new(context: &'a Context<'a>, name: String) -> Module<'a> {
        Module {
            context,

            name,

            inputs: Default::default(),
            outputs: Default::default(),
            registers: Default::default(),
            instances: Default::default(),
            mems: Default::default(),
        }
    }

    /// Creates a [`Signal`] that represents the constant literal specified by `value` with `bit_width` bits.
    ///
    /// The bit width of the type provided by `value` doesn't need to match `bit_width`, but the value represented by `value` must fit into `bit_width` bits.
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
    /// let m = c.module("MyModule");
    ///
    /// let eight_bit_const = m.lit(0xffu32, 8);
    /// let one_bit_const = m.lit(0u32, 1);
    /// let twenty_seven_bit_const = m.lit(true, 27);
    /// ```
    pub fn lit<C: Into<Constant>>(&'a self, value: C, bit_width: u32) -> &Signal<'a> {
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
    /// let m = c.module("MyModule");
    ///
    /// // The following two signals are semantically equivalent:
    /// let low1 = m.low();
    /// let low2 = m.lit(false, 1);
    /// ```
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
    /// let m = c.module("MyModule");
    ///
    /// // The following two signals are semantically equivalent:
    /// let high1 = m.high();
    /// let high2 = m.lit(true, 1);
    /// ```
    pub fn high(&'a self) -> &Signal<'a> {
        self.lit(true, 1)
    }

    /// Creates an input for this `Module` called `name` with `bit_width` bits, and returns a [`Signal`] that represents the value of this input.
    ///
    /// # Panics
    ///
    /// Panics if `bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively,
    /// or if the name of the input is already used in the module.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let my_input = m.input("my_input", 80);
    /// ```
    pub fn input<S: Into<String>>(&'a self, name: S, bit_width: u32) -> &Signal<'a> {
        let name = name.into();
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
        let mut map = self.inputs.borrow_mut();
        match map.entry(name.clone()) {
            Entry::Vacant(v) => {
                let input = self.context.signal_arena.alloc(Signal {
                    context: self.context,
                    module: self,

                    data: SignalData::Input {
                        name,
                        bit_width,
                    },
                });


                v.insert(input);

                input
            }
            Entry::Occupied(_) => {
                panic!("Cannot create an input with a name that already exists in this module.")
            }
        }
    }

    /// Creates an output for this `Module` called `name` with the same number of bits as `source`, and drives this output with `source`.
    ///
    /// # Panics
    ///
    /// Panics of `source` doesn't belong to this `Module` or if the name of the input is already used in the module.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let some_signal = m.high();
    /// m.output("my_output", some_signal);
    /// ```
    pub fn output<S: Into<String>>(&'a self, name: S, source: &'a Signal<'a>) {
        if !ptr::eq(self, source.module) {
            panic!("Cannot output a signal from another module.");
        }

        let mut map = self.outputs.borrow_mut();
        match map.entry(name.into()) {
            Entry::Vacant(v) => {
                v.insert(source);
            }
            Entry::Occupied(_) => {
                panic!("Cannot create an output with a name that already exists in this module.")
            }
        }
    }

    /// Creates a [`Register`] in this `Module` called `name` with `bit_width` bits.
    ///
    /// # Panics
    ///
    /// Panics if `bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively,
    /// or when the name already exists in the module.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let my_reg = m.reg("my_reg", 32);
    /// my_reg.default_value(0xfadebabeu32); // Optional
    /// my_reg.drive_next(!my_reg.value);
    /// m.output("my_output", my_reg.value);
    /// ```
    pub fn reg<S: Into<String>>(&'a self, name: S, bit_width: u32) -> &Register<'a> {
        if bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a register with {} bit(s). Signals must not be narrower than {} bit(s).",
                bit_width, MIN_SIGNAL_BIT_WIDTH
            );
        }
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a register with {} bit(s). Signals must not be wider than {} bit(s).",
                bit_width, MAX_SIGNAL_BIT_WIDTH
            );
        }
        let name = name.into();

        let mut map = self.registers.borrow_mut();
        match map.entry(name.clone()) {
            Entry::Vacant(v) => {
                let data = self.context.register_data_arena.alloc(RegisterData {
                    module: self,

                    name: name.clone(),
                    initial_value: RefCell::new(None),
                    bit_width,
                    next: RefCell::new(None),
                });
                let value = self.context.signal_arena.alloc(Signal {
                    context: self.context,
                    module: self,

                    data: SignalData::Reg { data },
                });

                v.insert(value);

                self.context.register_arena.alloc(Register { data, value })
            }
            Entry::Occupied(_) => {
                panic!("Cannot create a register with a name that already exists in this module.")
            }
        }
    }

    /// Creates a 2:1 [multiplexer](https://en.wikipedia.org/wiki/Multiplexer) that represents `when_true`'s value when `cond` is high, and `when_false`'s value when `cond` is low.
    ///
    /// # Panics
    ///
    /// Panics if `cond`, `when_true`, or `when_false` belong to a different `Module` than `self`, if `cond`'s bit width is not 1, or if the bit widths of `when_true` and `when_false` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let cond = m.input("cond", 1);
    /// let a = m.input("a", 8);
    /// let b = m.input("b", 8);
    /// m.output("my_output", m.mux(cond, a, b)); // Outputs a when cond is high, b otherwise
    /// ```
    pub fn mux(
        &'a self,
        cond: &'a Signal<'a>,
        when_true: &'a Signal<'a>,
        when_false: &'a Signal<'a>,
    ) -> &Signal<'a> {
        // TODO: This is an optimization to support sugar; if that doesn't go well, remove this
        if when_true == when_false {
            return when_true;
        }

        if !ptr::eq(self, cond.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if !ptr::eq(self, when_true.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if !ptr::eq(self, when_false.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if cond.bit_width() != 1 {
            panic!("Multiplexer conditionals can only be 1 bit wide.");
        }
        if when_true.bit_width() != when_false.bit_width() {
            panic!(
                "Cannot multiplex signals with different bit widths ({} and {}, respectively).",
                when_true.bit_width(),
                when_false.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Mux {
                cond,
                when_true,
                when_false,
                bit_width: when_true.bit_width(),
            },
        })
    }

    /// Creates an [`Instance`] called `instance_name` of the `Module` identified by `module_name` in this [`Context`] inside this `Module` definition.
    ///
    /// # Panics
    ///
    /// Panics if a `Module` identified by `module_name` doesn't exist in this [`Context`] or when the instance name already exists in this module.
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
    /// // Outer module (wraps a single `inner` instance)
    /// let outer = c.module("Outer");
    /// let inner_inst = outer.instance("inner_inst", "Inner");
    /// inner_inst.drive_input("i", outer.input("i", 32));
    /// outer.output("o", inner_inst.output("o"));
    /// ```
    pub fn instance<S: Into<String>>(
        &'a self,
        instance_name: S,
        module_name: &str,
    ) -> &Instance<'a> {
        match self.context.modules.borrow().get(module_name) {
            Some(instantiated_module) => {
                let instance_name = instance_name.into();

                let mut map = self.instances.borrow_mut();
                match map.entry(instance_name.clone()) {
                    Entry::Vacant(v) => {
                        let ret = self.context.instance_arena.alloc(Instance {
                            context: self.context,
                            module: self,

                            instantiated_module,
                            name: instance_name,
                            driven_inputs: Default::default(),
                        });
                        v.insert(ret);
                        ret
                    }
                    Entry::Occupied(_) => {
                        panic!("Cannot create an instance with a name that already exists in this module.")
                    }
                }
            }
            _ => panic!("Attempted to instantiate a module identified by \"{}\", but no such module exists in this context.", module_name)
        }
    }

    /// Creates a [`Mem`] in this `Module` called `name` with `address_bit_width` address bits and `element_bit_width` element bits.
    ///
    /// The size of this memory will be `1 << address_bit_width` elements, each `element_bit_width` bits wide.
    ///
    /// # Panics
    ///
    /// Panics if `address_bit_width` or `element_bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively,
    /// or when the memory name already exists in this module.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let my_mem = m.mem("my_mem", 1, 32);
    /// // Optional, unless no write port is specified
    /// my_mem.initial_contents(&[0xfadebabeu32, 0xdeadbeefu32]);
    /// // Optional, unless no initial contents are specified
    /// my_mem.write_port(m.high(), m.lit(0xabad1deau32, 32), m.high());
    /// m.output("my_output", my_mem.read_port(m.high(), m.high()));
    /// ```
    pub fn mem<S: Into<String>>(
        &'a self,
        name: S,
        address_bit_width: u32,
        element_bit_width: u32,
    ) -> &Mem<'a> {
        if address_bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a memory with {} address bit(s). Signals must not be narrower than {} bit(s).",
                address_bit_width, MIN_SIGNAL_BIT_WIDTH
            );
        }
        if address_bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a memory with {} address bit(s). Signals must not be wider than {} bit(s).",
                address_bit_width, MAX_SIGNAL_BIT_WIDTH
            );
        }
        if element_bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a memory with {} element bit(s). Signals must not be narrower than {} bit(s).",
                element_bit_width, MIN_SIGNAL_BIT_WIDTH
            );
        }
        if element_bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!(
                "Cannot create a memory with {} element bit(s). Signals must not be wider than {} bit(s).",
                element_bit_width, MAX_SIGNAL_BIT_WIDTH
            );
        }
        let name = name.into();

        let mut map = self.mems.borrow_mut();
        match map.entry(name.clone()) {
            Entry::Vacant(v) => {
                let ret = self.context.mem_arena.alloc(Mem {
                    context: self.context,
                    module: self,

                    name,
                    address_bit_width,
                    element_bit_width,

                    initial_contents: RefCell::new(None),

                    read_ports: RefCell::new(Vec::new()),
                    write_port: RefCell::new(None),
                });

                v.insert(ret);
                ret
            }
            Entry::Occupied(_) => {
                panic!("Cannot create a memory with a name that already exists in this module.")
            }
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

        let m = c.module("A");

        // Panic
        let _ = m.lit(false, 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a literal with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn lit_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.lit(false, 129);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into the specified bit width '7'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_1() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.lit(128u32, 7);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into the specified bit width '2'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_2() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.lit(128u64, 2);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '1023' into the specified bit width '4'. The value '1023' requires a bit width of at least 10 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_3() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.lit(1023u128, 4);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '65536' into the specified bit width '1'. The value '65536' requires a bit width of at least 17 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_4() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.lit(65536u32, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create an input with 0 bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn input_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.input("i", 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create an input with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn input_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.input("i", 129);
    }

    #[test]
    #[should_panic(expected = "Cannot output a signal from another module.")]
    fn output_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");

        let m2 = c.module("B");
        let i = m2.high();

        // Panic
        m1.output("a", i);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a register with 0 bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn reg_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.reg("r", 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a register with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn reg_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.reg("r", 129);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mux_cond_separate_module_error() {
        let c = Context::new();

        let a = c.module("A");
        let l1 = a.lit(false, 1);

        let b = c.module("B");
        let l2 = b.lit(32u8, 8);
        let l3 = b.lit(32u8, 8);

        // Panic
        let _ = b.mux(l1, l2, l3);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mux_when_true_separate_module_error() {
        let c = Context::new();

        let a = c.module("A");
        let l1 = a.lit(32u8, 8);

        let b = c.module("B");
        let l2 = b.lit(true, 1);
        let l3 = b.lit(32u8, 8);

        // Panic
        let _ = b.mux(l2, l1, l3);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mux_when_false_separate_module_error() {
        let c = Context::new();

        let a = c.module("A");
        let l1 = a.lit(32u8, 8);

        let b = c.module("B");
        let l2 = b.lit(true, 1);
        let l3 = b.lit(32u8, 8);

        // Panic
        let _ = b.mux(l2, l3, l1);
    }

    #[test]
    #[should_panic(expected = "Multiplexer conditionals can only be 1 bit wide.")]
    fn mux_cond_bit_width_error() {
        let c = Context::new();

        let a = c.module("A");
        let l1 = a.lit(2u8, 2);
        let l2 = a.lit(32u8, 8);
        let l3 = a.lit(32u8, 8);

        // Panic
        let _ = a.mux(l1, l2, l3);
    }

    #[test]
    #[should_panic(
        expected = "Cannot multiplex signals with different bit widths (3 and 5, respectively)."
    )]
    fn mux_true_false_bit_width_error() {
        let c = Context::new();

        let a = c.module("A");
        let l1 = a.lit(false, 1);
        let l2 = a.lit(3u8, 3);
        let l3 = a.lit(3u8, 5);

        // Panic
        let _ = a.mux(l1, l2, l3);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to instantiate a module identified by \"nope\", but no such module exists in this context."
    )]
    fn instantiate_nonexistent_module_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.instance("lol", "nope");
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 0 address bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn mem_address_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.mem("mem", 0, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 129 address bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn mem_address_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.mem("mem", 129, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 0 element bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn mem_element_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.mem("mem", 1, 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 129 element bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn mem_element_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("A");

        // Panic
        let _ = m.mem("mem", 1, 129);
    }

    #[test]
    #[should_panic(
        expected="Cannot create an output with a name that already exists in this module."
    )]
    fn outputs_same_name() {
        let c = Context::new();
        let m = c.module("A");

        m.output("o", m.input("i1", 1));
        m.output("o", m.input("i2", 1));
    }

    #[test]
    #[should_panic(
        expected="Cannot create an input with a name that already exists in this module."
    )]
    fn inputs_same_name() {
        let c = Context::new();
        let m = c.module("A");

        m.output("o1", m.input("i", 1));
        m.output("o2", m.input("i", 1));
    }

    #[test]
    #[should_panic(
    expected="Cannot create a register with a name that already exists in this module."
    )]
    fn regs_same_name() {
        let c = Context::new();
        let m = c.module("A");

        let _ = m.reg("r", 1);
        let _ = m.reg("r", 1);
    }

    #[test]
    #[should_panic(
    expected="Cannot create an instance with a name that already exists in this module."
    )]
    fn instances_same_name() {
        let c = Context::new();
        let ma = c.module("A");
        let _ = c.module("B");

        let _ = ma.instance("i", "B");
        let _ = ma.instance("i", "B");
    }

    #[test]
    #[should_panic(
    expected="Cannot create a memory with a name that already exists in this module."
    )]
    fn memory_same_name() {
        let c = Context::new();
        let ma = c.module("A");

        let _ = ma.mem("m", 1, 1);
        let _ = ma.mem("m", 1, 1);
    }
}

