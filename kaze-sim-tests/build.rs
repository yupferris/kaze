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

    sim::generate(&input_masking(&c), &mut file)?;
    sim::generate(&widest_input(&c), &mut file)?;
    sim::generate(&bitand_test_module(&c), &mut file)?;
    sim::generate(&bitor_test_module(&c), &mut file)?;
    sim::generate(&not_test_module(&c), &mut file)?;
    sim::generate(&reg_test_module(&c), &mut file)?;
    sim::generate(&simple_reg_delay(&c), &mut file)?;
    sim::generate(&bit_test_module_0(&c), &mut file)?;
    sim::generate(&bit_test_module_1(&c), &mut file)?;
    sim::generate(&bits_test_module_0(&c), &mut file)?;
    sim::generate(&bits_test_module_1(&c), &mut file)?;
    sim::generate(&repeat_test_module(&c), &mut file)?;
    sim::generate(&concat_test_module(&c), &mut file)?;
    sim::generate(&mux_test_module(&c), &mut file)?;

    Ok(())
}

fn input_masking<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("input_masking");

    m.output("o", m.input("i", 27));

    m
}

fn widest_input<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("widest_input");

    m.output("o", m.input("i", 128));

    m
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

fn not_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("not_test_module");

    let i = m.input("i", 4);
    m.output("o", !i);

    m
}

fn reg_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("reg_test_module");

    let r = m.reg(32, None);
    r.drive_next_with(m.input("i", 32));
    m.output("o", r.value());

    m
}

fn simple_reg_delay<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("simple_reg_delay");

    let r1 = m.reg(100, None);
    r1.drive_next_with(m.input("i", 100));
    let r2 = m.reg(100, None);
    r2.drive_next_with(r1.value());
    let r3 = m.reg(100, None);
    r3.drive_next_with(r2.value());
    m.output("o", r3.value());

    m
}

fn bit_test_module_0<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("bit_test_module_0");

    let i = m.input("i", 1);
    m.output("o", i.bit(0));

    m
}

fn bit_test_module_1<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("bit_test_module_1");

    let i = m.input("i", 4);
    m.output("o0", i.bit(0));
    m.output("o1", i.bit(1));
    m.output("o2", i.bit(2).bit(0));
    m.output("o3", i.bit(3));

    m
}

fn bits_test_module_0<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("bits_test_module_0");

    let i = m.input("i", 4);

    m.output("o210", i.bits(2, 0));
    m.output("o321", i.bits(3, 1).bits(2, 0));
    m.output("o10", i.bits(1, 0).bits(1, 0));
    m.output("o32", i.bits(3, 2));
    m.output("o2", i.bits(2, 2));

    m
}

fn bits_test_module_1<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("bits_test_module_1");

    let i = m.input("i", 128);

    m.output("o0", i.bits(127, 0));
    m.output("o1", i.bits(126, 0));
    m.output("o2", i.bits(127, 64));
    m.output("o3", i.bits(63, 0));
    m.output("o4", i.bits(127, 96));
    m.output("o5", i.bits(95, 64));
    m.output("o6", i.bits(63, 32).bits(31, 0));
    m.output("o7", i.bits(31, 0));
    m.output("o8", i.bits(123, 60));
    m.output("o9", i.bits(99, 99).bits(0, 0).bits(0, 0));
    m.output("o10", i.bits(63, 48));
    m.output("o11", i.bits(63, 0).bits(31, 0).bits(15, 0).bits(0, 0));

    m
}

fn repeat_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("repeat_test_module");

    let i = m.input("i", 4);

    m.output("o0", i.repeat(1));
    m.output("o1", i.repeat(2));
    m.output("o2", i.repeat(5));
    m.output("o3", i.repeat(8));
    m.output("o4", i.repeat(16));
    m.output("o5", i.repeat(32));
    m.output("o6", i.bit(0).repeat(3));
    m.output("o7", i.bit(0).repeat(64));
    m.output("o8", i.bit(0).repeat(128));

    m
}

fn concat_test_module<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("concat_test_module");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    let i3 = m.input("i3", 32);

    m.output("o0", i1.concat(i2));
    m.output("o1", i2.concat(i1));
    m.output("o2", m.low().concat(i1));
    m.output("o3", m.high().concat(i1));
    m.output("o4", i2.bit(0).concat(i1));
    m.output("o5", i3.concat(i3));
    m.output("o6", i3.concat(i3).concat(i3).concat(i3));

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
