/// A container for different types of integer constant values.
///
/// This type isn't typically used explicitly, as the graph API always takes `Value` parameters as `Into<Value>`, and `Value` implements `From` for most of Rust's unsigned integer types. If an API entry point requires a `Value`, prefer passing integer values/literals directly.
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
/// let a = m.lit(true, 16);
/// let b = m.lit(0xdeadbeefu32, 47);
/// let c = m.reg("data", 20);
/// c.default_value(5u32);
/// let d = m.lit(42u32, 8);
/// ```
pub enum Value {
    /// Contains a boolean value
    Bool(bool),
    /// Contains an unsigned, 32-bit value
    U32(u32),
    /// Contains an unsigned, 64-bit value
    U64(u64),
    /// Contains an unsigned, 128-bit value
    U128(u128),
}

impl Value {
    // TODO: Specific tests? I don't necessarily want to make this part of the public API at least.
    pub(super) fn required_bits(&self) -> u32 {
        match *self {
            Value::Bool(value) => 32 - (value as u32).leading_zeros(),
            Value::U32(value) => 32 - value.leading_zeros(),
            Value::U64(value) => 64 - value.leading_zeros(),
            Value::U128(value) => 128 - value.leading_zeros(),
        }
    }

    pub(super) fn numeric_value(&self) -> u128 {
        match *self {
            Value::Bool(value) => value.into(),
            Value::U32(value) => value.into(),
            Value::U64(value) => value.into(),
            Value::U128(value) => value,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::U32(value as _)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::U32(value as _)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::U32(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::U64(value)
    }
}

impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value::U128(value)
    }
}
