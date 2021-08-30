use super::constant::*;
use super::context::*;
use super::instance::*;
use super::mem::*;
use super::module::*;
use super::register::*;

use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr, Sub};
use std::ptr;

/// The minimum allowed bit width for any given [`Signal`].
///
/// This is currently set to `1`, and is not likely to change in future versions of this library.
pub const MIN_SIGNAL_BIT_WIDTH: u32 = 1;
/// The maximum allowed bit width for any given [`Signal`].
///
/// This is currently set to `128` to simplify simulator code generation, since it allows the generated code to rely purely on native integer types provided by Rust's standard library for storage, arithmetic, etc. Larger widths may be supported in a future version of this library.
pub const MAX_SIGNAL_BIT_WIDTH: u32 = 128;

/// A collection of 1 or more bits driven by some source.
///
/// A `Signal` can be created by several [`Module`] methods (eg. [`lit`]) or as a result of combining existing `Signal`s (eg. [`concat`]). `Signal`s are local to their respective [`Module`]s.
///
/// A `Signal` behaves similarly to a `wire` in Verilog, except that it's always driven.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// let m = c.module("MyModule");
/// let a = m.lit(0xffu8, 8); // 8-bit signal
/// let b = m.input("my_input", 27); // 27-bit signal
/// let c = b.bits(7, 0); // 8-bit signal
/// let d = a + c; // 8-bit signal
/// m.output("my_output", d); // 8-bit output driven by d
/// ```
///
/// [`concat`]: Self::concat
/// [`lit`]: Module::lit
#[must_use]
pub struct Signal<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) module: &'a Module<'a>,

    pub(crate) data: SignalData<'a>,
}

impl<'a> Signal<'a> {
    /// Returns the bit width of the given `Signal`.
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
    /// assert_eq!(m.lit(42u32, 7).bit_width(), 7);
    /// assert_eq!(m.input("i", 27).bit_width(), 27);
    /// assert_eq!(m.reg("some_reg", 46).value.bit_width(), 46);
    /// assert_eq!((!m.low()).bit_width(), 1);
    /// assert_eq!((m.lit(25u8, 8) + m.lit(42u8, 8)).bit_width(), 8);
    /// assert_eq!((m.lit(1u8, 1) * m.lit(2u8, 2)).bit_width(), 3);
    /// assert_eq!(m.lit(1u8, 1).mul_signed(m.lit(2u8, 2)).bit_width(), 3);
    /// assert_eq!(m.lit(false, 1).reg_next("some_other_reg").bit_width(), 1);
    /// assert_eq!(m.lit(true, 1).reg_next_with_default("yet_another_reg", false).bit_width(), 1);
    /// assert_eq!((m.high() & m.low()).bit_width(), 1);
    /// assert_eq!((m.high() | m.low()).bit_width(), 1);
    /// assert_eq!(m.lit(12u32, 100).bit(30).bit_width(), 1);
    /// assert_eq!(m.lit(1u32, 99).bits(37, 29).bit_width(), 9);
    /// assert_eq!(m.high().repeat(35).bit_width(), 35);
    /// assert_eq!(m.lit(1u32, 20).concat(m.high()).bit_width(), 21);
    /// assert_eq!((m.lit(0x80u32, 8) << m.lit(true, 1)).bit_width(), 8);
    /// assert_eq!((m.lit(0x80u32, 8) >> m.lit(true, 1)).bit_width(), 8);
    /// assert_eq!(m.lit(0x80u32, 8).shr_arithmetic(m.lit(true, 1)).bit_width(), 8);
    /// assert_eq!(m.lit(0xaau32, 8).eq(m.lit(0xaau32, 8)).bit_width(), 1);
    /// assert_eq!(m.lit(0xaau32, 8).ne(m.lit(0xaau32, 8)).bit_width(), 1);
    /// assert_eq!(m.lit(0xaau32, 8).lt(m.lit(0xaau32, 8)).bit_width(), 1);
    /// assert_eq!(m.lit(0xaau32, 8).le(m.lit(0xaau32, 8)).bit_width(), 1);
    /// assert_eq!(m.lit(0xaau32, 8).gt(m.lit(0xaau32, 8)).bit_width(), 1);
    /// assert_eq!(m.lit(0xaau32, 8).ge(m.lit(0xaau32, 8)).bit_width(), 1);
    /// assert_eq!(m.mux(m.low(), m.lit(5u32, 4), m.lit(6u32, 4)).bit_width(), 4);
    /// ```
    #[must_use]
    pub fn bit_width(&self) -> u32 {
        match self.data {
            SignalData::Lit { bit_width, .. } => bit_width,
            SignalData::Input { bit_width, .. } => bit_width,
            SignalData::Reg { ref data } => data.bit_width,
            SignalData::UnOp { bit_width, .. } => bit_width,
            SignalData::SimpleBinOp { bit_width, .. } => bit_width,
            SignalData::AdditiveBinOp { bit_width, .. } => bit_width,
            SignalData::ComparisonBinOp { .. } => 1,
            SignalData::ShiftBinOp { bit_width, .. } => bit_width,
            SignalData::Mul { bit_width, .. } => bit_width,
            SignalData::MulSigned { bit_width, .. } => bit_width,
            SignalData::Bits {
                range_high,
                range_low,
                ..
            } => range_high - range_low + 1,
            SignalData::Repeat { bit_width, .. } => bit_width,
            SignalData::Concat { bit_width, .. } => bit_width,
            SignalData::Mux { bit_width, .. } => bit_width,
            SignalData::InstanceOutput { bit_width, .. } => bit_width,
            SignalData::MemReadPortOutput { mem, .. } => mem.element_bit_width,
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
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let lit = m.lit(0b0110u32, 4);
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

            data: SignalData::Bits {
                source: self,
                range_high: index,
                range_low: index,
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
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let lit = m.lit(0b0110u32, 4);
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
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let lit = m.lit(0xau32, 4);
    /// let repeat_1 = lit.repeat(1); // Equivalent to just lit
    /// let repeat_2 = lit.repeat(2); // Equivalent to 8-bit lit with value 0xaa
    /// let repeat_5 = lit.repeat(5); // Equivalent to 20-bit lit with value 0xaaaaa
    /// let repeat_8 = lit.repeat(8); // Equivalent to 32-bit lit with value 0xaaaaaaaa
    /// ```
    pub fn repeat(&'a self, count: u32) -> &Signal<'a> {
        let bit_width = self.bit_width() * count;
        if bit_width < MIN_SIGNAL_BIT_WIDTH {
            panic!("Attempted to repeat a {}-bit signal {} times, but this would result in a bit width of {}, which is less than the minimal signal bit width of {} bit(s).", self.bit_width(), count, bit_width, MIN_SIGNAL_BIT_WIDTH);
        }
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!("Attempted to repeat a {}-bit signal {} times, but this would result in a bit width of {}, which is greater than the maximum signal bit width of {} bit(s).", self.bit_width(), count, bit_width, MAX_SIGNAL_BIT_WIDTH);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Repeat {
                source: self,
                count,
                bit_width,
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
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
    ///
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xffu32, 8);
    /// let concat_1 = lit_a.concat(lit_b); // Equivalent to 12-bit lit with value 0xaff
    /// let concat_2 = lit_b.concat(lit_a); // Equivalent to 12-bit lit with value 0xffa
    /// let concat_3 = lit_a.concat(lit_a); // Equivalent to 8-bit lit with value 0xaa
    /// ```
    pub fn concat(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        let bit_width = self.bit_width() + rhs.bit_width();
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!("Attempted to concatenate signals with {} bit(s) and {} bit(s) respectively, but this would result in a bit width of {}, which is greater than the maximum signal bit width of {} bit(s).", self.bit_width(), rhs.bit_width(), bit_width, MAX_SIGNAL_BIT_WIDTH);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Concat {
                lhs: self,
                rhs,
                bit_width,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a bitwise boolean equality comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, or if the bit widths of `self` and `rhs` aren't equal.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
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

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::Equal,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a bitwise boolean inequality comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, or if the bit widths of `self` and `rhs` aren't equal.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
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

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::NotEqual,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `<` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, or if the bit widths of `self` and `rhs` aren't equal.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
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

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::LessThan,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `<=` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, or if the bit widths of `self` and `rhs` aren't equal.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
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

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::LessThanEqual,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `>` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, or if the bit widths of `self` and `rhs` aren't equal.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
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

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::GreaterThan,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of an unsigned `>=` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, or if the bit widths of `self` and `rhs` aren't equal.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
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

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::GreaterThanEqual,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a signed `<` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, if the bit widths of `self` and `rhs` aren't equal, or if the bit widths of `self` and `rhs` are 1.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
    /// let lt_signed_1 = lit_a.lt_signed(lit_a); // Equivalent to m.low()
    /// let lt_signed_2 = lit_b.lt_signed(lit_b); // Equivalent to m.low()
    /// let lt_signed_3 = lit_a.lt_signed(lit_b); // Equivalent to m.high()
    /// let lt_signed_4 = lit_b.lt_signed(lit_a); // Equivalent to m.low()
    /// ```
    pub fn lt_signed(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
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
        if self.bit_width() == 1 {
            panic!("Cannot perform signed comparison of 1-bit signals.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::LessThanSigned,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a signed `<=` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, if the bit widths of `self` and `rhs` aren't equal, or if the bit widths of `self` and `rhs` are 1.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
    /// let le_signed_1 = lit_a.le_signed(lit_a); // Equivalent to m.high()
    /// let le_signed_2 = lit_b.le_signed(lit_b); // Equivalent to m.high()
    /// let le_signed_3 = lit_a.le_signed(lit_b); // Equivalent to m.high()
    /// let le_signed_4 = lit_b.le_signed(lit_a); // Equivalent to m.low()
    /// ```
    pub fn le_signed(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
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
        if self.bit_width() == 1 {
            panic!("Cannot perform signed comparison of 1-bit signals.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::LessThanEqualSigned,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a signed `>` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, if the bit widths of `self` and `rhs` aren't equal, or if the bit widths of `self` and `rhs` are 1.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
    /// let gt_signed_1 = lit_a.gt_signed(lit_a); // Equivalent to m.low()
    /// let gt_signed_2 = lit_b.gt_signed(lit_b); // Equivalent to m.low()
    /// let gt_signed_3 = lit_a.gt_signed(lit_b); // Equivalent to m.low()
    /// let gt_signed_4 = lit_b.gt_signed(lit_a); // Equivalent to m.high()
    /// ```
    pub fn gt_signed(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
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
        if self.bit_width() == 1 {
            panic!("Cannot perform signed comparison of 1-bit signals.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::GreaterThanSigned,
            },
        })
    }

    /// Creates a `Signal` that represents the single-bit result of a signed `>=` comparison between `self` and `rhs`.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `rhs` belong to different [`Module`]s, if the bit widths of `self` and `rhs` aren't equal, or if the bit widths of `self` and `rhs` are 1.
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
    /// let lit_a = m.lit(0xau32, 4);
    /// let lit_b = m.lit(0xbu32, 4);
    /// let ge_signed_1 = lit_a.ge_signed(lit_a); // Equivalent to m.high()
    /// let ge_signed_2 = lit_b.ge_signed(lit_b); // Equivalent to m.high()
    /// let ge_signed_3 = lit_a.ge_signed(lit_b); // Equivalent to m.low()
    /// let ge_signed_4 = lit_b.ge_signed(lit_a); // Equivalent to m.high()
    /// ```
    pub fn ge_signed(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
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
        if self.bit_width() == 1 {
            panic!("Cannot perform signed comparison of 1-bit signals.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ComparisonBinOp {
                lhs: self,
                rhs,
                op: ComparisonBinOp::GreaterThanEqualSigned,
            },
        })
    }

    /// Combines two `Signal`s, producing a new `Signal` that represents `self` arithmetically shifted right by `rhs` bits.
    ///
    /// The result is truncated to `self`'s `bit_width`. If `rhs` specifies a value that's greater than or equal to `self`'s `bit_width`, the resulting value will be all `self`'s top bit repeated `self`'s `bit_width` times.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s.
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
    /// let lhs = m.lit(0x80000000u32, 32);
    /// let rhs = m.lit(1u32, 1);
    /// let shifted = lhs.shr_arithmetic(rhs); // Equivalent to m.lit(0xc0000000u32, 32)
    /// ```
    pub fn shr_arithmetic(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ShiftBinOp {
                lhs: self,
                rhs,
                op: ShiftBinOp::ShrArithmetic,
                bit_width: self.bit_width(),
            },
        })
    }

    /// Combines two `Signal`s, producing a new `Signal` that represents the signed product of the original two `Signal`s.
    ///
    /// The product's `bit_width` is equal to `self.bit_width() + rhs.bit_width()`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if `self.bit_width() + rhs.bit_width()` is greater than [`MAX_SIGNAL_BIT_WIDTH`].
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
    /// let lhs = m.lit(4u32, 3); // -4
    /// let rhs = m.lit(5u32, 4);
    /// let sum = lhs.mul_signed(rhs); // Equivalent to m.lit(108u32, 7), -20
    /// ```
    pub fn mul_signed(&'a self, rhs: &'a Signal<'a>) -> &Signal<'a> {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        let bit_width = self.bit_width() + rhs.bit_width();
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!("Attempted to multiply a {}-bit with a {}-bit signal, but this would result in a bit width of {}, which is greater than the maximum signal bit width of {} bit(s).", self.bit_width(), rhs.bit_width(), bit_width, MAX_SIGNAL_BIT_WIDTH);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::MulSigned {
                lhs: self,
                rhs,
                bit_width,
            },
        })
    }

    /// Creates a 2:1 [multiplexer](https://en.wikipedia.org/wiki/Multiplexer) that represents `when_true`'s value when `self` is high, and `when_false`'s value when `self` is low.
    ///
    /// This is a convenience wrapper for [`Module::mux`].
    ///
    /// # Panics
    ///
    /// Panics if `when_true` or `when_false` belong to a different [`Module`] than `self`, if `self`'s bit width is not 1, or if the bit widths of `when_true` and `when_false` aren't equal.
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
    /// m.output("my_output", cond.mux(a, b)); // Outputs a when cond is high, b otherwise
    /// ```
    // TODO: This is currently only used to support sugar; if it doesn't work out, remove this
    pub fn mux(&'a self, when_true: &'a Signal<'a>, when_false: &'a Signal<'a>) -> &Signal<'a> {
        self.module.mux(self, when_true, when_false)
    }

    /// Creates a [`Signal`] that represents the same value as this [`Signal`], but delayed by one cycle.
    ///
    /// This is achieved by creating a new [`Register`] called `name`, and specifying this [`Signal`] as the next value for the [`Register`]. Note that no default value is provided for this [`Register`], so the returned [`Signal`]'s value is undefined until the first clock edge, and its value is not affected by its [`Module`]'s implicit reset. If a default value is desired, use [`reg_next_with_default`] instead.
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
    /// let a = m.lit(true, 1);
    /// let a_delayed = a.reg_next("a_delayed");
    /// ```
    ///
    /// [`reg_next_with_default`]: Self::reg_next_with_default
    pub fn reg_next<S: Into<String>>(&'a self, name: S) -> &Signal<'a> {
        let reg = self.module.reg(name, self.bit_width());
        reg.drive_next(self);
        reg.value
    }

    /// Creates a [`Signal`] that represents the same value as this [`Signal`], but delayed by one cycle, except when this [`Signal`]'s [`Module`]'s implicit reset is asserted, at which point it will represent `default_value`.
    ///
    /// This is achieved by creating a new [`Register`] called `name`, specifying `default_value` as its default value, and specifying this [`Signal`] as the next value for the [`Register`]. If a default value is not desired, use [`reg_next`] instead.
    ///
    /// # Panics
    ///
    /// Panics if the specified `value` doesn't fit into this [`Signal`]'s bit width.
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
    /// let a = m.lit(true, 1);
    /// let a_delayed = a.reg_next_with_default("a_delayed", false);
    /// ```
    ///
    /// [`reg_next`]: Self::reg_next
    pub fn reg_next_with_default<S: Into<String>, C: Into<Constant>>(
        &'a self,
        name: S,
        default_value: C,
    ) -> &Signal<'a> {
        let reg = self.module.reg(name, self.bit_width());
        reg.default_value(default_value);
        reg.drive_next(self);
        reg.value
    }
}

pub(crate) enum SignalData<'a> {
    Lit {
        value: Constant,
        bit_width: u32,
    },

    Input {
        name: String,
        bit_width: u32,
    },

    Reg {
        data: &'a RegisterData<'a>,
    },

    UnOp {
        source: &'a Signal<'a>,
        op: UnOp,
        bit_width: u32,
    },
    SimpleBinOp {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        op: SimpleBinOp,
        bit_width: u32,
    },
    AdditiveBinOp {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        op: AdditiveBinOp,
        bit_width: u32,
    },
    ComparisonBinOp {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        op: ComparisonBinOp,
    },
    ShiftBinOp {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        op: ShiftBinOp,
        bit_width: u32,
    },

    Mul {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        bit_width: u32,
    },
    MulSigned {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        bit_width: u32,
    },

    Bits {
        source: &'a Signal<'a>,
        range_high: u32,
        range_low: u32,
    },

    Repeat {
        source: &'a Signal<'a>,
        count: u32,
        bit_width: u32,
    },
    Concat {
        lhs: &'a Signal<'a>,
        rhs: &'a Signal<'a>,
        bit_width: u32,
    },

    Mux {
        cond: &'a Signal<'a>,
        when_true: &'a Signal<'a>,
        when_false: &'a Signal<'a>,
        bit_width: u32,
    },

    InstanceOutput {
        instance: &'a Instance<'a>,
        name: String,
        bit_width: u32,
    },

    MemReadPortOutput {
        mem: &'a Mem<'a>,
        address: &'a Signal<'a>,
        enable: &'a Signal<'a>,
    },
}

impl<'a> Add for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` that represents the sum of the original two `Signal`s.
    ///
    /// The sum is truncated to the `Signal`'s `bit_width`. If a carry bit is desired, the operands can be [`concat`]enated with a `0` bit before the operation.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if the bit widths of `lhs` and `rhs` aren't equal.
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
    /// let lhs = m.lit(1u32, 32);
    /// let rhs = m.lit(2u32, 32);
    /// let sum = lhs + rhs; // Equivalent to m.lit(3u32, 32)
    ///
    /// let lhs = m.low().concat(m.lit(0xffffffffu32, 32)); // Concat 0 bits with operands for carry
    /// let rhs = m.low().concat(m.lit(0x00000001u32, 32));
    /// let carry_sum = lhs + rhs; // Equivalent to m.lit(0x100000000u64, 33)
    /// let sum = carry_sum.bits(31, 0); // Equivalent to m.lit(0u32, 32)
    /// let carry = carry_sum.bit(32); // Equivalent to m.lit(true, 1)
    /// ```
    ///
    /// [`concat`]: Signal::concat
    fn add(self, rhs: Self) -> Self {
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

            data: SignalData::AdditiveBinOp {
                lhs: self,
                rhs,
                op: AdditiveBinOp::Add,
                bit_width: self.bit_width(),
            },
        })
    }
}

impl<'a> BitAnd for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` whose bits represent the bitwise `&` of each of the bits of the original two `Signal`s.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if the bit widths of `lhs` and `rhs` aren't equal.
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

            data: SignalData::SimpleBinOp {
                lhs: self,
                rhs,
                op: SimpleBinOp::BitAnd,
                bit_width: self.bit_width(),
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
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if the bit widths of `lhs` and `rhs` aren't equal.
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

            data: SignalData::SimpleBinOp {
                lhs: self,
                rhs,
                op: SimpleBinOp::BitOr,
                bit_width: self.bit_width(),
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
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if the bit widths of `lhs` and `rhs` aren't equal.
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

            data: SignalData::SimpleBinOp {
                lhs: self,
                rhs,
                op: SimpleBinOp::BitXor,
                bit_width: self.bit_width(),
            },
        })
    }
}

impl<'a> Eq for &'a Signal<'a> {}

impl<'a> Hash for &'a Signal<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self as *const _ as usize)
    }
}

impl<'a> PartialEq for &'a Signal<'a> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(*self, *other)
    }
}

impl<'a> Mul for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` that represents the unsigned product of the original two `Signal`s.
    ///
    /// The product's `bit_width` is equal to `self.bit_width() + rhs.bit_width()`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if `self.bit_width() + rhs.bit_width()` is greater than [`MAX_SIGNAL_BIT_WIDTH`].
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
    /// let lhs = m.lit(4u32, 3);
    /// let rhs = m.lit(5u32, 4);
    /// let sum = lhs * rhs; // Equivalent to m.lit(20u32, 7)
    /// ```
    fn mul(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        let bit_width = self.bit_width() + rhs.bit_width();
        if bit_width > MAX_SIGNAL_BIT_WIDTH {
            panic!("Attempted to multiply a {}-bit with a {}-bit signal, but this would result in a bit width of {}, which is greater than the maximum signal bit width of {} bit(s).", self.bit_width(), rhs.bit_width(), bit_width, MAX_SIGNAL_BIT_WIDTH);
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::Mul {
                lhs: self,
                rhs,
                bit_width,
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
    /// use kaze::*;
    ///
    /// let c = Context::new();
    ///
    /// let m = c.module("MyModule");
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
                bit_width: self.bit_width(),
            },
        })
    }
}

impl<'a> Shl for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` that represents `self` logically shifted left by `rhs` bits.
    ///
    /// The result is truncated to `self`'s `bit_width`. If `rhs` specifies a value that's greater than or equal to `self`'s `bit_width`, the resulting value will be zero.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s.
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
    /// let lhs = m.lit(3u32, 32);
    /// let rhs = m.lit(2u32, 2);
    /// let shifted = lhs << rhs; // Equivalent to m.lit(12u32, 32)
    /// ```
    fn shl(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ShiftBinOp {
                lhs: self,
                rhs,
                op: ShiftBinOp::Shl,
                bit_width: self.bit_width(),
            },
        })
    }
}

impl<'a> Shr for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` that represents `self` logically shifted right by `rhs` bits.
    ///
    /// The result is truncated to `self`'s `bit_width`. If `rhs` specifies a value that's greater than or equal to `self`'s `bit_width`, the resulting value will be zero.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s.
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
    /// let lhs = m.lit(12u32, 32);
    /// let rhs = m.lit(2u32, 2);
    /// let shifted = lhs >> rhs; // Equivalent to m.lit(3u32, 32)
    /// ```
    fn shr(self, rhs: Self) -> Self {
        if !ptr::eq(self.module, rhs.module) {
            panic!("Attempted to combine signals from different modules.");
        }
        self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::ShiftBinOp {
                lhs: self,
                rhs,
                op: ShiftBinOp::Shr,
                bit_width: self.bit_width(),
            },
        })
    }
}

impl<'a> Sub for &'a Signal<'a> {
    type Output = Self;

    /// Combines two `Signal`s, producing a new `Signal` that represents the difference of the original two `Signal`s.
    ///
    /// The difference is truncated to the `Signal`'s `bit_width`.
    ///
    /// # Panics
    ///
    /// Panics if `lhs` and `rhs` belong to different [`Module`]s, or if the bit widths of `lhs` and `rhs` aren't equal.
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
    /// let lhs = m.lit(3u32, 32);
    /// let rhs = m.lit(2u32, 32);
    /// let difference = lhs - rhs; // Equivalent to m.lit(1u32, 32)
    /// ```
    fn sub(self, rhs: Self) -> Self {
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

            data: SignalData::AdditiveBinOp {
                lhs: self,
                rhs,
                op: AdditiveBinOp::Sub,
                bit_width: self.bit_width(),
            },
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) enum UnOp {
    Not,
}

#[derive(Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SimpleBinOp {
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Clone, Copy)]
pub(crate) enum ComparisonBinOp {
    Equal,
    GreaterThan,
    GreaterThanEqual,
    GreaterThanEqualSigned,
    GreaterThanSigned,
    LessThan,
    LessThanEqual,
    LessThanEqualSigned,
    LessThanSigned,
    NotEqual,
}

#[derive(Clone, Copy)]
pub(crate) enum AdditiveBinOp {
    Add,
    Sub,
}

#[derive(Clone, Copy)]
pub(crate) enum ShiftBinOp {
    Shl,
    Shr,
    ShrArithmetic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Attempted to take bit index 3 from a signal with a width of 3 bits. Bit indices must be in the range [0, 2] for a signal with a width of 3 bits."
    )]
    fn bit_index_oob_error() {
        let c = Context::new();

        let m = c.module("A");
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

        let m = c.module("A");
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

        let m = c.module("A");
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

        let m = c.module("A");
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

        let m = c.module("A");
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

        let m = c.module("A");
        let i = m.input("i", 1);

        // Panic
        let _ = i.repeat(129);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn concat_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.concat(i2);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to concatenate signals with 128 bit(s) and 1 bit(s) respectively, but this would result in a bit width of 129, which is greater than the maximum signal bit width of 128 bit(s)."
    )]
    fn concat_oob_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("i1", 128);
        let i2 = m.input("i2", 1);

        // Panic
        let _ = i1.concat(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn eq_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.eq(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn eq_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.eq(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn ne_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.ne(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn ne_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.ne(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn lt_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.lt(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn lt_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.lt(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn le_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.le(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn le_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.le(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn gt_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.gt(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn gt_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.gt(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn ge_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.ge(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn ge_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.ge(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn lt_signed_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.lt_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn lt_signed_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.lt_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Cannot perform signed comparison of 1-bit signals.")]
    fn lt_signed_bit_width_1_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 1);
        let i2 = m.input("b", 1);

        // Panic
        let _ = i1.lt_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn le_signed_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.le_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn le_signed_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.le_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Cannot perform signed comparison of 1-bit signals.")]
    fn le_signed_bit_width_1_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 1);
        let i2 = m.input("b", 1);

        // Panic
        let _ = i1.le_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn gt_signed_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.gt_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn gt_signed_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.gt_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Cannot perform signed comparison of 1-bit signals.")]
    fn gt_signed_bit_width_1_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 1);
        let i2 = m.input("b", 1);

        // Panic
        let _ = i1.gt_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn ge_signed_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.ge_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn ge_signed_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1.ge_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Cannot perform signed comparison of 1-bit signals.")]
    fn ge_signed_bit_width_1_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 1);
        let i2 = m.input("b", 1);

        // Panic
        let _ = i1.ge_signed(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn shr_arithmetic_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.shr_arithmetic(i2);
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mul_signed_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1.mul_signed(i2);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to multiply a 128-bit with a 1-bit signal, but this would result in a bit width of 129, which is greater than the maximum signal bit width of 128 bit(s)."
    )]
    fn mul_signed_oob_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 128);
        let i2 = m.input("b", 1);

        // Panic
        let _ = i1.mul_signed(i2);
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
        let _ = l1.mux(l2, l3);
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
        let _ = l2.mux(l1, l3);
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
        let _ = l2.mux(l3, l1);
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
        let _ = l1.mux(l2, l3);
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
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn add_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 + i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn add_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 + i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn bitand_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 & i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn bitand_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 & i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn bitor_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 | i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn bitor_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 | i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn bitxor_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 ^ i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn bitxor_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 ^ i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn mul_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 * i2;
    }

    #[test]
    #[should_panic(
        expected = "Attempted to multiply a 128-bit with a 1-bit signal, but this would result in a bit width of 129, which is greater than the maximum signal bit width of 128 bit(s)."
    )]
    fn mul_oob_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 128);
        let i2 = m.input("b", 1);

        // Panic
        let _ = i1 * i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn shl_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 << i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn shr_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 >> i2;
    }

    #[test]
    #[should_panic(expected = "Attempted to combine signals from different modules.")]
    fn sub_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("A");
        let i1 = m1.input("a", 1);

        let m2 = c.module("B");
        let i2 = m2.high();

        // Panic
        let _ = i1 - i2;
    }

    #[test]
    #[should_panic(expected = "Signals have different bit widths (3 and 5, respectively).")]
    fn sub_incompatible_bit_widths_error() {
        let c = Context::new();

        let m = c.module("A");
        let i1 = m.input("a", 3);
        let i2 = m.input("b", 5);

        // Panic
        let _ = i1 - i2;
    }
}
