//! SystemVerilog code generation.

use crate::code_writer;
use crate::graph;
use crate::module_context::*;
use crate::state_elements::*;
use crate::validation::*;

use typed_arena::Arena;

use std::collections::HashMap;
use std::io::{Result, Write};
use std::ptr;

struct NodeDecl {
    name: String,
    bit_width: u32,
}

impl NodeDecl {
    fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
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

struct AssignmentContext {
    assignments: Vec<Assignment>,
    local_decls: Vec<NodeDecl>,
}

impl AssignmentContext {
    fn new() -> AssignmentContext {
        AssignmentContext {
            assignments: Vec::new(),
            local_decls: Vec::new(),
        }
    }

    fn gen_temp(&mut self, expr: Expr, bit_width: u32) -> Expr {
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

    fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    fn push(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
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

struct Assignment {
    target_name: String,
    expr: Expr,
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
enum Expr {
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

    fn write<W: Write>(&self, w: &mut code_writer::CodeWriter<W>) -> Result<()> {
        match self {
            Expr::BinOp { lhs, rhs, op } => {
                lhs.write(w)?;
                w.append(&format!(
                    " {} ",
                    match op {
                        BinOp::Add => "+",
                        BinOp::Equal => "==",
                        BinOp::NotEqual => "!=",
                        BinOp::LessThan => "<",
                        BinOp::LessThanEqual => "<=",
                        BinOp::GreaterThan => ">",
                        BinOp::GreaterThanEqual => ">=",
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
enum BinOp {
    Add,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Sub,
}

#[derive(Clone)]
enum UnOp {
    Not,
}

struct Compiler<'graph, 'arena> {
    state_elements: &'arena StateElements<'graph, 'arena>,

    signal_exprs: HashMap<
        (
            &'arena ModuleContext<'graph, 'arena>,
            &'graph graph::Signal<'graph>,
        ),
        Expr,
    >,
}

impl<'graph, 'arena> Compiler<'graph, 'arena> {
    pub fn new(state_elements: &'arena StateElements<'graph, 'arena>) -> Compiler<'graph, 'arena> {
        Compiler {
            state_elements,

            signal_exprs: HashMap::new(),
        }
    }

    pub fn compile_signal(
        &mut self,
        signal: &'graph graph::Signal<'graph>,
        context: &'arena ModuleContext<'graph, 'arena>,
        a: &mut AssignmentContext,
    ) -> Expr {
        let key = (context, signal);
        if !self.signal_exprs.contains_key(&key) {
            let expr = match signal.data {
                graph::SignalData::Lit {
                    ref value,
                    bit_width,
                } => Expr::from_constant(value, bit_width),

                graph::SignalData::Input { ref name, .. } => Expr::Ref { name: name.clone() },

                graph::SignalData::Reg { .. } => Expr::Ref {
                    name: self.state_elements.regs[&key].value_name.clone(),
                },

                graph::SignalData::UnOp { source, op } => {
                    let bit_width = source.bit_width();
                    let source = self.compile_signal(source, context, a);
                    a.gen_temp(
                        Expr::UnOp {
                            source: Box::new(source),
                            op: match op {
                                graph::UnOp::Not => UnOp::Not,
                            },
                        },
                        bit_width,
                    )
                }
                graph::SignalData::SimpleBinOp { .. } => unimplemented!(),
                graph::SignalData::AdditiveBinOp { lhs, rhs, op } => {
                    let bit_width = lhs.bit_width();
                    let lhs = self.compile_signal(lhs, context, a);
                    let rhs = self.compile_signal(rhs, context, a);
                    a.gen_temp(
                        Expr::BinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: match op {
                                graph::AdditiveBinOp::Add => BinOp::Add,
                                graph::AdditiveBinOp::Sub => BinOp::Sub,
                            },
                        },
                        bit_width,
                    )
                }
                graph::SignalData::ComparisonBinOp { lhs, rhs, op } => {
                    let bit_width = signal.bit_width();
                    let mut lhs = self.compile_signal(lhs, context, a);
                    let mut rhs = self.compile_signal(rhs, context, a);
                    match op {
                        graph::ComparisonBinOp::GreaterThanEqualSigned
                        | graph::ComparisonBinOp::GreaterThanSigned
                        | graph::ComparisonBinOp::LessThanEqualSigned
                        | graph::ComparisonBinOp::LessThanSigned => {
                            lhs = self.gen_signed(lhs, bit_width, a);
                            rhs = self.gen_signed(rhs, bit_width, a);
                        }
                        _ => (),
                    }
                    a.gen_temp(
                        Expr::BinOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op: match op {
                                graph::ComparisonBinOp::Equal => BinOp::Equal,
                                graph::ComparisonBinOp::NotEqual => BinOp::NotEqual,
                                graph::ComparisonBinOp::LessThan
                                | graph::ComparisonBinOp::LessThanSigned => BinOp::LessThan,
                                graph::ComparisonBinOp::LessThanEqual
                                | graph::ComparisonBinOp::LessThanEqualSigned => {
                                    BinOp::LessThanEqual
                                }
                                graph::ComparisonBinOp::GreaterThan
                                | graph::ComparisonBinOp::GreaterThanSigned => BinOp::GreaterThan,
                                graph::ComparisonBinOp::GreaterThanEqual
                                | graph::ComparisonBinOp::GreaterThanEqualSigned => {
                                    BinOp::GreaterThanEqual
                                }
                            },
                        },
                        bit_width,
                    )
                }
                graph::SignalData::ShiftBinOp { .. } => unimplemented!(),

                graph::SignalData::Bits {
                    source,
                    range_high,
                    range_low,
                } => {
                    let bit_width = signal.bit_width();
                    let source = self.compile_signal(source, context, a);
                    a.gen_temp(
                        Expr::Bits {
                            source: Box::new(source),
                            range_high,
                            range_low,
                        },
                        bit_width,
                    )
                }

                graph::SignalData::Repeat { .. } => unimplemented!(),
                graph::SignalData::Concat { lhs, rhs } => {
                    let bit_width = signal.bit_width();
                    let lhs = self.compile_signal(lhs, context, a);
                    let rhs = self.compile_signal(rhs, context, a);
                    a.gen_temp(
                        Expr::Concat {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        bit_width,
                    )
                }

                graph::SignalData::Mux {
                    cond,
                    when_true,
                    when_false,
                } => {
                    let bit_width = when_true.bit_width();
                    let cond = self.compile_signal(cond, context, a);
                    let when_true = self.compile_signal(when_true, context, a);
                    let when_false = self.compile_signal(when_false, context, a);
                    a.gen_temp(
                        Expr::Ternary {
                            cond: Box::new(cond),
                            when_true: Box::new(when_true),
                            when_false: Box::new(when_false),
                        },
                        bit_width,
                    )
                }

                graph::SignalData::InstanceOutput { .. } => unimplemented!(),

                graph::SignalData::MemReadPortOutput { .. } => unimplemented!(),
            };
            self.signal_exprs.insert(key.clone(), expr);
        }

        self.signal_exprs[&key].clone()
    }

    fn gen_signed(&mut self, expr: Expr, bit_width: u32, a: &mut AssignmentContext) -> Expr {
        a.gen_temp(
            Expr::Signed {
                source: Box::new(expr),
            },
            bit_width,
        )
    }
}

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
    validate_module_hierarchy(m);

    let context_arena = Arena::new();
    let root_context = context_arena.alloc(ModuleContext::new());

    let mut state_elements = StateElements::new();
    for (_, output) in m.outputs.borrow().iter() {
        state_elements.gather(&output, root_context, &context_arena);
    }

    let mut c = Compiler::new(&state_elements);

    let mut assignments = AssignmentContext::new();
    for (name, output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(&output, root_context, &mut assignments);
        assignments.push(Assignment {
            target_name: name.clone(),
            expr,
        });
    }
    let mut regs = Vec::new();
    let mut node_decls = Vec::new();
    for ((context, _), reg) in state_elements.regs.iter() {
        // TODO: Impl Eq/PartialEq for ModuleContext
        if ptr::eq(*context, root_context) {
            regs.push(reg);

            node_decls.push(NodeDecl {
                name: reg.value_name.clone(),
                bit_width: reg.data.bit_width,
            });
            node_decls.push(NodeDecl {
                name: reg.next_name.clone(),
                bit_width: reg.data.bit_width,
            });

            let expr = c.compile_signal(reg.data.next.borrow().unwrap(), context, &mut assignments);
            assignments.push(Assignment {
                target_name: reg.next_name.clone(),
                expr,
            });
        }
    }

    let mut w = code_writer::CodeWriter::new(w);

    w.append_line(&format!("module {}(", m.name))?;
    w.indent();

    // TODO: Make conditional based on the presents of (resetable) state elements
    w.append_line("input wire logic reset_n,")?;
    w.append_indent()?;
    w.append("input wire logic clk")?;
    if m.inputs.borrow().len() > 0 || m.outputs.borrow().len() > 0 {
        w.append(",")?;
        w.append_newline()?;
    }
    w.append_newline()?;
    let inputs = m.inputs.borrow();
    let num_inputs = inputs.len();
    for (i, (name, source)) in inputs.iter().enumerate() {
        w.append_indent()?;
        w.append("input wire logic ")?;
        if source.bit_width() > 1 {
            w.append(&format!("[{}:{}] ", source.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if m.outputs.borrow().len() > 0 || i < num_inputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    let outputs = m.outputs.borrow();
    let num_outputs = outputs.len();
    for (i, (name, output)) in outputs.iter().enumerate() {
        w.append_indent()?;
        w.append("output wire logic ")?;
        if output.bit_width() > 1 {
            w.append(&format!("[{}:{}] ", output.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if i < num_outputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    w.append_line(");")?;
    w.append_newline()?;

    if node_decls.len() > 0 {
        for node_decl in node_decls {
            node_decl.write(&mut w)?;
        }
        w.append_newline()?;
    }

    if regs.len() > 0 {
        for reg in regs {
            w.append_indent()?;
            w.append("always_ff @(posedge clk")?;
            if reg.data.initial_value.borrow().is_some() {
                w.append(", negedge reset_n")?;
            }
            w.append(") begin")?;
            w.append_newline()?;
            w.indent();
            if let Some(ref initial_value) = *reg.data.initial_value.borrow() {
                w.append_line("if (~reset_n) begin")?;
                w.indent();
                w.append_line(&format!(
                    "{} <= {}'h{:0x};",
                    reg.value_name,
                    reg.data.bit_width,
                    initial_value.numeric_value()
                ))?;
                w.unindent()?;
                w.append_line("end")?;
                w.append_line("else begin")?;
                w.indent();
            }
            w.append_line(&format!("{} <= {};", reg.value_name, reg.next_name))?;
            if reg.data.initial_value.borrow().is_some() {
                w.unindent()?;
                w.append_line("end")?;
            }
            w.unindent()?;
            w.append_line("end")?;
        }
        w.append_newline()?;
    }

    if !assignments.is_empty() {
        assignments.write(&mut w)?;
        w.append_newline()?;
    }

    w.unindent()?;
    w.append_line("endmodule")?;
    w.append_newline()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::*;

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because it has a recursive definition formed by an instance of itself called \"a\"."
    )]
    fn recursive_module_definition_error1() {
        let c = Context::new();

        let a = c.module("A");

        let _ = a.instance("a", "A");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because it has a recursive definition formed by an instance of itself called \"a\" in module \"B\"."
    )]
    fn recursive_module_definition_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");

        let _ = a.instance("b", "B");
        let _ = b.instance("a", "A");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains an instance of module \"B\" called \"b\" whose input \"i\" is not driven."
    )]
    fn undriven_instance_input_error() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.input("i", 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error1() {
        let c = Context::new();

        let a = c.module("A");
        let _ = a.reg("r", 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.reg("r", 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error1() {
        let c = Context::new();

        let a = c.module("A");
        let _ = a.mem("m", 1, 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.mem("m", 1, 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error1() {
        let c = Context::new();

        let a = c.module("A");
        let m = a.mem("m", 1, 1);
        let _ = m.read_port(a.low(), a.low());

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let m = b.mem("m", 1, 1);
        let _ = m.read_port(b.low(), b.low());

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"b\" because module \"a\" contains an output called \"o\" which forms a combinational loop with itself."
    )]
    fn combinational_loop_error() {
        let c = Context::new();

        let a = c.module("a");
        a.output("o", a.input("i", 1));

        let b = c.module("b");
        let a_inst = b.instance("a_inst", "a");
        let a_inst_o = a_inst.output("o");
        a_inst.drive_input("i", a_inst_o);

        // Panic
        generate(b, Vec::new()).unwrap();
    }
}
