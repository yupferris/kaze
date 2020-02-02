/// A container for different types of integer constant values.
///
/// This type isn't typically used explicitly, as the graph API always takes `Constant` parameters as `Into<Constant>`, and `Constant` implements `From` for most of Rust's unsigned integer types. If an API entry point requires a `Constant`, prefer passing integer values/literals directly.
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
/// let a = m.lit(true, 16);
/// let b = m.lit(0xdeadbeefu32, 47);
/// let c = m.reg("data", 20);
/// c.default_value(5u32);
/// let d = m.lit(42u32, 8);
/// ```
pub enum Constant {
    /// Contains a boolean value
    Bool(bool),
    /// Contains an unsigned, 32-bit value
    U32(u32),
    /// Contains an unsigned, 64-bit value
    U64(u64),
    /// Contains an unsigned, 128-bit value
    U128(u128),
}

impl Constant {
    // TODO: Specific tests? I don't necessarily want to make this part of the public API at least.
    pub(super) fn required_bits(&self) -> u32 {
        match *self {
            Constant::Bool(value) => 32 - (value as u32).leading_zeros(),
            Constant::U32(value) => 32 - value.leading_zeros(),
            Constant::U64(value) => 64 - value.leading_zeros(),
            Constant::U128(value) => 128 - value.leading_zeros(),
        }
    }

    pub(super) fn numeric_value(&self) -> u128 {
        match *self {
            Constant::Bool(value) => value.into(),
            Constant::U32(value) => value.into(),
            Constant::U64(value) => value.into(),
            Constant::U128(value) => value,
        }
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Constant::Bool(value)
    }
}

impl From<u8> for Constant {
    fn from(value: u8) -> Self {
        Constant::U32(value as _)
    }
}

impl From<u16> for Constant {
    fn from(value: u16) -> Self {
        Constant::U32(value as _)
    }
}

impl From<u32> for Constant {
    fn from(value: u32) -> Self {
        Constant::U32(value)
    }
}

impl From<u64> for Constant {
    fn from(value: u64) -> Self {
        Constant::U64(value)
    }
}

impl From<u128> for Constant {
    fn from(value: u128) -> Self {
        Constant::U128(value)
    }
}
