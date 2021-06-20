use super::constant::*;
use super::context::*;
use super::mem::*;
use super::module::*;
use super::register::*;

use std::hash::{Hash, Hasher};
use std::ptr;

pub struct InternalSignal<'a> {
    pub(super) context: &'a Context<'a>,
    pub(crate) module: &'a Module<'a>,

    pub(crate) data: SignalData<'a>,
}

impl<'a> InternalSignal<'a> {
    pub fn bit_width(&'a self) -> u32 {
        match self.data {
            SignalData::Lit { bit_width, .. } => bit_width,
            SignalData::Input { data } => data.bit_width,
            // TODO: Test above
            SignalData::Output { data } => data.bit_width,
            SignalData::Reg { data } => data.bit_width,
            SignalData::UnOp { bit_width, .. } => bit_width,
            SignalData::SimpleBinOp { bit_width, .. } => bit_width,
            SignalData::AdditiveBinOp { bit_width, .. } => bit_width,
            SignalData::ComparisonBinOp { .. } => 1,
            SignalData::ShiftBinOp { bit_width, .. } => bit_width,
            SignalData::Mul { bit_width, .. } => bit_width,
            SignalData::MulSigned { bit_width, .. } => bit_width,
            SignalData::Bits {
                range_high,
                range_low,
                ..
            } => range_high - range_low + 1,
            SignalData::Repeat { bit_width, .. } => bit_width,
            SignalData::Concat { bit_width, .. } => bit_width,
            SignalData::Mux { bit_width, .. } => bit_width,
            SignalData::MemReadPortOutput { mem, .. } => mem.element_bit_width,
        }
    }
}

impl<'a> Eq for &'a InternalSignal<'a> {}

impl<'a> Hash for &'a InternalSignal<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self as *const _ as usize)
    }
}

impl<'a> PartialEq for &'a InternalSignal<'a> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(*self, *other)
    }
}

pub(crate) enum SignalData<'a> {
    Lit {
        value: Constant,
        bit_width: u32,
    },

    Input {
        data: &'a InputData<'a>,
    },
    Output {
        data: &'a OutputData<'a>,
    },

    // TODO: Rename to Register?
    Reg {
        data: &'a RegisterData<'a>,
    },

    UnOp {
        source: &'a InternalSignal<'a>,
        op: UnOp,
        bit_width: u32,
    },
    SimpleBinOp {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        op: SimpleBinOp,
        bit_width: u32,
    },
    AdditiveBinOp {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        op: AdditiveBinOp,
        bit_width: u32,
    },
    ComparisonBinOp {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        op: ComparisonBinOp,
    },
    ShiftBinOp {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        op: ShiftBinOp,
        bit_width: u32,
    },

    Mul {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        bit_width: u32,
    },
    MulSigned {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        bit_width: u32,
    },

    Bits {
        source: &'a InternalSignal<'a>,
        range_high: u32,
        range_low: u32,
    },

    Repeat {
        source: &'a InternalSignal<'a>,
        count: u32,
        bit_width: u32,
    },
    Concat {
        lhs: &'a InternalSignal<'a>,
        rhs: &'a InternalSignal<'a>,
        bit_width: u32,
    },

    Mux {
        cond: &'a InternalSignal<'a>,
        when_true: &'a InternalSignal<'a>,
        when_false: &'a InternalSignal<'a>,
        bit_width: u32,
    },

    MemReadPortOutput {
        mem: &'a Mem<'a>,
        address: &'a InternalSignal<'a>,
        enable: &'a InternalSignal<'a>,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum UnOp {
    Not,
}

#[derive(Clone, Copy)]
pub(crate) enum SimpleBinOp {
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Clone, Copy)]
pub(crate) enum ComparisonBinOp {
    Equal,
    GreaterThan,
    GreaterThanEqual,
    GreaterThanEqualSigned,
    GreaterThanSigned,
    LessThan,
    LessThanEqual,
    LessThanEqualSigned,
    LessThanSigned,
    NotEqual,
}

#[derive(Clone, Copy)]
pub(crate) enum AdditiveBinOp {
    Add,
    Sub,
}

#[derive(Clone, Copy)]
pub(crate) enum ShiftBinOp {
    Shl,
    Shr,
    ShrArithmetic,
}

pub trait GetInternalSignal<'a> {
    // TODO: Rename to `get_internal_signal` ?
    fn internal_signal(&'a self) -> &'a InternalSignal<'a>;
}

impl<'a> GetInternalSignal<'a> for InternalSignal<'a> {
    fn internal_signal(&'a self) -> &'a InternalSignal<'a> {
        self
    }
}
