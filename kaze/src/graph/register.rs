use super::signal::*;
use super::value::*;

#[must_use]
pub struct Register<'a> {
    pub value: &'a Signal<'a>,
}

impl<'a> Register<'a> {
    pub fn default_value<V: Into<Value>>(&'a self, value: V) {
        // TODO: Panic if this register already has a default value
        // TODO: Value range check
        let value = value.into();
        match self.value.data {
            SignalData::Reg {
                ref initial_value, ..
            } => {
                *initial_value.borrow_mut() = Some(value);
            }
            _ => unreachable!(),
        }
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
