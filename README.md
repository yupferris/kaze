# kaze

An [HDL](https://en.wikipedia.org/wiki/Hardware_description_language) embedded in [Rust](https://www.rust-lang.org/).

kaze provides an API to describe `Module`s composed of `Signal`s, which can then be used to generate Rust simulator code or verilog modules.

kaze's API is designed to be as minimal as possible while still being expressive.
It's designed to keep the user from being able to describe buggy or incorrect hardware as much as possible.
This enables a user to hack on designs fearlessly, while the API and generators ensure that these designs are sound.

## Usage

```toml
[dependencies]
kaze = "0.1"
```

## Example

```rust
use kaze::*;

fn main() -> std::io::Result<()> {
    // Create a context, which will contain our module(s)
    let c = Context::new();

    // Create a module
    let inverter = c.module("inverter");
    let i = inverter.input("i", 1); // 1-bit input
    inverter.output("o", !i); // 1-bit output, representing inverted input

    // Generate Rust simulator code
    sim::generate(inverter, std::io::stdout())?;

    // Generate verilog code
    verilog::generate(inverter, std::io::stdout())?;

    Ok(())
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
