use super::constant::*;
use super::context::*;
use super::module::*;
use super::signal::*;

use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::ptr;

/// A synchronous memory, created by the [`Module`]::[`mem`] method.
///
/// Memories in kaze are always sequential/synchronous-read, sequential/synchronous-write memories.
/// This means that when a read and/or write is asserted, the read/write will be visible on the cycle immediately following the cycle in which it's asserted.
/// If both a write and a read to the same location occurs within the same cycle, the read will return the previous value at the memory location, **not** the newly-written value.
///
/// Memories must have at least one read port specified.
/// Multiple reads to the same location within the same cycle will return the same value.
///
/// Memories may optionally have initial contents and/or a write port specified.
/// If either of these are missing, the contents of the memory can't be determined, so this is a logical error.
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
///
/// [`Module`]: ./struct.Module.html
/// [`mem`]: ./struct.Module.html#method.mem
#[must_use]
pub struct Mem<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) module: &'a Module<'a>,

    pub(crate) name: String,
    pub(crate) address_bit_width: u32,
    pub(crate) element_bit_width: u32,

    pub(crate) initial_contents: RefCell<Option<Vec<Constant>>>,

    pub(crate) read_ports: RefCell<Vec<(&'a Signal<'a>, &'a Signal<'a>)>>,
    pub(crate) write_port: RefCell<Option<(&'a Signal<'a>, &'a Signal<'a>, &'a Signal<'a>)>>,
}

impl<'a> Mem<'a> {
    /// Specifies the initial contents for this `Mem`.
    ///
    /// Reads from this `Mem` will reflect the values specified unless writes have overwritten them (if the `Mem` has a write port).
    ///
    /// By default, a `Mem` does not have initial contents, and it is not required to specify them unless the `Mem` does not have a write port.
    /// If initial contents are not specified, then this `Mem`'s contents will be undefined initially.
    ///
    /// Note that these contents are **not** restored when the containing [`Module`]'s implicit reset is asserted.
    ///
    /// # Panics
    ///
    /// Panics if this `Mem` already has initial contents specified, if `contents.len()` doesn't match the number of elements in this `Mem`, or if any of the specified element values don't fit into this `Mem`'s element bit width.
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
    ///
    /// [`Module`]: ./struct.Module.html
    pub fn initial_contents<C: Clone + Into<Constant>>(&'a self, contents: &[C]) {
        if self.initial_contents.borrow().is_some() {
            panic!("Attempted to specify initial contents for memory \"{}\" in module \"{}\", but this memory already has initial contents.", self.name, self.module.name);
        }
        let expected_contents_len = 1 << self.address_bit_width;
        if contents.len() != expected_contents_len {
            panic!("Attempted to specify initial contents for memory \"{}\" in module \"{}\" that contains {} element(s), but this memory has {} address bit(s), and requires {} element(s).", self.name, self.module.name, contents.len(), self.address_bit_width, expected_contents_len);
        }
        *self.initial_contents.borrow_mut() = Some(contents.iter().cloned().enumerate().map(|(i, x)| {
            let ret = x.into();
            if ret.required_bits() > self.element_bit_width {
                panic!("Attempted to specify initial contents for memory \"{}\" in module \"{}\", but this memory has an element width of {} bit(s), and these initial contents specify element {} with value {} which requires {} bit(s).", self.name, self.module.name, self.element_bit_width, i, ret.numeric_value(), ret.required_bits());
            }
            ret
        }).collect());
    }

    /// Specifies a read port for this `Mem` and returns a [`Signal`] representing the data read from this port.
    ///
    /// `Mem`s are required to have at least one read port, otherwise the memory contents could never be read, which would be a logical error.
    /// There is no upper bound to the number of read ports specified in kaze, however a target device may not be able to synthesize the resulting SystemVerilog code if too many are used.
    ///
    /// Read ports always have an `address` signal and an `enable` signal.
    /// When `enable` is asserted, the returned [`Signal`] will reflect the data read from the location specified by `address` on the following cycle.
    /// If `enable` is not asserted, then the value of the returned [`Signal`] is undefined on the following cycle.
    ///
    /// # Panics
    ///
    /// Panics if `address`'s bit width doesn't match this `Mem`'s address bit width, or if `enable`'s bit width is not `1`.
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
    ///
    /// [`Module`]: ./struct.Module.html
    /// [`Signal`]: ./struct.Signal.html
    pub fn read_port(&'a self, address: &'a Signal<'a>, enable: &'a Signal<'a>) -> &Signal<'a> {
        // TODO: Limit amount of read ports added?
        if address.bit_width() != self.address_bit_width {
            panic!("Attempted to specify a read port for memory \"{}\" in module \"{}\" with an address signal with {} bit(s), but this memory has {} address bit(s).", self.name, self.module.name, address.bit_width(), self.address_bit_width);
        }
        if enable.bit_width() != 1 {
            panic!("Attempted to specify a read port for memory \"{}\" in module \"{}\" with an enable signal with {} bit(s), but memory read/write ports are required to be 1 bit wide.", self.name, self.module.name, enable.bit_width());
        }
        let ret = self.context.signal_arena.alloc(Signal {
            context: self.context,
            module: self.module,

            data: SignalData::MemReadPortOutput {
                mem: self,
                address,
                enable,
            },
        });
        self.read_ports.borrow_mut().push((address, enable));
        ret
    }

    /// Specifies a write port for this `Mem`.
    ///
    /// By default, a `Mem` does not have any write ports, and it is not required to specify one unless the `Mem` does not have initial contents.
    ///
    /// Write ports always have an `address` signal, a `value` signal, and an `enable` signal.
    /// When `enable` is asserted, the value at the location specified by `address` will reflect the value of the `value` signal on the following cycle.
    /// If `enable` is not asserted, then the memory contents will not change.
    ///
    /// # Panics
    ///
    /// Panics if this `Mem` already has a write port specified, if `address`'s bit width doesn't match this `Mem`'s address bit width, if `value`'s bit width doesn't match this `Mem`'s element bit width, or if `enable`'s bit width is not `1`.
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
    ///
    /// [`Module`]: ./struct.Module.html
    /// [`Signal`]: ./struct.Signal.html
    // TODO: byte/word enable? How might that interface look?
    pub fn write_port(
        &'a self,
        address: &'a Signal<'a>,
        value: &'a Signal<'a>,
        enable: &'a Signal<'a>,
    ) {
        if self.write_port.borrow().is_some() {
            panic!("Attempted to specify a write port for memory \"{}\" in module \"{}\", but this memory already has a write port.", self.name, self.module.name);
        }
        if address.bit_width() != self.address_bit_width {
            panic!("Attempted to specify a write port for memory \"{}\" in module \"{}\" with an address signal with {} bit(s), but this memory has {} address bit(s).", self.name, self.module.name, address.bit_width(), self.address_bit_width);
        }
        if value.bit_width() != self.element_bit_width {
            panic!("Attempted to specify a write port for memory \"{}\" in module \"{}\" with a value signal with {} bit(s), but this memory has {} element bit(s).", self.name, self.module.name, value.bit_width(), self.element_bit_width);
        }
        if enable.bit_width() != 1 {
            panic!("Attempted to specify a write port for memory \"{}\" in module \"{}\" with an enable signal with {} bit(s), but memory read/write ports are required to be 1 bit wide.", self.name, self.module.name, enable.bit_width());
        }
        *self.write_port.borrow_mut() = Some((address, value, enable));
    }
}

impl<'a> Eq for &'a Mem<'a> {}

impl<'a> Hash for &'a Mem<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self as *const _ as usize)
    }
}

impl<'a> PartialEq for &'a Mem<'a> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(*self, *other)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[should_panic(
        expected = "Attempted to specify initial contents for memory \"mem\" in module \"A\", but this memory already has initial contents."
    )]
    fn initial_contents_already_specified_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        mem.initial_contents(&[true, false]);

        // Panic
        mem.initial_contents(&[true, false]);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify initial contents for memory \"mem\" in module \"A\" that contains 3 element(s), but this memory has 1 address bit(s), and requires 2 element(s)."
    )]
    fn initial_contents_length_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        mem.initial_contents(&[true, false, true]);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify initial contents for memory \"mem\" in module \"A\", but this memory has an element width of 1 bit(s), and these initial contents specify element 0 with value 2 which requires 2 bit(s)."
    )]
    fn initial_contents_element_bit_width_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        mem.initial_contents(&[2u32, 0u32]);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify a read port for memory \"mem\" in module \"A\" with an address signal with 2 bit(s), but this memory has 1 address bit(s)."
    )]
    fn read_port_address_bit_width_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        let _ = mem.read_port(m.lit(0u32, 2), m.low());
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify a read port for memory \"mem\" in module \"A\" with an enable signal with 2 bit(s), but memory read/write ports are required to be 1 bit wide."
    )]
    fn read_port_enable_bit_width_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        let _ = mem.read_port(m.low(), m.lit(0u32, 2));
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify a write port for memory \"mem\" in module \"A\", but this memory already has a write port."
    )]
    fn write_port_already_specified_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        mem.write_port(m.low(), m.low(), m.low());

        // Panic
        mem.write_port(m.low(), m.low(), m.low());
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify a write port for memory \"mem\" in module \"A\" with an address signal with 2 bit(s), but this memory has 1 address bit(s)."
    )]
    fn write_port_address_bit_width_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        mem.write_port(m.lit(0u32, 2), m.low(), m.low());
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify a write port for memory \"mem\" in module \"A\" with a value signal with 2 bit(s), but this memory has 1 element bit(s)."
    )]
    fn write_port_value_bit_width_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        mem.write_port(m.low(), m.lit(0u32, 2), m.low());
    }

    #[test]
    #[should_panic(
        expected = "Attempted to specify a write port for memory \"mem\" in module \"A\" with an enable signal with 2 bit(s), but memory read/write ports are required to be 1 bit wide."
    )]
    fn write_port_enable_bit_width_error() {
        let c = Context::new();

        let m = c.module("A");
        let mem = m.mem("mem", 1, 1);

        // Panic
        mem.write_port(m.low(), m.low(), m.lit(0u32, 2));
    }
}
