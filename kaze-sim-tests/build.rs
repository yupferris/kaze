use kaze::*;
use kaze::module::*;

use std::env;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
enum Error {
    CodeWriter(code_writer::Error),
}

impl From<code_writer::Error> for Error {
    fn from(error: code_writer::Error) -> Error {
        Error::CodeWriter(error)
    }
}

fn main() -> Result<(), Error> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    sim::generate(&bitor_test_module(&c), &mut file)?;

    Ok(())
}

fn bitor_test_module<'a>(c: &'a Context<'a>) -> &'a Module<'a> {
    let m = c.module("bitor_test_module");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o", i1 | i2);

    m
}
