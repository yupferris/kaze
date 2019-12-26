#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

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
