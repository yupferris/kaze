use super::constant::*;
use super::module::*;
use super::signal::*;

use std::cell::RefCell;

/// A hardware register, created by the [`Module`]::[`reg`] method.
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
/// let m = c.module("my_module");
///
/// let my_reg = m.reg("my_reg", 32);
/// my_reg.default_value(0xfadebabeu32); // Optional
/// my_reg.drive_next(!my_reg.value);
/// m.output("my_output", my_reg.value);
/// ```
///
/// [`default_value`]: #method.default_value
/// [`drive_next`]: #method.drive_next
/// [`Module`]: ./struct.Module.html
/// [`reg`]: ./struct.Module.html#method.reg
/// [`value`]: #structfield.value
#[must_use]
pub struct Register<'a> {
    pub(crate) data: &'a RegisterData<'a>,
    /// This `Register`'s current value.
    ///
    /// [`Signal`]: ./struct.Signal.html
    pub value: &'a Signal<'a>,
}

impl<'a> Register<'a> {
    /// Specifies the default value for this `Register`.
    ///
    /// This `Register`'s `value` will reflect this default value when this `Register`'s [`Module`]'s implicit reset is asserted.
    ///
    /// By default, a `Register` does not have a default value, and it is not required to specify one. If a default value is not specified, then this `Register`'s `value` will not change when its [`Module`]'s implicit reset is asserted.
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
    /// let m = c.module("my_module");
    ///
    /// let my_reg = m.reg("my_reg", 32);
    /// my_reg.default_value(0xfadebabeu32); // Optional
    /// my_reg.drive_next(!my_reg.value);
    /// m.output("my_output", my_reg.value);
    /// ```
    ///
    /// [`Module`]: ./struct.Module.html
    pub fn default_value<C: Into<Constant>>(&'a self, value: C) {
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

    pub fn drive_next(&'a self, n: &'a Signal<'a>) {
        // TODO: Ensure n is in the same module as self
        // TODO: Ensure n's bit_width is the same as self.value.data.bit_width
        // TODO: Ensure this register isn't already driven
        *self.data.next.borrow_mut() = Some(n);
    }
}

pub(crate) struct RegisterData<'a> {
    pub module: &'a Module<'a>,

    pub name: String,
    pub initial_value: RefCell<Option<Constant>>,
    pub bit_width: u32,
    pub next: RefCell<Option<&'a Signal<'a>>>,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[should_panic(
        expected = "Attempted to specify a default value for register \"r\" in module \"a\", but this register already has a default value."
    )]
    fn default_value_already_specified_error() {
        let c = Context::new();

        let m = c.module("a");
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

        let m = c.module("a");
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

        let m = c.module("a");
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

        let m = c.module("a");
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

        let m = c.module("a");
        let r = m.reg("r", 1);

        // Panic
        r.default_value(65536u32);
    }
}
