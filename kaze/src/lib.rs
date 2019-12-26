//! An HDL embedded in Rust.
//!
//! kaze has two primary goals:
//! - this is one TODO
//! - this is the other one TODO
//!
//! kaze's primary construct is the [`Module`]. Similar to Verilog/VHDL, a [`Module`] represents a blueprint or template for a given hardware module. [`Module`]s can be used to generate code directly, or be instantiated in other [`Module`]s.
//!
//! [`Module`]: ./module/struct.Module.html

pub mod code_writer;
pub mod module;
pub mod sim;
pub mod verilog;
