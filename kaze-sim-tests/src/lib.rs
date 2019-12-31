#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn input_masking() {
        let mut m = input_masking::default();

        m.i = 0xffffffff;
        m.prop();
        assert_eq!(m.o, 0x07ffffff);
    }

    #[test]
    fn widest_input() {
        let mut m = widest_input::default();

        m.i = 0xfadebabedeadbeefabad1deabadc0de5;
        m.prop();
        assert_eq!(m.o, 0xfadebabedeadbeefabad1deabadc0de5);
    }

    #[test]
    fn bitand_test_module() {
        let mut m = bitand_test_module::default();

        m.i1 = false;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = false;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = false;
        assert_eq!(m.o, true); // No propagation
        m.prop();
        assert_eq!(m.o, false);
    }

    #[test]
    fn bitor_test_module() {
        let mut m = bitor_test_module::default();

        m.i1 = false;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = true;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = false;
        assert_eq!(m.o, true); // No propagation
        m.prop();
        assert_eq!(m.o, false);
    }

    #[test]
    fn not_test_module() {
        let mut m = not_test_module::default();

        m.i = 0;
        m.prop();
        assert_eq!(m.o, 0xf);

        m.i = 0xff;
        m.prop();
        assert_eq!(m.o, 0);

        m.i = 0xa;
        m.prop();
        assert_eq!(m.o, 0x5);

        m.i = 0x5;
        m.prop();
        assert_eq!(m.o, 0xa);
    }

    #[test]
    fn reg_test_module() {
        let mut m = reg_test_module::default();

        // Check initial value
        m.reset();
        m.prop();
        assert_eq!(m.o, 0);

        // Register value doesn't change without clock edge
        m.i = 0xdeadbeef;
        m.prop();
        assert_eq!(m.o, 0);
        m.posedge_clk();
        assert_eq!(m.o, 0); // No propagation
        m.prop();
        assert_eq!(m.o, 0xdeadbeef);
        m.i = 0xfadebabe;
        m.prop();
        assert_eq!(m.o, 0xdeadbeef);
    }

    #[test]
    fn simple_reg_delay() {
        let mut m = simple_reg_delay::default();

        // Check initial value
        m.reset();
        m.prop();
        assert_eq!(m.o, 0);

        // The input propagates through 3 registers, so we won't see it output for 3 cycles
        m.i = 0xffffffffffffffffffffffffffffffff;
        m.prop(); // Propagate to first register input
        assert_eq!(m.o, 0);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0xfffffffffffffffffffffffff);
    }

    #[test]
    fn bit_test_module_0() {
        let mut m = bit_test_module_0::default();

        m.i = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i = true;
        m.prop();
        assert_eq!(m.o, true);
    }

    #[test]
    fn bit_test_module_1() {
        let mut m = bit_test_module_1::default();

        m.i = 0b0110;
        m.prop();
        assert_eq!(m.o0, false);
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);
        assert_eq!(m.o3, false);
    }

    #[test]
    fn bits_test_module_0() {
        let mut m = bits_test_module_0::default();

        m.i = 0b0110;
        m.prop();
        assert_eq!(m.o210, 0b110);
        assert_eq!(m.o321, 0b011);
        assert_eq!(m.o10, 0b10);
        assert_eq!(m.o32, 0b01);
        assert_eq!(m.o2, true);

        m.i = 0b1001;
        m.prop();
        assert_eq!(m.o210, 0b001);
        assert_eq!(m.o321, 0b100);
        assert_eq!(m.o10, 0b01);
        assert_eq!(m.o32, 0b10);
        assert_eq!(m.o2, false);

        m.i = 0b1111;
        m.prop();
        assert_eq!(m.o210, 0b111);
        assert_eq!(m.o321, 0b111);
        assert_eq!(m.o10, 0b11);
        assert_eq!(m.o32, 0b11);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn bits_test_module_1() {
        let mut m = bits_test_module_1::default();

        m.i = 0xfadebabedeadbeefabad1deabadc0de5;
        m.prop();
        assert_eq!(m.o0, 0xfadebabedeadbeefabad1deabadc0de5u128);
        assert_eq!(m.o1, 0x7adebabedeadbeefabad1deabadc0de5u128);
        assert_eq!(m.o2, 0xfadebabedeadbeefu64);
        assert_eq!(m.o3, 0xabad1deabadc0de5u64);
        assert_eq!(m.o4, 0xfadebabeu32);
        assert_eq!(m.o5, 0xdeadbeefu32);
        assert_eq!(m.o6, 0xabad1deau32);
        assert_eq!(m.o7, 0xbadc0de5u32);
        assert_eq!(m.o8, 0xadebabedeadbeefau64);
        assert_eq!(m.o9, true);
        assert_eq!(m.o10, 0xabadu32);
        assert_eq!(m.o11, true);
    }

    #[test]
    fn repeat_test_module() {
        let mut m = repeat_test_module::default();

        m.i = 0xa;
        m.prop();
        assert_eq!(m.o0, 0xau32);
        assert_eq!(m.o1, 0xaau32);
        assert_eq!(m.o2, 0xaaaaau32);
        assert_eq!(m.o3, 0xaaaaaaaau32);
        assert_eq!(m.o4, 0xaaaaaaaaaaaaaaaau64);
        assert_eq!(m.o5, 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaau128);
        assert_eq!(m.o6, 0b000u32);
        assert_eq!(m.o7, 0x0u64);
        assert_eq!(m.o8, 0x0u128);

        m.i = 0x5;
        m.prop();
        assert_eq!(m.o0, 0x5u32);
        assert_eq!(m.o1, 0x55u32);
        assert_eq!(m.o2, 0x55555u32);
        assert_eq!(m.o3, 0x55555555u32);
        assert_eq!(m.o4, 0x5555555555555555u64);
        assert_eq!(m.o5, 0x55555555555555555555555555555555u128);
        assert_eq!(m.o6, 0b111u32);
        assert_eq!(m.o7, 0xffffffffffffffffu64);
        assert_eq!(m.o8, 0xffffffffffffffffffffffffffffffffu128);
    }

    #[test]
    fn concat_test_module() {
        let mut m = concat_test_module::default();

        m.i1 = 0xa;
        m.i2 = 0x5;
        m.i3 = 0xfadebabe;
        m.prop();
        assert_eq!(m.o0, 0xa5u32);
        assert_eq!(m.o1, 0x5au32);
        assert_eq!(m.o2, 0xau32);
        assert_eq!(m.o3, 0x1au32);
        assert_eq!(m.o4, 0x1au32);
        assert_eq!(m.o5, 0xfadebabefadebabeu64);
        assert_eq!(m.o6, 0xfadebabefadebabefadebabefadebabeu128);
    }

    #[test]
    fn mux_test_module() {
        let mut m = mux_test_module::default();

        m.i = false;
        m.invert = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i = true;
        m.invert = false;
        m.prop();
        assert_eq!(m.o, true);

        m.i = false;
        m.invert = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i = true;
        m.invert = true;
        m.prop();
        assert_eq!(m.o, false);
    }
}
