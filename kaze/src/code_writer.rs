use std::convert::From;
use std::io;

pub struct CodeWriter<'a, W: io::Write> {
    w: &'a mut W,
    indent_level: u32,
}

impl<'a, W: io::Write> CodeWriter<'a, W> {
    pub fn new(w: &'a mut W) -> CodeWriter<'a, W> {
        CodeWriter {
            w,
            indent_level: 0,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn unindent(&mut self) -> Result<(), Error> {
        if self.indent_level == 0 {
            return Err(Error::IndentUnderflow);
        }
        self.indent_level -= 1;
        Ok(())
    }

    pub fn append_indent(&mut self) -> Result<(), Error> {
        for _ in 0..self.indent_level {
            write!(self.w, "    ")?;
        }
        Ok(())
    }

    pub fn append_newline(&mut self) -> Result<(), Error> {
        writeln!(self.w, "")?;
        Ok(())
    }

    pub fn append(&mut self, s: &str) -> Result<(), Error> {
        write!(self.w, "{}", s)?;
        Ok(())
    }

    pub fn append_line(&mut self, s: &str) -> Result<(), Error> {
        self.append_indent()?;
        self.append(s)?;
        self.append_newline()?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    IndentUnderflow,
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}
