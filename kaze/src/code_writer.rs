use std::io::{Result, Write};

pub struct CodeWriter<W: Write> {
    w: W,
    indent_level: u32,
}

impl<W: Write> CodeWriter<W> {
    pub fn new(w: W) -> CodeWriter<W> {
        CodeWriter { w, indent_level: 0 }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn unindent(&mut self) -> Result<()> {
        if self.indent_level == 0 {
            panic!("Indent level underflow");
        }
        self.indent_level -= 1;
        Ok(())
    }

    pub fn append_indent(&mut self) -> Result<()> {
        for _ in 0..self.indent_level {
            write!(self.w, "    ")?;
        }
        Ok(())
    }

    pub fn append_newline(&mut self) -> Result<()> {
        writeln!(self.w, "")?;
        Ok(())
    }

    pub fn append(&mut self, s: &str) -> Result<()> {
        write!(self.w, "{}", s)?;
        Ok(())
    }

    pub fn append_line(&mut self, s: &str) -> Result<()> {
        self.append_indent()?;
        self.append(s)?;
        self.append_newline()?;
        Ok(())
    }
}
