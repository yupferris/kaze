//! An [HDL](https://en.wikipedia.org/wiki/Hardware_description_language) embedded in [Rust](https://www.rust-lang.org/).
//!
//! kaze provides an API to describe [`Module`]s composed of [`Signal`]s, which can then be used to generate [Rust simulator code](sim::generate) or [Verilog modules](verilog::generate).
//!
//! kaze's API is designed to be as minimal as possible while still being expressive.
//! It's designed to prevent the user from being able to describe buggy or incorrect hardware as much as possible.
//! This enables a user to hack on designs fearlessly, while the API and generators ensure that these designs are sound.
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! kaze = "0.1"
//! ```
//!
//! # Examples
//!
//! ```rust
//! # fn main() -> std::io::Result<()> {
//! use kaze::*;
//!
//! // Create a context, which will contain our module(s)
//! let c = Context::new();
//!
//! // Create a module
//! let inverter = c.module("Inverter");
//! let i = inverter.input("i", 1); // 1-bit input
//! inverter.output("o", !i); // Output inverted input
//!
//! // Generate Rust simulator code
//! sim::generate(inverter, sim::GenerationOptions::default(), std::io::stdout())?;
//!
//! // Generate Verilog code
//! verilog::generate(inverter, std::io::stdout())?;
//! # Ok(())
//! # }
//! ```

// Must be kept up-to-date with version in Cargo.toml
#![doc(html_root_url = "https://docs.rs/kaze/0.1.14")]

mod code_writer;
mod graph;
mod module_context;
pub mod runtime;
pub mod sim;
mod validation;
pub mod verilog;

pub use graph::*;
