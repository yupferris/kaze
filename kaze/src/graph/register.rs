use super::signal::*;
use super::value::*;

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
    pub fn default_value<V: Into<Value>>(&'a self, value: V) {
        // TODO: Panic if this register already has a default value
        // TODO: Value range check
        let value = value.into();
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
    pub name: String,
    pub initial_value: RefCell<Option<Value>>,
    pub bit_width: u32,
    pub next: RefCell<Option<&'a Signal<'a>>>,
}
