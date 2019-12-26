use kaze::module::*;
use kaze::*;

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

    sim::generate(&bitand_test_module(&c), &mut file)?;
    sim::generate(&bitor_test_module(&c), &mut file)?;
    sim::generate(&mux_test_module(&c), &mut file)?;

    Ok(())
}

fn bitand_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("bitand_test_module");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o", i1 & i2);

    m
}

fn bitor_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("bitor_test_module");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o", i1 | i2);

    m
}

fn mux_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("mux_test_module");

    let invert = m.input("invert", 1);

    let mut i = m.input("i", 1);
    kaze_sugar! {
        i = i;
        i = !i;
        i = !i;
        if (invert) {
            i = i; // TODO: Why does it break when we remove this?
            if (!m.low()) {
                i = !i;
            }
            //i = i; // TODO: Why does this not parse?
        }
        //i = i; // TODO: Why does this not parse?
    }

    m.output("o", i);

    m
}

// TODO: Better name?
#[macro_export]
macro_rules! kaze_sugar {
    ($($contents:tt)*) => {
        kaze_sugar_impl!([], [ $($contents)* ])
    };
}

#[macro_export(local__inner_macros)]
macro_rules! kaze_sugar_impl {
    ([], [ if ($sel:expr) { $($rest:tt)* } ]) => {
        kaze_sugar_impl!([ $sel ], [ $($rest)* ])
    };
    ([], [ $name:ident = $value:expr; $($rest:tt)* ]) => {
        $name = $value;
        kaze_sugar_impl!([], [ $($rest)* ]);
    };
    ([], []) => {};

    ([ $prev_sel:expr ], [ if ($sel:expr) { $($rest:tt)* } ]) => {
        kaze_sugar_impl!([ $prev_sel & $sel ], [ $($rest)* ])
    };
    ([ $sel:expr ], [ $name:ident = $value:expr; $($rest:tt)* ]) => {
        let prev = $name;
        kaze_sugar_impl!([ $sel ], [ $($rest)* ]);
        $name = prev.mux($value, $sel);
    };
    ([ $_:expr ], []) => {};
}
