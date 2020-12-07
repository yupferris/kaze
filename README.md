# kaze [風](https://jisho.org/search/%E9%A2%A8%20%23kanji)

An [HDL](https://en.wikipedia.org/wiki/Hardware_description_language) embedded in [Rust](https://www.rust-lang.org/).

[<img alt="github" src="https://img.shields.io/badge/github-yupferris/kaze-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/yupferris/kaze)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kaze.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kaze)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-kaze-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/kaze)
[<img alt="license" src="https://img.shields.io/crates/l/kaze?style=for-the-badge" height="20">](#license)

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
