#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn bitand_test_module() {
        let mut m = bitand_test_module::default();

        m.posedge_clk(); // Suppress unused method warning

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

        m.posedge_clk(); // Suppress unused method warning

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
    fn mux_test_module() {
        let mut m = mux_test_module::default();

        m.posedge_clk(); // Suppress unused method warning

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
