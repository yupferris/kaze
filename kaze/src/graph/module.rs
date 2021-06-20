use super::constant::*;
use super::context::*;
use super::internal_signal::*;
use super::mem::*;
use super::register::*;
use super::signal::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::ptr;

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
/// let m = c.module("m", "MyModule");
/// m.output("out", m.input("in", 1));
/// ```
// TODO: Validation error if a module has no inputs/outputs
// TODO: Document composing modules (even if it's really basic)
#[must_use]
pub struct Module<'a> {
    context: &'a Context<'a>,

    pub(crate) parent: Option<&'a Module<'a>>,

    pub(crate) instance_name: String,
    pub(crate) name: String,

    // TODO: Do we need to duplicate the input/output names here?
    pub(crate) inputs: RefCell<BTreeMap<String, &'a Input<'a>>>,
    pub(crate) outputs: RefCell<BTreeMap<String, &'a Output<'a>>>,
    pub(crate) registers: RefCell<Vec<&'a InternalSignal<'a>>>,
    pub(crate) modules: RefCell<Vec<&'a Module<'a>>>,
    pub(crate) mems: RefCell<Vec<&'a Mem<'a>>>,
}

impl<'a> Module<'a> {
    pub(super) fn new(
        context: &'a Context<'a>,
        parent: Option<&'a Module<'a>>,
        instance_name: String,
        name: String,
    ) -> Module<'a> {
        Module {
            context,

            parent,

            instance_name,
            name,

            inputs: RefCell::new(BTreeMap::new()),
            outputs: RefCell::new(BTreeMap::new()),
            registers: RefCell::new(Vec::new()),
            modules: RefCell::new(Vec::new()),
            mems: RefCell::new(Vec::new()),
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
    /// let m = c.module("m", "MyModule");
    ///
    /// let eight_bit_const = m.lit(0xffu32, 8);
    /// let one_bit_const = m.lit(0u32, 1);
    /// let twenty_seven_bit_const = m.lit(true, 27);
    /// ```
    pub fn lit(&'a self, value: impl Into<Constant>, bit_width: u32) -> &dyn Signal<'a> {
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
        self.context.signal_arena.alloc(InternalSignal {
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
    /// let m = c.module("m", "MyModule");
    ///
    /// // The following two signals are semantically equivalent:
    /// let low1 = m.low();
    /// let low2 = m.lit(false, 1);
    /// ```
    pub fn low(&'a self) -> &dyn Signal<'a> {
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
    /// let m = c.module("m", "MyModule");
    ///
    /// // The following two signals are semantically equivalent:
    /// let high1 = m.high();
    /// let high2 = m.lit(true, 1);
    /// ```
    pub fn high(&'a self) -> &dyn Signal<'a> {
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
    /// let m = c.module("m", "MyModule");
    ///
    /// let my_input = m.input("my_input", 80);
    /// ```
    pub fn input(&'a self, name: impl Into<String>, bit_width: u32) -> &Input<'a> {
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
        let data = self.context.input_data_arena.alloc(InputData {
            name: name.clone(),
            bit_width,
            driven_value: RefCell::new(None),
        });
        let value = self.context.signal_arena.alloc(InternalSignal {
            context: self.context,
            module: self,

            data: SignalData::Input { data },
        });
        let input = self.context.input_arena.alloc(Input {
            module: self,

            data,
            value,
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
    /// let m = c.module("m", "MyModule");
    ///
    /// let some_signal = m.high();
    /// m.output("my_output", some_signal);
    /// ```
    pub fn output(&'a self, name: impl Into<String>, source: &'a dyn Signal<'a>) -> &Output<'a> {
        let name = name.into();
        let source = source.internal_signal();
        if !ptr::eq(self, source.module) {
            panic!("Cannot output a signal from another module.");
        }
        // TODO: Error if name already exists in this context
        let data = self.context.output_data_arena.alloc(OutputData {
            module: self,

            name: name.clone(),
            source,
            bit_width: source.bit_width(),
        });
        let output = self.context.output_arena.alloc(Output { data });
        self.outputs.borrow_mut().insert(name, output);
        output
    }

    /// Creates a [`Register`] in this `Module` called `name` with `bit_width` bits.
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
    /// let m = c.module("m", "MyModule");
    ///
    /// let my_reg = m.reg("my_reg", 32);
    /// my_reg.default_value(0xfadebabeu32); // Optional
    /// my_reg.drive_next(!my_reg);
    /// m.output("my_output", my_reg);
    /// ```
    pub fn reg(&'a self, name: impl Into<String>, bit_width: u32) -> &Register<'a> {
        // TODO: Error if name already exists in this context and update docs for Signal::reg_next and Signal::reg_next_with_default to reflect this
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
        let data = self.context.register_data_arena.alloc(RegisterData {
            module: self,

            name: name.into(),
            initial_value: RefCell::new(None),
            bit_width,
            next: RefCell::new(None),
        });
        let value = self.context.signal_arena.alloc(InternalSignal {
            context: self.context,
            module: self,

            data: SignalData::Reg { data },
        });
        self.registers.borrow_mut().push(value);
        self.context.register_arena.alloc(Register { data, value })
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
    /// let m = c.module("m", "MyModule");
    ///
    /// let cond = m.input("cond", 1);
    /// let a = m.input("a", 8);
    /// let b = m.input("b", 8);
    /// m.output("my_output", m.mux(cond, a, b)); // Outputs a when cond is high, b otherwise
    /// ```
    pub fn mux(
        &'a self,
        cond: &'a dyn Signal<'a>,
        when_true: &'a dyn Signal<'a>,
        when_false: &'a dyn Signal<'a>,
    ) -> &dyn Signal<'a> {
        let cond = cond.internal_signal();
        let when_true = when_true.internal_signal();
        let when_false = when_false.internal_signal();

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
        self.context.signal_arena.alloc(InternalSignal {
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

    /// Creates a [`Mem`] in this `Module` called `name` with `address_bit_width` address bits and `element_bit_width` element bits.
    ///
    /// The size of this memory will be `1 << address_bit_width` elements, each `element_bit_width` bits wide.
    ///
    /// # Panics
    ///
    /// Panics if `address_bit_width` or `element_bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("m", "MyModule");
    ///
    /// let my_mem = m.mem("my_mem", 1, 32);
    /// // Optional, unless no write port is specified
    /// my_mem.initial_contents(&[0xfadebabeu32, 0xdeadbeefu32]);
    /// // Optional, unless no initial contents are specified
    /// my_mem.write_port(m.high(), m.lit(0xabad1deau32, 32), m.high());
    /// m.output("my_output", my_mem.read_port(m.high(), m.high()));
    /// ```
    pub fn mem(
        &'a self,
        name: impl Into<String>,
        address_bit_width: u32,
        element_bit_width: u32,
    ) -> &Mem<'a> {
        // TODO: Error if name already exists in this context
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
        let ret = self.context.mem_arena.alloc(Mem {
            context: self.context,
            module: self,

            name: name.into(),
            address_bit_width,
            element_bit_width,

            initial_contents: RefCell::new(None),

            read_ports: RefCell::new(Vec::new()),
            write_port: RefCell::new(None),
        });
        self.mems.borrow_mut().push(ret);
        ret
    }
}

impl<'a> ModuleParent<'a> for Module<'a> {
    // TODO: Docs, error handling
    fn module(&'a self, instance_name: impl Into<String>, name: impl Into<String>) -> &Module {
        let instance_name = instance_name.into();
        let name = name.into();
        let module = self.context.module_arena.alloc(Module::new(
            self.context,
            Some(self),
            instance_name,
            name,
        ));
        self.modules.borrow_mut().push(module);
        module
    }
}

impl<'a> Eq for &'a Module<'a> {}

impl<'a> Hash for &'a Module<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self as *const _ as usize)
    }
}

impl<'a> PartialEq for &'a Module<'a> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(*self, *other)
    }
}

// TODO: Move?
// TODO: Doc
// TODO: bit width as const generic param?
#[must_use]
pub struct Input<'a> {
    // TODO: Double-check that we need this (I think we need it for error checking in drive)
    pub(crate) module: &'a Module<'a>,

    pub(crate) data: &'a InputData<'a>,
    pub(crate) value: &'a InternalSignal<'a>,
}

impl<'a> Input<'a> {
    // TODO: Doc
    // TODO: Merge error cases with Instance::drive_input?
    // TODO: Rename i?
    pub fn drive(&'a self, i: &'a dyn Signal<'a>) {
        let i = i.internal_signal();
        // TODO: Change text from instance -> module in appropriate places?
        if let Some(parent) = self.module.parent {
            if !ptr::eq(parent, i.module) {
                // TODO: Clarify?
                panic!("Attempted to drive an instance input with a signal from a different module than that instance's parent module.");
            }
        } else {
            // TODO: Proper panic + test!
            panic!("OH NOES");
        }
        let mut driven_value = self.data.driven_value.borrow_mut();
        if driven_value.is_some() {
            panic!("Attempted to drive an input called \"{}\" on an instance of \"{}\", but this input is already driven for this instance.", self.data.name, self.module.name);
        }
        if self.data.bit_width != i.bit_width() {
            panic!("Attempted to drive an input called \"{}\" on an instance of \"{}\", but this input and the provided signal have different bit widths ({} and {}, respectively).", self.data.name, self.module.name, self.data.bit_width, i.bit_width());
        }
        *driven_value = Some(i);
    }
}

impl<'a> GetInternalSignal<'a> for Input<'a> {
    fn internal_signal(&'a self) -> &'a InternalSignal<'a> {
        self.value
    }
}

impl<'a> GetInternalSignal<'a> for Output<'a> {
    fn internal_signal(&'a self) -> &'a InternalSignal<'a> {
        let parent = self.data.module.parent.expect("TODO better error pls");
        self.data.module.context.signal_arena.alloc(InternalSignal {
            context: self.data.module.context,
            module: parent,

            data: SignalData::Output { data: self.data },
        })
    }
}

pub(crate) struct InputData<'a> {
    // TODO: Do we need this stored here too?
    pub name: String,
    pub bit_width: u32,
    // TODO: Rename?
    pub driven_value: RefCell<Option<&'a InternalSignal<'a>>>,
}

// TODO: Move?
// TODO: Doc
// TODO: must_use?
// TODO: bit width as const generic param?
pub struct Output<'a> {
    pub(crate) data: &'a OutputData<'a>,
}

pub(crate) struct OutputData<'a> {
    // TODO: Do we need this?
    pub module: &'a Module<'a>,

    // TODO: Do we need this stored here too?
    pub name: String,
    pub source: &'a InternalSignal<'a>,
    pub bit_width: u32,
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

        let m = c.module("a", "A");

        // Panic
        let _ = m.lit(false, 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a literal with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn lit_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.lit(false, 129);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into the specified bit width '7'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_1() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.lit(128u32, 7);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into the specified bit width '2'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_2() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.lit(128u64, 2);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '1023' into the specified bit width '4'. The value '1023' requires a bit width of at least 10 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_3() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.lit(1023u128, 4);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '65536' into the specified bit width '1'. The value '65536' requires a bit width of at least 17 bit(s)."
    )]
    fn lit_value_cannot_bit_into_bit_width_error_4() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.lit(65536u32, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create an input with 0 bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn input_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.input("i", 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create an input with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn input_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.input("i", 129);
    }

    #[test]
    #[should_panic(expected = "Cannot output a signal from another module.")]
    fn output_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a", "A");

        let m2 = c.module("b", "B");
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

        let m = c.module("a", "A");

        // Panic
        let _ = m.reg("r", 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a register with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn reg_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.reg("r", 129);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mux_cond_separate_module_error() {
        let c = Context::new();

        let a = c.module("a", "A");
        let l1 = a.lit(false, 1);

        let b = c.module("b", "B");
        let l2 = b.lit(32u8, 8);
        let l3 = b.lit(32u8, 8);

        // Panic
        let _ = b.mux(l1, l2, l3);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mux_when_true_separate_module_error() {
        let c = Context::new();

        let a = c.module("a", "A");
        let l1 = a.lit(32u8, 8);

        let b = c.module("b", "B");
        let l2 = b.lit(true, 1);
        let l3 = b.lit(32u8, 8);

        // Panic
        let _ = b.mux(l2, l1, l3);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mux_when_false_separate_module_error() {
        let c = Context::new();

        let a = c.module("a", "A");
        let l1 = a.lit(32u8, 8);

        let b = c.module("b", "B");
        let l2 = b.lit(true, 1);
        let l3 = b.lit(32u8, 8);

        // Panic
        let _ = b.mux(l2, l3, l1);
    }

    #[test]
    #[should_panic(expected = "Multiplexer conditionals can only be 1 bit wide.")]
    fn mux_cond_bit_width_error() {
        let c = Context::new();

        let a = c.module("a", "A");
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

        let a = c.module("a", "A");
        let l1 = a.lit(false, 1);
        let l2 = a.lit(3u8, 3);
        let l3 = a.lit(3u8, 5);

        // Panic
        let _ = a.mux(l1, l2, l3);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 0 address bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn mem_address_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.mem("mem", 0, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 129 address bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn mem_address_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.mem("mem", 129, 1);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 0 element bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn mem_element_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.mem("mem", 1, 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a memory with 129 element bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn mem_element_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        // Panic
        let _ = m.mem("mem", 1, 129);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive an instance input with a signal from a different module than that instance's parent module."
    )]
    fn input_drive_different_module_than_parent_module_error() {
        let c = Context::new();

        let m1 = c.module("a", "A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b", "B");

        let inner = m2.module("inner", "Inner");
        let a = inner.input("a", 1);

        // Panic
        a.drive(i1);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive an input called \"a\" on an instance of \"Inner\", but this input is already driven for this instance."
    )]
    fn input_drive_already_driven_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        let inner = m.module("inner", "Inner");
        let a = inner.input("a", 1);

        a.drive(m.input("i1", 1));

        // Panic
        a.drive(m.input("i2", 1));
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive an input called \"a\" on an instance of \"Inner\", but this input and the provided signal have different bit widths (1 and 32, respectively)."
    )]
    fn input_drive_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a", "A");

        let inner = m.module("inner", "Inner");
        let a = inner.input("a", 1);

        // Panic
        a.drive(m.input("i1", 32));
    }
}
