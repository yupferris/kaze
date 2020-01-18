use super::signal::*;
use super::value::*;

use std::cell::RefCell;

#[must_use]
pub struct Register<'a> {
    pub(crate) data: &'a RegisterData<'a>,
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
