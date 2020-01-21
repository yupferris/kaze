use crate::code_writer;

use std::io::{Result, Write};

pub struct Assignment {
    pub target_scope: TargetScope,
    pub target_name: String,
    pub expr: Expr,
}

impl Assignment {
    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        match self.target_scope {
            TargetScope::Local => {
                w.append("let ")?;
            }
            TargetScope::Member => {
                w.append("self.")?;
            }
        }
        w.append(&format!("{} = ", self.target_name))?;
        self.expr.write(w)?;
        w.append(";")?;
        w.append_newline()?;

        Ok(())
    }
}

pub enum TargetScope {
    Local,
    Member,
}

#[derive(Clone)]
pub enum Expr {
    BinOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinOp,
    },
    Cast {
        source: Box<Expr>,
        target_type: ValueType,
    },
    Constant {
        value: Constant,
    },
    Ref {
        name: String,
        scope: RefScope,
    },
    Ternary {
        cond: Box<Expr>,
        when_true: Box<Expr>,
        when_false: Box<Expr>,
    },
    UnOp {
        source: Box<Expr>,
        op: UnOp,
    },
}

impl Expr {
    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        match self {
            Expr::BinOp { lhs, rhs, op } => {
                lhs.write(w)?;
                match op {
                    BinOp::Add => {
                        w.append(".wrapping_add(")?;
                        rhs.write(w)?;
                        w.append(")")?;
                    }
                    _ => {
                        w.append(&format!(
                            " {} ",
                            match op {
                                BinOp::Add => unreachable!(),
                                BinOp::BitAnd => "&",
                                BinOp::BitOr => "|",
                                BinOp::BitXor => "^",
                                BinOp::Equal => "==",
                                BinOp::NotEqual => "!=",
                                BinOp::LessThan => "<",
                                BinOp::LessThanEqual => "<=",
                                BinOp::GreaterThan => ">",
                                BinOp::GreaterThanEqual => ">=",
                                BinOp::Shl => "<<",
                                BinOp::Shr => ">>",
                            }
                        ))?;
                        rhs.write(w)?;
                    }
                }
            }
            Expr::Cast {
                source,
                target_type,
            } => {
                source.write(w)?;
                w.append(&format!(" as {}", target_type.name()))?;
            }
            Expr::Constant { value } => {
                w.append(&match value {
                    Constant::Bool(value) => format!("{}", value),
                    Constant::U32(value) => format!("0x{:x}u32", value),
                    Constant::U64(value) => format!("0x{:x}u64", value),
                    Constant::U128(value) => format!("0x{:x}u128", value),
                })?;
            }
            Expr::Ref { name, scope } => {
                if let RefScope::Member = scope {
                    w.append("self.")?;
                }
                w.append(name)?;
            }
            Expr::Ternary {
                cond,
                when_true,
                when_false,
            } => {
                w.append("if ")?;
                cond.write(w)?;
                w.append(" { ")?;
                when_true.write(w)?;
                w.append(" } else { ")?;
                when_false.write(w)?;
                w.append(" }")?;
            }
            Expr::UnOp { source, op } => {
                w.append(match op {
                    UnOp::Not => "!",
                })?;
                source.write(w)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum UnOp {
    Not,
}

#[derive(Clone)]
pub enum BinOp {
    Add,
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Shl,
    Shr,
}

#[derive(Clone)]
pub enum RefScope {
    Local,
    Member,
}

#[derive(Clone)]
pub enum Constant {
    Bool(bool),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[derive(Clone, Copy, PartialEq)]
pub enum ValueType {
    Bool,
    U32,
    U64,
    U128,
}

impl ValueType {
    pub fn from_bit_width(bit_width: u32) -> ValueType {
        if bit_width == 1 {
            ValueType::Bool
        } else if bit_width <= 32 {
            ValueType::U32
        } else if bit_width <= 64 {
            ValueType::U64
        } else if bit_width <= 128 {
            ValueType::U128
        } else {
            unreachable!()
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ValueType::Bool => "bool",
            ValueType::U32 => "u32",
            ValueType::U64 => "u64",
            ValueType::U128 => "u128",
        }
    }

    pub fn bit_width(&self) -> u32 {
        match self {
            ValueType::Bool => 1,
            ValueType::U32 => 32,
            ValueType::U64 => 64,
            ValueType::U128 => 128,
        }
    }
}
