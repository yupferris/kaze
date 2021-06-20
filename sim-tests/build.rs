use kaze::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let p = Context::new();

    sim::generate(
        input_masking(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        widest_input(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        add_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        sub_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        mul_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        mul_signed_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        shl_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        shr_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        shr_arithmetic_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bit_and_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bit_or_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bit_xor_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        not_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        reg_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        simple_reg_delay(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bit_test_module_0(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bit_test_module_1(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bits_test_module_0(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        bits_test_module_1(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        repeat_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        concat_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        eq_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        ne_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        lt_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        le_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        gt_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        ge_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        lt_signed_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        le_signed_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        gt_signed_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        ge_signed_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        mux_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        reg_next_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        reg_next_with_default_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        instantiation_test_module_comb(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        instantiation_test_module_reg(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        nested_instantiation_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        mem_test_module_0(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        mem_test_module_1(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        mem_test_module_2(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;
    sim::generate(
        trace_test_module_0(&p),
        sim::GenerationOptions {
            tracing: true,
            ..sim::GenerationOptions::default()
        },
        &mut file,
    )?;
    sim::generate(
        trace_test_module_1(&p),
        sim::GenerationOptions {
            tracing: true,
            ..sim::GenerationOptions::default()
        },
        &mut file,
    )?;
    sim::generate(
        trace_test_module_2(&p),
        sim::GenerationOptions {
            tracing: true,
            ..sim::GenerationOptions::default()
        },
        &mut file,
    )?;
    sim::generate(
        trace_test_module_3(&p),
        sim::GenerationOptions {
            tracing: true,
            ..sim::GenerationOptions::default()
        },
        &mut file,
    )?;
    sim::generate(
        deep_graph_test_module(&p),
        sim::GenerationOptions::default(),
        &mut file,
    )?;

    Ok(())
}

fn input_masking<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("input_masking", "InputMasking");

    m.output("o", m.input("i", 27));

    m
}

fn widest_input<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("widest_input", "WidestInput");

    m.output("o", m.input("i", 128));

    m
}

fn add_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("add_test_module", "AddTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1 + i2);

    let i3 = m.input("i3", 16);
    let i4 = m.input("i4", 16);
    m.output("o2", i3 + i4);

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 32);
    m.output("o3", i5 + i6);

    let i7 = m.input("i7", 64);
    let i8_ = m.input("i8", 64);
    m.output("o4", i7 + i8_);

    let i9 = m.input("i9", 128);
    let i10 = m.input("i10", 128);
    m.output("o5", i9 + i10);

    let i11 = m.input("i11", 7);
    let i12 = m.input("i12", 7);
    m.output("o6", i11 + i12);

    m
}

fn sub_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("sub_test_module", "SubTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1 - i2);

    let i3 = m.input("i3", 16);
    let i4 = m.input("i4", 16);
    m.output("o2", i3 - i4);

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 32);
    m.output("o3", i5 - i6);

    let i7 = m.input("i7", 64);
    let i8_ = m.input("i8", 64);
    m.output("o4", i7 - i8_);

    let i9 = m.input("i9", 128);
    let i10 = m.input("i10", 128);
    m.output("o5", i9 - i10);

    let i11 = m.input("i11", 7);
    let i12 = m.input("i12", 7);
    m.output("o6", i11 - i12);

    m
}

fn mul_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("mul_test_module", "MulTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1 * i2);

    let i3 = m.input("i3", 3);
    let i4 = m.input("i4", 4);
    m.output("o2", i3 * i4);

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 1);
    m.output("o3", i5 * i6);

    let i7 = m.input("i7", 32);
    let i8_ = m.input("i8", 32);
    m.output("o4", i7 * i8_);

    let i9 = m.input("i9", 64);
    let i10 = m.input("i10", 1);
    m.output("o5", i9 * i10);

    let i11 = m.input("i11", 64);
    let i12 = m.input("i12", 64);
    m.output("o6", i11 * i12);

    let i13 = m.input("i13", 127);
    let i14 = m.input("i14", 1);
    m.output("o7", i13 * i14);

    m
}

fn mul_signed_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("mul_signed_test_module", "MulSignedTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1.mul_signed(i2));

    let i3 = m.input("i3", 3);
    let i4 = m.input("i4", 4);
    m.output("o2", i3.mul_signed(i4));

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 1);
    m.output("o3", i5.mul_signed(i6));

    let i7 = m.input("i7", 32);
    let i8_ = m.input("i8", 32);
    m.output("o4", i7.mul_signed(i8_));

    let i9 = m.input("i9", 64);
    let i10 = m.input("i10", 1);
    m.output("o5", i9.mul_signed(i10));

    let i11 = m.input("i11", 64);
    let i12 = m.input("i12", 64);
    m.output("o6", i11.mul_signed(i12));

    let i13 = m.input("i13", 127);
    let i14 = m.input("i14", 1);
    m.output("o7", i13.mul_signed(i14));

    m
}

fn shl_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("shl_test_module", "ShlTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1 << i2);

    let i3 = m.input("i3", 16);
    let i4 = m.input("i4", 6);
    m.output("o2", i3 << i4);

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 32);
    m.output("o3", i5 << i6);

    let i7 = m.input("i7", 64);
    let i8_ = m.input("i8", 64);
    m.output("o4", i7 << i8_);

    let i9 = m.input("i9", 128);
    let i10 = m.input("i10", 128);
    m.output("o5", i9 << i10);

    let i11 = m.input("i11", 7);
    let i12 = m.input("i12", 7);
    m.output("o6", i11 << i12);

    let i13 = m.input("i13", 32);
    let i14 = m.input("i14", 1);
    m.output("o7", i13 << i14);

    let i15 = m.input("i15", 64);
    let i16_ = m.input("i16", 1);
    m.output("o8", i15 << i16_);

    let i17 = m.input("i17", 128);
    let i18 = m.input("i18", 1);
    m.output("o9", i17 << i18);

    m
}

fn shr_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("shr_test_module", "ShrTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1 >> i2);

    let i3 = m.input("i3", 16);
    let i4 = m.input("i4", 6);
    m.output("o2", i3 >> i4);

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 32);
    m.output("o3", i5 >> i6);

    let i7 = m.input("i7", 64);
    let i8_ = m.input("i8", 64);
    m.output("o4", i7 >> i8_);

    let i9 = m.input("i9", 128);
    let i10 = m.input("i10", 128);
    m.output("o5", i9 >> i10);

    let i11 = m.input("i11", 7);
    let i12 = m.input("i12", 7);
    m.output("o6", i11 >> i12);

    let i13 = m.input("i13", 32);
    let i14 = m.input("i14", 1);
    m.output("o7", i13 >> i14);

    let i15 = m.input("i15", 64);
    let i16_ = m.input("i16", 1);
    m.output("o8", i15 >> i16_);

    let i17 = m.input("i17", 128);
    let i18 = m.input("i18", 1);
    m.output("o9", i17 >> i18);

    m
}

fn shr_arithmetic_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("shr_arithmetic_test_module", "ShrArithmeticTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o1", i1.shr_arithmetic(i2));

    let i3 = m.input("i3", 16);
    let i4 = m.input("i4", 6);
    m.output("o2", i3.shr_arithmetic(i4));

    let i5 = m.input("i5", 32);
    let i6 = m.input("i6", 32);
    m.output("o3", i5.shr_arithmetic(i6));

    let i7 = m.input("i7", 64);
    let i8_ = m.input("i8", 64);
    m.output("o4", i7.shr_arithmetic(i8_));

    let i9 = m.input("i9", 128);
    let i10 = m.input("i10", 128);
    m.output("o5", i9.shr_arithmetic(i10));

    let i11 = m.input("i11", 7);
    let i12 = m.input("i12", 7);
    m.output("o6", i11.shr_arithmetic(i12));

    let i13 = m.input("i13", 32);
    let i14 = m.input("i14", 1);
    m.output("o7", i13.shr_arithmetic(i14));

    let i15 = m.input("i15", 64);
    let i16_ = m.input("i16", 1);
    m.output("o8", i15.shr_arithmetic(i16_));

    let i17 = m.input("i17", 128);
    let i18 = m.input("i18", 1);
    m.output("o9", i17.shr_arithmetic(i18));

    m
}

fn bit_and_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bit_and_test_module", "BitAndTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o", i1 & i2);

    m
}

fn bit_or_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bit_or_test_module", "BitOrTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o", i1 | i2);

    m
}

fn bit_xor_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bit_xor_test_module", "BitXorTestModule");

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);
    m.output("o", i1 ^ i2);

    m
}

fn not_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("not_test_module", "NotTestModule");

    let i = m.input("i", 4);
    m.output("o", !i);

    m
}

fn reg_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("reg_test_module", "RegTestModule");

    let r1 = m.reg("r1", 32);
    r1.default_value(0u32);
    r1.drive_next(m.input("i1", 32));
    m.output("o1", r1);

    let r2 = m.reg("r2", 32);
    r2.drive_next(m.input("i2", 32));
    m.output("o2", r2);

    m
}

fn simple_reg_delay<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("simple_reg_delay", "SimpleRegDelay");

    let r1 = m.reg("r1", 100);
    r1.default_value(0u32);
    r1.drive_next(m.input("i", 100));
    let r2 = m.reg("r2", 100);
    r2.default_value(0u32);
    r2.drive_next(r1);
    let r3 = m.reg("r3", 100);
    r3.default_value(0u32);
    r3.drive_next(r2);
    m.output("o", r3);

    m
}

fn bit_test_module_0<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bit_test_module_0", "BitTestModule0");

    let i = m.input("i", 1);
    m.output("o", i.bit(0));

    m
}

fn bit_test_module_1<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bit_test_module_1", "BitTestModule1");

    let i = m.input("i", 4);
    m.output("o0", i.bit(0));
    m.output("o1", i.bit(1));
    m.output("o2", i.bit(2).bit(0));
    m.output("o3", i.bit(3));

    m
}

fn bits_test_module_0<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bits_test_module_0", "BitsTestModule0");

    let i = m.input("i", 4);

    m.output("o210", i.bits(2, 0));
    m.output("o321", i.bits(3, 1).bits(2, 0));
    m.output("o10", i.bits(1, 0).bits(1, 0));
    m.output("o32", i.bits(3, 2));
    m.output("o2", i.bits(2, 2));

    m
}

fn bits_test_module_1<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("bits_test_module_1", "BitsTestModule1");

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

fn repeat_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("repeat_test_module", "RepeatTestModule");

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

fn concat_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("concat_test_module", "ConcatTestModule");

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

fn eq_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("eq_test_module", "EqTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.eq(i2));
    m.output("o2", i1.bit(0).eq(i2.bit(0)));

    m
}

fn ne_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("ne_test_module", "NeTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.ne(i2));
    m.output("o2", i1.bit(0).ne(i2.bit(0)));

    m
}

fn lt_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("lt_test_module", "LtTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.lt(i2));
    m.output("o2", i1.bit(0).lt(i2.bit(0)));

    m
}

fn le_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("le_test_module", "LeTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.le(i2));
    m.output("o2", i1.bit(0).le(i2.bit(0)));

    m
}

fn gt_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("gt_test_module", "GtTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.gt(i2));
    m.output("o2", i1.bit(0).gt(i2.bit(0)));

    m
}

fn ge_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("ge_test_module", "GeTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.ge(i2));
    m.output("o2", i1.bit(0).ge(i2.bit(0)));

    m
}

fn lt_signed_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("lt_signed_test_module", "LtSignedTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.lt_signed(i2));
    m.output("o2", i1.bits(1, 0).lt_signed(i2.bits(1, 0)));

    m
}

fn le_signed_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("le_signed_test_module", "LeSignedTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.le_signed(i2));
    m.output("o2", i1.bits(1, 0).le_signed(i2.bits(1, 0)));

    m
}

fn gt_signed_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("gt_signed_test_module", "GtSignedTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.gt_signed(i2));
    m.output("o2", i1.bits(1, 0).gt_signed(i2.bits(1, 0)));

    m
}

fn ge_signed_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("ge_signed_test_module", "GeSignedTestModule");

    let i1 = m.input("i1", 4);
    let i2 = m.input("i2", 4);
    m.output("o1", i1.ge_signed(i2));
    m.output("o2", i1.bits(1, 0).ge_signed(i2.bits(1, 0)));

    m
}

fn mux_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("mux_test_module", "MuxTestModule");

    let invert = m.input("invert", 1);

    let i1 = m.input("i1", 1);
    let i2 = m.input("i2", 1);

    let (i1, i2) = if_(m.high(), {
        let i1 = !i1;
        let i1 = i1;
        let i1 = !i1;
        if_(invert, {
            let i2 = i2;
            if_(!m.low(), {
                let i1 = !i1;
                let i1 = i1;
                let i2 = !i2;
                (i1, i2)
            })
            .else_((i1, i2))
        })
        .else_((i1, i2))
    })
    .else_((m.low(), m.low()));

    m.output("o1", i1);
    m.output("o2", i2);

    m
}

fn reg_next_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("reg_next_test_module", "RegNextTestModule");

    let i = m.input("i", 1);

    m.output("o1", i);
    m.output("o2", i.reg_next("i_delayed"));

    m
}

fn reg_next_with_default_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module(
        "reg_next_with_default_test_module",
        "RegNextWithDefaultTestModule",
    );

    let i = m.input("i", 1);

    m.output("o1", i);
    m.output("o2", i.reg_next_with_default("i_delayed", true));

    m
}

fn instantiation_test_module_comb<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    // TODO: Do we want to restructure all of these test modules to follow this pattern?
    struct InstantiationTestModuleCombInner<'a> {
        i1: &'a Input<'a>,
        i2: &'a Input<'a>,
        o: &'a Output<'a>,
    }

    impl<'a> InstantiationTestModuleCombInner<'a> {
        fn new(
            instance_name: impl Into<String>,
            p: &'a impl ModuleParent<'a>,
        ) -> InstantiationTestModuleCombInner<'a> {
            let m = p.module(instance_name, "InstantiationTestModuleCombInner");
            let i1 = m.input("i1", 32);
            let i2 = m.input("i2", 32);
            let o = m.output("o", i1 & i2);
            InstantiationTestModuleCombInner { i1, i2, o }
        }
    }

    let m = p.module(
        "instantiation_test_module_comb",
        "InstantiationTestModuleComb",
    );
    let inner1 = InstantiationTestModuleCombInner::new("inner1", m);
    inner1.i1.drive(m.input("i1", 32));
    inner1.i2.drive(m.input("i2", 32));
    let inner2 = InstantiationTestModuleCombInner::new("inner2", m);
    inner2.i1.drive(m.input("i3", 32));
    inner2.i2.drive(m.input("i4", 32));
    let inner3 = InstantiationTestModuleCombInner::new("inner3", m);
    inner3.i1.drive(inner1.o);
    inner3.i2.drive(inner2.o);
    m.output("o", inner3.o);

    m
}

fn instantiation_test_module_reg<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    struct InstantiationTestModuleRegInner<'a> {
        i1: &'a Input<'a>,
        i2: &'a Input<'a>,
        o: &'a Output<'a>,
    }

    impl<'a> InstantiationTestModuleRegInner<'a> {
        fn new(
            instance_name: impl Into<String>,
            p: &'a impl ModuleParent<'a>,
        ) -> InstantiationTestModuleRegInner<'a> {
            let m = p.module(instance_name, "InstantiationTestModuleRegInner");
            let i1 = m.input("i1", 32);
            let i2 = m.input("i2", 32);
            let r = m.reg("r", 32);
            r.default_value(0u32);
            r.drive_next(i1 & i2);
            let o = m.output("o", r);
            InstantiationTestModuleRegInner { i1, i2, o }
        }
    }

    let m = p.module(
        "instantiation_test_module_reg",
        "InstantiationTestModuleReg",
    );
    let inner1 = InstantiationTestModuleRegInner::new("inner1", m);
    inner1.i1.drive(m.input("i1", 32));
    inner1.i2.drive(m.input("i2", 32));
    let inner2 = InstantiationTestModuleRegInner::new("inner2", m);
    inner2.i1.drive(m.input("i3", 32));
    inner2.i2.drive(m.input("i4", 32));
    let inner3 = InstantiationTestModuleRegInner::new("inner3", m);
    inner3.i1.drive(inner1.o);
    inner3.i2.drive(inner2.o);
    m.output("o", inner3.o);

    m
}

fn nested_instantiation_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    struct NestedInstantiationTestModuleInnerInner<'a> {
        i: &'a Input<'a>,
        o: &'a Output<'a>,
    }

    impl<'a> NestedInstantiationTestModuleInnerInner<'a> {
        fn new(
            instance_name: impl Into<String>,
            p: &'a impl ModuleParent<'a>,
        ) -> NestedInstantiationTestModuleInnerInner<'a> {
            let m = p.module(instance_name, "NestedInstantiationTestModuleInnerInner");
            let i = m.input("i", 32);
            let o = m.output("o", i);
            NestedInstantiationTestModuleInnerInner { i, o }
        }
    }

    struct NestedInstantiationTestModuleInner<'a> {
        i1: &'a Input<'a>,
        i2: &'a Input<'a>,
        o: &'a Output<'a>,
    }

    impl<'a> NestedInstantiationTestModuleInner<'a> {
        fn new(
            instance_name: impl Into<String>,
            p: &'a impl ModuleParent<'a>,
        ) -> NestedInstantiationTestModuleInner<'a> {
            let m = p.module(instance_name, "NestedInstantiationTestModuleInner");
            let inner = NestedInstantiationTestModuleInnerInner::new("inner", m);
            let i1 = m.input("i1", 32);
            let i2 = m.input("i2", 32);
            inner.i.drive(i1 & i2);
            let o = m.output("o", inner.o);
            NestedInstantiationTestModuleInner { i1, i2, o }
        }
    }

    let m = p.module(
        "nested_instantiation_test_module",
        "NestedInstantiationTestModule",
    );
    let inner1 = NestedInstantiationTestModuleInner::new("inner1", m);
    inner1.i1.drive(m.input("i1", 32));
    inner1.i2.drive(m.input("i2", 32));
    let inner2 = NestedInstantiationTestModuleInner::new("inner2", m);
    inner2.i1.drive(m.input("i3", 32));
    inner2.i2.drive(m.input("i4", 32));
    let inner3 = NestedInstantiationTestModuleInner::new("inner3", m);
    inner3.i1.drive(inner1.o);
    inner3.i2.drive(inner2.o);
    m.output("o", inner3.o);

    m
}

fn mem_test_module_0<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("mem_test_module_0", "MemTestModule0");

    // No initial contents, single write port, single read port
    let mem = m.mem("mem", 1, 4);
    mem.write_port(
        m.input("write_addr", 1),
        m.input("write_value", 4),
        m.input("write_enable", 1),
    );
    m.output(
        "read_data",
        mem.read_port(m.input("read_addr", 1), m.input("read_enable", 1)),
    );

    m
}

fn mem_test_module_1<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("mem_test_module_1", "MemTestModule1");

    // Initial contents, no write ports, single read port
    let mem = m.mem("mem", 2, 32);
    mem.initial_contents(&[0xfadebabeu32, 0xdeadbeefu32, 0xabadcafeu32, 0xabad1deau32]);
    m.output(
        "read_data",
        mem.read_port(m.input("read_addr", 2), m.input("read_enable", 1)),
    );

    m
}

fn mem_test_module_2<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("mem_test_module_2", "MemTestModule2");

    // No initial contents, single write port, single read port
    let mem = m.mem("mem", 1, 1);
    mem.write_port(
        m.input("write_addr", 1),
        m.input("write_value", 1),
        m.input("write_enable", 1),
    );
    m.output(
        "read_data",
        mem.read_port(m.input("read_addr", 1), m.input("read_enable", 1)),
    );

    m
}

fn trace_test_module_0<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("trace_test_module_0", "TraceTestModule0");

    m.output("o0", m.input("i0", 1));
    m.output("o1", m.input("i1", 2));
    m.output("o2", m.input("i2", 32));
    m.output("o3", m.input("i3", 64));
    m.output("o4", m.input("i4", 128));

    m
}

fn trace_test_module_1<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("trace_test_module_1", "TraceTestModule1");

    let r1 = m.reg("r1", 32);
    r1.default_value(0u32);
    r1.drive_next(m.input("i1", 32));
    m.output("o1", r1);

    let r2 = m.reg("r2", 32);
    r2.drive_next(m.input("i2", 32));
    m.output("o2", r2);

    m
}

fn trace_test_module_2<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    struct TraceTestModule2Inner<'a> {
        i1: &'a Input<'a>,
        i2: &'a Input<'a>,
        o: &'a Output<'a>,
    }

    impl<'a> TraceTestModule2Inner<'a> {
        fn new(
            instance_name: impl Into<String>,
            p: &'a impl ModuleParent<'a>,
        ) -> TraceTestModule2Inner<'a> {
            let m = p.module(instance_name, "TraceTestModule2Inner");
            let i1 = m.input("i1", 32);
            let i2 = m.input("i2", 32);
            let r = m.reg("r", 32);
            r.default_value(0u32);
            r.drive_next(i1 & i2);
            let o = m.output("o", r);
            TraceTestModule2Inner { i1, i2, o }
        }
    }

    let m = p.module("trace_test_module_2", "TraceTestModule2");
    let inner1 = TraceTestModule2Inner::new("inner1", m);
    inner1.i1.drive(m.input("i1", 32));
    inner1.i2.drive(m.input("i2", 32));
    let inner2 = TraceTestModule2Inner::new("inner2", m);
    inner2.i1.drive(m.input("i3", 32));
    inner2.i2.drive(m.input("i4", 32));
    let inner3 = TraceTestModule2Inner::new("inner3", m);
    inner3.i1.drive(inner1.o);
    inner3.i2.drive(inner2.o);
    m.output("o", inner3.o);

    m
}

fn trace_test_module_3<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("trace_test_module_3", "TraceTestModule3");

    // No initial contents, single write port, single read port
    let mem = m.mem("mem", 1, 4);
    mem.write_port(
        m.input("write_addr", 1),
        m.input("write_value", 4),
        m.input("write_enable", 1),
    );
    m.output(
        "read_data",
        mem.read_port(m.input("read_addr", 1), m.input("read_enable", 1)),
    );

    m
}

fn deep_graph_test_module<'a>(p: &'a impl ModuleParent<'a>) -> &Module<'a> {
    let m = p.module("deep_graph_test_module", "DeepGraphTestModule");

    let mut x: &'a dyn Signal<'a> = m.input("i", 1);

    for _ in 0..3001 {
        x = !x;
    }

    m.output("o", x);

    m
}
