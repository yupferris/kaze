use crate::code_writer;
use crate::graph;

use std::io::{Result, Write};

pub struct AssignmentContext {
    assignments: Vec<Assignment>,
    local_count: u32,
}

impl AssignmentContext {
    pub fn new() -> AssignmentContext {
        AssignmentContext {
            assignments: Vec::new(),
            local_count: 0,
        }
    }

    pub fn gen_temp(&mut self, expr: Expr) -> Expr {
        let name = format!("__temp_{}", self.local_count);
        self.local_count += 1;

        self.assignments.push(Assignment {
            target: Expr::Ref {
                name: name.clone(),
                scope: Scope::Local,
            },
            expr,
        });

        Expr::Ref {
            name,
            scope: Scope::Local,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    pub fn push(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        for assignment in self.assignments.iter() {
            assignment.write(w)?;
        }

        Ok(())
    }
}

pub struct Assignment {
    pub target: Expr,
    pub expr: Expr,
}

impl Assignment {
    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        w.append_indent()?;
        // TODO: I hate these kind of conditionals...
        if let Expr::Ref { ref scope, .. } = self.target {
            match scope {
                Scope::Local => {
                    w.append("let ")?;
                }
                Scope::Member => (),
            }
        }
        self.target.write(w)?;
        w.append(" = ")?;
        self.expr.write(w)?;
        w.append(";")?;
        w.append_newline()?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum Expr {
    ArrayIndex {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Cast {
        source: Box<Expr>,
        target_type: ValueType,
    },
    BinaryFunctionCall {
        name: String,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Constant {
        value: Constant,
    },
    InfixBinOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: InfixBinOp,
    },
    Ref {
        name: String,
        scope: Scope,
    },
    Ternary {
        cond: Box<Expr>,
        when_true: Box<Expr>,
        when_false: Box<Expr>,
    },
    UnaryMemberCall {
        target: Box<Expr>,
        name: String,
        arg: Box<Expr>,
    },
    UnOp {
        source: Box<Expr>,
        op: UnOp,
    },
}

impl Expr {
    pub fn from_constant(value: &graph::Constant, bit_width: u32) -> Expr {
        let value = value.numeric_value();

        let target_type = ValueType::from_bit_width(bit_width);
        Expr::Constant {
            value: match target_type {
                ValueType::Bool => Constant::Bool(value != 0),
                ValueType::I32 | ValueType::I64 | ValueType::I128 => unreachable!(),
                ValueType::U32 => Constant::U32(value as _),
                ValueType::U64 => Constant::U64(value as _),
                ValueType::U128 => Constant::U128(value),
            },
        }
    }

    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        match self {
            Expr::ArrayIndex { target, index } => {
                target.write(w)?;
                w.append("[")?;
                index.write(w)?;
                w.append(" as usize]")?;
            }
            Expr::Cast {
                source,
                target_type,
            } => {
                source.write(w)?;
                w.append(&format!(" as {}", target_type.name()))?;
            }
            Expr::BinaryFunctionCall { name, lhs, rhs } => {
                w.append(&format!("{}(", name))?;
                lhs.write(w)?;
                w.append(", ")?;
                rhs.write(w)?;
                w.append(")")?;
            }
            Expr::Constant { value } => {
                w.append(&match value {
                    Constant::Bool(value) => format!("{}", value),
                    Constant::U32(value) => format!("0x{:x}u32", value),
                    Constant::U64(value) => format!("0x{:x}u64", value),
                    Constant::U128(value) => format!("0x{:x}u128", value),
                })?;
            }
            Expr::InfixBinOp { lhs, rhs, op } => {
                lhs.write(w)?;
                w.append(&format!(
                    " {} ",
                    match op {
                        InfixBinOp::BitAnd => "&",
                        InfixBinOp::BitOr => "|",
                        InfixBinOp::BitXor => "^",
                        InfixBinOp::Equal => "==",
                        InfixBinOp::NotEqual => "!=",
                        InfixBinOp::LessThan => "<",
                        InfixBinOp::LessThanEqual => "<=",
                        InfixBinOp::GreaterThan => ">",
                        InfixBinOp::GreaterThanEqual => ">=",
                        InfixBinOp::Shl => "<<",
                        InfixBinOp::Shr => ">>",
                        InfixBinOp::Mul => "*",
                    }
                ))?;
                rhs.write(w)?;
            }
            Expr::Ref { name, scope } => {
                if let Scope::Member = scope {
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
            Expr::UnaryMemberCall { target, name, arg } => {
                target.write(w)?;
                w.append(&format!(".{}(", name))?;
                arg.write(w)?;
                w.append(")")?;
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
pub enum Constant {
    Bool(bool),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[derive(Clone)]
pub enum InfixBinOp {
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
    Mul,
}

#[derive(Clone)]
pub enum Scope {
    Local,
    Member,
}

#[derive(Clone)]
pub enum UnOp {
    Not,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ValueType {
    Bool,
    I32,
    I64,
    I128,
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

    pub fn to_signed(&self) -> ValueType {
        match self {
            ValueType::Bool | ValueType::I32 | ValueType::I64 | ValueType::I128 => unreachable!(),
            ValueType::U32 => ValueType::I32,
            ValueType::U64 => ValueType::I64,
            ValueType::U128 => ValueType::I128,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ValueType::Bool => "bool",
            ValueType::I32 => "i32",
            ValueType::I64 => "i64",
            ValueType::I128 => "i128",
            ValueType::U32 => "u32",
            ValueType::U64 => "u64",
            ValueType::U128 => "u128",
        }
    }

    pub fn bit_width(&self) -> u32 {
        match self {
            ValueType::Bool => 1,
            ValueType::I32 | ValueType::U32 => 32,
            ValueType::I64 | ValueType::U64 => 64,
            ValueType::I128 | ValueType::U128 => 128,
        }
    }

    pub fn zero_str(&self) -> &'static str {
        match self {
            ValueType::Bool => "false",
            _ => "0",
        }
    }
}
