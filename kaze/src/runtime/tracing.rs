//! Rust simulator runtime dependencies for tracing.

pub mod vcd;

use std::io;

// TODO: Do we want to re-use graph::Constant for this? They're equivalent but currently distinct in their usage, so I'm not sure it's the right API design decision.
#[derive(Debug, Eq, PartialEq)]
pub enum TraceValue {
    /// Contains a boolean value
    Bool(bool),
    /// Contains an unsigned, 32-bit value
    U32(u32),
    /// Contains an unsigned, 64-bit value
    U64(u64),
    /// Contains an unsigned, 128-bit value
    U128(u128),
}

#[derive(Debug, Eq, PartialEq)]
pub enum TraceValueType {
    Bool,
    U32,
    U64,
    U128,
}

impl TraceValueType {
    pub(crate) fn from_bit_width(bit_width: u32) -> TraceValueType {
        if bit_width == 1 {
            TraceValueType::Bool
        } else if bit_width <= 32 {
            TraceValueType::U32
        } else if bit_width <= 64 {
            TraceValueType::U64
        } else if bit_width <= 128 {
            TraceValueType::U128
        } else {
            unreachable!()
        }
    }
}

pub trait Trace {
    type SignalId;

    fn push_module(&mut self, name: &'static str) -> io::Result<()>;
    fn pop_module(&mut self) -> io::Result<()>;
    fn add_signal(
        &mut self,
        name: &'static str,
        bit_width: u32,
        type_: TraceValueType,
    ) -> io::Result<Self::SignalId>;

    fn update_time_stamp(&mut self, time_stamp: u64) -> io::Result<()>;
    fn update_signal(&mut self, signal_id: &Self::SignalId, value: TraceValue) -> io::Result<()>;
}
