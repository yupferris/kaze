//! `Module` graph API.
//!
//! TODO: This is where most of the meat should go!
//!
//! # Modules
//! # Signals
//! # Registers

use typed_arena::Arena;

use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::ptr;

/// A top-level container/owner object for a module graph.
///
/// A `Context` owns all parts of a module graph, and provides an API for creating `Module` objects.
///
/// # Examples
///
/// ```
/// use kaze::module::*;
///
/// let c = Context::new();
/// let m = c.module("my_module");
/// m.output("out", m.input("in", 1));
/// ```
pub struct Context<'a> {
    module_arena: Arena<Module<'a>>,
    signal_arena: Arena<Signal<'a>>,
    output_arena: Arena<Output<'a>>,
    register_arena: Arena<Register<'a>>,
    instance_arena: Arena<Instance<'a>>,

    modules: RefCell<BTreeMap<String, &'a Module<'a>>>,
}

impl<'a> Context<'a> {
    /// Creates a new, empty `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    /// ```
    pub fn new() -> Context<'a> {
        Context {
            module_arena: Arena::new(),
            signal_arena: Arena::new(),
            output_arena: Arena::new(),
            register_arena: Arena::new(),
            instance_arena: Arena::new(),

            modules: RefCell::new(BTreeMap::new()),
        }
    }

    /// Creates a new `Module` called `name` in this `Context`.
    ///
    /// Conventionally, `name` should be `snake_case`, though this is not enforced.
    ///
    /// # Panics
    ///
    /// Panics if a `Module` with the same `name` already exists in this `Context`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let my_module = c.module("my_module");
    /// let another_mod = c.module("another_mod");
    /// ```
    ///
    /// The following example panics by creating a `Module` with the same `name` as a previously-created `Module` in the same `Context`:
    ///
    /// ```should_panic
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let _ = c.module("a"); // Unique name, OK
    /// let _ = c.module("b"); // Unique name, OK
    ///
    /// let _ = c.module("a"); // Non-unique name, panic!
    /// ```
    pub fn module<S: Into<String>>(&'a self, name: S) -> &Module {
        let name = name.into();
        let mut modules = self.modules.borrow_mut();
        if modules.contains_key(&name) {
            panic!(
                "A module with the name \"{}\" already exists in this context.",
                name
            );
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

    /// Creates a `Signal` that represents the constant literal specified by `value` and with `bit_width` bits.
    ///
    /// The bit width of the type provided by `value` doesn't need to match `bit_width`.
    ///
    /// # Panics
    ///
    /// Panics if `bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let eight_bit_const = m.lit(Value::U32(0xff), 8);
    /// let one_bit_const = m.lit(Value::U128(0), 1);
    /// let twenty_seven_bit_const = m.lit(Value::Bool(true), 27);
    /// ```
    ///
    /// [`MIN_SIGNAL_BIT_WIDTH`]: ./constant.MIN_SIGNAL_BIT_WIDTH.html
    /// [`MAX_SIGNAL_BIT_WIDTH`]: ./constant.MAX_SIGNAL_BIT_WIDTH.html
    pub fn lit(&'a self, value: Value, bit_width: u32) -> &Signal<'a> {
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
        // TODO: Ensure value fits within bit_width bits
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Lit { value, bit_width },
        })
    }

    /// Convenience method to create a `Signal` that represents a single `0` bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// // The following two signals are semantically equivalent:
    /// let low1 = m.low();
    /// let low2 = m.lit(Value::Bool(false), 1);
    /// ```
    pub fn low(&'a self) -> &Signal<'a> {
        self.lit(Value::Bool(false), 1)
    }

    /// Convenience method to create a `Signal` that represents a single `1` bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// // The following two signals are semantically equivalent:
    /// let high1 = m.high();
    /// let high2 = m.lit(Value::Bool(true), 1);
    /// ```
    pub fn high(&'a self) -> &Signal<'a> {
        self.lit(Value::Bool(true), 1)
    }

    /// Creates an input for this `Module` called `name` with `bit_width` bits, and returns a `Signal` that represents the value of this input.
    ///
    /// # Panics
    ///
    /// Panics if `bit_width` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`], respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
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

    /// Creates an output for this `Module` called `name` and with the same number of bits as `source`, and drives this output with `source`.
    ///
    /// # Panics
    ///
    /// Panics of `source` doesn't belong to this `Module`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
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
        let output = self.context.output_arena.alloc(Output { source });
        self.outputs.borrow_mut().insert(name.into(), output);
    }

    pub fn reg(&'a self, bit_width: u32, initial_value: Option<Value>) -> &Register<'a> {
        // TODO: bit_width bounds checks
        // TODO: Ensure initial_value fits within bit_width bits
        let value = self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Reg {
                initial_value: initial_value.unwrap_or(Value::U32(0)),
                bit_width,
                next: RefCell::new(None),
            },
        });
        self.context.register_arena.alloc(Register { value })
    }

    pub fn mux(&'a self, a: &'a Signal<'a>, b: &'a Signal<'a>, sel: &'a Signal<'a>) -> &Signal<'a> {
        // TODO: Ensure a and b have the same bit widths
        // TODO: Ensure sel is 1 bit wide
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self,

            data: SignalData::Mux { a, b, sel },
        })
    }

    /// Instantiates the `Module` called `name` in this `Context` inside this `Module` definition, and returns the new `Instance`.
    ///
    /// # Panics
    ///
    /// Panics if a `Module` called `name` doesn't exist in this `Context` or if creating this `Instance` would cause this `Module` definition to become recursive.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// // Inner module (simple pass-through)
    /// let inner = c.module("inner");
    /// inner.output("o", inner.input("i", 32));
    ///
    /// // Outer module (wraps a single `inner` instance)
    /// let outer = c.module("outer");
    /// let inner_inst = outer.instance("inner");
    /// inner_inst.drive_input("i", outer.input("i", 32));
    /// outer.output("o", inner_inst.output("o"));
    /// ```
    pub fn instance(&'a self, name: &str) -> &Instance<'a> {
        // TODO: Error if this creates a recursive module definition (can we actually do this here?)
        match self.context.modules.borrow().get(name) {
            Some(instantiated_module) => {
                self.context.instance_arena.alloc(Instance {
                    context: self.context,
                    module: self,

                    instantiated_module,
                    driven_inputs: RefCell::new(BTreeMap::new()),
                })
            }
            _ => panic!("Attempted to instantiate a module with the name \"{}\", but no such module with this name exists in this context.", name)
        }
    }
}

/// The minimum allowed bit width for any given `Signal`.
///
/// This is currently set to `1`, and is not likely to change in future versions of this library.
pub const MIN_SIGNAL_BIT_WIDTH: u32 = 1;
/// The maximum allowed bit width for any given `Signal`.
///
/// This is currently set to `128` to simplify simulator code generation, since it allows the generated code to rely purely on native integer types provided by Rust's standard library for storage, arithmetic, etc. Larger widths may be supported in a future version of this library.
pub const MAX_SIGNAL_BIT_WIDTH: u32 = 128;

pub struct Signal<'a> {
    context: &'a Context<'a>,
    module: &'a Module<'a>,

    pub data: SignalData<'a>,
}

impl<'a> Signal<'a> {
    /// Returns the bit width of the given `Signal`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// assert_eq!(m.lit(Value::U32(42), 7).bit_width(), 7);
    /// assert_eq!(m.input("i", 27).bit_width(), 27);
    /// assert_eq!(m.reg(46, None).value().bit_width(), 46);
    /// assert_eq!((!m.low()).bit_width(), 1);
    /// assert_eq!((m.high() | m.low()).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(12), 100).bit(30).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(1), 99).bits(37, 29).bit_width(), 9);
    /// assert_eq!(m.high().repeat(35).bit_width(), 35);
    /// assert_eq!(m.lit(Value::U32(1), 20).concat(m.high()).bit_width(), 21);
    /// assert_eq!(m.lit(Value::U32(0xaa), 8).eq(m.lit(Value::U32(0xaa), 8)).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(0xaa), 8).ne(m.lit(Value::U32(0xaa), 8)).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(0xaa), 8).lt(m.lit(Value::U32(0xaa), 8)).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(0xaa), 8).le(m.lit(Value::U32(0xaa), 8)).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(0xaa), 8).gt(m.lit(Value::U32(0xaa), 8)).bit_width(), 1);
    /// assert_eq!(m.lit(Value::U32(0xaa), 8).ge(m.lit(Value::U32(0xaa), 8)).bit_width(), 1);
    /// assert_eq!(m.mux(m.lit(Value::U32(5), 4), m.lit(Value::U32(6), 4), m.low()).bit_width(), 4);
    /// ```
    pub fn bit_width(&self) -> u32 {
        match &self.data {
            SignalData::Lit { bit_width, .. } => *bit_width,
            SignalData::Input { bit_width, .. } => *bit_width,
            SignalData::Reg { bit_width, .. } => *bit_width,
            SignalData::UnOp { source, .. } => source.bit_width(),
            SignalData::BinOp { bit_width, .. } => *bit_width,
            SignalData::Bit { .. } => 1,
            SignalData::Bits {
                range_high,
                range_low,
                ..
            } => range_high - range_low + 1,
            SignalData::Repeat { source, count } => source.bit_width() * count,
            SignalData::Concat { lhs, rhs } => lhs.bit_width() + rhs.bit_width(),
            SignalData::Mux { a, .. } => a.bit_width(),
            SignalData::InstanceOutput { instance, name } => instance.instantiated_module.outputs()
                [name]
                .source
                .bit_width(),
        }
    }

    /// Creates a `Signal` that represents the value of the single bit of this `Signal` at index `index`, where `index` equal to `0` represents this `Signal`'s least significant bit.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than or equal to this `Signal`'s `bit_width`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit = m.lit(Value::U32(0b0110), 4);
    /// let bit_0 = lit.bit(0); // Represents 0
    /// let bit_1 = lit.bit(1); // Represents 1
    /// let bit_2 = lit.bit(2); // Represents 1
    /// let bit_3 = lit.bit(3); // Represents 0
    /// ```
    pub fn bit(&'a self, index: u32) -> &Signal<'a> {
        if index >= self.bit_width() {
            panic!("Attempted to take bit index {} from a signal with a width of {} bits. Bit indices must be in the range [0, {}] for a signal with a width of {} bits.", index, self.bit_width(), self.bit_width() - 1, self.bit_width());
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Bit {
                source: self,
                index,
            },
        })
    }

    /// Creates a `Signal` that represents a contiguous subset of the bits of this `Signal`, starting at `range_low` as the least significant bit and ending at `range_high` as the most significant bit, inclusive.
    ///
    /// # Panics
    ///
    /// Panics if either `range_low` or `range_high` is greater than or equal to the bit width of this `Signal`, or if `range_low` is greater than `range_high`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit = m.lit(Value::U32(0b0110), 4);
    /// let bits_210 = lit.bits(2, 0); // Represents 0b110
    /// let bits_321 = lit.bits(3, 1); // Represents 0b011
    /// let bits_10 = lit.bits(1, 0); // Represents 0b10
    /// let bits_32 = lit.bits(3, 2); // Represents 0b01
    /// let bits_2 = lit.bits(2, 2); // Represents 1, equivalent to lit.bit(2)
    /// ```
    pub fn bits(&'a self, range_high: u32, range_low: u32) -> &Signal<'a> {
        if range_low >= self.bit_width() {
            panic!("Cannot specify a range of bits where the lower bound is greater than or equal to the number of bits in the source signal. The bounds must be in the range [0, {}] for a signal with a width of {} bits, but a lower bound of {} was given.", self.bit_width() - 1, self.bit_width(), range_low);
        }
        if range_high >= self.bit_width() {
            panic!("Cannot specify a range of bits where the upper bound is greater than or equal to the number of bits in the source signal. The bounds must be in the range [0, {}] for a signal with a width of {} bits, but an upper bound of {} was given.", self.bit_width() - 1, self.bit_width(), range_high);
        }
        if range_low > range_high {
            panic!("Cannot specify a range of bits where the lower bound is greater than the upper bound.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Bits {
                source: self,
                range_high,
                range_low,
            },
        })
    }

    /// Creates a `Signal` that represents this `Signal` repeated `count` times.
    ///
    /// # Panics
    ///
    /// Panics if `self.bit_width() * count` is less than [`MIN_SIGNAL_BIT_WIDTH`] or greater than [`MAX_SIGNAL_BIT_WIDTH`].
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit = m.lit(Value::U32(0xa), 4);
    /// let repeat_1 = lit.repeat(1); // Equivalent to just lit
    /// let repeat_2 = lit.repeat(2); // Equivalent to 8-bit lit with value 0xaa
    /// let repeat_5 = lit.repeat(5); // Equivalent to 20-bit lit with value 0xaaaaa
    /// let repeat_8 = lit.repeat(8); // Equivalent to 32-bit lit with value 0xaaaaaaaa
    /// ```
    ///
    /// [`MIN_SIGNAL_BIT_WIDTH`]: ./constant.MIN_SIGNAL_BIT_WIDTH.html
    /// [`MAX_SIGNAL_BIT_WIDTH`]: ./constant.MAX_SIGNAL_BIT_WIDTH.html
    pub fn repeat(&'a self, count: u32) -> &Signal<'a> {
        let target_bit_width = self.bit_width() * count;
        if target_bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!("Attempted to repeat a {}-bit signal {} times, but this would result in a bit width of {}, which is less than the minimal signal bit width of {} bit(s).", self.bit_width(), count, target_bit_width, MIN_SIGNAL_BIT_WIDTH);
        }
        if target_bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!("Attempted to repeat a {}-bit signal {} times, but this would result in a bit width of {}, which is greater than the maximum signal bit width of {} bit(s).", self.bit_width(), count, target_bit_width, MAX_SIGNAL_BIT_WIDTH);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Repeat {
                source: self,
                count,
            },
        })
    }

    /// Creates a `Signal` that represents this `Signal` concatenated with `rhs`.
    ///
    /// `self` represents the upper bits in the resulting `Signal`, and `rhs` represents the lower bits.
    ///
    /// # Panics
    ///
    /// Panics if `self.bit_width() + rhs.bit_width()` is greater than [`MAX_SIGNAL_BIT_WIDTH`].
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xff), 8);
    /// let concat_1 = lit_a.concat(lit_b); // Equivalent to 12-bit lit with value 0xaff
    /// let concat_2 = lit_b.concat(lit_a); // Equivalent to 12-bit lit with value 0xffa
    /// let concat_3 = lit_a.concat(lit_a); // Equivalent to 8-bit lit with value 0xaa
    /// ```
    ///
    /// [`MAX_SIGNAL_BIT_WIDTH`]: ./constant.MAX_SIGNAL_BIT_WIDTH.html
    pub fn concat(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        let target_bit_width = self.bit_width() + rhs.bit_width();
        if target_bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!("Attempted to concatenate signals with {} bit(s) and {} bit(s) respectively, but this would result in a bit width of {}, which is greater than the maximum signal bit width of {} bit(s).", self.bit_width(), rhs.bit_width(), target_bit_width, MAX_SIGNAL_BIT_WIDTH);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Concat { lhs: self, rhs },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a bitwise boolean equality comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xb), 4);
    /// let eq_1 = lit_a.eq(lit_a); // Equivalent to m.high()
    /// let eq_2 = lit_b.eq(lit_b); // Equivalent to m.high()
    /// let eq_3 = lit_a.eq(lit_b); // Equivalent to m.low()
    /// let eq_4 = lit_b.eq(lit_a); // Equivalent to m.low()
    /// ```
    pub fn eq(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: 1,
                lhs: self,
                rhs,
                op: BinOp::Equal,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a bitwise boolean inequality comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xb), 4);
    /// let ne_1 = lit_a.ne(lit_a); // Equivalent to m.low()
    /// let ne_2 = lit_b.ne(lit_b); // Equivalent to m.low()
    /// let ne_3 = lit_a.ne(lit_b); // Equivalent to m.high()
    /// let ne_4 = lit_b.ne(lit_a); // Equivalent to m.high()
    /// ```
    pub fn ne(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: 1,
                lhs: self,
                rhs,
                op: BinOp::NotEqual,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `<` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xb), 4);
    /// let lt_1 = lit_a.lt(lit_a); // Equivalent to m.low()
    /// let lt_2 = lit_b.lt(lit_b); // Equivalent to m.low()
    /// let lt_3 = lit_a.lt(lit_b); // Equivalent to m.high()
    /// let lt_4 = lit_b.lt(lit_a); // Equivalent to m.low()
    /// ```
    pub fn lt(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: 1,
                lhs: self,
                rhs,
                op: BinOp::LessThan,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `<=` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xb), 4);
    /// let le_1 = lit_a.le(lit_a); // Equivalent to m.high()
    /// let le_2 = lit_b.le(lit_b); // Equivalent to m.high()
    /// let le_3 = lit_a.le(lit_b); // Equivalent to m.high()
    /// let le_4 = lit_b.le(lit_a); // Equivalent to m.low()
    /// ```
    pub fn le(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: 1,
                lhs: self,
                rhs,
                op: BinOp::LessThanEqual,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `>` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xb), 4);
    /// let gt_1 = lit_a.gt(lit_a); // Equivalent to m.low()
    /// let gt_2 = lit_b.gt(lit_b); // Equivalent to m.low()
    /// let gt_3 = lit_a.gt(lit_b); // Equivalent to m.low()
    /// let gt_4 = lit_b.gt(lit_a); // Equivalent to m.high()
    /// ```
    pub fn gt(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: 1,
                lhs: self,
                rhs,
                op: BinOp::GreaterThan,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit resugt of an unsigned `>=` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lit_a = m.lit(Value::U32(0xa), 4);
    /// let lit_b = m.lit(Value::U32(0xb), 4);
    /// let ge_1 = lit_a.ge(lit_a); // Equivalent to m.high()
    /// let ge_2 = lit_b.ge(lit_b); // Equivalent to m.high()
    /// let ge_3 = lit_a.ge(lit_b); // Equivalent to m.low()
    /// let ge_4 = lit_b.ge(lit_a); // Equivalent to m.high()
    /// ```
    pub fn ge(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: 1,
                lhs: self,
                rhs,
                op: BinOp::GreaterThanEqual,
            },
        })
    }

    // TODO: This is currently only used to support macro conditional syntax; if it doesn't work out, remove this
    pub fn mux(&'a self, b: &'a Signal<'a>, sel: &'a Signal<'a>) -> &Signal<'a> {
        self.module.mux(self, b, sel)
    }
}

pub enum SignalData<'a> {
    Lit {
        value: Value,
        bit_width: u32,
    },

    Input {
        name: String,
        bit_width: u32,
    },

    Reg {
        initial_value: Value,
        bit_width: u32,
        next: RefCell<Option<&'a Signal<'a>>>,
    },

    UnOp {
        source: &'a Signal<'a>,
        op: UnOp,
    },
    BinOp {
        bit_width: u32,
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        op: BinOp,
    },

    Bit {
        source: &'a Signal<'a>,
        index: u32,
    },
    Bits {
        source: &'a Signal<'a>,
        range_high: u32,
        range_low: u32,
    },

    Repeat {
        source: &'a Signal<'a>,
        count: u32,
    },
    Concat {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
    },

    Mux {
        a: &'a Signal<'a>,
        b: &'a Signal<'a>,
        sel: &'a Signal<'a>,
    },

    InstanceOutput {
        instance: &'a Instance<'a>,
        name: String,
    },
}

impl<'a> BitAnd for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` whose bits represent the bitwise `&` of each of the bits of the original two `Signal`s.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kaze::module::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lhs = m.low();
    /// let rhs = m.high();
    /// let single_bitand = lhs & rhs;
    ///
    /// let lhs = m.input("in1", 3);
    /// let rhs = m.input("in2", 3);
    /// let multi_bitand = lhs & rhs;
    /// ```
    fn bitand(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: self.bit_width(),
                lhs: self,
                rhs,
                op: BinOp::BitAnd,
            },
        })
    }
}

impl<'a> BitOr for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` whose bits represent the bitwise `|` of each of the bits of the original two `Signal`s.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kaze::module::*;
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lhs = m.low();
    /// let rhs = m.high();
    /// let single_bitor = lhs | rhs;
    ///
    /// let lhs = m.input("in1", 3);
    /// let rhs = m.input("in2", 3);
    /// let multi_bitor = lhs | rhs;
    /// ```
    fn bitor(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: self.bit_width(),
                lhs: self,
                rhs,
                op: BinOp::BitOr,
            },
        })
    }
}

impl<'a> BitXor for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` whose bits represent the bitwise `^` of each of the bits of the original two `Signal`s.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different `Module`s, or if the bit widths of `lhs` and `rhs` aren't equal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kaze::module::*;
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let lhs = m.low();
    /// let rhs = m.high();
    /// let single_bitxor = lhs ^ rhs;
    ///
    /// let lhs = m.input("in1", 3);
    /// let rhs = m.input("in2", 3);
    /// let multi_bitxor = lhs ^ rhs;
    /// ```
    fn bitxor(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        if self.bit_width() != rhs.bit_width() {
            panic!(
                "Signals have different bit widths ({} and {}, respectively).",
                self.bit_width(),
                rhs.bit_width()
            );
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::BinOp {
                bit_width: self.bit_width(),
                lhs: self,
                rhs,
                op: BinOp::BitXor,
            },
        })
    }
}

impl<'a> Not for &'a Signal<'a> {
    type Output = Self;

    /// Produces a new `Signal` whose bits represent the bitwise `!` of each of the bits of the original `Signal`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kaze::module::*;
    /// let c = Context::new();
    ///
    /// let m = c.module("my_module");
    ///
    /// let input1 = m.input("input1", 1);
    /// let single_not = !input1;
    ///
    /// let input2 = m.input("input2", 6);
    /// let multi_not = !input2;
    /// ```
    fn not(self) -> Self {
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::UnOp {
                source: self,
                op: UnOp::Not,
            },
        })
    }
}

#[derive(Clone, Copy)]
pub enum UnOp {
    Not,
}

#[derive(Clone, Copy)]
pub enum BinOp {
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    NotEqual,
}

pub struct Output<'a> {
    pub source: &'a Signal<'a>,
}

pub struct Register<'a> {
    pub value: &'a Signal<'a>,
}

impl<'a> Register<'a> {
    pub fn value(&'a self) -> &Signal<'a> {
        self.value
    }

    pub fn drive_next(&'a self, n: &'a Signal<'a>) {
        match self.value.data {
            SignalData::Reg { ref next, .. } => {
                // TODO: Ensure n is in the same module as self
                // TODO: Ensure n's bit_width is the same as self.value.data.bit_width
                // TODO: Ensure this register isn't already driven
                *next.borrow_mut() = Some(n);
            }
            _ => unreachable!(),
        }
    }
}

pub struct Instance<'a> {
    context: &'a Context<'a>,
    module: &'a Module<'a>,

    pub instantiated_module: &'a Module<'a>,
    driven_inputs: RefCell<BTreeMap<String, &'a Signal<'a>>>,
}

impl<'a> Instance<'a> {
    pub fn driven_inputs(&self) -> Ref<BTreeMap<String, &'a Signal<'a>>> {
        self.driven_inputs.borrow()
    }

    pub fn drive_input<S: Into<String>>(&'a self, name: S, i: &'a Signal<'a>) {
        // TODO: Error if i is from a different module than self.module
        // TODO: Error if there's no input called `name` on this module
        // TODO: Error if this input is already driven
        // TODO: Error if the signal's bit width and the input's bit width don't match
        // TODO: Should we try to detect loops and error here? Is that sufficient? Efficient?
        self.driven_inputs.borrow_mut().insert(name.into(), i);
    }

    pub fn output<S: Into<String>>(&'a self, name: S) -> &Signal<'a> {
        // TODO: Error if there's no output called `name` on this module
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::InstanceOutput {
                instance: self,
                name: name.into(),
            },
        })
    }
}

// TODO: Should this be named Literal or Const or something?
pub enum Value {
    Bool(bool),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "A module with the name \"a\" already exists in this context.")]
    fn unique_module_names() {
        let c = Context::new();

        let _ = c.module("a"); // Unique name, OK
        let _ = c.module("b"); // Unique name, OK

        // Panic
        let _ = c.module("a");
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a literal with 0 bit(s). Signals must not be narrower than 1 bit(s)."
    )]
    fn lit_bit_width_lt_min_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(Value::Bool(false), 0);
    }

    #[test]
    #[should_panic(
        expected = "Cannot create a literal with 129 bit(s). Signals must not be wider than 128 bit(s)."
    )]
    fn lit_bit_width_gt_max_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.lit(Value::Bool(false), 129);
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
        expected = "Attempted to instantiate a module with the name \"nope\", but no such module with this name exists in this context."
    )]
    fn instantiate_nonexistent_module_error() {
        let c = Context::new();

        let m = c.module("a");

        // Panic
        let _ = m.instance("nope");
    }

    #[test]
    #[should_panic(
        expected = "Attempted to take bit index 3 from a signal with a width of 3 bits. Bit indices must be in the range [0, 2] for a signal with a width of 3 bits."
    )]
    fn bit_index_oob_error() {
        let c = Context::new();

        let m = c.module("a");
        let i = m.input("i", 3);

        let _ = i.bit(0); // OK
        let _ = i.bit(1); // OK
        let _ = i.bit(2); // OK

        let _ = i.bit(3); // Panic, `index` too high
    }

    #[test]
    #[should_panic(
        expected = "Cannot specify a range of bits where the lower bound is greater than or equal to the number of bits in the source signal. The bounds must be in the range [0, 2] for a signal with a width of 3 bits, but a lower bound of 3 was given."
    )]
    fn bits_range_low_oob_error() {
        let c = Context::new();

        let m = c.module("a");
        let i = m.input("i", 3);

        // Panic
        let _ = i.bits(4, 3);
    }

    #[test]
    #[should_panic(
        expected = "Cannot specify a range of bits where the upper bound is greater than or equal to the number of bits in the source signal. The bounds must be in the range [0, 2] for a signal with a width of 3 bits, but an upper bound of 3 was given."
    )]
    fn bits_range_high_oob_error() {
        let c = Context::new();

        let m = c.module("a");
        let i = m.input("i", 3);

        // Panic
        let _ = i.bits(3, 2);
    }

    #[test]
    #[should_panic(
        expected = "Cannot specify a range of bits where the lower bound is greater than the upper bound."
    )]
    fn bits_range_low_gt_high_error() {
        let c = Context::new();

        let m = c.module("a");
        let i = m.input("i", 3);

        // Panic
        let _ = i.bits(0, 1);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to repeat a 1-bit signal 0 times, but this would result in a bit width of 0, which is less than the minimal signal bit width of 1 bit(s)."
    )]
    fn repeat_count_zero_error() {
        let c = Context::new();

        let m = c.module("a");
        let i = m.input("i", 1);

        // Panic
        let _ = i.repeat(0);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to repeat a 1-bit signal 129 times, but this would result in a bit width of 129, which is greater than the maximum signal bit width of 128 bit(s)."
    )]
    fn repeat_count_oob_error() {
        let c = Context::new();

        let m = c.module("a");
        let i = m.input("i", 1);

        // Panic
        let _ = i.repeat(129);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to concatenate signals with 128 bit(s) and 1 bit(s) respectively, but this would result in a bit width of 129, which is greater than the maximum signal bit width of 128 bit(s)."
    )]
    fn concat_oob_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("i1", 128);
        let i2 = m.input("i2", 1);

        // Panic
        let _ = i1.concat(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn eq_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1.eq(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn eq_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.eq(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn ne_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1.ne(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn ne_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.ne(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn lt_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1.lt(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn lt_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.lt(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn le_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1.le(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn le_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.le(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn gt_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1.gt(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn gt_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.gt(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn ge_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1.ge(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn ge_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.ge(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
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
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn bitor_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 | i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn bitxor_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a");
        let i1 = m1.input("a", 1);

        let m2 = c.module("b");
        let i2 = m2.high();

        // Panic
        let _ = i1 ^ i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn bitxor_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("a");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 ^ i2;
    }
}
