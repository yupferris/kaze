use super::constant::*;
use super::internal_signal::*;
use super::module::*;
use super::signal::*;

use std::cell::RefCell;
use std::ptr;

/// A hardware register, created by the [`Module::reg`] method.
///
/// A `Register` is a stateful component that behaves like a [D flip-flop](https://en.wikipedia.org/wiki/Flip-flop_(electronics)#D_flip-flop) (more precisely as a [positive-edge-triggered D flip-flop](https://en.wikipedia.org/wiki/Flip-flop_(electronics)#Classical_positive-edge-triggered_D_flip-flop)).
///
/// It always has a current value represented by the [`value`] field (often referred to as `Q`) and a next value specified by the [`drive_next`] method (often referred to as `D`).
/// It will hold its [`value`] until a positive edge of its [`Module`]'s implicit clock occurs, at which point [`value`] will be updated to reflect the next value.
///
/// Optionally, it also has a default value specified by the [`default_value`] method. If at any time its [`Module`]'s implicit reset is driven low, the register's [`value`] will reflect the default value.
/// Default values are used to provide a known register state on system power-on and reset, but are often omitted to reduce combinational logic (which ultimately is how default values are typically implemented), especially for registers on timing-critical data paths.
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
///
/// [`default_value`]: Self::default_value
/// [`drive_next`]: Self::drive_next
/// [`value`]: Self::value
#[must_use]
pub struct Register<'a> {
    pub(crate) data: &'a RegisterData<'a>,
    /// This `Register`'s current value.
    pub(crate) value: &'a InternalSignal<'a>,
}

impl<'a> Register<'a> {
    /// Specifies the default value for this `Register`.
    ///
    /// This `Register`'s [`value`] will reflect this default value when this `Register`'s [`Module`]'s implicit reset is asserted.
    ///
    /// By default, a `Register` does not have a default value, and it is not required to specify one. If a default value is not specified, then this `Register`'s [`value`] will not change when its [`Module`]'s implicit reset is asserted.
    ///
    /// # Panics
    ///
    /// Panics if this `Register` already has a default value specified, or if the specified `value` doesn't fit into this `Register`'s bit width.
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
    ///
    /// [`value`]: Self::value
    pub fn default_value(&'a self, value: impl Into<Constant>) {
        if self.data.initial_value.borrow().is_some() {
            panic!("Attempted to specify a default value for register \"{}\" in module \"{}\", but this register already has a default value.", self.data.name, self.data.module.name);
        }
        let value = value.into();
        let required_bits = value.required_bits();
        if required_bits > self.data.bit_width {
            let numeric_value = value.numeric_value();
            panic!("Cannot fit the specified value '{}' into register \"{}\"'s bit width '{}'. The value '{}' requires a bit width of at least {} bit(s).", numeric_value, self.data.name, self.data.bit_width, numeric_value, required_bits);
        }
        *self.data.initial_value.borrow_mut() = Some(value);
    }

    /// Specifies the next value for this `Register`.
    ///
    /// A `Register` will hold its [`value`] until a positive edge of its [`Module`]'s implicit clock occurs, at which point [`value`] will be updated to reflect this next value.
    ///
    /// # Panics
    ///
    /// Panics if `self` and `n` belong to different [`Module`]s, if the bit widths of `self` and `n` aren't equal, or if this `Register`'s next value is already driven.
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
    /// my_reg.drive_next(!my_reg); // my_reg's value will toggle with each positive clock edge
    /// m.output("my_output", my_reg);
    /// ```
    ///
    /// [`value`]: Self::value
    pub fn drive_next(&'a self, n: &'a dyn Signal<'a>) {
        let n = n.internal_signal();
        if !ptr::eq(self.data.module, n.module) {
            panic!("Attempted to drive register \"{}\"'s next value with a signal from another module.", self.data.name);
        }
        if n.bit_width() != self.data.bit_width {
            panic!("Attempted to drive register \"{}\"'s next value with a signal that has a different bit width than the register ({} and {}, respectively).", self.data.name, n.bit_width(), self.data.bit_width);
        }
        if self.data.next.borrow().is_some() {
            panic!("Attempted to drive register \"{}\"'s next value in module \"{}\", but this register's next value is already driven.", self.data.name, self.data.module.name);
        }
        *self.data.next.borrow_mut() = Some(n);
    }
}

pub(crate) struct RegisterData<'a> {
    pub module: &'a Module<'a>,

    pub name: String,
    pub initial_value: RefCell<Option<Constant>>,
    pub bit_width: u32,
    pub next: RefCell<Option<&'a InternalSignal<'a>>>,
}

impl<'a> GetInternalSignal<'a> for Register<'a> {
    fn internal_signal(&'a self) -> &'a InternalSignal<'a> {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[should_panic(
        expected = "Attempted to specify a default value for register \"r\" in module \"A\", but this register already has a default value."
    )]
    fn default_value_already_specified_error() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 32);

        r.default_value(0xfadebabeu32);

        // Panic
        r.default_value(0xdeadbeefu32);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into register \"r\"'s bit width '7'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn default_value_cannot_bit_into_bit_width_error_1() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 7);

        // Panic
        r.default_value(128u32);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '128' into register \"r\"'s bit width '2'. The value '128' requires a bit width of at least 8 bit(s)."
    )]
    fn default_value_cannot_bit_into_bit_width_error_2() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 2);

        // Panic
        r.default_value(128u64);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '1023' into register \"r\"'s bit width '4'. The value '1023' requires a bit width of at least 10 bit(s)."
    )]
    fn default_value_cannot_bit_into_bit_width_error_3() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 4);

        // Panic
        r.default_value(1023u128);
    }

    #[test]
    #[should_panic(
        expected = "Cannot fit the specified value '65536' into register \"r\"'s bit width '1'. The value '65536' requires a bit width of at least 17 bit(s)."
    )]
    fn default_value_cannot_bit_into_bit_width_error_4() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 1);

        // Panic
        r.default_value(65536u32);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive register \"r\"'s next value with a signal from another module."
    )]
    fn drive_next_separate_module_error() {
        let c = Context::new();

        let m1 = c.module("a", "A");
        let l = m1.lit(true, 1);

        let m2 = c.module("b", "B");
        let r = m2.reg("r", 1);

        // Panic
        r.drive_next(l);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive register \"r\"'s next value with a signal that has a different bit width than the register (5 and 3, respectively)."
    )]
    fn drive_next_incompatible_bit_width_error() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 3);
        let i = m.input("i", 5);

        // Panic
        r.drive_next(i);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to drive register \"r\"'s next value in module \"A\", but this register's next value is already driven."
    )]
    fn drive_next_already_driven_error() {
        let c = Context::new();

        let m = c.module("a", "A");
        let r = m.reg("r", 32);
        let i = m.input("i", 32);

        r.drive_next(i);

        // Panic
        r.drive_next(i);
    }
}
