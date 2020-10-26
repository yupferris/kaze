# kaze [風](https://jisho.org/search/%E9%A2%A8%20%23kanji)

An [HDL](https://en.wikipedia.org/wiki/Hardware_description_language) embedded in [Rust](https://www.rust-lang.org/).

[![Latest version](https://img.shields.io/crates/v/kaze)](https://crates.io/crates/kaze)
[![Documentation](https://docs.rs/kaze/badge.svg)](https://docs.rs/kaze)
![License](https://img.shields.io/crates/l/kaze)

kaze provides an API to describe `Module`s composed of `Signal`s, which can then be used to generate Rust simulator code or Verilog modules.

kaze's API is designed to be as minimal as possible while still being expressive.
It's designed to prevent the user from being able to describe buggy or incorrect hardware as much as possible.
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
    let inverter = c.module("Inverter");
    let i = inverter.input("i", 1); // 1-bit input
    inverter.output("o", !i); // Output inverted input

    // Generate Rust simulator code
    sim::generate(inverter, sim::GenerationOptions::default(), std::io::stdout())?;

    // Generate Verilog code
    verilog::generate(inverter, std::io::stdout())?;

    Ok(())
}
```

## Releases

See [changelog](https://github.com/yupferris/kaze/blob/master/CHANGELOG.md) for release information.

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
