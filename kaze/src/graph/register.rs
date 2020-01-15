use super::signal::*;

#[must_use]
pub struct Register<'a> {
    pub value: &'a Signal<'a>,
}

impl<'a> Register<'a> {
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
