use crate::code_writer;
use crate::graph;

use std::io::{Result, Write};

pub struct NodeDecl {
    pub name: String,
    pub bit_width: u32,
}

impl NodeDecl {
    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        w.append_indent()?;
        w.append("logic ")?;
        if self.bit_width > 1 {
            w.append(&format!("[{}:{}] ", self.bit_width - 1, 0))?;
        }
        w.append(&format!("{};", self.name))?;
        w.append_newline()?;

        Ok(())
    }
}

pub struct AssignmentContext {
    assignments: Vec<Assignment>,
    local_decls: Vec<NodeDecl>,
}

impl AssignmentContext {
    pub fn new() -> AssignmentContext {
        AssignmentContext {
            assignments: Vec::new(),
            local_decls: Vec::new(),
        }
    }

    pub fn gen_temp(&mut self, expr: Expr, bit_width: u32) -> Expr {
        let name = format!("__temp_{}", self.local_decls.len());

        self.local_decls.push(NodeDecl {
            name: name.clone(),
            bit_width,
        });

        self.assignments.push(Assignment {
            target_name: name.clone(),
            expr,
        });

        Expr::Ref { name }
    }

    pub fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    pub fn push(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        for node_decl in self.local_decls.iter() {
            node_decl.write(w)?;
        }
        w.append_newline()?;

        for assignment in self.assignments.iter() {
            assignment.write(w)?;
        }

        Ok(())
    }
}

pub struct Assignment {
    pub target_name: String,
    pub expr: Expr,
}

impl Assignment {
    fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        w.append_indent()?;
        w.append(&format!("assign {}", self.target_name))?;
        w.append(" = ")?;
        self.expr.write(w)?;
        w.append(";")?;
        w.append_newline()?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum Expr {
    BinOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: BinOp,
    },
    Bits {
        source: Box<Expr>,
        range_high: u32,
        range_low: u32,
    },
    Concat {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Constant {
        bit_width: u32,
        value: u128,
    },
    Ref {
        name: String,
    },
    Repeat {
        source: Box<Expr>,
        count: u32,
    },
    Signed {
        source: Box<Expr>,
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
    pub fn from_constant(value: &graph::Constant, bit_width: u32) -> Expr {
        Expr::Constant {
            bit_width,
            value: value.numeric_value(),
        }
    }

    pub fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        match self {
            Expr::BinOp { lhs, rhs, op } => {
                lhs.write(w)?;
                w.append(&format!(
                    " {} ",
                    match op {
                        BinOp::Add => "+",
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
                        BinOp::ShrArithmetic => ">>>",
                        BinOp::Sub => "-",
                    }
                ))?;
                rhs.write(w)?;
            }
            Expr::Bits {
                source,
                range_high,
                range_low,
            } => {
                source.write(w)?;
                if range_high != range_low {
                    w.append(&format!("[{}:{}]", range_high, range_low))?;
                } else {
                    w.append(&format!("[{}]", range_high))?;
                }
            }
            Expr::Concat { lhs, rhs } => {
                w.append("{")?;
                lhs.write(w)?;
                w.append(", ")?;
                rhs.write(w)?;
                w.append("}")?;
            }
            Expr::Constant { bit_width, value } => {
                w.append(&format!("{}'h{:0x}", bit_width, value))?;
            }
            Expr::Ref { name } => {
                w.append(name)?;
            }
            Expr::Repeat { source, count } => {
                w.append(&format!("{{{}, {{", count))?;
                source.write(w)?;
                w.append("}}")?;
            }
            Expr::Signed { source } => {
                w.append("$signed(")?;
                source.write(w)?;
                w.append(")")?;
            }
            Expr::Ternary {
                cond,
                when_true,
                when_false,
            } => {
                cond.write(w)?;
                w.append(" ? ")?;
                when_true.write(w)?;
                w.append(" : ")?;
                when_false.write(w)?;
            }
            Expr::UnOp { source, op } => {
                w.append(match op {
                    UnOp::Not => "~",
                })?;
                source.write(w)?;
            }
        }

        Ok(())
    }
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
    ShrArithmetic,
    Sub,
}

#[derive(Clone)]
pub enum UnOp {
    Not,
}
